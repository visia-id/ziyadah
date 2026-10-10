# Ziyadah

Aplikasi Qur'an desktop gratis dan open source untuk tilawah, mendengarkan murottal sambil bekerja, dan menghafal. Ziyadah berarti tambahan: harapannya pahala membaca, mendengarkan, dan menghafal Al-Qur'an terus bertambah, insya Allah.

> **Status: uji coba (beta), belum ditashih LPMQ.** Teks Al-Qur'an diambil apa adanya dari Tanzil Project (Uthmani 1.1, mengikuti Mushaf Madinah) dan tidak diubah; sebagian tanda baca dan waqaf bisa berbeda dari Mushaf Standar Indonesia. Kami sedang mengurus komunikasi dengan LPMQ tentang proses tashih. Pernyataan lengkap: [PERNYATAAN.txt](PERNYATAAN.txt). Laporkan kesalahan lewat [Issues](https://github.com/visia-id/ziyadah/issues).

## Yang sudah ada di spike ini

- Panel Ambient: jendela transparan, selalu di atas, tanpa bingkai, tidak muncul di taskbar, bisa diseret, kontrol muncul saat kursor di atas panel
- Mode klik-tembus (panel tidak menangkap klik) dari jendela utama atau menu tray
- Murottal diputar dari inti Rust (rodio), satu file per ayat dari EveryAyah, dengan cache lokal
- Basmalah otomatis sebelum ayat 1 (kecuali Al-Fatihah dan At-Taubah)
- Ayat berikutnya selalu sudah diantrikan supaya pergantian ayat tanpa jeda
- Terjemah Indonesia opsional di panel (tombol "ID")
- Panel tampil saat murottal diputar dan bisa disembunyikan dengan tombol ×; tinggi panel mengikuti panjang ayat sehingga teks Arab tidak terpotong
- Menutup jendela utama tidak menghentikan murottal; aplikasi tetap hidup di tray

## Prasyarat (Windows)

1. [Node.js](https://nodejs.org) 22.13 atau lebih baru (memakai `node:sqlite` bawaan)
2. [Rust](https://rustup.rs) (toolchain MSVC)
3. [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) dengan workload **Desktop development with C++**
4. WebView2 (sudah ada di Windows 10/11 yang ter-update)

Cek cepat:

```powershell
node -v
cargo -V
```

Untuk macOS: Xcode Command Line Tools (`xcode-select --install`) dan Rust.

## Menjalankan

```powershell
npm install
npm run fetch-data      # unduh teks Tanzil, cek checksum, bangun src-tauri/resources/quran.db (tidak di-commit)
npm run tauri dev
```

Build pertama Rust butuh beberapa menit. Build berikutnya jauh lebih cepat.

## Checklist uji spike

Ini gerbang Fase 1 di PRD. Windows 11 lolos semua (10 Okt 2026, F1-01); macOS belum (F1-03). Poin 1, 2, dan 4 bisa diuji otomatis: jalankan Ziyadah dengan `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9334`, putar murottal, lalu `powershell -STA -File scripts/uji-spike.ps1`. Jangan memakai keyboard dan mouse selama uji.

- [x] Panel tampil transparan di atas aplikasi lain (editor, browser)
- [x] Saat ayat berganti, fokus keyboard tidak pindah ke panel (coba sambil mengetik di editor)
- [x] Panel bisa diseret dan kontrolnya muncul saat kursor di atasnya
- [x] Klik-tembus aktif: klik di area panel mengenai jendela di bawahnya
- [x] Pergantian ayat tanpa jeda yang terasa
- [x] Menutup jendela utama, murottal tetap jalan; tray bisa jeda dan keluar
- [x] Pemakaian CPU saat memutar (Task Manager) di bawah 3%

## Rilis dan pembaruan otomatis

Aplikasi memeriksa pembaruan dari [GitHub Releases](https://github.com/visia-id/ziyadah/releases) dan hanya memasang rilis yang ditandatangani kunci updater milik pemelihara.

1. Samakan versi di `package.json`, `src-tauri/Cargo.toml`, dan `src-tauri/tauri.conf.json`.
2. Commit, lalu `git tag vX.Y.Z` dan `git push origin vX.Y.Z`.
3. Workflow `release.yml` membangun installer Windows bertanda tangan, menambahkan checksum SHA-256 (`SHA256SUMS.txt` dan di catatan rilis), lalu membuat draf rilis. Periksa, lalu publikasikan.

Membangun installer di komputer sendiri membutuhkan variabel `TAURI_SIGNING_PRIVATE_KEY` berisi kunci privat updater.

## Situs

Landing page statis ada di folder `website/` (HTML, CSS, sedikit JS, tanpa build), untuk `ziyadah.untungkasirin.com`. Tombol unduh otomatis menunjuk ke installer rilis terbaru lewat API GitHub. Situs tanpa cookie; kunjungan dihitung Cloudflare Web Analytics tanpa data pribadi. Semua font di-host sendiri.

## Struktur

```
src/
  main/       jendela utama (React)
  panel/      panel Ambient (React)
  shared/     API pemutar dan pemuat data
src-tauri/
  src/audio.rs  mesin audio: mode putar, antrian, unduhan, cache, status
  src/quran.rs  membaca quran.db, tes integritas teks
  src/lib.rs    perintah, panel, tray
scripts/
  build-quran-db.mjs   bangun quran.db dari berkas resmi Tanzil
  tanzil-sources.json  alamat sumber dan checksum teks
```

## Sumber data dan lisensi

- Kode: MIT, lihat [LICENSE](LICENSE)
- Teks: [Tanzil Project](https://tanzil.net), Tanzil Quran Text (Uthmani, versi 1.1). Disalin verbatim tanpa perubahan sesuai syarat Tanzil; lihat [keputusan 0004](docs/decisions/0004-teks-tanzil.md)
- Metadata juz, halaman, rub' hizb, sajdah: Tanzil `quran-data.xml`
- Terjemah: Kementerian Agama RI, via Tanzil (`id.indonesian`)
- Timing per kata untuk sorot kata: [quran-align](https://github.com/cpfair/quran-align) oleh Collin Fair, [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/)
- Latar Mode Layar Penuh: foto NASA, ESA, CSA, STScI (domain publik dan CC BY 4.0); rincian di [public/backgrounds/KREDIT.md](public/backgrounds/KREDIT.md)
- Font Arab: [Amiri Quran](https://github.com/aliftype/amiri), SIL Open Font License 1.1, dibundel di `public/fonts`
- Audio: [EveryAyah](https://everyayah.com), hak rekaman milik qari atau pemegang haknya

Proyek ini bukan mushaf yang sudah ditashih. Data Qur'an sengaja tidak di-commit sampai izin dan tashih selesai.
