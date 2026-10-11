//! Unduhan audio offline per surah, per juz, atau seluruh mushaf per qari (F1-11).
//!
//! - File disimpan di folder cache audio yang sama dengan pemutaran (`audio/<qari>/SSSAAA.mp3`), jadi ayat yang
//!   pernah diputar ikut terhitung dan tidak ada audio dobel di disk.
//! - Satu koneksi, berurutan, supaya sopan ke server EveryAyah. File yang sudah ada dilewati, sehingga
//!   melanjutkan unduhan otomatis mulai dari yang belum.
//! - Ukuran tiap file diverifikasi terhadap Content-Length (lihat `audio::download`).
//! - Tiap file dicoba ulang beberapa kali. Bila beberapa file berturut-turut gagal, dianggap koneksi putus dan
//!   unduhan dihentikan dengan status "terputus"; pekerjaan tetap tersimpan untuk dilanjutkan.
//! - Pekerjaan disimpan di `offline.json`. Setelah aplikasi dibuka lagi, unduhan tidak jalan sendiri (bisa memakai
//!   kuota tanpa izin); pengguna menekan "Lanjutkan".
//! - Kemajuan disiarkan lewat event `offline://progress`.

use crate::audio::{self, BASE_URL};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

/// Percobaan per file sebelum dianggap gagal.
const TRIES: u32 = 3;
/// Bila sekian file berturut-turut gagal, koneksi dianggap putus dan unduhan dihentikan.
const MAX_CONSECUTIVE_FAILURES: u32 = 5;
/// Selang minimum pengiriman event kemajuan.
const EMIT_INTERVAL: Duration = Duration::from_millis(250);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Scope {
    Surah { surah: u16 },
    Juz { juz: u8 },
    All,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Job {
    pub reciter: String,
    pub scope: Scope,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    /// idle | running | done | failed | interrupted | cancelled | pending
    /// (`pending`: pekerjaan dari sesi sebelumnya yang belum selesai, menunggu "Lanjutkan").
    pub status: &'static str,
    pub job: Option<Job>,
    pub done: u32,
    pub total: u32,
    pub failed: u32,
    pub error: Option<String>,
}

impl Progress {
    fn idle() -> Self {
        Self { status: "idle", job: None, done: 0, total: 0, failed: 0, error: None }
    }
}

/// Daftar `(surah, ayat)` yang harus ada untuk `ayahs`, ditambah 1:1 bila ada surah yang diawali basmalah
/// (basmalah diputar dari berkas Al-Fatihah ayat 1). Tanpa duplikat, urut mushaf.
pub fn files_for(ayahs: impl IntoIterator<Item = (u16, u16)>) -> Vec<(u16, u16)> {
    let mut set: BTreeSet<(u16, u16)> = BTreeSet::new();
    for (surah, ayah) in ayahs {
        set.insert((surah, ayah));
        if ayah == 1 && surah != 1 && surah != 9 {
            set.insert((1, 1));
        }
    }
    set.into_iter().collect()
}

pub fn file_name(surah: u16, ayah: u16) -> String {
    format!("{surah:03}{ayah:03}.mp3")
}

/// Jumlah berkas ayat yang sudah ada per surah (indeks 0 = surah 1), dari nama berkas di folder qari.
pub fn counts_per_surah<'a>(names: impl IntoIterator<Item = &'a str>) -> Vec<u16> {
    let mut counts = vec![0u16; 114];
    for name in names {
        let Some(stem) = name.strip_suffix(".mp3") else { continue };
        if stem.len() != 6 || !stem.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let surah: usize = stem[..3].parse().unwrap_or(0);
        let ayah: u16 = stem[3..].parse().unwrap_or(0);
        if (1..=114).contains(&surah) && ayah >= 1 && ayah <= audio::ayah_count(surah as u16) {
            counts[surah - 1] += 1;
        }
    }
    counts
}

/// Pemakaian disk satu folder qari (F1-12).
#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    pub reciter: String,
    pub files: u32,
    pub bytes: u64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Deleted {
    pub files: u32,
    pub bytes: u64,
    /// Berkas yang tidak bisa dihapus, biasanya karena sedang diputar.
    pub skipped: u32,
}

/// Nama folder qari yang aman dipakai sebagai satu komponen path di dalam folder audio.
pub fn is_safe_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 100
        && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        && !name.starts_with('-')
}

