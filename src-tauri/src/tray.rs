//! Menu tray (F1-16): pilih surah, qari, dan mode putar tanpa membuka jendela utama.
//!
//! Qari terakhir dipegang di sini supaya tray bisa memutar surah walau pemutar sedang idle; jendela utama
//! mengabarkan pilihannya lewat `tray_set_reciter` dan mendengar `tray://reciter` bila qari diganti dari tray.
//! Mode putar tetap dipegang mesin audio; centang di menu mengikuti `player://state`.

use crate::audio::PlayMode;
use crate::quran::SurahIndexItem;
use std::sync::Mutex;
use tauri::menu::{CheckMenuItem, IsMenuItem, MenuItem, Submenu};
use tauri::{AppHandle, Wry};

/// Daftar qari, harus sama dengan `RECITERS` di `src/shared/player.ts`.
pub const RECITERS: [(&str, &str); 4] = [
    ("Alafasy_128kbps", "Mishary Rashid Alafasy"),
    ("Husary_128kbps", "Mahmoud Khalil Al-Husary"),
    ("Minshawy_Murattal_128kbps", "Mohamed Siddiq Al-Minshawi"),
    ("Abdul_Basit_Murattal_192kbps", "Abdul Basit Abdus Samad"),
];

/// Mode yang bisa dipilih dari tray. Rentang ayat butuh isian angka, jadi tetap diatur di jendela utama.
const MODES: [(&str, &str); 4] = [
    ("stop", "Berhenti di akhir surah"),
    ("continue", "Lanjut ke surah berikutnya"),
    ("repeatAyah", "Ulang ayat"),
    ("repeatSurah", "Ulang surah"),
];

/// Banyak surah per submenu, supaya daftar 114 surah tidak memanjang melewati layar.
const GROUP: usize = 20;

#[derive(Debug, PartialEq)]
pub enum TrayAction {
    Surah(u16),
    Reciter(String),
    Mode(PlayMode),
}

fn mode_key(mode: &PlayMode) -> Option<&'static str> {
    match mode {
        PlayMode::Stop => Some("stop"),
        PlayMode::Continue => Some("continue"),
        PlayMode::RepeatAyah => Some("repeatAyah"),
        PlayMode::RepeatSurah => Some("repeatSurah"),
        PlayMode::Range { .. } => None,
    }
}

fn mode_from_key(key: &str) -> Option<PlayMode> {
    match key {
        "stop" => Some(PlayMode::Stop),
        "continue" => Some(PlayMode::Continue),
        "repeatAyah" => Some(PlayMode::RepeatAyah),
        "repeatSurah" => Some(PlayMode::RepeatSurah),
        _ => None,
    }
}

pub fn is_reciter(id: &str) -> bool {
    RECITERS.iter().any(|(r, _)| *r == id)
}

/// Terjemahkan id item menu (`surah:12`, `qari:...`, `mode:...`) menjadi aksi.
pub fn parse_id(id: &str) -> Option<TrayAction> {
    let (kind, value) = id.split_once(':')?;
    match kind {
        "surah" => value.parse().ok().filter(|n| (1..=114).contains(n)).map(TrayAction::Surah),
        "qari" if is_reciter(value) => Some(TrayAction::Reciter(value.to_string())),
        "mode" => mode_from_key(value).map(TrayAction::Mode),
        _ => None,
    }
}

/// Judul submenu surah, misalnya "1–20 · Al-Faatiha sampai Taa-Haa".
fn group_title(items: &[SurahIndexItem]) -> String {
    let (first, last) = (&items[0], &items[items.len() - 1]);
    format!("{}–{} · {} sampai {}", first.number, last.number, first.name_latin, last.name_latin)
}

