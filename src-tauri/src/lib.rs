//! Inti aplikasi Ziyadah (spike mode Ambient).
//!
//! Jendela:
//! - `main`  : jendela utama (pilih surah, qari, kontrol pemutaran)
//! - `panel` : panel Ambient transparan, selalu di atas, tidak mengambil fokus.
//!   Tampil otomatis saat murottal mulai diputar dan tersembunyi saat pemutar idle.
//!
//! Aplikasi tetap hidup di tray saat jendela utama ditutup; keluar lewat menu tray.

mod audio;
mod quran;

use audio::{Audio, Command, PlayMode, PlayerState};
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use std::sync::atomic::{AtomicBool, Ordering};
use quran::{Quran, Surah, SurahIndexItem};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Listener, Manager, PhysicalPosition, PhysicalSize, State, WebviewWindow, Wry};

struct ClickThroughItem(CheckMenuItem<Wry>);

// ---------- Perintah pemutar ----------

#[tauri::command]
fn player_play(app: AppHandle, audio: State<Audio>, surah: u16, start_ayah: u16, reciter: String) {
    // Memutar dari jendela utama selalu memunculkan panel, walau sebelumnya disembunyikan.
    let _ = show_panel(&app);
    audio.send(Command::Play {
        surah,
        start_ayah,
        reciter,
    });
}

#[tauri::command]
fn player_set_mode(audio: State<Audio>, mode: PlayMode) {
    audio.send(Command::SetMode(mode));
}

#[tauri::command]
fn player_pause(audio: State<Audio>) {
    audio.send(Command::Pause);
}

#[tauri::command]
fn player_resume(audio: State<Audio>) {
    audio.send(Command::Resume);
}

#[tauri::command]
fn player_next(audio: State<Audio>) {
    audio.send(Command::Next);
}

#[tauri::command]
fn player_prev(audio: State<Audio>) {
    audio.send(Command::Prev);
}

#[tauri::command]
fn player_stop(audio: State<Audio>) {
    audio.send(Command::Stop);
}

#[tauri::command]
fn player_state(audio: State<Audio>) -> PlayerState {
    audio.state()
}

// ---------- Perintah data Qur'an ----------

#[tauri::command]
fn quran_index(quran: State<Quran>) -> Result<Vec<SurahIndexItem>, String> {
    quran.index()
}

#[tauri::command]
fn quran_surah(quran: State<Quran>, number: u16) -> Result<Surah, String> {
    quran.surah(number)
}

// ---------- Perintah panel ----------

#[tauri::command]
fn panel_toggle(app: AppHandle) -> Result<bool, String> {
    toggle_panel(&app)
}

#[tauri::command]
fn panel_hide(app: AppHandle) -> Result<(), String> {
    panel_window(&app)?.hide().map_err(|e| e.to_string())
}

/// Ubah tinggi panel mengikuti isinya (tinggi logis dari frontend), dengan tepi bawah tetap
/// di tempat supaya panel tumbuh ke atas dan tidak masuk ke bawah taskbar.
#[tauri::command]
fn panel_fit(app: AppHandle, height: f64) -> Result<(), String> {
    let panel = panel_window(&app)?;
    let scale = panel.scale_factor().map_err(|e| e.to_string())?;
    let size = panel.outer_size().map_err(|e| e.to_string())?;
    let pos = panel.outer_position().map_err(|e| e.to_string())?;
    let target = (height * scale).round() as u32;
    if target == size.height {
        return Ok(());
    }
    let bottom = pos.y + size.height as i32;
    panel
        .set_size(PhysicalSize::new(size.width, target))
        .map_err(|e| e.to_string())?;
    // Baca ulang: OS bisa membatasi ke minHeight.
    let applied = panel.outer_size().map(|s| s.height).unwrap_or(target);
    panel
        .set_position(PhysicalPosition::new(pos.x, bottom - applied as i32))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn panel_set_click_through(app: AppHandle, on: bool) -> Result<(), String> {
    set_click_through(&app, on)
}

fn panel_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    app.get_webview_window("panel")
        .ok_or_else(|| "jendela panel tidak ditemukan".to_string())
}

fn toggle_panel(app: &AppHandle) -> Result<bool, String> {
    let panel = panel_window(app)?;
    let visible = panel.is_visible().map_err(|e| e.to_string())?;
    if visible {
        panel.hide().map_err(|e| e.to_string())?;
    } else {
        show_panel(app)?;
    }
    Ok(!visible)
}

fn show_panel(app: &AppHandle) -> Result<(), String> {
    let panel = panel_window(app)?;
    panel.show().map_err(|e| e.to_string())?;
    // Pastikan tetap di atas setelah ditampilkan ulang.
    let _ = panel.set_always_on_top(true);
    Ok(())
}

/// Panel disembunyikan saat pemutar berubah dari aktif menjadi idle (berhenti atau surah selesai).
/// Panel tidak dimunculkan dari sini; itu tugas `player_play`, supaya panel yang sengaja
/// disembunyikan pengguna tidak muncul lagi saat ayat berganti.
fn should_hide_panel(was_active: bool, status: &str) -> bool {
    was_active && status == "idle"
}

