//! Mesin audio murottal.
//!
//! - Satu file MP3 per ayat (format EveryAyah: SSSAAA.mp3).
//! - Basmalah (001001) diputar sebelum ayat 1 untuk surah 2 sampai 114, kecuali surah 9.
//! - File diunduh ke cache oleh thread pengunduh, lalu diantrikan ke `Sink` rodio.
//!   Selalu ada satu ayat berikutnya yang sudah masuk antrian supaya pergantian ayat tanpa jeda.
//! - Ayat berikutnya ditentukan mode putar (`PlayMode`): berhenti di akhir surah, lanjut,
//!   ulang ayat, ulang surah, atau ulang rentang. Pindah surah dan kembali ke awal diberi jeda 1 detik.
//! - Status disiarkan ke semua jendela lewat event `player://state`.

use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rodio::source::Zero;
use rodio::{Decoder, OutputStream, Sink, Source};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

const BASE_URL: &str = "https://everyayah.com/data";
/// Jumlah entri yang dijaga ada di antrian Sink (yang sedang diputar + berikutnya).
const LOOKAHEAD: usize = 2;
/// Jumlah ayat ke depan yang diunduh lebih dulu ke cache.
const DOWNLOAD_AHEAD: usize = 20;
/// Jeda hening saat pindah surah atau kembali ke awal surah/rentang (F1-09).
const GAP: Duration = Duration::from_secs(1);

/// Jumlah ayat per surah (hitungan Kufi, total 6.236). Sementara di sini sampai `quran.db` ada (F1-05).
const AYAH_COUNTS: [u16; 114] = [
    7, 286, 200, 176, 120, 165, 206, 75, 129, 109, 123, 111, 43, 52, 99, 128, 111, 110, 98, 135, 112, 78,
    118, 64, 77, 227, 93, 88, 69, 60, 34, 30, 73, 54, 45, 83, 182, 88, 75, 85, 54, 53, 89, 59, 37, 35, 38,
    29, 18, 45, 60, 49, 62, 55, 78, 96, 29, 22, 24, 13, 14, 11, 11, 18, 12, 12, 30, 52, 52, 44, 28, 28, 20,
    56, 40, 31, 50, 40, 46, 42, 29, 19, 36, 25, 22, 17, 19, 26, 30, 20, 15, 21, 11, 8, 8, 19, 5, 8, 8, 11,
    11, 8, 3, 9, 5, 4, 7, 3, 6, 3, 5, 4, 5, 6,
];

/// Apa yang terjadi setelah sebuah ayat selesai (F1-08).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PlayMode {
    /// Berhenti di akhir surah.
    #[default]
    Stop,
    /// Lanjut ke surah berikutnya sampai An-Nas.
    Continue,
    /// Ulang ayat yang sedang diputar.
    RepeatAyah,
    /// Kembali ke awal surah setelah ayat terakhir.
    RepeatSurah,
    /// Ulang rentang ayat `from..=to` dalam surah yang sedang diputar.
    Range { from: u16, to: u16 },
}

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
    pub mode: PlayMode,
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
            mode: PlayMode::default(),
        }
    }
}

pub enum Command {
    Play {
        surah: u16,
        start_ayah: u16,
        reciter: String,
    },
    SetMode(PlayMode),
    Pause,
    Resume,
    Next,
    Prev,
    Stop,
}

/// Satu ayat: satu file audio.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

/// Isi antrian Sink: ayat, atau jeda hening yang membawa ayat sebelumnya untuk ditampilkan selama jeda.
#[derive(Clone, Copy)]
enum Entry {
    Ayah(Item),
    Gap(Item),
}

impl Entry {
    fn item(&self) -> Item {
        match *self {
            Entry::Ayah(it) | Entry::Gap(it) => it,
        }
    }
}

fn ayah_count(surah: u16) -> u16 {
    AYAH_COUNTS[(surah.clamp(1, 114) - 1) as usize]
}

