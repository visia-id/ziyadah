# Roadmap Ziyadah

Daftar tugas yang dikerjakan bertahap. Satu tugas = satu sesi kerja atau satu commit, kira-kira setengah sampai satu hari.

**Cara pakai**

- Kerjakan tugas berurutan dalam satu fase, kecuali ditandai bisa paralel.
- Status: `[ ]` belum, `[~]` sedang dikerjakan, `[x]` selesai, `[-]` dibatalkan (tulis alasannya).
- Setelah selesai: centang, tulis tanggal, dan catat di `CHANGELOG.md` bagian "Belum dirilis".
- Tugas baru yang muncul di tengah jalan ditambahkan di fase yang relevan dengan ID berikutnya, bukan langsung dikerjakan.
- Gerbang fase harus terpenuhi sebelum mulai fase berikutnya.

Rujukan kebutuhan: `docs/PRD.md`.

---

## Fase 0: Fondasi (Oktober 2026)

- [x] **F0-01** Scaffold Tauri v2 + React + Vite + TypeScript (6 Okt 2026)
- [x] **F0-02** PRD, CLAUDE.md, roadmap, changelog, catatan keputusan (6 Okt 2026)
- [x] **F0-03** Pasang toolchain di laptop Windows (Rust, C++ Build Tools) dan `npm run tauri dev` berhasil jalan (6 Okt 2026)
- [x] **F0-04** Inisialisasi git, commit pertama, push ke repo GitHub `ziyadah` (6 Okt 2026)
- [x] **F0-05** CI GitHub Actions: build frontend, `cargo test`, build Tauri untuk Windows dan macOS di setiap pull request (6 Okt 2026)
- [ ] **F0-06** Amankan nama: cek merek di PDKI, ambil domain, akun/organisasi GitHub *(non-kode, paralel)*
- [~] **F0-07** Kirim surat ke Kemenag/LPMQ: syarat pemakaian teks, terjemah, font, dan alur tashih aplikasi open source *(non-kode, paralel)*

## Fase 1: Inti + mode Ambient, v0.1 (target 29 Nov 2026)

### Spike (gerbang paling berisiko, kerjakan dulu)

- [ ] **F1-01** Uji spike di Windows sesuai checklist di `README.md`, catat hasilnya di bagian Catatan di bawah
- [ ] **F1-02** Perbaiki temuan spike Windows (fokus, transparansi, klik-tembus, drag)
- [ ] **F1-03** Uji spike di macOS (pinjam Mac bila perlu) dan perbaiki temuannya

### Data

- [x] **F1-04** Putuskan sumber teks utama (Mushaf Standar Indonesia atau Tanzil), catat di `docs/decisions/` (6 Okt 2026: Tanzil, keputusan 0004)
- [x] **F1-05** Script pembangun `quran.db` (SQLite) dari berkas resmi Tanzil: 114 surah, metadata juz/halaman/hizb, terjemah Kemenag (6 Okt 2026)
- [x] **F1-06** Checksum teks dan tes integritas (jumlah ayat 6.236, basmalah, tidak ada teks berubah) (6 Okt 2026)
- [x] **F1-07** Inti Rust membaca `quran.db`; frontend berhenti memakai JSON di `public/data` (6 Okt 2026)

### Audio

- [x] **F1-08** Mode putar: berhenti di akhir surah, lanjut ke surah berikutnya, ulang ayat, ulang surah, rentang ayat (6 Okt 2026)
- [x] **F1-09** Jeda 1 detik antar surah saat mode lanjut (6 Okt 2026)
- [ ] **F1-10** Kontrol media OS (tombol media keyboard/headset, kontrol media Windows, Now Playing macOS)
- [ ] **F1-11** Manajer unduhan offline per surah, per juz, dan seluruh mushaf per qari, bisa dilanjutkan
- [ ] **F1-12** Tampilan ruang disk per qari dan tombol hapus audio

### Panel Ambient