fn watch_player_for_panel(app: &AppHandle) {
    let handle = app.clone();
    let was_active = AtomicBool::new(false);
    app.listen_any("player://state", move |event| {
        let Ok(state) = serde_json::from_str::<serde_json::Value>(event.payload()) else { return };
        let status = state["status"].as_str().unwrap_or("idle");
        if should_hide_panel(was_active.load(Ordering::Relaxed), status) {
            if let Ok(panel) = panel_window(&handle) {
                let _ = panel.hide();
            }
        }
        was_active.store(status != "idle", Ordering::Relaxed);
    });
}

fn set_click_through(app: &AppHandle, on: bool) -> Result<(), String> {
    let panel = panel_window(app)?;
    panel.set_ignore_cursor_events(on).map_err(|e| e.to_string())?;
    if let Some(item) = app.try_state::<ClickThroughItem>() {
        let _ = item.0.set_checked(on);
    }
    Ok(())
}

/// Letakkan panel di tengah bawah monitor utama, sedikit di atas taskbar.
fn place_panel(panel: &WebviewWindow) {
    let Ok(Some(monitor)) = panel.primary_monitor() else { return };
    let Ok(size) = panel.outer_size() else { return };
    let area = monitor.size();
    let pos = monitor.position();
    let x = pos.x + (area.width as i32 - size.width as i32) / 2;
    let y = pos.y + area.height as i32 - size.height as i32 - (96.0 * monitor.scale_factor()) as i32;
    let _ = panel.set_position(PhysicalPosition::new(x, y));
}

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

pub fn run() {
    tauri::Builder::default()
        // Harus plugin pertama: instance kedua langsung keluar dan jendela utama instance pertama dimunculkan,
        // supaya tidak ada dua tray dan murottal yang berbunyi dobel.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| show_main(app)))
        .setup(|app| {
            let handle = app.handle().clone();

            // Mesin audio dengan cache di folder cache aplikasi.
            let cache_dir = app.path().app_cache_dir()?.join("audio");
            app.manage(Audio::start(handle.clone(), cache_dir));

            // Data Qur'an dibundel sebagai resource; bila belum dibangun, perintah quran_* mengembalikan pesan galat.
            let db_path = app.path().resolve("resources/quran.db", BaseDirectory::Resource)?;
            app.manage(Quran::open(&db_path));

            if let Some(panel) = app.get_webview_window("panel") {
                place_panel(&panel);
            }
            watch_player_for_panel(&handle);

            // Menu tray / menu bar.
            let toggle = MenuItem::with_id(app, "toggle_panel", "Tampilkan/sembunyikan panel", true, None::<&str>)?;
            let click = CheckMenuItem::with_id(app, "click_through", "Klik-tembus panel", true, false, None::<&str>)?;
            let play_pause = MenuItem::with_id(app, "play_pause", "Putar/jeda", true, None::<&str>)?;
            let open = MenuItem::with_id(app, "open_main", "Buka jendela utama", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Keluar", true, None::<&str>)?;
            let sep = PredefinedMenuItem::separator(app)?;
            let menu = Menu::with_items(app, &[&play_pause, &toggle, &click, &sep, &open, &quit])?;
            app.manage(ClickThroughItem(click.clone()));

            TrayIconBuilder::with_id("ziyadah-tray")
                .icon(app.default_window_icon().cloned().expect("ikon aplikasi tidak ada"))
                .tooltip("Ziyadah")
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "toggle_panel" => {
                        let _ = toggle_panel(app);
                    }
                    "click_through" => {
                        // CheckMenuItem sudah membalik centangnya sendiri; ikuti statusnya.
                        let on = app
                            .try_state::<ClickThroughItem>()
                            .and_then(|i| i.0.is_checked().ok())
                            .unwrap_or(false);
                        let _ = set_click_through(app, on);
                    }
                    "play_pause" => {
                        let audio = app.state::<Audio>();
                        match audio.state().status {
                            "playing" => audio.send(Command::Pause),
                            "paused" => audio.send(Command::Resume),
                            _ => show_main(app),
                        }
                    }
                    "open_main" => show_main(app),
                    "quit" => {
                        app.state::<Audio>().send(Command::Stop);
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            // Menutup jendela utama hanya menyembunyikannya; murottal tetap jalan dari tray.
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            player_play,
            player_set_mode,
            player_pause,
            player_resume,
            player_next,
            player_prev,
            player_stop,
            player_state,
            quran_index,
            quran_surah,
            panel_toggle,
            panel_hide,
            panel_fit,
            panel_set_click_through,
        ])
        .run(tauri::generate_context!())
        .expect("gagal menjalankan Ziyadah");
}

#[cfg(test)]
mod tests {
    use super::should_hide_panel;

    #[test]
    fn panel_disembunyikan_hanya_saat_aktif_menjadi_idle() {
        assert!(should_hide_panel(true, "idle"));
        assert!(!should_hide_panel(false, "idle"));
        assert!(!should_hide_panel(true, "playing"));
        assert!(!should_hide_panel(true, "paused"));
        assert!(!should_hide_panel(false, "loading"));
    }
}
