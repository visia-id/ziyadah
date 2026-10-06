//! Mesin audio murottal.
//!
//! - Satu file MP3 per ayat (format EveryAyah: SSSAAA.mp3).
//! - Basmalah (001001) diputar sebelum ayat 1 untuk surah 2 sampai 114, kecuali surah 9.
//! - File diunduh ke cache oleh thread pengunduh, lalu diantrikan ke `Sink` rodio.
//!   Selalu ada satu ayat berikutnya yang sudah masuk antrian supaya pergantian ayat tanpa jeda.
//! - Status disiarkan ke semua jendela lewat event `player://state`.

use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rodio::{Decoder, OutputStream, Sink};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

const BASE_URL: &str = "https://everyayah.com/data";
/// Jumlah ayat yang dijaga ada di antrian Sink (yang sedang diputar + berikutnya).
const LOOKAHEAD: usize = 2;

#[derive(Clone, Serialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlayerState {
    pub status: &'static str, // idle | loading | playing | paused
    pub surah: u16,
    /// 0 = basmalah, 1..n = nomor ayat
    pub ayah: u16,
    pub reciter: String,
    pub buffering: bool,
    pub error: Option<String>,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            status: "idle",
            surah: 0,
            ayah: 0,
            reciter: String::new(),
            buffering: false,
            error: None,
        }
    }
}

pub enum Command {
    Play {
        surah: u16,
        ayah_count: u16,
        start_ayah: u16,
        reciter: String,
    },
    Pause,
    Resume,
    Next,
    Prev,
    Stop,
}

/// Satu item antrian: satu file audio.
#[derive(Clone, Copy)]
struct Item {
    surah: u16,
    /// 0 = basmalah
    ayah: u16,
}

impl Item {
    fn file_name(&self) -> String {
        if self.ayah == 0 {
            "001001.mp3".to_string()
        } else {
            format!("{:03}{:03}.mp3", self.surah, self.ayah)
        }
    }
}

#[derive(Clone)]
pub struct Audio {
    tx: Sender<Command>,
    state: Arc<Mutex<PlayerState>>,
}

impl Audio {
    pub fn start(app: AppHandle, cache_dir: PathBuf) -> Self {
        let (tx, rx) = mpsc::channel();
        let state = Arc::new(Mutex::new(PlayerState::default()));
        let st = state.clone();
        thread::Builder::new()
            .name("ziyadah-audio".into())
            .spawn(move || run_audio(app, cache_dir, rx, st))
            .expect("gagal menjalankan thread audio");
        Self { tx, state }
    }

    pub fn send(&self, cmd: Command) {
        let _ = self.tx.send(cmd);
    }

    pub fn state(&self) -> PlayerState {
        self.state.lock().unwrap().clone()
    }
}

/// Pekerjaan untuk thread pengunduh: unduh daftar file berurutan.
struct DownloadJob {
    generation: u64,
    files: Vec<(String, PathBuf)>,
}

fn spawn_downloader(current_gen: Arc<AtomicU64>) -> Sender<DownloadJob> {
    let (tx, rx) = mpsc::channel::<DownloadJob>();
    thread::Builder::new()
        .name("ziyadah-download".into())
        .spawn(move || {
            let client = reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(60))
                .user_agent("Ziyadah/0.0.1")
                .build()
                .expect("gagal membuat HTTP client");
            let mut pending: Option<DownloadJob> = None;
            loop {
                let job = match pending.take() {
                    Some(j) => j,
                    None => match rx.recv() {
                        Ok(j) => j,
                        Err(_) => return,
                    },
                };
                for (url, path) in &job.files {
                    // Ada pekerjaan baru (pengguna ganti surah/ayat): hentikan yang lama.
                    if let Ok(newer) = rx.try_recv() {
                        pending = Some(newer);
                        break;
                    }
                    if current_gen.load(Ordering::SeqCst) != job.generation {
                        break;
                    }
                    if path.exists() {
                        continue;
                    }
                    if let Err(e) = download(&client, url, path) {
                        eprintln!("[ziyadah] gagal mengunduh {url}: {e}");
                    }
                }
            }
        })
        .expect("gagal menjalankan thread pengunduh");
    tx
}

fn download(client: &reqwest::blocking::Client, url: &str, path: &Path) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let res = client.get(url).send().map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(format!("HTTP {}", res.status()));
    }
    let bytes = res.bytes().map_err(|e| e.to_string())?;
    // Tulis ke .part dulu lalu rename, supaya pemutar tidak membaca file setengah jadi.
    let part = path.with_extension("part");
    fs::write(&part, &bytes).map_err(|e| e.to_string())?;
    fs::rename(&part, path).map_err(|e| e.to_string())?;
    Ok(())
}

