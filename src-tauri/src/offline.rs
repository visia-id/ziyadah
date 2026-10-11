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
    fn nama_berkas_everyayah() {
        assert_eq!(file_name(2, 255), "002255.mp3");
    }
}
