//! Data Qur'an read-only dari `quran.db` (dibangun oleh scripts/build-quran-db.mjs dari Tanzil).
//!
//! Teks diambil apa adanya. Kolom `text_display` hanya memisahkan basmalah di awal ayat 1
//! supaya basmalah ditampilkan dan diputar sebagai item tersendiri; isi teks tidak diubah.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::{Connection, OpenFlags};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SurahIndexItem {
    pub number: u16,
    pub name_ar: String,
    pub name_latin: String,
    pub ayah_count: u16,
}

#[derive(Serialize)]
pub struct Ayah {
    pub n: u16,
    pub ar: String,
    pub id: String,
}

#[derive(Serialize)]
pub struct Basmalah {
    pub ar: String,
    pub id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Surah {
    pub number: u16,
    pub name_ar: String,
    pub name_latin: String,
    pub ayah_count: u16,
    /// Basmalah di awal surah persis seperti di Tanzil; `None` untuk Al-Fatihah dan At-Taubah.
    pub basmalah: Option<Basmalah>,
    pub ayahs: Vec<Ayah>,
}

pub struct Quran {
    conn: Result<Mutex<Connection>, String>,
}

impl Quran {
    pub fn open(path: &Path) -> Self {
        let conn = if path.exists() {
            Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
                .map(Mutex::new)
                .map_err(|e| format!("Gagal membuka data Qur'an: {e}"))
        } else {
            Err("Data Qur'an belum ada. Jalankan: npm run fetch-data".to_string())
        };
        Self { conn }
    }

    fn with<T>(&self, f: impl FnOnce(&Connection) -> rusqlite::Result<T>) -> Result<T, String> {
        let conn = self.conn.as_ref().map_err(|e| e.clone())?;
        let conn = conn.lock().map_err(|e| e.to_string())?;
        f(&conn).map_err(|e| e.to_string())
    }

    pub fn index(&self) -> Result<Vec<SurahIndexItem>, String> {
        self.with(|c| {
            let mut st = c.prepare("SELECT number, name_ar, name_latin, ayah_count FROM surah ORDER BY number")?;
            let rows = st.query_map([], |r| {
                Ok(SurahIndexItem {
                    number: r.get(0)?,
                    name_ar: r.get(1)?,
                    name_latin: r.get(2)?,
                    ayah_count: r.get(3)?,
                })
            })?;
            rows.collect()
        })
    }

    pub fn surah(&self, number: u16) -> Result<Surah, String> {
        self.with(|c| {
            let basmalah_id: String =
                c.query_row("SELECT value FROM meta WHERE key = 'basmalah_id'", [], |r| r.get(0))?;
            let (name_ar, name_latin, ayah_count, basmalah): (String, String, u16, Option<String>) = c.query_row(
                "SELECT name_ar, name_latin, ayah_count, basmalah FROM surah WHERE number = ?1",
                [number],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )?;
            let mut st = c.prepare(
                "SELECT a.ayah, a.text_display, t.text FROM ayah a
                 JOIN translation t ON t.lang = 'id' AND t.surah = a.surah AND t.ayah = a.ayah
                 WHERE a.surah = ?1 ORDER BY a.ayah",
            )?;
            let ayahs = st
                .query_map([number], |r| Ok(Ayah { n: r.get(0)?, ar: r.get(1)?, id: r.get(2)? }))?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            Ok(Surah {
                number,
                name_ar,
                name_latin,
                ayah_count,
                basmalah: basmalah.map(|ar| Basmalah { ar, id: basmalah_id }),
                ayahs,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    //! Tes integritas (F1-06) terhadap quran.db hasil `npm run fetch-data`.
    use super::*;
    use std::path::PathBuf;

    fn db() -> Quran {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources").join("quran.db");
        assert!(path.exists(), "quran.db belum ada. Jalankan: npm run fetch-data");
        Quran::open(&path)
    }

    #[test]
    fn jumlah_surah_dan_ayat() {
        let q = db();
        let index = q.index().unwrap();
        assert_eq!(index.len(), 114);
        assert_eq!(index.iter().map(|s| s.ayah_count as u32).sum::<u32>(), 6236);
        let total: u32 = (1..=114).map(|n| q.surah(n).unwrap().ayahs.len() as u32).sum();
        assert_eq!(total, 6236);
    }

    #[test]
    fn basmalah_dipisah_tanpa_mengubah_teks() {
        let q = db();
        q.with(|c| {
            let mut st = c.prepare("SELECT a.surah, a.text, a.text_display, s.basmalah FROM ayah a JOIN surah s ON s.number = a.surah WHERE a.ayah = 1")?;
            let rows = st.query_map([], |r| {
                Ok((r.get::<_, u16>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, Option<String>>(3)?))
            })?;
            for row in rows {
                let (surah, text, display, basmalah) = row?;
                match basmalah {
                    Some(b) => assert_eq!(format!("{b} {display}"), text, "surah {surah}"),
                    None => {
                        assert!(surah == 1 || surah == 9, "surah {surah} tanpa basmalah");
                        assert_eq!(display, text, "surah {surah}");
                    }
                }
            }
            Ok(())
        })
        .unwrap();
        assert!(q.surah(1).unwrap().basmalah.is_none());
        assert!(q.surah(9).unwrap().basmalah.is_none());
        assert!(q.surah(67).unwrap().basmalah.is_some());
    }

    #[test]
    fn ayat_selain_ayat_pertama_tidak_berubah() {
        db().with(|c| {
            let changed: u32 =
                c.query_row("SELECT count(*) FROM ayah WHERE ayah > 1 AND text <> text_display", [], |r| r.get(0))?;
            assert_eq!(changed, 0);
            Ok(())
        })
        .unwrap();
    }
}