pub struct TrayMenus {
    reciter: Mutex<String>,
    reciter_items: Vec<(&'static str, CheckMenuItem<Wry>)>,
    mode_items: Vec<(&'static str, CheckMenuItem<Wry>)>,
}

impl TrayMenus {
    /// Bangun submenu Surah, Qari, dan Mode putar. Submenu Surah kosong bila data Qur'an belum ada.
    pub fn build(app: &AppHandle, index: &[SurahIndexItem]) -> tauri::Result<(Self, [Submenu<Wry>; 3])> {
        let mut groups = Vec::new();
        for chunk in index.chunks(GROUP) {
            let items = chunk
                .iter()
                .map(|s| {
                    let label = format!("{}. {} ({} ayat)", s.number, s.name_latin, s.ayah_count);
                    MenuItem::with_id(app, format!("surah:{}", s.number), label, true, None::<&str>)
                })
                .collect::<tauri::Result<Vec<_>>>()?;
            let refs: Vec<&dyn IsMenuItem<Wry>> = items.iter().map(|i| i as &dyn IsMenuItem<Wry>).collect();
            groups.push(Submenu::with_items(app, group_title(chunk), true, &refs)?);
        }
        let group_refs: Vec<&dyn IsMenuItem<Wry>> = groups.iter().map(|g| g as &dyn IsMenuItem<Wry>).collect();
        let surah = Submenu::with_items(app, "Putar surah", !index.is_empty(), &group_refs)?;

        let reciter_items = RECITERS
            .iter()
            .map(|(id, name)| {
                CheckMenuItem::with_id(app, format!("qari:{id}"), *name, true, *id == RECITERS[0].0, None::<&str>)
                    .map(|item| (*id, item))
            })
            .collect::<tauri::Result<Vec<_>>>()?;
        let refs: Vec<&dyn IsMenuItem<Wry>> = reciter_items.iter().map(|(_, i)| i as &dyn IsMenuItem<Wry>).collect();
        let reciter = Submenu::with_items(app, "Qari", true, &refs)?;

        let mode_items = MODES
            .iter()
            .map(|(key, label)| {
                CheckMenuItem::with_id(app, format!("mode:{key}"), *label, true, *key == "stop", None::<&str>)
                    .map(|item| (*key, item))
            })
            .collect::<tauri::Result<Vec<_>>>()?;
        let refs: Vec<&dyn IsMenuItem<Wry>> = mode_items.iter().map(|(_, i)| i as &dyn IsMenuItem<Wry>).collect();
        let mode = Submenu::with_items(app, "Mode putar", true, &refs)?;

        let menus = Self { reciter: Mutex::new(RECITERS[0].0.to_string()), reciter_items, mode_items };
        Ok((menus, [surah, reciter, mode]))
    }

    pub fn reciter(&self) -> String {
        self.reciter.lock().map(|r| r.clone()).unwrap_or_else(|_| RECITERS[0].0.to_string())
    }

    /// Simpan qari terakhir dan perbarui centangnya. Qari yang tidak dikenal diabaikan.
    pub fn set_reciter(&self, id: &str) {
        if !is_reciter(id) {
            return;
        }
        if let Ok(mut r) = self.reciter.lock() {
            *r = id.to_string();
        }
        for (rid, item) in &self.reciter_items {
            let _ = item.set_checked(*rid == id);
        }
    }

    /// Centang mode yang sedang berlaku; rentang ayat tidak punya item, jadi tidak ada yang dicentang.
    pub fn sync_mode(&self, mode: &PlayMode) {
        let current = mode_key(mode);
        for (key, item) in &self.mode_items {
            let _ = item.set_checked(Some(*key) == current);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn surah(number: u16, name: &str) -> SurahIndexItem {
        SurahIndexItem { number, name_ar: String::new(), name_latin: name.into(), ayah_count: 1 }
    }

    #[test]
    fn id_menu_diterjemahkan_ke_aksi() {
        assert_eq!(parse_id("surah:67"), Some(TrayAction::Surah(67)));
        assert_eq!(parse_id("surah:0"), None);
        assert_eq!(parse_id("surah:115"), None);
        assert_eq!(parse_id("qari:Husary_128kbps"), Some(TrayAction::Reciter("Husary_128kbps".into())));
        assert_eq!(parse_id("qari:Sudais"), None);
        assert_eq!(parse_id("mode:repeatAyah"), Some(TrayAction::Mode(PlayMode::RepeatAyah)));
        assert_eq!(parse_id("mode:range"), None);
        assert_eq!(parse_id("quit"), None);
    }

    #[test]
    fn mode_bolak_balik_kecuali_rentang() {
        for (key, _) in MODES {
            assert_eq!(mode_from_key(key).as_ref().and_then(mode_key), Some(key));
        }
        assert_eq!(mode_key(&PlayMode::Range { from: 1, to: 3 }), None);
    }

    #[test]
    fn surah_dibagi_per_dua_puluh() {
        let index: Vec<_> = (1..=114).map(|n| surah(n, &format!("S{n}"))).collect();
        let titles: Vec<_> = index.chunks(GROUP).map(group_title).collect();
        assert_eq!(titles.len(), 6);
        assert_eq!(titles[0], "1–20 · S1 sampai S20");
        assert_eq!(titles[5], "101–114 · S101 sampai S114");
    }
}