/// Berkas yang dibuat Ziyadah di folder qari: audio ayat dan berkas sementara unduhan.
fn is_audio_file(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or("");
    let ext = &name[stem.len()..];
    stem.len() == 6 && stem.bytes().all(|b| b.is_ascii_digit()) && matches!(ext, ".mp3" | ".part" | ".offline-part")
}

/// Pemakaian disk per folder qari di `cache_dir`, terbesar dulu. Hanya berkas audio Ziyadah yang dihitung.
pub fn usage_in(cache_dir: &Path) -> Vec<Usage> {
    let mut list: Vec<Usage> = fs::read_dir(cache_dir)
        .map(|rd| rd.filter_map(|e| e.ok()).collect::<Vec<_>>())
        .unwrap_or_default()
        .into_iter()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter_map(|e| {
            let reciter = e.file_name().to_string_lossy().into_owned();
            if !is_safe_name(&reciter) {
                return None;
            }
            let (mut files, mut bytes) = (0u32, 0u64);
            for f in fs::read_dir(e.path()).ok()?.filter_map(|f| f.ok()) {
                if is_audio_file(&f.file_name().to_string_lossy()) {
                    files += 1;
                    bytes += f.metadata().map(|m| m.len()).unwrap_or(0);
                }
            }
            (files > 0).then_some(Usage { reciter, files, bytes })
        })
        .collect();
    list.sort_by(|a, b| b.bytes.cmp(&a.bytes));
    list
}

/// Hapus berkas audio Ziyadah di folder satu qari, lalu foldernya bila sudah kosong.
/// Berkas lain yang mungkin ditaruh pengguna di sana tidak disentuh.
pub fn delete_in(cache_dir: &Path, reciter: &str) -> Result<Deleted, String> {
    if !is_safe_name(reciter) {
        return Err("Nama qari tidak valid".into());
    }
    let dir = cache_dir.join(reciter);
    let mut out = Deleted { files: 0, bytes: 0, skipped: 0 };
    let Ok(rd) = fs::read_dir(&dir) else { return Ok(out) };
    for f in rd.filter_map(|f| f.ok()) {
        if !is_audio_file(&f.file_name().to_string_lossy()) || !f.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let len = f.metadata().map(|m| m.len()).unwrap_or(0);
        match fs::remove_file(f.path()) {
            Ok(()) => {
                out.files += 1;
                out.bytes += len;
            }
            Err(_) => out.skipped += 1,
        }
    }
    let _ = fs::remove_dir(&dir); // hanya berhasil bila kosong
    Ok(out)
}

enum Msg {
    Start { job: Job, files: Vec<(u16, u16)> },
}

pub struct Offline {
    tx: Sender<Msg>,
    progress: Arc<Mutex<Progress>>,
    cancel: Arc<AtomicBool>,
    cache_dir: PathBuf,
    state_file: PathBuf,
}

impl Offline {
    pub fn start(app: AppHandle, cache_dir: PathBuf, state_file: PathBuf) -> Self {
        let mut initial = Progress::idle();
        if let Some(job) = load_job(&state_file) {
            initial.status = "pending";
            initial.job = Some(job);
        }
        let progress = Arc::new(Mutex::new(initial));
        let cancel = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel::<Msg>();
        let (p, c, dir, file) = (progress.clone(), cancel.clone(), cache_dir.clone(), state_file.clone());
        thread::Builder::new()
            .name("ziyadah-offline".into())
            .spawn(move || {
                let client = reqwest::blocking::Client::builder()
                    .timeout(Duration::from_secs(60))
                    .user_agent(concat!("Ziyadah/", env!("CARGO_PKG_VERSION")))
                    .build()
                    .expect("gagal membuat HTTP client");
                while let Ok(Msg::Start { job, files }) = rx.recv() {
                    run_job(&app, &client, &dir, &file, &p, &c, job, files);
                }
            })
            .expect("gagal menjalankan thread unduhan offline");
        Self { tx, progress, cancel, cache_dir, state_file }
    }

    pub fn status(&self) -> Progress {
        self.progress.lock().map(|p| p.clone()).unwrap_or_else(|_| Progress::idle())
    }

