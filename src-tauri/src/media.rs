//! Kontrol media OS (F1-10): tombol media keyboard dan headset, kontrol media Windows, Now Playing macOS.
//!
//! Tombol media diteruskan ke mesin audio. Saat pemutar idle, Ziyadah melapor "berhenti" dan mengabaikan tombol
//! putar, supaya tombol media tetap bisa dipakai aplikasi lain dan jendela tidak tiba-tiba muncul.

use crate::audio::{Audio, Command, PlayerState};
use crate::tray::RECITERS;
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

pub struct Media {
    controls: Mutex<MediaControls>,
    /// Isi terakhir yang dikirim ke OS, supaya tidak memperbarui berulang tiap event status.
    last: Mutex<Option<(String, String, &'static str)>>,
}

// MediaControls di Windows (SMTC) dan macOS hanya dipakai lewat Mutex dari event pemutar; aman dibagi antarthread.
unsafe impl Send for Media {}
unsafe impl Sync for Media {}

/// Judul yang tampil di kontrol media OS, misalnya "Al-Mulk · ayat 3".
pub fn title(surah_name: &str, ayah: u16) -> String {
    if ayah == 0 {
        format!("{surah_name} · Basmalah")
    } else {
        format!("{surah_name} · ayat {ayah}")
    }
}

fn reciter_name(id: &str) -> &str {
    RECITERS.iter().find(|(rid, _)| *rid == id).map(|(_, name)| *name).unwrap_or(id)
}

/// Terjemahkan tombol media menjadi perintah untuk mesin audio, dengan status pemutar saat ini.
pub fn command_for(event: &MediaControlEvent, status: &str) -> Option<Command> {
    let active = status != "idle";
    match event {
        MediaControlEvent::Toggle if status == "playing" => Some(Command::Pause),
        MediaControlEvent::Toggle | MediaControlEvent::Play if status == "paused" => Some(Command::Resume),
        MediaControlEvent::Pause if status == "playing" => Some(Command::Pause),
        MediaControlEvent::Next if active => Some(Command::Next),
        MediaControlEvent::Previous if active => Some(Command::Prev),
        MediaControlEvent::Stop if active => Some(Command::Stop),
        _ => None,
    }
}

impl Media {
    /// Daftarkan Ziyadah ke kontrol media OS. Di Windows SMTC butuh handle jendela; dipakai jendela utama.
    pub fn start(app: &AppHandle) -> Option<Self> {
        #[cfg(windows)]
        let hwnd = app.get_webview_window("main").and_then(|w| w.hwnd().ok()).map(|h| h.0 as *mut std::ffi::c_void);
        #[cfg(not(windows))]
        let hwnd = None;
        let config = PlatformConfig { dbus_name: "ziyadah", display_name: "Ziyadah", hwnd };
        let mut controls = MediaControls::new(config).ok()?;
        let handle = app.clone();
        controls
            .attach(move |event: MediaControlEvent| {
                let audio = handle.state::<Audio>();
                if let Some(cmd) = command_for(&event, audio.state().status) {
                    audio.send(cmd);
                }
            })
            .ok()?;
        let _ = controls.set_playback(MediaPlayback::Stopped);
        Some(Self { controls: Mutex::new(controls), last: Mutex::new(None) })
    }

    /// Perbarui judul, qari, dan status putar di OS sesuai status pemutar.
    pub fn update(&self, state: &PlayerState, surah_name: &str) {
        let shown = (title(surah_name, state.ayah), state.reciter.clone(), state.status);
        let Ok(mut last) = self.last.lock() else { return };
        if last.as_ref() == Some(&shown) {
            return;
        }
        let Ok(mut controls) = self.controls.lock() else { return };
        if state.status == "idle" {
            let _ = controls.set_playback(MediaPlayback::Stopped);
        } else {
            let _ = controls.set_metadata(MediaMetadata {
                title: Some(&shown.0),
                artist: Some(reciter_name(&state.reciter)),
                album: Some("Ziyadah"),
                ..Default::default()
            });
            let playback = if state.status == "paused" {
                MediaPlayback::Paused { progress: None }
            } else {
                MediaPlayback::Playing { progress: None }
            };
            let _ = controls.set_playback(playback);
        }
        *last = Some(shown);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn judul_ayat_dan_basmalah() {
        assert_eq!(title("Al-Mulk", 3), "Al-Mulk · ayat 3");
        assert_eq!(title("Al-Mulk", 0), "Al-Mulk · Basmalah");
    }

    #[test]
    fn tombol_media_mengikuti_status() {
        use MediaControlEvent::*;
        assert!(matches!(command_for(&Toggle, "playing"), Some(Command::Pause)));
        assert!(matches!(command_for(&Toggle, "paused"), Some(Command::Resume)));
        assert!(matches!(command_for(&Play, "paused"), Some(Command::Resume)));
        assert!(matches!(command_for(&Next, "playing"), Some(Command::Next)));
        assert!(matches!(command_for(&Previous, "paused"), Some(Command::Prev)));
        // Idle: tombol media dibiarkan untuk aplikasi lain, tidak memunculkan atau memutar apa pun.
        assert!(command_for(&Toggle, "idle").is_none());
        assert!(command_for(&Play, "idle").is_none());
        assert!(command_for(&Next, "idle").is_none());
    }
}