/// Item pertama sebuah surah: basmalah untuk surah 2 sampai 114 kecuali 9, selain itu ayat 1.
fn first_item(surah: u16) -> Item {
    let ayah = if surah != 1 && surah != 9 { 0 } else { 1 };
    Item { surah, ayah }
}

fn linear_next(x: Item) -> Option<Item> {
    (x.ayah < ayah_count(x.surah)).then(|| Item { surah: x.surah, ayah: x.ayah + 1 })
}

fn linear_prev(x: Item) -> Item {
    if x.ayah > first_item(x.surah).ayah {
        Item { surah: x.surah, ayah: x.ayah - 1 }
    } else {
        x
    }
}

/// Titik mulai: mulai dari ayat 1 berarti basmalah ikut diputar.
fn range_start(surah: u16, from: u16) -> Item {
    if from <= 1 {
        first_item(surah)
    } else {
        Item { surah, ayah: from.min(ayah_count(surah)) }
    }
}

/// Item yang diputar setelah `x` menurut mode. `None` berarti pemutaran selesai.
fn next_item(mode: PlayMode, x: Item) -> Option<Item> {
    match mode {
        PlayMode::Stop => linear_next(x),
        PlayMode::Continue => linear_next(x).or_else(|| (x.surah < 114).then(|| first_item(x.surah + 1))),
        // Basmalah tidak diulang; yang diulang ayat sesudahnya.
        PlayMode::RepeatAyah => Some(if x.ayah == 0 { Item { surah: x.surah, ayah: 1 } } else { x }),
        PlayMode::RepeatSurah => linear_next(x).or(Some(first_item(x.surah))),
        PlayMode::Range { from, to } => {
            let count = ayah_count(x.surah);
            let from = from.clamp(1, count);
            let to = to.clamp(from, count);
            if x.ayah == 0 {
                Some(Item { surah: x.surah, ayah: from })
            } else if x.ayah >= from && x.ayah < to {
                Some(Item { surah: x.surah, ayah: x.ayah + 1 })
            } else {
                Some(range_start(x.surah, from))
            }
        }
    }
}

/// Tombol berikutnya: ayat berikutnya, menyeberang surah hanya di mode lanjut.
fn nav_next(mode: PlayMode, x: Item) -> Item {
    let next = match mode {
        PlayMode::Continue => next_item(mode, x),
        _ => linear_next(x),
    };
    next.unwrap_or(x)
}

/// Jeda dipasang saat pindah surah atau kembali ke posisi lebih awal (ulang surah/rentang),
/// tidak saat mengulang ayat yang sama.
fn needs_gap(prev: Item, next: Item) -> bool {
    next.surah != prev.surah || next.ayah < prev.ayah
}