    /// Mulai pekerjaan baru; yang sedang berjalan dihentikan lebih dulu.
    pub fn begin(&self, job: Job, files: Vec<(u16, u16)>) {
        self.cancel.store(true, Ordering::SeqCst);
        save_job(&self.state_file, &job);
        if let Ok(mut p) = self.progress.lock() {
            *p = Progress { status: "running", job: Some(job.clone()), done: 0, total: files.len() as u32, failed: 0, error: None };
        }
        let _ = self.tx.send(Msg::Start { job, files });
    }

    /// Pekerjaan yang tersimpan (untuk "Lanjutkan" setelah aplikasi dibuka lagi atau setelah terputus).
    pub fn saved_job(&self) -> Option<Job> {
        load_job(&self.state_file)
    }

    pub fn cancel(&self, app: &AppHandle) {
        self.cancel.store(true, Ordering::SeqCst);
        let _ = fs::remove_file(&self.state_file);
        if let Ok(mut p) = self.progress.lock() {
            if p.status != "running" {
                *p = Progress::idle();
                let _ = app.emit("offline://progress", p.clone());
            }
        }
    }

    pub fn usage(&self) -> Vec<Usage> {
        usage_in(&self.cache_dir)
    }

    /// Hapus audio satu qari. Unduhan offline qari itu dihentikan dan dilupakan lebih dulu.
    pub fn delete(&self, app: &AppHandle, reciter: &str) -> Result<Deleted, String> {
        let downloading = self.status().job.is_some_and(|j| j.reciter == reciter);
        if downloading || self.saved_job().is_some_and(|j| j.reciter == reciter) {
            self.cancel(app);
            // Beri waktu thread unduhan berhenti sebelum berkasnya dihapus.
            thread::sleep(Duration::from_millis(300));
        }
        delete_in(&self.cache_dir, reciter)
    }

    pub fn counts(&self, reciter: &str) -> Vec<u16> {
        let names: Vec<String> = fs::read_dir(self.cache_dir.join(reciter))
            .map(|rd| rd.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().into_owned()).collect())
            .unwrap_or_default();
        counts_per_surah(names.iter().map(String::as_str))
    }
}

fn load_job(path: &Path) -> Option<Job> {
    serde_json::from_str(&fs::read_to_string(path).ok()?).ok()
}

fn save_job(path: &Path, job: &Job) {
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Ok(text) = serde_json::to_string(job) {
        let _ = fs::write(path, text);
    }
}

