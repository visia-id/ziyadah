//! Posisi panel Ambient (F1-14).
//!
//! - Bawaan: pojok kanan bawah area kerja monitor utama (di atas taskbar), lebar ramping. Bagian bawah
//!   tengah layar sengaja dihindari karena di situ biasanya kolom ketik, terminal, dan tombol kirim.
//! - Setelah digeser atau diubah lebarnya, posisi diingat per monitor dan dipulihkan saat aplikasi dibuka.
//! - Posisi disimpan relatif terhadap tepi kanan dan tepi bawah area kerja: tinggi panel berubah mengikuti
//!   panjang ayat dengan tepi bawah tetap (lihat `panel_fit`), jadi tepi bawah yang perlu diingat.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::{Monitor, PhysicalPosition, PhysicalSize, Runtime, WebviewWindow, Window};

/// Lebar bawaan dan jarak ke tepi area kerja, dalam piksel logis.
const DEFAULT_WIDTH: f64 = 520.0;
const DEFAULT_MARGIN: f64 = 24.0;
/// Selang minimum menulis berkas saat panel sedang diseret; posisi terakhir tetap disimpan saat keluar.
const WRITE_INTERVAL: Duration = Duration::from_millis(800);

/// Jarak tepi kanan dan tepi bawah panel ke tepi area kerja, serta lebar panel. Piksel fisik.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Placement {
    pub right: i32,
    pub bottom: i32,
    pub width: u32,
}

/// Area kerja monitor: posisi kiri atas dan ukuran, piksel fisik.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Area {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Area {
    fn of(monitor: &Monitor) -> Self {
        let wa = monitor.work_area();
        Self { x: wa.position.x, y: wa.position.y, width: wa.size.width, height: wa.size.height }
    }
}

#[derive(Serialize, Deserialize, Default)]
struct Saved {
    last: Option<String>,
    monitors: HashMap<String, Placement>,
}

fn monitor_key(monitor: &Monitor) -> String {
    monitor.name().cloned().unwrap_or_else(|| "default".to_string())
}

fn default_placement(scale: f64) -> Placement {
    let margin = (DEFAULT_MARGIN * scale).round() as i32;
    Placement { right: margin, bottom: margin, width: (DEFAULT_WIDTH * scale).round() as u32 }
}

/// Posisi kiri atas panel untuk penempatan dan tinggi tertentu, selalu di dalam area kerja.
fn position_in(area: Area, placement: Placement, height: u32) -> (i32, i32) {
    let width = placement.width.min(area.width) as i32;
    let height = height.min(area.height) as i32;
    let x = area.x + area.width as i32 - placement.right - width;
    let y = area.y + area.height as i32 - placement.bottom - height;
    let x = x.clamp(area.x, area.x + area.width as i32 - width);
    let y = y.clamp(area.y, area.y + area.height as i32 - height);
    (x, y)
}

/// Kebalikan `position_in`: penempatan dari posisi dan ukuran panel saat ini.
fn placement_from(area: Area, x: i32, y: i32, width: u32, height: u32) -> Placement {
    Placement {
        right: area.x + area.width as i32 - (x + width as i32),
        bottom: area.y + area.height as i32 - (y + height as i32),
        width,
    }
}

pub struct PanelPlacement {
    path: PathBuf,
    saved: Mutex<Saved>,
    last_write: Mutex<Option<Instant>>,
}

impl PanelPlacement {
    pub fn load(path: PathBuf) -> Self {
        let saved = fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Self { path, saved: Mutex::new(saved), last_write: Mutex::new(None) }
    }

