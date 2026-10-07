# Changelog

Semua perubahan penting dicatat di sini. Format mengikuti [Keep a Changelog](https://keepachangelog.com/id-ID/1.1.0/), versi mengikuti [Semantic Versioning](https://semver.org/lang/id/).

Setiap tugas roadmap yang selesai dicatat di "Belum dirilis" dengan ID-nya. Saat rilis, bagian itu diberi nomor versi dan tanggal.

## [Belum dirilis]

### Ditambahkan

- Lebar panel Ambient bisa diatur dengan menyeret tepi kirinya (minimum 280) dan diingat; lebar bawaan kini 380 (F1-34)
- Mode Layar Penuh bisa dibuka langsung dari panel Ambient (ikon ⛶) dan menu tray; Esc atau Keluar kembali ke panel tanpa membuka jendela utama (F1-33)
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
- Hanya satu Ziyadah yang berjalan; membuka lagi memunculkan jendela yang sudah ada (F1-19)
- Sorot per kata di panel dan jendela utama memakai timing quran-align untuk Alafasy, Al-Husary, Minshawi, dan Abdul Basit; ayat tanpa timing yang konsisten memakai sorot per ayat (F1-17)
- Panel Ambient bawaannya di pojok kanan bawah area kerja dengan lebar lebih ramping; posisi dan lebar yang dipilih pengguna diingat per monitor (F1-14)
- Ayat panjang di panel otomatis bergulir mengikuti baris yang sedang dibaca, berhenti sebentar bila pengguna menggulir sendiri; tepi memudar menggantikan scrollbar; daftar ayat di jendela utama ikut menampilkan ayat aktif (F1-27)
- Pernyataan status tashih yang jujur sejak awal: halaman pernyataan saat instal, bar status di jendela utama, dan README (F1-29)
- Pembaruan otomatis bertanda tangan dari GitHub Releases dan workflow rilis yang membuat draf rilis dari tag versi (F4-03)
- Mode Layar Penuh: ayat besar dengan sorot per kata, ayat sebelum dan sesudah, terjemah, latar bergerak pelan (Galaksi Whirlpool, Tebing Kosmik, Bumi, Bintang, Gradasi) berlisensi NASA/ESA dengan kredit, kontrol keyboard dan remote presentasi (F2-11)
- Kenyamanan Mode Layar Penuh: ayat sekitar bawaannya mati dan tidak pernah tampil terpotong, transisi ayat bergulir, ukuran huruf bisa diatur dan diingat, latar Bumi ditata ulang sebagai ufuk di bawah layar (F2-13)
- Font Arab Amiri Quran dibundel di aplikasi; tidak ada lagi permintaan ke Google Fonts dan tampilan tetap sama tanpa internet (F1-30)
- Landing page https://ziyadah.untungkasirin.com: unduhan rilis terbaru, status tashih jujur, atribusi lengkap, tanpa cookie dan analitik (F1-31)

### Diperbaiki

- Mode Layar Penuh: harakat di bawah baris terakhir (misalnya kasratain pada Al-Mulk 3) tidak lagi terpotong (F2-11)
- Mode Layar Penuh: huruf Arab tidak lagi mengecil tanpa perlu karena animasi masuk ayat ikut terhitung sebagai isi yang tidak muat (F2-11)
- Jendela utama yang ditutup ke tray tidak lagi menggambar ulang sorot kata tanpa terlihat; CPU saat murottal sekitar 1% (F1-20)
- Panel tidak lagi kosong bila pemutaran dimulai sebelum data surah selesai dimuat (F1-17)
- Versi installer tidak lagi gagal memuat data Qur'an saat dibuka ("state not managed"): jendela kini dibuat setelah audio dan data Qur'an siap
- Panel tidak lagi menempel ke taskbar: jarak ke taskbar sama dengan jarak ke sisi kanan, posisi tidak lagi tercatat keliru saat tinggi panel berubah, dan lebar bawaan 440 (F1-28)
- Mode Layar Penuh: huruf Arab tidak lagi mengecil drastis saat terjemah dimatikan (harakat bawah sempat terbaca sebagai teks yang meluap)
- Mode Layar Penuh: ayat lama dan ayat baru tidak lagi bertumpuk saat berganti; terjemah ikut transisi yang sama (F2-14)
- Pemeriksaan pembaruan tiap 1 jam, sebelumnya tiap 6 jam
- Keputusan 0004: Tanzil sebagai sumber teks Qur'an utama (F1-04)