fn run_audio(app: AppHandle, cache_dir: PathBuf, rx: Receiver<Command>, shared: Arc<Mutex<PlayerState>>) {
    let (_stream, handle) = match OutputStream::try_default() {
        Ok(v) => v,
        Err(e) => {
            let mut s = shared.lock().unwrap();
            s.error = Some(format!("Perangkat audio tidak tersedia: {e}"));
            let _ = app.emit("player://state", s.clone());
            return;
        }
    };
    let sink = Sink::try_new(&handle).expect("gagal membuat sink audio");

    let generation = Arc::new(AtomicU64::new(0));
    let downloader = spawn_downloader(generation.clone());

    let mut queue: Vec<Item> = Vec::new();
    let mut reciter = String::new();
    // Indeks item berikutnya yang akan dimasukkan ke Sink.
    let mut next_to_append: usize = 0;
    let mut active = false;
    let mut paused = false;
    let mut last_error: Option<String> = None;
    let mut last_sent = PlayerState::default();

    let path_of = |reciter: &str, item: &Item| cache_dir.join(reciter).join(item.file_name());

    // Mulai (ulang) pemutaran dari indeks antrian tertentu.
    let start_from = |idx: usize,
                      queue: &Vec<Item>,
                      reciter: &str,
                      next_to_append: &mut usize| {
        sink.clear();
        sink.play();
        *next_to_append = idx;
        let gen = generation.fetch_add(1, Ordering::SeqCst) + 1;
        let files = queue[idx..]
            .iter()
            .map(|it| {
                (
                    format!("{BASE_URL}/{reciter}/{}", it.file_name()),
                    path_of(reciter, it),
                )
            })
            .collect();
        let _ = downloader.send(DownloadJob { generation: gen, files });
    };

    loop {
        match rx.recv_timeout(Duration::from_millis(60)) {
            Ok(cmd) => match cmd {
                Command::Play {
                    surah,
                    ayah_count,
                    start_ayah,
                    reciter: r,
                } => {
                    queue = build_queue(surah, ayah_count);
                    reciter = r;
                    active = true;
                    paused = false;
                    last_error = None;
                    let start = queue
                        .iter()
                        .position(|it| it.ayah == start_ayah.max(1))
                        .unwrap_or(0);
                    // Mulai dari ayat 1 berarti basmalah ikut diputar.
                    let start = if start_ayah <= 1 { 0 } else { start };
                    start_from(start, &queue, &reciter, &mut next_to_append);
                }
                Command::Pause => {
                    if active {
                        sink.pause();
                        paused = true;
                    }
                }
                Command::Resume => {
                    if active {
                        sink.play();
                        paused = false;
                    }
                }
                Command::Next | Command::Prev => {
                    if active && !queue.is_empty() {
                        let cur = current_index(next_to_append, sink.len()).min(queue.len() - 1);
                        let target = match cmd {
                            Command::Next => (cur + 1).min(queue.len() - 1),
                            _ => cur.saturating_sub(1),
                        };
                        paused = false;
                        start_from(target, &queue, &reciter, &mut next_to_append);
                    }
                }
                Command::Stop => {
                    sink.clear();
                    active = false;
                    paused = false;
                    generation.fetch_add(1, Ordering::SeqCst);
                }
            },
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }

        let mut buffering = false;
        if active {
            // Isi antrian Sink selama file berikutnya sudah siap di cache.
            while sink.len() < LOOKAHEAD && next_to_append < queue.len() {
                let path = path_of(&reciter, &queue[next_to_append]);
                if !path.exists() {
                    break;
                }
                match File::open(&path)
                    .map_err(|e| e.to_string())
                    .and_then(|f| Decoder::new(BufReader::new(f)).map_err(|e| e.to_string()))
                {
                    Ok(src) => sink.append(src),
                    Err(e) => {
                        // File rusak: hapus supaya diunduh ulang lain kali, lalu lewati.
                        let _ = fs::remove_file(&path);
                        last_error = Some(format!("Audio rusak dilewati: {e}"));
                    }
                }
                next_to_append += 1;
            }

            if sink.empty() {
                if next_to_append >= queue.len() {
                    // Surah selesai.
                    active = false;
                } else {
                    buffering = true;
                }
            }
        }

        let mut next_state = PlayerState {
            status: if !active {
                "idle"
            } else if paused {
                "paused"
            } else if buffering {
                // Ayat yang harus diputar belum selesai diunduh.
                "loading"
            } else {
                "playing"
            },
            surah: 0,
            ayah: 0,
            reciter: reciter.clone(),
            buffering,
            error: last_error.clone(),
        };
        if active && !queue.is_empty() {
            let idx = current_index(next_to_append, sink.len()).min(queue.len() - 1);
            next_state.surah = queue[idx].surah;
            next_state.ayah = queue[idx].ayah;
        } else if let Some(first) = queue.first() {
            next_state.surah = first.surah;
        }

        if next_state != last_sent {
            *shared.lock().unwrap() = next_state.clone();
            let _ = app.emit("player://state", next_state.clone());
            last_sent = next_state;
        }
    }
}

/// Indeks item yang sedang terdengar: item yang sudah dimasukkan dikurangi yang masih menunggu di Sink.
fn current_index(next_to_append: usize, sink_len: usize) -> usize {
    if sink_len == 0 {
        next_to_append
    } else {
        next_to_append - sink_len
    }
}

fn build_queue(surah: u16, ayah_count: u16) -> Vec<Item> {
    let mut q = Vec::with_capacity(ayah_count as usize + 1);
    if surah != 1 && surah != 9 {
        q.push(Item { surah, ayah: 0 });
    }
    for a in 1..=ayah_count {
        q.push(Item { surah, ayah: a });
    }
    q
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basmalah_ditambahkan_kecuali_fatihah_dan_taubah() {
        assert_eq!(build_queue(67, 30).len(), 31);
        assert_eq!(build_queue(67, 30)[0].ayah, 0);
        assert_eq!(build_queue(1, 7).len(), 7);
        assert_eq!(build_queue(9, 129).len(), 129);
        assert_eq!(build_queue(9, 129)[0].ayah, 1);
    }

    #[test]
    fn nama_file_format_everyayah() {
        assert_eq!(Item { surah: 67, ayah: 0 }.file_name(), "001001.mp3");
        assert_eq!(Item { surah: 67, ayah: 5 }.file_name(), "067005.mp3");
        assert_eq!(Item { surah: 114, ayah: 6 }.file_name(), "114006.mp3");
    }

    #[test]
    fn indeks_yang_sedang_terdengar() {
        assert_eq!(current_index(3, 2), 1);
        assert_eq!(current_index(3, 1), 2);
        assert_eq!(current_index(3, 0), 3);
    }
}
