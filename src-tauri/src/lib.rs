//! Inti aplikasi Ziyadah (spike mode Ambient).
//!
//! Jendela:
//! - `main`  : jendela utama (pilih surah, qari, kontrol pemutaran)
//! - `panel` : panel Ambient transparan, selalu di atas, tidak mengambil fokus
//!
//! Aplikasi tetap hidup di tray saat jendela utama ditutup; keluar lewat menu tray.

mod audio;

use audio::{Audio, Command, PlayerState};
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, PhysicalPosition, State, WebviewWindow, Wry};

struct ClickThroughItem(CheckMenuItem<Wry>);

// ---------- Perintah pemutar ----------

#[tauri::command]
fn player_play(audio: State<Audio>, surah: u16, ayah_count: u16, start_ayah: u16, reciter: String) {
    audio.send(Command::Play {
        surah,
        ayah_count,
        start_ayah,
        reciter,
    });
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

// ---------- Perintah panel ----------

#[tauri::command]
fn panel_toggle(app: AppHandle) -> Result<bool, String> {
    toggle_panel(&app)
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
        panel.show().map_err(|e| e.to_string())?;
        // Pastikan tetap di atas setelah ditampilkan ulang.
        let _ = panel.set_always_on_top(true);
    }
    Ok(!visible)
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
        .setup(|app| {
            let handle = app.handle().clone();

            // Mesin audio dengan cache di folder cache aplikasi.
            let cache_dir = app.path().app_cache_dir()?.join("audio");
            app.manage(Audio::start(handle.clone(), cache_dir));

            if let Some(panel) = app.get_webview_window("panel") {
                place_panel(&panel);
            }

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
            player_pause,
            player_resume,
            player_next,
            player_prev,
            player_stop,
            player_state,
            panel_toggle,
            panel_set_click_through,
        ])
        .run(tauri::generate_context!())
        .expect("gagal menjalankan Ziyadah");
}