- [ ] **F1-13** Pengaturan tampilan panel: ukuran huruf, lebar, warna, transparansi, kurangi gerakan
- [x] **F1-14** Simpan posisi dan ukuran panel per monitor (tanpa plugin window-state: posisi diingat relatif terhadap tepi bawah karena tinggi panel mengikuti ayat, dan plugin ikut memulihkan status tampil yang bentrok dengan F1-22) (6 Okt 2026)
- [ ] **F1-15** Pintasan global tampil/sembunyi panel dan putar/jeda
- [ ] **F1-16** Tray: pilih surah, qari, dan mode putar langsung dari menu
- [x] **F1-17** Sorot per kata memakai data quran-align untuk qari yang tersedia, turun ke sorot per ayat bila tidak ada (6 Okt 2026)
- [x] **F1-22** Panel bisa disembunyikan: tombol × di kontrol panel, panel hanya tampil saat murottal diputar dan tersembunyi saat idle (6 Okt 2026)
- [x] **F1-23** Teks Arab di panel tidak pernah terpotong: tinggi panel mengikuti isi (maks 40% layar), huruf Arab mengecil sampai 18 px bila perlu, lalu bisa di-scroll (6 Okt 2026)
- [ ] **F1-24** Opsi teks panel memudar beberapa detik setelah ayat berganti, muncul lagi saat kursor di atas panel (bawaan: mati)
- [x] **F1-25** Terjemah di panel tidak dipotong 2 baris: tinggi panel mengikuti ayat dan terjemah, bila melebihi batas keduanya bisa di-scroll (6 Okt 2026)
- [x] **F1-26** Ingat surah, qari, dan mode putar terakhir antar sesi; pertama kali dibuka mulai dari Al-Fatihah, bukan Al-Mulk (sementara di localStorage, dipindah saat F1-18) (6 Okt 2026)
- [x] **F1-27** Ayat panjang yang di-scroll: panel mengikuti baris yang sedang dibaca, berhenti saat pengguna menggulir, tepi memudar menggantikan scrollbar; jendela utama ikut menampilkan ayat aktif (6 Okt 2026)
- [x] **F1-28** Jarak panel ke taskbar sama dengan jarak ke sisi kanan (posisi sempat tercatat menembus taskbar), lebar bawaan 440 (6 Okt 2026)
- [x] **F1-29** Pernyataan status tashih yang jujur sejak awal: halaman pernyataan saat instal (PERNYATAAN.txt), bar status di jendela utama, README (6 Okt 2026)
- [x] **F1-30** Bundel font Arab di aplikasi; jangan memuat Amiri Quran dari Google Fonts (melanggar offline-first dan privasi), cek lisensi font (6 Okt 2026)
- [~] **F1-31** Landing page `website/` untuk ziyadah.untungkasirin.com: unduhan rilis terbaru, status tashih jujur, atribusi lengkap, tanpa cookie dan analitik

### Rilis

- [ ] **F1-18** Penyimpanan preferensi pengguna (`user.db` atau file pengaturan)
- [x] **F1-19** Satu instance saja (plugin single-instance) (6 Okt 2026)
- [ ] **F1-20** Ukur CPU dan memori saat memutar; optimalkan bila di atas target PRD
- [ ] **F1-21** Installer Windows dan macOS, checksum SHA-256, rilis v0.1 di GitHub Releases berlabel pratinjau

**Gerbang Fase 1:** panel transparan dan klik-tembus terbukti jalan di macOS dan Windows; satu juz penuh diputar tanpa jeda terdengar; CPU rata-rata di bawah 3%.

## Fase 2: Mode Tilawah, v0.2 (target 31 Jan 2027)

