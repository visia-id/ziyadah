//! Posisi panel Ambient (F1-14).
//!
//! - Bawaan: pojok kanan bawah area kerja monitor utama (di atas taskbar), lebar ramping. Bagian bawah
//!   tengah layar sengaja dihindari karena di situ biasanya kolom ketik, terminal, dan tombol kirim.
//! - Setelah digeser atau diubah lebarnya, posisi diingat per monitor dan dipulihkan saat aplikasi dibuka.
//! - Posisi disimpan relatif terhadap tepi kanan dan tepi bawah area kerja: tinggi panel berubah mengikuti
//!   panjang ayat dengan tepi bawah tetap (lihat `panel_fit`), jadi tepi bawah yang perlu diingat.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{Monitor, PhysicalPosition, PhysicalSize, Runtime, WebviewWindow, Window};

/// Lebar bawaan dan jarak ke tepi area kerja (kanan dan bawah sama), dalam piksel logis.
const DEFAULT_WIDTH: f64 = 380.0;
const DEFAULT_MARGIN: f64 = 24.0;
/// Penulisan berkas ditunda sebentar setelah perubahan terakhir supaya tidak menulis terus saat panel diseret.
/// Yang ditulis selalu posisi terbaru, jadi posisi akhir tidak pernah terlewat.
const WRITE_DELAY: Duration = Duration::from_millis(800);
/// Versi format berkas. Posisi dari versi 1 (0.0.4 dan 0.0.5) bisa tercemar keadaan antara saat tinggi panel
/// berubah, jadi diabaikan sekali dan panel kembali ke posisi bawaan.
const FORMAT_VERSION: u32 = 2;

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
    #[serde(default)]
    version: u32,
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
    saved: Arc<Mutex<Saved>>,
    write_scheduled: Arc<AtomicBool>,
}

fn write_file(path: &Path, saved: &Mutex<Saved>) {
    let json = serde_json::to_string_pretty(&*saved.lock().unwrap()).unwrap_or_default();
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let _ = fs::write(path, json);
}

impl PanelPlacement {
    pub fn load(path: PathBuf) -> Self {
        let saved = fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<Saved>(&s).ok())
            .filter(|s| s.version == FORMAT_VERSION)
            .unwrap_or_else(|| Saved { version: FORMAT_VERSION, ..Default::default() });
        Self { path, saved: Arc::new(Mutex::new(saved)), write_scheduled: Arc::new(AtomicBool::new(false)) }
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
        let mut placement = placement_from(Area::of(&monitor), pos.x, pos.y, size.width, size.height);
        // Panel yang diseret sebagian keluar layar disimpan menempel tepi, bukan di luar area kerja.
        placement.right = placement.right.max(0);
        placement.bottom = placement.bottom.max(0);
        {
            let mut saved = self.saved.lock().unwrap();
            if saved.last.as_deref() == Some(key.as_str()) && saved.monitors.get(&key) == Some(&placement) {
                return;
            }
            saved.monitors.insert(key.clone(), placement);
            saved.last = Some(key);
        }
        self.schedule_write();
    }

    /// Tulis posisi terbaru sebentar lagi; perubahan berikutnya dalam selang itu ikut tertulis.
    fn schedule_write(&self) {
        if self.write_scheduled.swap(true, Ordering::SeqCst) {
            return;
        }
        let (path, saved, scheduled) = (self.path.clone(), self.saved.clone(), self.write_scheduled.clone());
        thread::spawn(move || {
            thread::sleep(WRITE_DELAY);
            scheduled.store(false, Ordering::SeqCst);
            write_file(&path, &saved);
        });
    }

    /// Tulis sekarang juga; dipakai saat aplikasi keluar.
    pub fn save_now(&self) {
        write_file(&self.path, &self.saved);
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
        assert_eq!(p, Placement { right: 24, bottom: 24, width: 380 });
        assert_eq!(position_in(AREA, p, 150), (1920 - 24 - 380, 1032 - 24 - 150));
        // Skala 125%: jarak kanan dan bawah tetap sama.
        assert_eq!(default_placement(1.25), Placement { right: 30, bottom: 30, width: 475 });
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
    fn posisi_dari_versi_lama_diabaikan() {
        let dir = std::env::temp_dir().join(format!("ziyadah-panel-test-{}", std::process::id()));
        let path = dir.join("panel.json");
        fs::create_dir_all(&dir).unwrap();
        fs::write(&path, r#"{"last":"A","monitors":{"A":{"right":30,"bottom":-46,"width":650}}}"#).unwrap();
        let p = PanelPlacement::load(path.clone());
        assert!(p.saved.lock().unwrap().monitors.is_empty());
        p.saved.lock().unwrap().monitors.insert("A".into(), Placement { right: 30, bottom: 30, width: 550 });
        p.save_now();
        assert_eq!(PanelPlacement::load(path).saved.lock().unwrap().monitors.len(), 1);
        let _ = fs::remove_dir_all(dir);
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
