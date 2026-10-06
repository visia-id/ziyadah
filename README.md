# Ziyadah

Aplikasi Qur'an desktop gratis dan open source untuk tilawah, mendengarkan murottal sambil bekerja, dan menghafal. Ziyadah berarti tambahan: harapannya pahala membaca, mendengarkan, dan menghafal Al-Qur'an terus bertambah, insya Allah.

> **Status: prototipe (spike mode Ambient).** Belum untuk diedarkan. Teks dan terjemah belum ditashih LPMQ.

## Yang sudah ada di spike ini

- Panel Ambient: jendela transparan, selalu di atas, tanpa bingkai, tidak muncul di taskbar, bisa diseret, kontrol muncul saat kursor di atas panel
- Mode klik-tembus (panel tidak menangkap klik) dari jendela utama atau menu tray
- Murottal diputar dari inti Rust (rodio), satu file per ayat dari EveryAyah, dengan cache lokal
- Basmalah otomatis sebelum ayat 1 (kecuali Al-Fatihah dan At-Taubah)
- Ayat berikutnya selalu sudah diantrikan supaya pergantian ayat tanpa jeda
- Terjemah Indonesia opsional di panel (tombol "ID")
- Menutup jendela utama tidak menghentikan murottal; aplikasi tetap hidup di tray

## Prasyarat (Windows)

1. [Node.js](https://nodejs.org) 20 atau lebih baru
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
npm run fetch-data      # ambil teks surah uji coba ke public/data (tidak di-commit)
npm run tauri dev
```

Build pertama Rust butuh beberapa menit. Build berikutnya jauh lebih cepat.

## Checklist uji spike

Ini gerbang Fase 1 di PRD. Centang setelah dicoba:

- [ ] Panel tampil transparan di atas aplikasi lain (editor, browser)
- [ ] Saat ayat berganti, fokus keyboard tidak pindah ke panel (coba sambil mengetik di editor)
- [ ] Panel bisa diseret dan kontrolnya muncul saat kursor di atasnya
- [ ] Klik-tembus aktif: klik di area panel mengenai jendela di bawahnya
- [ ] Pergantian ayat tanpa jeda yang terasa
- [ ] Menutup jendela utama, murottal tetap jalan; tray bisa jeda dan keluar
- [ ] Pemakaian CPU saat memutar (Task Manager) di bawah 3%

## Struktur

```
src/
  main/       jendela utama (React)
  panel/      panel Ambient (React)
  shared/     API pemutar dan pemuat data
src-tauri/
  src/audio.rs  mesin audio: antrian, unduhan, cache, status
  src/lib.rs    perintah, panel, tray
scripts/
  fetch-data.mjs  ambil teks dan terjemah untuk uji coba
```

## Sumber data dan lisensi

- Kode: MIT, lihat [LICENSE](LICENSE)
- Teks: [Al Quran Cloud](https://alquran.cloud) edisi `quran-uthmani` (uji coba; sumber final ditentukan bersama urusan tashih)
- Terjemah: Kementerian Agama RI, via Al Quran Cloud edisi `id.indonesian`
- Audio: [EveryAyah](https://everyayah.com), hak rekaman milik qari atau pemegang haknya

Proyek ini bukan mushaf yang sudah ditashih. Data Qur'an sengaja tidak di-commit sampai izin dan tashih selesai.