- [ ] **F2-01** Tampilan per ayat dengan terjemah dan tombol putar per ayat
- [ ] **F2-02** Tampilan per halaman mushaf (604 halaman), satu dan dua halaman
- [ ] **F2-03** Palet perintah (Ctrl/Cmd+K) untuk lompat ke surah, juz, halaman, ayat
- [ ] **F2-04** Pencarian teks Arab (dengan/tanpa harakat) dan terjemah
- [ ] **F2-05** Tema terang, gelap, sepia; ukuran huruf
- [ ] **F2-06** Terakhir dibaca dan bookmark dengan catatan
- [ ] **F2-07** Target khatam dan riwayat harian (kalender)
- [ ] **F2-08** Pengingat tilawah opsional
- [ ] **F2-09** Siapkan dan ajukan permohonan tashih LPMQ *(target 4 Jan 2027)*
- [ ] **F2-10** Rilis v0.2
- [x] **F2-11** Mode Layar Penuh tahap 1 (dimajukan ke v0.0.8): ayat besar dengan sorot per kata, latar gambar bergerak berlisensi jelas, kontrol keyboard (6 Okt 2026)
- [x] **F2-13** Kenyamanan Mode Layar Penuh: ayat sekitar bawaannya mati dan tidak pernah terpotong, transisi ayat bergulir, ukuran huruf bisa diatur (v0.0.9) (6 Okt 2026)
- [ ] **F2-12** Mode Layar Penuh tahap 2: video latar unduhan terpisah, mode masjid (huruf ekstra besar, pindah ayat manual tanpa audio)

**Gerbang Fase 2:** integritas teks lolos checksum; target khatam teruji simulasi 30 hari.

## Fase 3: Mode Hafalan, v0.3 (target 11 Apr 2027)

- [ ] **F3-01** Putuskan algoritma murajaah (SM-2 atau FSRS), catat di `docs/decisions/`
- [ ] **F3-02** Model data hafalan: item, status, jadwal review
- [ ] **F3-03** Layar "Hari ini": target hafalan baru dan murajaah jatuh tempo
- [ ] **F3-04** Loop ayat/rentang N kali dengan jeda
- [ ] **F3-05** Tingkat penutupan teks dan buka kata satu per satu
- [ ] **F3-06** Tes sambung ayat
- [ ] **F3-07** Penilaian diri (lancar, ragu, lupa) dan penjadwalan ulang
- [ ] **F3-08** Rekam suara dan putar bergantian dengan qari
- [ ] **F3-09** Peta hafalan per juz/surah dan riwayat setoran
- [ ] **F3-10** Uji coba 14 hari dengan minimal 5 penghafal; rilis v0.3

**Gerbang Fase 3:** satu siklus murajaah 14 hari diuji 5 penghafal; satu sesi hafalan baru bisa diselesaikan tanpa mouse.

## Fase 4: Pemolesan, v1.0 (target 23 Mei 2027)

- [ ] **F4-01** Aksesibilitas: navigasi keyboard penuh, label pembaca layar, kontras WCAG AA
- [ ] **F4-02** Bahasa Inggris dan struktur i18n
- [x] **F4-03** Auto-update bertanda tangan (plugin updater) (6 Okt 2026: 0.0.7 -> 0.0.8 berhasil diperbarui otomatis)
- [ ] **F4-04** Tanda tangan kode Windows dan notarisasi macOS
- [ ] **F4-05** Ekspor/impor data pengguna (JSON)
- [ ] **F4-06** Dokumentasi kontributor (CONTRIBUTING, panduan arsitektur, label good first issue)
- [ ] **F4-07** Rilis v1.0

**Gerbang Fase 4:** build bertanda tangan untuk semua target; dokumentasi kontributor lengkap.

---

## Catatan

Tulis temuan, hasil uji, dan hal yang mengganjal di sini, dengan tanggal. Yang sudah jadi keputusan dipindah ke `docs/decisions/`.

