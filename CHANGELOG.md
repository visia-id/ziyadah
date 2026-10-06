# Changelog

Semua perubahan penting dicatat di sini. Format mengikuti [Keep a Changelog](https://keepachangelog.com/id-ID/1.1.0/), versi mengikuti [Semantic Versioning](https://semver.org/lang/id/).

Setiap tugas roadmap yang selesai dicatat di "Belum dirilis" dengan ID-nya. Saat rilis, bagian itu diberi nomor versi dan tanggal.

## [Belum dirilis]

### Ditambahkan

- Spike mode Ambient: panel transparan selalu di atas, klik-tembus, tray, murottal per ayat dari inti Rust dengan cache dan basmalah otomatis (F0-01)
- PRD, CLAUDE.md, roadmap, changelog, dan catatan keputusan (F0-02)
- Toolchain Windows terpasang (Rust 1.99, VS 2022 C++ Build Tools); `cargo test` dan `npm run tauri dev` berjalan (F0-03)
- Repo git dan push pertama ke GitHub `visia-id/ziyadah` (F0-04)
- CI GitHub Actions: build frontend, `cargo test`, dan build Tauri di Windows dan macOS untuk setiap pull request dan push ke main (F0-05)
- Panel Ambient bisa disembunyikan lewat tombol ×; panel tampil otomatis saat murottal diputar dan tersembunyi saat pemutar idle (F1-22)
- Teks Arab di panel Ambient tidak lagi terpotong: tinggi panel mengikuti isi (maks 40% layar), huruf Arab mengecil sampai 18 px lalu bisa di-scroll (F1-23)
- Mode putar: berhenti di akhir surah, lanjut ke surah berikutnya, ulang ayat, ulang surah, dan ulang rentang ayat; bisa diganti saat murottal berjalan (F1-08)
- Jeda 1 detik saat pindah surah dan saat kembali ke awal surah atau rentang (F1-09)
- Terjemah di panel tidak lagi dipotong 2 baris; ayat dan terjemah di-scroll bersama bila melebihi batas tinggi panel (F1-25)
- Data Qur'an lengkap 114 surah dari berkas resmi Tanzil (Uthmani 1.1) beserta terjemah Kemenag, juz, halaman, rub' hizb, dan sajdah, dibangun menjadi `quran.db` lewat `npm run fetch-data` (F1-05)
- Checksum teks sumber dan tes integritas: 6.236 ayat, basmalah dipisah tanpa mengubah teks (termasuk penulisan khusus di surah 95 dan 97), ayat lain sama persis dengan sumber (F1-06)
- Inti Rust membaca `quran.db`; data JSON uji coba dan script Al Quran Cloud dihapus (F1-07)
- Surah, qari, dan mode putar terakhir diingat antar sesi; pertama kali dibuka mulai dari Al-Fatihah (F1-26)
- Keputusan 0004: Tanzil sebagai sumber teks Qur'an utama (F1-04)