#[allow(clippy::too_many_arguments)]
fn run_job(
    app: &AppHandle,
    client: &reqwest::blocking::Client,
    cache_dir: &Path,
    state_file: &Path,
    progress: &Mutex<Progress>,
    cancel: &AtomicBool,
    job: Job,
    files: Vec<(u16, u16)>,
) {
    cancel.store(false, Ordering::SeqCst);
    let dir = cache_dir.join(&job.reciter);
    let total = files.len() as u32;
    let (mut done, mut failed, mut streak) = (0u32, 0u32, 0u32);
    let mut last_emit = Instant::now() - EMIT_INTERVAL;
    let mut last_error = None;
    let set = |status: &'static str, done: u32, failed: u32, error: Option<String>| {
        if let Ok(mut p) = progress.lock() {
            *p = Progress { status, job: Some(job.clone()), done, total, failed, error };
            let _ = app.emit("offline://progress", p.clone());
        }
    };
    for (surah, ayah) in files {
        if cancel.load(Ordering::SeqCst) {
            // Dibatalkan pengguna, atau digantikan pekerjaan baru (yang sudah mengisi status sendiri).
            if load_job(state_file).as_ref() != Some(&job) {
                set("cancelled", done, failed, None);
            }
            return;
        }
        let name = file_name(surah, ayah);
        let path = dir.join(&name);
        let exists = fs::metadata(&path).map(|m| m.len() > 0).unwrap_or(false);
        if !exists {
            let url = format!("{BASE_URL}/{}/{name}", job.reciter);
            let mut result = Err(String::new());
            for attempt in 0..TRIES {
                if attempt > 0 {
                    thread::sleep(Duration::from_secs(2 * attempt as u64));
                }
                result = audio::download(client, &url, &path, "offline-part");
                if result.is_ok() || cancel.load(Ordering::SeqCst) {
                    break;
                }
            }
            if let Err(e) = result {
                failed += 1;
                streak += 1;
                last_error = Some(format!("{name}: {e}"));
                if streak >= MAX_CONSECUTIVE_FAILURES {
                    set("interrupted", done, failed, Some("Koneksi terputus. Unduhan bisa dilanjutkan.".into()));
                    return;
                }
                continue;
            }
        }
        streak = 0;
        done += 1;
        if last_emit.elapsed() >= EMIT_INTERVAL {
            set("running", done, failed, None);
            last_emit = Instant::now();
        }
    }
    if failed == 0 {
        let _ = fs::remove_file(state_file);
        set("done", done, 0, None);
    } else {
        set("failed", done, failed, last_error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basmalah_ikut_diunduh_sekali() {
        assert_eq!(files_for([(67, 1), (67, 2)]), vec![(1, 1), (67, 1), (67, 2)]);
        assert_eq!(files_for([(9, 1), (9, 2)]), vec![(9, 1), (9, 2)]);
        assert_eq!(files_for([(1, 1), (1, 2)]), vec![(1, 1), (1, 2)]);
        // Juz 30 dimulai dari An-Naba' 1 dan memuat banyak surah: berkas 1:1 tetap satu.
        let juz: Vec<_> = (1..=40).map(|a| (78, a)).chain((1..=46).map(|a| (79, a))).collect();
        let files = files_for(juz);
        assert_eq!(files.iter().filter(|f| **f == (1, 1)).count(), 1);
        assert_eq!(files.len(), 1 + 40 + 46);
    }

    #[test]
    fn hitung_berkas_per_surah() {
        let names = ["001001.mp3", "001007.mp3", "067001.mp3", "067030.mp3", "067031.mp3", "067002.part", "x.mp3", "115001.mp3"];
        let counts = counts_per_surah(names);
        assert_eq!(counts[0], 2);
        assert_eq!(counts[66], 2, "ayat 31 tidak ada di Al-Mulk, berkas .part tidak dihitung");
        assert_eq!(counts.iter().map(|&c| c as u32).sum::<u32>(), 4);
    }

    #[test]
    fn nama_qari_aman_untuk_path() {
        assert!(is_safe_name("Alafasy_128kbps"));
        assert!(is_safe_name("Abdurrahmaan_As-Sudais_192kbps"));
        for bad in ["", "..", "../x", "a/b", r"a\b", "C:", "-rf", "a b", "Alafasy.."] {
            assert!(!is_safe_name(bad), "{bad}");
        }
    }

    #[test]
    fn hanya_berkas_audio_ziyadah() {
        assert!(is_audio_file("067001.mp3"));
        assert!(is_audio_file("067001.part"));
        assert!(is_audio_file("067001.offline-part"));
        for other in ["catatan.txt", "67001.mp3", "067001.mp3.bak", "desktop.ini", "067001"] {
            assert!(!is_audio_file(other), "{other}");
        }
    }

    #[test]
    fn hitung_lalu_hapus_satu_qari() {
        let root = std::env::temp_dir().join(format!("ziyadah-uji-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let a = root.join("Alafasy_128kbps");
        let b = root.join("Husary_128kbps");
        fs::create_dir_all(&a).unwrap();
        fs::create_dir_all(&b).unwrap();
        fs::write(a.join("001001.mp3"), [0u8; 100]).unwrap();
        fs::write(a.join("067001.mp3"), [0u8; 300]).unwrap();
        fs::write(a.join("catatan.txt"), b"milik pengguna").unwrap();
        fs::write(b.join("067001.mp3"), [0u8; 50]).unwrap();

        let usage = usage_in(&root);
        assert_eq!(usage[0], Usage { reciter: "Alafasy_128kbps".into(), files: 2, bytes: 400 });
        assert_eq!(usage[1], Usage { reciter: "Husary_128kbps".into(), files: 1, bytes: 50 });

        assert_eq!(delete_in(&root, "Alafasy_128kbps").unwrap(), Deleted { files: 2, bytes: 400, skipped: 0 });
        assert!(a.join("catatan.txt").exists(), "berkas lain tidak disentuh, folder tetap ada");
        assert!(b.join("067001.mp3").exists(), "qari lain tidak terhapus");
        assert!(delete_in(&root, "../Husary_128kbps").is_err());

        assert_eq!(delete_in(&root, "Husary_128kbps").unwrap().files, 1);
        assert!(!b.exists(), "folder kosong ikut dihapus");
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn nama_berkas_everyayah() {
        assert_eq!(file_name(2, 255), "002255.mp3");
    }
}