- 6 Okt 2026: data uji coba (5 surah) berhasil diambil di laptop Windows; basmalah terpisah dengan benar. `cargo` belum terpasang.
- 6 Okt 2026: toolchain Windows terpasang lewat winget (Rust 1.99.0 stable MSVC, VS 2022 Build Tools + Windows SDK). Build Rust pertama sekitar 8 menit. `cargo test` lolos 3 tes, `npm run tauri dev` membuka aplikasi.
- 6 Okt 2026: panel Ambient tampil saat idle di tengah bawah layar, menutupi input aplikasi lain, dan tidak bisa ditutup dari panel (tidak ada tombol, hanya lewat tray). Ditindaklanjuti di F1-22; pintasan global tetap di F1-15 karena tombol × tidak bisa diklik saat klik-tembus aktif.
- 6 Okt 2026: dengan terjemah aktif, baris kedua teks Arab terpotong (Al-Mulk 15) karena tinggi panel dikunci 180 px. Teks Arab juga sengaja di-clamp 3 baris, sehingga ayat panjang selalu terpotong tanpa tanda. Ditindaklanjuti di F1-23.
- 6 Okt 2026: tinjauan riset tentang Qur'an sebagai suara latar saat bekerja. Yang berdasar kuat: suara ucapan mengganggu tugas verbal (irrelevant speech effect), dan murattal bertempo stabil lebih tidak mengganggu daripada mujawwad (changing-state). Klaim EEG gelombang alfa dan penurunan kortisol berasal dari studi kecil, sebagian pada pasien, dan tidak mengukur kinerja kerja, jadi tidak dipakai sebagai klaim. Hasilnya: F1-24, Pomodoro-istirahat di daftar setelah 1.0, serta pertanyaan terbuka soal qari bawaan dan adab menyimak di PRD.
- 6 Okt 2026: tes integritas menemukan basmalah di awal surah 95 dan 97 pada teks Tanzil ditulis dengan tasydid pada ba (بِّسْمِ), berbeda dari Al-Fatihah 1. Basmalah kini disimpan per surah apa adanya, tidak disalin dari Al-Fatihah.
- 6 Okt 2026: build release pertama di laptop Windows (16 GB RAM) gagal kehabisan memori karena profil release memakai LTO dan codegen-units = 1, ditambah Chrome dan VM yang sedang berjalan. Berhasil dengan `CARGO_BUILD_JOBS=1` (installer NSIS 3,5 MB, MSI 4,9 MB). Vite juga crash karena memantau `src-tauri/target`; folder itu kini dikecualikan di vite.config.ts.
- 6 Okt 2026: logo awal (buku terbuka dengan tanda + di atasnya) diganti karena mudah terbaca sebagai salib di atas Injil atau simbol P3K. Logo baru: huruf ز (zay), huruf awal Ziyadah, sumbernya `app-icon.svg`.
- 6 Okt 2026: installer pratinjau 0.0.1 (logo lama, belum F1-19 dan F1-26) sudah terkirim ke beberapa penguji. Versi dinaikkan ke 0.0.2 supaya pembaruannya mudah dibedakan.
- 6 Okt 2026 (F1-17): indeks kata quran-align tidak menghitung basmalah dan token tanda waqaf yang berdiri sendiri; dengan aturan itu 99,8% ayat cocok persis. Ayat yang tidak konsisten (antara lain muqatta'ah di awal surah 10 sampai 15) memakai sorot per ayat. Data Sudais di rilis quran-align rusak (berisi log galat). Data Al-Husary dan Abdul Basit dari bitrate 64 kbps, durasinya sama dengan rekaman yang diputar (selisih sekitar 0,1 detik).
- 6 Okt 2026: panel crash (kosong) bila pemutaran mulai sebelum data surah selesai dimuat; diperbaiki bersama F1-17.
- 6 Okt 2026: installer 0.0.3 pertama gagal memuat data ("state not managed") karena Tauri membuat jendela sebelum setup selesai dan build release memuat halaman sangat cepat. Jendela kini dibuat di setup setelah state siap. Pelajaran: uji build release, bukan hanya `tauri dev`, sebelum installer dikirim.
