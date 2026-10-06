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
- [ ] **F0-05** CI GitHub Actions: build frontend, `cargo test`, build Tauri untuk Windows dan macOS di setiap pull request
- [ ] **F0-06** Amankan nama: cek merek di PDKI, ambil domain, akun/organisasi GitHub *(non-kode, paralel)*
- [ ] **F0-07** Kirim surat ke Kemenag/LPMQ: syarat pemakaian teks, terjemah, font, dan alur tashih aplikasi open source *(non-kode, paralel)*

## Fase 1: Inti + mode Ambient, v0.1 (target 29 Nov 2026)

### Spike (gerbang paling berisiko, kerjakan dulu)

- [ ] **F1-01** Uji spike di Windows sesuai checklist di `README.md`, catat hasilnya di bagian Catatan di bawah
- [ ] **F1-02** Perbaiki temuan spike Windows (fokus, transparansi, klik-tembus, drag)
- [ ] **F1-03** Uji spike di macOS (pinjam Mac bila perlu) dan perbaiki temuannya

### Data

- [ ] **F1-04** Putuskan sumber teks utama (Mushaf Standar Indonesia atau Tanzil), catat di `docs/decisions/`
- [ ] **F1-05** Script pembangun `quran.db` (SQLite): 114 surah, metadata juz/halaman/hizb, terjemah Kemenag
- [ ] **F1-06** Checksum teks dan tes integritas (jumlah ayat 6.236, basmalah, tidak ada teks berubah)
- [ ] **F1-07** Inti Rust membaca `quran.db`; frontend berhenti memakai JSON di `public/data`

### Audio

- [ ] **F1-08** Mode putar: berhenti di akhir surah, lanjut ke surah berikutnya, ulang ayat, ulang surah, rentang ayat
- [ ] **F1-09** Jeda 1 detik antar surah saat mode lanjut
- [ ] **F1-10** Kontrol media OS (tombol media keyboard/headset, kontrol media Windows, Now Playing macOS)
- [ ] **F1-11** Manajer unduhan offline per surah, per juz, dan seluruh mushaf per qari, bisa dilanjutkan
- [ ] **F1-12** Tampilan ruang disk per qari dan tombol hapus audio

### Panel Ambient

- [ ] **F1-13** Pengaturan tampilan panel: ukuran huruf, lebar, warna, transparansi, kurangi gerakan
- [ ] **F1-14** Simpan posisi dan ukuran panel per monitor (plugin window-state)
- [ ] **F1-15** Pintasan global tampil/sembunyi panel dan putar/jeda
- [ ] **F1-16** Tray: pilih surah, qari, dan mode putar langsung dari menu
- [ ] **F1-17** Sorot per kata memakai data quran-align untuk qari yang tersedia, turun ke sorot per ayat bila tidak ada
- [x] **F1-22** Panel bisa disembunyikan: tombol × di kontrol panel, panel hanya tampil saat murottal diputar dan tersembunyi saat idle (6 Okt 2026)
- [x] **F1-23** Teks Arab di panel tidak pernah terpotong: tinggi panel mengikuti isi (maks 40% layar), huruf Arab mengecil sampai 18 px bila perlu, lalu bisa di-scroll (6 Okt 2026)

### Rilis

- [ ] **F1-18** Penyimpanan preferensi pengguna (`user.db` atau file pengaturan)
- [ ] **F1-19** Satu instance saja (plugin single-instance)
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
- [ ] **F4-03** Auto-update bertanda tangan (plugin updater)
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