    /// Pasang panel di posisi tersimpan pada monitor terakhir bila monitor itu masih ada;
    /// selain itu di posisi bawaan pada monitor utama.
    pub fn restore<R: Runtime>(&self, panel: &WebviewWindow<R>) {
        let monitors = panel.available_monitors().unwrap_or_default();
        let saved = self.saved.lock().unwrap();
        let remembered = saved.last.as_ref().and_then(|name| {
            let monitor = monitors.iter().find(|m| &monitor_key(m) == name)?;
            Some((monitor.clone(), *saved.monitors.get(name)?))
        });
        drop(saved);
        let (monitor, placement) = match remembered {
            Some(found) => found,
            None => {
                let Ok(Some(monitor)) = panel.primary_monitor() else { return };
                let placement = default_placement(monitor.scale_factor());
                (monitor, placement)
            }
        };
        let area = Area::of(&monitor);
        let height = panel.outer_size().map(|s| s.height).unwrap_or(0);
        let width = placement.width.min(area.width);
        let _ = panel.set_size(PhysicalSize::new(width, height));
        let (x, y) = position_in(area, placement, height);
        let _ = panel.set_position(PhysicalPosition::new(x, y));
    }

    /// Catat posisi panel setelah digeser, diubah ukurannya, atau tingginya disesuaikan dengan ayat.
    pub fn remember<R: Runtime>(&self, panel: &Window<R>) {
        if !panel.is_visible().unwrap_or(false) {
            return;
        }
        let (Ok(Some(monitor)), Ok(pos), Ok(size)) = (panel.current_monitor(), panel.outer_position(), panel.outer_size())
        else {
            return;
        };
        let key = monitor_key(&monitor);
        let placement = placement_from(Area::of(&monitor), pos.x, pos.y, size.width, size.height);
        {
            let mut saved = self.saved.lock().unwrap();
            if saved.last.as_deref() == Some(key.as_str()) && saved.monitors.get(&key) == Some(&placement) {
                return;
            }
            saved.monitors.insert(key.clone(), placement);
            saved.last = Some(key);
        }
        self.save(false);
    }

    /// Tulis ke berkas. Saat panel diseret penulisan dibatasi; `force` dipakai saat aplikasi keluar.
    pub fn save(&self, force: bool) {
        let mut last = self.last_write.lock().unwrap();
        if !force && last.is_some_and(|t| t.elapsed() < WRITE_INTERVAL) {
            return;
        }
        *last = Some(Instant::now());
        let json = serde_json::to_string_pretty(&*self.saved.lock().unwrap()).unwrap_or_default();
        if let Some(dir) = self.path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        let _ = fs::write(&self.path, json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Layar 1920x1080 dengan taskbar 48 px di bawah.
    const AREA: Area = Area { x: 0, y: 0, width: 1920, height: 1032 };

    #[test]
    fn bawaan_di_pojok_kanan_bawah_area_kerja() {
        let p = default_placement(1.0);
        assert_eq!(p, Placement { right: 24, bottom: 24, width: 520 });
        assert_eq!(position_in(AREA, p, 150), (1920 - 24 - 520, 1032 - 24 - 150));
        assert_eq!(default_placement(1.25).width, 650);
    }

    #[test]
    fn tepi_bawah_tetap_saat_tinggi_berubah() {
        let p = Placement { right: 100, bottom: 40, width: 600 };
        let (_, y_pendek) = position_in(AREA, p, 120);
        let (_, y_panjang) = position_in(AREA, p, 300);
        assert_eq!(y_pendek + 120, y_panjang + 300);
    }

    #[test]
    fn simpan_lalu_pulihkan_menghasilkan_posisi_sama() {
        let p = placement_from(AREA, 300, 500, 600, 160);
        assert_eq!(position_in(AREA, p, 160), (300, 500));
    }

    #[test]
    fn panel_tidak_keluar_dari_area_kerja() {
        // Posisi tersimpan dari monitor yang lebih besar: panel ditarik masuk ke area kerja.
        let p = Placement { right: -400, bottom: -300, width: 600 };
        assert_eq!(position_in(AREA, p, 200), (1920 - 600, 1032 - 200));
        let area_kedua = Area { x: 1920, y: 0, width: 1280, height: 1024 };
        let (x, _) = position_in(area_kedua, Placement { right: 5000, bottom: 0, width: 600 }, 100);
        assert_eq!(x, 1920);
    }
}