/// Daftar ayat yang akan diputar mulai `start`, berhenti bila berulang atau sudah `n` item.
fn upcoming(mode: PlayMode, start: Item, n: usize) -> Vec<Item> {
    let mut v = vec![start];
    let mut x = start;
    while v.len() < n {
        match next_item(mode, x) {
            Some(y) if !v.contains(&y) => {
                v.push(y);
                x = y;
            }
            _ => break,
        }
    }
    v
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

/// Status pemutar yang hidup di thread audio.
struct Player {
    sink: Sink,
    downloader: Sender<DownloadJob>,
    generation: Arc<AtomicU64>,
    cache_dir: PathBuf,
    mode: PlayMode,
    reciter: String,
    /// Semua entri yang sudah dimasukkan ke Sink sejak mulai atau lompat terakhir.
    appended: Vec<Entry>,
    /// Ayat berikutnya yang akan dimasukkan ke Sink; `None` berarti tidak ada lagi.
    cursor: Option<Item>,
    active: bool,
    paused: bool,
    last_error: Option<String>,
}

impl Player {
    fn path_of(&self, item: &Item) -> PathBuf {
        self.cache_dir.join(&self.reciter).join(item.file_name())
    }

    /// Mulai (ulang) pemutaran dari `item`, membuang antrian lama.
    fn start_at(&mut self, item: Item) {
        self.sink.clear();
        self.sink.play();
        self.appended.clear();
        self.cursor = Some(item);
        self.active = true;
        self.paused = false;
        self.generation.fetch_add(1, Ordering::SeqCst);
        self.request_download();
    }

    fn stop(&mut self) {
        self.sink.clear();
        self.appended.clear();
        self.cursor = None;
        self.active = false;
        self.paused = false;
        self.generation.fetch_add(1, Ordering::SeqCst);
    }

    /// Unduh beberapa ayat ke depan mulai dari kursor; pekerjaan lama yang belum selesai diganti.
    fn request_download(&self) {
        let Some(start) = self.cursor else { return };
        let files = upcoming(self.mode, start, DOWNLOAD_AHEAD)
            .iter()
            .map(|it| (format!("{BASE_URL}/{}/{}", self.reciter, it.file_name()), self.path_of(it)))
            .collect();
        let generation = self.generation.load(Ordering::SeqCst);
        let _ = self.downloader.send(DownloadJob { generation, files });
    }

    fn last_ayah(&self) -> Option<Item> {
        self.appended.iter().rev().find_map(|e| match e {
            Entry::Ayah(it) => Some(*it),
            Entry::Gap(_) => None,
        })
    }

    /// Ayat yang sedang terdengar, atau yang sedang ditunggu bila antrian Sink kosong.
    fn current(&self) -> Option<Item> {
        let in_sink = self.sink.len();
        if in_sink > 0 && in_sink <= self.appended.len() {
            Some(self.appended[current_index(self.appended.len(), in_sink)].item())
        } else {
            self.cursor.or_else(|| self.appended.last().map(|e| e.item()))
        }
    }

    fn set_mode(&mut self, mode: PlayMode) {
        self.mode = mode;
        if !self.active {
            return;
        }
        if mode == PlayMode::RepeatAyah {
            // Ayat berikutnya mungkin sudah masuk Sink; mulai ulang ayat yang sedang terdengar.
            if let Some(cur) = self.current() {
                self.start_at(cur);
            }
            return;
        }
        if let Some(last) = self.last_ayah() {
            self.cursor = next_item(mode, last);
        }
        self.request_download();
    }

    /// Isi Sink selama file berikutnya sudah ada di cache. Mengembalikan true bila sedang menunggu unduhan.
    fn fill(&mut self) -> bool {
        while self.sink.len() < LOOKAHEAD {
            let Some(next) = self.cursor else { break };
            let after_gap = matches!(self.appended.last(), Some(Entry::Gap(_)));
            if let Some(prev) = self.last_ayah() {
                if !after_gap && needs_gap(prev, next) {
                    self.sink.append(Zero::<f32>::new(2, 44_100).take_duration(GAP));
                    self.appended.push(Entry::Gap(prev));
                    continue;
                }
            }
            let path = self.path_of(&next);
            if !path.exists() {
                break;
            }
            let decoded = File::open(&path)
                .map_err(|e| e.to_string())
                .and_then(|f| Decoder::new(BufReader::new(f)).map_err(|e| e.to_string()));
            self.cursor = next_item(self.mode, next);
            match decoded {
                Ok(src) => {
                    self.sink.append(src);
                    self.appended.push(Entry::Ayah(next));
                    self.request_download();
                }
                Err(e) => {
                    // File rusak: hapus supaya diunduh ulang lain kali, lalu lewati.
                    let _ = fs::remove_file(&path);
                    self.last_error = Some(format!("Audio rusak dilewati: {e}"));
                    break;
                }
            }
        }

        if self.sink.empty() {
            if self.cursor.is_none() {
                // Selesai: akhir surah di mode berhenti, atau An-Nas di mode lanjut.
                self.active = false;
            } else {
                return true;
            }
        }
        false
    }

    fn snapshot(&self, buffering: bool) -> PlayerState {
        let status = if !self.active {
            "idle"
        } else if self.paused {
            "paused"
        } else if buffering {
            // Ayat yang harus diputar belum selesai diunduh.
            "loading"
        } else {
            "playing"
        };
        let cur = self.current();
        PlayerState {
            status,
            surah: cur.map(|c| c.surah).unwrap_or(0),
            ayah: if self.active { cur.map(|c| c.ayah).unwrap_or(0) } else { 0 },
            reciter: self.reciter.clone(),
            buffering,
            error: self.last_error.clone(),
            mode: self.mode,
        }
    }
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

    let mut p = Player {
        sink,
        downloader,
        generation,
        cache_dir,
        mode: PlayMode::default(),
        reciter: String::new(),
        appended: Vec::new(),
        cursor: None,
        active: false,
        paused: false,
        last_error: None,
    };
    let mut last_sent = PlayerState::default();

    loop {
        match rx.recv_timeout(Duration::from_millis(60)) {
            Ok(cmd) => match cmd {
                Command::Play { surah, start_ayah, reciter } => {
                    p.reciter = reciter;
                    p.last_error = None;
                    p.start_at(range_start(surah.clamp(1, 114), start_ayah));
                }
                Command::SetMode(mode) => p.set_mode(mode),
                Command::Pause => {
                    if p.active {
                        p.sink.pause();
                        p.paused = true;
                    }
                }
                Command::Resume => {
                    if p.active {
                        p.sink.play();
                        p.paused = false;
                    }
                }
                Command::Next | Command::Prev => {
                    if let (true, Some(cur)) = (p.active, p.current()) {
                        let target = match cmd {
                            Command::Next => nav_next(p.mode, cur),
                            _ => linear_prev(cur),
                        };
                        p.start_at(target);
                    }
                }
                Command::Stop => p.stop(),
            },
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }

        let buffering = p.active && p.fill();
        let next_state = p.snapshot(buffering);
        if next_state != last_sent {
            *shared.lock().unwrap() = next_state.clone();
            let _ = app.emit("player://state", next_state.clone());
            last_sent = next_state;
        }
    }
}

/// Indeks entri yang sedang terdengar: entri yang sudah dimasukkan dikurangi yang masih menunggu di Sink.
fn current_index(appended: usize, sink_len: usize) -> usize {
    if sink_len == 0 {
        appended
    } else {
        appended - sink_len
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn it(surah: u16, ayah: u16) -> Item {
        Item { surah, ayah }
    }

    #[test]
    fn jumlah_ayat_6236() {
        assert_eq!(AYAH_COUNTS.iter().map(|&n| n as u32).sum::<u32>(), 6236);
        assert_eq!(ayah_count(2), 286);
        assert_eq!(ayah_count(114), 6);
    }

    #[test]
    fn basmalah_kecuali_fatihah_dan_taubah() {
        assert_eq!(first_item(1), it(1, 1));
        assert_eq!(first_item(9), it(9, 1));
        assert_eq!(first_item(67), it(67, 0));
        assert_eq!(range_start(67, 1), it(67, 0));
        assert_eq!(range_start(67, 5), it(67, 5));
    }

    #[test]
    fn nama_file_format_everyayah() {
        assert_eq!(Item { surah: 67, ayah: 0 }.file_name(), "001001.mp3");
        assert_eq!(Item { surah: 67, ayah: 5 }.file_name(), "067005.mp3");
        assert_eq!(Item { surah: 114, ayah: 6 }.file_name(), "114006.mp3");
    }

    #[test]
    fn mode_berhenti_di_akhir_surah() {
        assert_eq!(next_item(PlayMode::Stop, it(67, 0)), Some(it(67, 1)));
        assert_eq!(next_item(PlayMode::Stop, it(67, 29)), Some(it(67, 30)));
        assert_eq!(next_item(PlayMode::Stop, it(67, 30)), None);
    }

    #[test]
    fn mode_lanjut_ke_surah_berikutnya() {
        assert_eq!(next_item(PlayMode::Continue, it(67, 30)), Some(it(68, 0)));
        // Al-Anfal ke At-Taubah: tanpa basmalah.
        assert_eq!(next_item(PlayMode::Continue, it(8, 75)), Some(it(9, 1)));
        assert_eq!(next_item(PlayMode::Continue, it(114, 6)), None);
    }

    #[test]
    fn mode_ulang_ayat() {
        assert_eq!(next_item(PlayMode::RepeatAyah, it(67, 5)), Some(it(67, 5)));
        assert_eq!(next_item(PlayMode::RepeatAyah, it(67, 0)), Some(it(67, 1)));
    }

    #[test]
    fn mode_ulang_surah() {
        assert_eq!(next_item(PlayMode::RepeatSurah, it(67, 10)), Some(it(67, 11)));
        assert_eq!(next_item(PlayMode::RepeatSurah, it(67, 30)), Some(it(67, 0)));
        assert_eq!(next_item(PlayMode::RepeatSurah, it(1, 7)), Some(it(1, 1)));
    }

    #[test]
    fn mode_rentang_ayat() {
        let r = PlayMode::Range { from: 3, to: 5 };
        assert_eq!(next_item(r, it(2, 3)), Some(it(2, 4)));
        assert_eq!(next_item(r, it(2, 5)), Some(it(2, 3)));
        // Di luar rentang: kembali ke awal rentang.
        assert_eq!(next_item(r, it(2, 9)), Some(it(2, 3)));
        let dari_awal = PlayMode::Range { from: 1, to: 2 };
        assert_eq!(next_item(dari_awal, it(2, 0)), Some(it(2, 1)));
        assert_eq!(next_item(dari_awal, it(2, 2)), Some(it(2, 0)));
        // Batas akhir melebihi jumlah ayat dipotong ke ayat terakhir.
        assert_eq!(next_item(PlayMode::Range { from: 3, to: 99 }, it(112, 4)), Some(it(112, 3)));
    }

    #[test]
    fn jeda_saat_pindah_surah_atau_kembali_ke_awal() {
        assert!(needs_gap(it(67, 30), it(68, 0)));
        assert!(needs_gap(it(67, 30), it(67, 0)));
        assert!(needs_gap(it(2, 5), it(2, 3)));
        assert!(!needs_gap(it(67, 5), it(67, 5)));
        assert!(!needs_gap(it(67, 0), it(67, 1)));
    }

    #[test]
    fn tombol_berikutnya_dan_sebelumnya() {
        assert_eq!(nav_next(PlayMode::Stop, it(67, 30)), it(67, 30));
        assert_eq!(nav_next(PlayMode::Continue, it(67, 30)), it(68, 0));
        assert_eq!(nav_next(PlayMode::RepeatAyah, it(67, 5)), it(67, 6));
        assert_eq!(linear_prev(it(67, 1)), it(67, 0));
        assert_eq!(linear_prev(it(67, 0)), it(67, 0));
        assert_eq!(linear_prev(it(1, 1)), it(1, 1));
    }

    #[test]
    fn unduhan_ke_depan_berhenti_saat_berulang() {
        assert_eq!(upcoming(PlayMode::RepeatAyah, it(67, 5), 20), vec![it(67, 5)]);
        assert_eq!(upcoming(PlayMode::Range { from: 1, to: 2 }, it(67, 0), 20).len(), 3);
        assert_eq!(upcoming(PlayMode::Continue, it(114, 5), 20), vec![it(114, 5), it(114, 6)]);
    }

    #[test]
    fn indeks_yang_sedang_terdengar() {
        assert_eq!(current_index(3, 2), 1);
        assert_eq!(current_index(3, 1), 2);
        assert_eq!(current_index(3, 0), 3);
    }
}
