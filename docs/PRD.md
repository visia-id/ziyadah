# PRD Ziyadah: Aplikasi Qur'an Desktop

Versi 6 Oktober 2026 · Untung Kasirin

Usulan perubahan PRD diajukan lewat issue atau pull request di repo ini.

## Ringkasan

Ziyadah adalah aplikasi Qur'an desktop gratis dan open source untuk macOS dan Windows, dibangun dengan Tauri v2, yang menyatukan tiga kebutuhan harian Muslim pekerja dalam satu aplikasi: tilawah, mendengarkan murottal sambil bekerja, dan menghafal.

**Positioning:** Qur'an teman kerja.

**Visi:** menjadi teman Qur'an yang selalu ada di depan laptop, menemani jam kerja tanpa mengganggu pekerjaan, tanpa iklan, dan tetap berjalan tanpa internet.

**Tujuan 12 bulan pertama:**

1. Merilis versi 1.0 dengan ketiga mode berjalan stabil di macOS (Apple Silicon dan Intel) serta Windows 10/11.
2. Menjadi aplikasi Qur'an desktop rujukan bagi pengguna Indonesia, dengan terjemah dan standar mushaf yang familiar.
3. Membangun komunitas kontributor open source yang aktif sehingga proyek tidak bergantung pada satu orang.

**Makna nama.** Ziyadah berarti tambahan. Harapannya, lewat aplikasi ini pahala membaca, mendengarkan, dan menghafal Al-Qur'an terus bertambah setiap hari, insya Allah.

## Pengguna dan masalah

Pengguna utama adalah Muslim yang menghabiskan 6 jam atau lebih per hari di depan laptop. Aplikasi Qur'an yang ada hampir semuanya dirancang untuk ponsel, sehingga interaksi dengan Qur'an di jam kerja berarti berpindah perangkat dan kehilangan fokus.

| Persona | Kebutuhan utama | Masalah saat ini |
| --- | --- | --- |
| Pekerja kantoran dan profesional | Mendengar murottal sambil bekerja, sesekali melihat ayat | Membuka YouTube atau ponsel; tidak ada teks yang mengikuti bacaan; iklan dan distraksi |
| Programmer dan kreator | Ambient Qur'an saat deep work, ringan dan tidak mengganggu | Floating Ayah hanya untuk Mac Apple Silicon dan tanpa terjemah |
| Penghafal mandiri (dewasa) | Mengulang ayat, menguji hafalan, menjadwalkan murajaah | Mengulang manual, tidak ada pencatatan progres, lupa jadwal murajaah |
| Pembaca rutin | Target khatam, bookmark, terjemah Kemenag | Progres tersebar di beberapa aplikasi, tidak ada versi desktop yang nyaman |

Yang sengaja tidak menjadi target versi 1.0: anak-anak (butuh desain dan pengawasan berbeda), lembaga tahfidz yang butuh manajemen santri, dan pengguna Linux.

## Prinsip produk

Setiap keputusan fitur diuji terhadap enam prinsip berikut; fitur yang melanggar salah satunya tidak masuk.

1. **Gratis dan terbuka selamanya.** Tidak ada iklan, langganan, fitur berbayar, maupun pelacakan. Kode berlisensi MIT.
2. **Offline-first.** Semua fungsi inti berjalan tanpa internet. Internet hanya dipakai untuk mengunduh audio dan memeriksa pembaruan.
3. **Teks Qur'an diperlakukan sebagai amanah.** Teks hanya diambil dari sumber terverifikasi, tidak pernah diubah oleh aplikasi, dan setiap perubahan data teks wajib ditinjau manusia.
4. **Tidak mengganggu pekerjaan.** Mode Ambient tidak boleh mencuri fokus keyboard, tidak memunculkan notifikasi tanpa izin, dan hemat CPU. Positioning Ziyadah adalah "Qur'an teman kerja": menemani, bukan menjanjikan kerja lebih fokus atau produktif. Klaim neurosains (gelombang otak, hormon stres) tidak dipakai di aplikasi, README, maupun promosi kecuali sumbernya sudah diperiksa.
5. **Privasi penuh.** Progres, rekaman suara, dan catatan tersimpan lokal. Tidak ada akun dan tidak ada telemetri tanpa persetujuan eksplisit.
6. **Akrab bagi pengguna Indonesia.** Bahasa antarmuka Indonesia sebagai bawaan, terjemah Kemenag, dan tampilan mushaf yang familiar. Bahasa Inggris tersedia sebagai pilihan.

## Ruang lingkup

Versi 1.0 dicapai dalam empat fase; tiap fase menghasilkan rilis yang bisa dipakai sendiri. Mode Ambient dikerjakan pertama karena paling cepat jadi, paling mudah didemokan, dan memaksa lapisan inti (data, audio, sinkronisasi) selesai lebih dulu.

| Fase | Rilis | Isi utama |
| --- | --- | --- |
| 1 | v0.1 | Lapisan inti, player murottal, mode Ambient |
| 2 | v0.2 | Mode Tilawah, bookmark, target khatam |
| 3 | v0.3 | Mode Hafalan, murajaah terjadwal, rekam suara |
| 4 | v1.0 | Pemolesan, aksesibilitas, auto-update, dokumentasi kontributor |

**Di luar ruang lingkup 1.0** (dicatat untuk setelahnya):

- Koreksi bacaan dengan AI (pengenalan suara lokal)
- Sinkronisasi antarperangkat dan aplikasi pendamping ponsel
- Tafsir lengkap selain terjemah Kemenag
- Fitur kelompok atau halaqah (setoran ke ustadz, kelas)
- Build resmi Linux (Tauri mendukungnya; dibuka untuk kontribusi komunitas)
- Sesi kerja ala Pomodoro: murottal diputar penuh untuk disimak saat jeda istirahat, bukan hanya sebagai suara latar

## Lapisan inti

Ketiga mode berbagi satu lapisan inti; mode hanyalah tampilan berbeda di atas data, player, dan penyimpanan yang sama.

**Data Qur'an (dibundel, read-only)**

- Teks Rasm Utsmani 114 surah, 6.236 ayat, beserta pemecahan per kata
- Metadata: juz, hizb, halaman mushaf, ruku, sajdah, makkiyah/madaniyah
- Terjemah Kemenag bahasa Indonesia; terjemah bahasa Inggris sebagai opsi unduhan
- Pemetaan ayat ke halaman untuk tampilan mushaf

**Player audio**

- Satu file audio per ayat (format EveryAyah), sehingga loop, lompat, dan cache bisa per ayat
- Antrian ayat berikutnya dimuat sebelum ayat berjalan selesai agar tidak ada jeda
- Basmalah otomatis di awal surah 2 sampai 114, kecuali surah 9 (At-Taubah)
- Kecepatan putar 0,5x sampai 1,5x tanpa mengubah nada
- Kontrol media sistem: tombol media keyboard dan headset, Now Playing di macOS, kontrol media Windows

**Sinkronisasi teks dan audio**

- Tingkat 1, semua qari: sorot per ayat, berdasarkan pergantian file audio
- Tingkat 2, qari dengan data timing: sorot per kata memakai timestamp per kata
- Jika data timing tidak ada atau rusak untuk suatu ayat, aplikasi turun ke tingkat 1, tidak menebak

**Manajer unduhan**

- Streaming bawaan, unduh offline per surah, per juz, atau seluruh mushaf per qari
- Unduhan bisa dilanjutkan setelah terputus; setiap file diverifikasi ukurannya
- Tampilkan ruang disk yang dipakai per qari dan opsi hapus

**Penyimpanan lokal**

- SQLite untuk data pengguna: bookmark, riwayat baca, progres hafalan, jadwal murajaah, preferensi
- Ekspor dan impor seluruh data pengguna ke satu file JSON sebagai cadangan manual

## Mode Ambient (Fase 1)

Panel transparan yang melayang di atas semua jendela dan menampilkan ayat yang sedang dibaca qari, seperti lirik lagu, tanpa mengambil fokus dari pekerjaan.

**Kebutuhan fungsional**

- Jendela tanpa bingkai, latar transparan, selalu di atas, dan tidak mengambil fokus keyboard saat muncul
- Menampilkan sekitar 3 baris teks Arab; ayat aktif disorot, kata aktif diberi penanda bila data timing tersedia
- Opsi satu baris terjemah Kemenag di bawah teks Arab (bawaan: mati)
- Seret panel ke mana saja; posisi dan ukuran diingat per monitor
- Arahkan kursor untuk memunculkan kontrol (putar/jeda, sebelumnya, berikutnya, ulang); kontrol hilang lagi saat kursor pergi
- Mode klik-tembus: panel tidak menangkap klik sehingga jendela di bawahnya tetap bisa dipakai
- Ikon di menu bar (macOS) atau system tray (Windows) dengan menu: pilih surah, qari, mode putar, tampilkan/sembunyikan panel
- Mode putar: terus berlanjut, berhenti di akhir surah, ulang ayat, ulang surah, putar rentang ayat
- Pintasan global yang bisa diatur, misalnya untuk menampilkan/menyembunyikan panel
- Sembunyi otomatis saat aplikasi lain masuk layar penuh atau saat presentasi (opsional)

**Kustomisasi tampilan**

- Ukuran huruf 18 sampai 48 pt, lebar panel 280 sampai 900 pt
- Warna teks, bayangan, dan tingkat transparansi latar
- Mode "kurangi gerakan" untuk mematikan animasi gulir
- Opsi teks memudar beberapa detik setelah ayat berganti dan muncul lagi saat kursor di atas panel (bawaan: mati), untuk pengguna yang hanya ingin mendengar dan sesekali melirik

**Kriteria diterima**

- [ ] Panel tidak pernah mencuri fokus dari editor teks atau aplikasi lain saat ayat berganti
- [ ] Pergantian ayat tanpa jeda yang terdengar lebih dari 150 ms (di luar jeda antar surah)
- [ ] Pemakaian CPU rata-rata di bawah 3% saat memutar dengan panel terlihat

## Mode Tilawah (Fase 2)

Jendela utama untuk membaca Qur'an dengan nyaman di layar besar, dengan progres bacaan yang tercatat otomatis.

**Tampilan baca**

- Tampilan per halaman mengikuti tata letak mushaf standar (604 halaman), satu atau dua halaman berdampingan
- Tampilan per ayat: teks Arab, terjemah di bawahnya, nomor ayat, tombol putar per ayat
- Navigasi cepat ke surah, juz, halaman, atau ayat lewat palet perintah (Ctrl/Cmd+K)
- Pencarian teks Arab (dengan atau tanpa harakat) dan pencarian dalam terjemah
- Tema terang, gelap, dan sepia; ukuran huruf bisa diatur
- Pewarnaan tajwid sebagai opsi (masuk bila sumber data tajwid berlisensi terbuka tersedia)

**Progres dan kebiasaan**

- Penanda "terakhir dibaca" yang diperbarui otomatis
- Bookmark tanpa batas, dengan catatan pribadi per ayat
- Target khatam: pengguna memilih durasi (misalnya 30 hari) dan aplikasi menghitung target harian dalam halaman
- Riwayat harian: halaman dibaca dan menit mendengarkan, dalam bentuk kalender
- Pengingat tilawah opsional pada jam yang dipilih pengguna

**Integrasi dengan mode lain**

- Satu klik dari ayat mana pun untuk memutar murottal mulai dari ayat itu di mode Ambient
- Satu klik untuk menambahkan rentang ayat ke daftar hafalan

**Kriteria diterima**

- [ ] Pindah ke halaman mana pun di bawah 200 ms
- [ ] Target khatam menyesuaikan otomatis bila pengguna tertinggal atau lebih cepat

## Mode Layar Penuh (dimajukan ke v0.0.8)

Tampilan layar penuh ala video murottal: ayat besar di tengah dengan sorot per kata, terjemah di bawahnya, dan latar gambar yang bergerak pelan. Pengalaman yang biasa dicari orang di YouTube, tanpa iklan, tanpa rekomendasi, dan tetap jalan tanpa internet.

**Situasi pemakaian**

1. Jeda kerja: menyimak penuh beberapa menit (pasangan dari panel Ambient saat bekerja)
2. Tilawah santai, terutama malam hari dan Ramadan
3. Masjid, majelis, dan kajian: laptop disambung ke TV atau proyektor
4. Layar saat komputer menganggur (seperti screensaver)

**Kebutuhan tahap 1 (v0.0.8)**

- Masuk dari jendela utama atau tombol F11; keluar dengan Esc
- Ayat aktif besar di tengah, ayat sebelum dan sesudah samar, terjemah opsional, nama surah dan nomor ayat kecil
- Kontrol hanya muncul saat mouse bergerak; Spasi untuk jeda/lanjut, panah untuk pindah ayat
- Latar bawaan: foto langit dan alam yang bergerak pelan (zoom, geser, galaksi berputar), bintang berkelip yang digambar aplikasi, dan gradasi; pilihan latar diingat
- Teks selalu di atas lapisan gelap atau blur, kontras memenuhi WCAG AA

**Aturan latar**

- Tanpa gambar makhluk bernyawa; tema langit, luar angkasa, alam, dan gradasi
- Lisensi wajib jelas: domain publik (misalnya NASA) atau lisensi terbuka dengan atribusi (misalnya ESA, CC BY 4.0); kredit tiap gambar dicantumkan di aplikasi dan README
- Gambar tidak dipasangkan dengan ayat tertentu sebagai "bukti ilmiah"; latar hanya suasana
- Tahap berikutnya: video latar diunduh terpisah (misalnya timelapse Bumi dari ISS), mode masjid dengan huruf ekstra besar dan pindah ayat manual tanpa audio

## Mode Hafalan (Fase 3)

Mode Hafalan adalah pembeda utama aplikasi: ia memisahkan hafalan baru dari murajaah (pengulangan) dan menjadwalkan murajaah otomatis supaya hafalan lama tidak hilang. Di antarmuka, istilah yang dipakai adalah "Hafalan baru" dan "Murajaah"; kata Ziyadah hanya dipakai sebagai nama aplikasi agar tidak ambigu.

**Alur harian pengguna**

1. Buka "Hari ini": aplikasi menampilkan target hafalan baru hari ini dan daftar murajaah yang jatuh tempo.
2. Hafalan baru: dengar, ulang, lalu uji ayat baru dengan teks yang makin lama makin tertutup.
3. Murajaah: uji ayat yang jatuh tempo, lalu beri nilai diri sendiri (lancar, ragu, lupa).
4. Aplikasi menghitung ulang jadwal murajaah berdasarkan nilai tersebut.

**Alat bantu menghafal**

- Loop ayat atau rentang ayat N kali, dengan jeda antarulangan yang bisa diatur
- Tingkat penutupan teks: tampil penuh, huruf awal tiap kata, kata pertama ayat saja, tertutup penuh
- Klik atau tekan spasi untuk membuka kata berikutnya satu per satu
- Tes sambung ayat: aplikasi menampilkan akhir ayat sebelumnya, pengguna melanjutkan dari hafalan
- Rekam suara sendiri lalu putar bergantian dengan bacaan qari untuk membandingkan

**Murajaah terjadwal**

- Satuan hafalan bisa per ayat, per halaman, atau per rentang yang ditentukan pengguna
- Penjadwalan memakai algoritma spaced repetition (misalnya SM-2 atau FSRS); nilai "lupa" mengembalikan interval ke awal
- Batas beban harian bisa diatur agar murajaah tidak menumpuk setelah beberapa hari terlewat

**Progres**

- Peta hafalan per juz dan per surah dengan status: belum, sedang dihafal, hafal, perlu dikuatkan
- Riwayat setoran harian dan jumlah ayat hafal per minggu

**Kriteria diterima**

- [ ] Pengguna bisa menyelesaikan satu sesi hafalan baru tanpa menyentuh mouse (seluruhnya via keyboard)
- [ ] Rekaman suara tersimpan lokal dan bisa dihapus per sesi atau seluruhnya

## Arsitektur teknis

Semua jendela berbagi satu inti Rust: audio, data, dan jadwal hanya hidup di sana, sehingga panel Ambient dan jendela utama selalu menampilkan status yang sama dan murottal tetap berjalan saat semua jendela disembunyikan.

```
Antarmuka: React + Vite di WebView
  [Jendela utama]      [Panel Ambient]        [Menu bar / system tray]
  Tilawah, Hafalan,    transparan, selalu     kontrol cepat,
  pengaturan, unduhan  di atas, klik-tembus   tampil/sembunyikan panel
        |  perintah (invoke) ke bawah, event status pemutaran ke atas  |
Inti aplikasi: Rust (Tauri v2)
  [Mesin audio]          [Sinkronisasi teks]       [Penjadwal murajaah]
  antrian per ayat,      sorot per ayat, per kata  spaced repetition,
  basmalah, media OS     bila data timing ada      beban harian
  [Data Qur'an SQLite]   [Data pengguna SQLite]    [Unduhan dan pembaruan]
  read-only, dibundel    bookmark, progres,        audio offline,
                         rekaman, ekspor JSON      pembaruan aplikasi
Layanan luar (perlu internet): EveryAyah (audio per ayat), GitHub Releases (pembaruan bertanda tangan)
```

Jendela hanya menampilkan dan mengirim perintah; inti Rust memegang status pemutaran dan menyiarkannya lewat event ke semua jendela sekaligus.

| Lapisan | Pilihan (kandidat) | Alasan |
| --- | --- | --- |
| Shell aplikasi | Tauri v2 | Installer kecil, multi-jendela, transparansi, tray bawaan |
| Antarmuka | React + Vite + TypeScript, Tailwind | Sesuai stack yang sudah dikuasai |
| State antarmuka | Zustand | Ringan, cukup untuk status per jendela |
| Audio | Crate rodio + symphonia di Rust | Audio di inti, bukan di WebView, agar tidak terikat satu jendela |
| Kontrol media OS | Crate souvlaki (Now Playing macOS, kontrol media Windows) | Belum ada plugin resmi Tauri untuk ini |
| Database | SQLite lewat rusqlite atau tauri-plugin-sql | Lokal, satu file, mudah dicadangkan |
| Plugin Tauri | global-shortcut, updater, single-instance, window-state | Pintasan global, auto-update, satu instance, posisi jendela diingat |

**Model data utama**

- `quran.db` (read-only, dibundel): surah, ayah (nomor, halaman, juz, hizb, teks), word (pecahan kata per ayat), translation, reciter, timing (qari, ayat, posisi kata, mulai dan selesai dalam milidetik)
- `user.db` (milik pengguna): bookmark, reading_log (tanggal, halaman, menit mendengar), khatam_plan, hifz_item (rentang ayat dan status), review (jatuh tempo, interval, nilai terakhir), recording (lokasi file), settings
- Audio offline disimpan di folder data aplikasi per qari, dengan nama file format EveryAyah (`SSSAAA.mp3`)

## Sumber data, lisensi, dan tashih

Kode aplikasi berlisensi MIT, tetapi setiap aset data punya syarat sendiri dan tidak ikut berlisensi MIT. Inventaris di bawah wajib dikonfirmasi sebelum rilis publik pertama.

| Aset | Kandidat sumber | Catatan lisensi dan risiko |
| --- | --- | --- |
| Teks Rasm Utsmani | Tanzil edisi Uthmani, dari berkas resmi tanzil.net (keputusan [0004](decisions/0004-teks-tanzil.md)) | Boleh disalin verbatim dengan menyebut Tanzil Project dan menautkan ke tanzil.net; teks tidak boleh diubah. Tashih LPMQ tetap wajib sebelum diedarkan luas |
| Terjemah Indonesia | Terjemah Kemenag | Izin redistribusi dalam aplikasi pihak ketiga perlu dikonfirmasi tertulis |
| Font mushaf | Font Uthmani gaya Madinah (misalnya KFGQPC), mengikuti teks Tanzil | Syarat redistribusi masing-masing font perlu dibaca sebelum dibundel |
| Audio murottal | EveryAyah (per ayat) | Hak atas rekaman tetap milik qari atau pemegang hak; aplikasi hanya mengunduh, tidak mengklaim |
| Data uji coba (sementara) | Al Quran Cloud: edisi `quran-uthmani` (bersumber dari Tanzil) dan `id.indonesian` (Kemenag) | Hanya untuk prototipe lewat `npm run fetch-data`, tidak di-commit dan tidak dibundel. Diganti sumber final setelah keputusan F1-04 |
| Timing per kata | quran-align (Colin Fair) | CC BY 4.0, wajib atribusi; hanya mencakup sebagian qari dan ada ayat yang meleset |
| Data tajwid | Belum ditentukan | Masuk hanya bila ada sumber berlisensi terbuka |

**Tashih LPMQ.** [PMA Nomor 44 Tahun 2016](https://www.peraturan.go.id/files/bn1605-2016.pdf) mendefinisikan mushaf termasuk media digital, dan mewajibkan setiap mushaf yang diterbitkan atau diedarkan di Indonesia memperoleh Surat Tanda Tashih atau Surat Izin Edar. Permohonan diajukan penerbit ke Kepala LPMQ dengan menyerahkan copy master mushaf. Konsekuensinya untuk proyek ini:

- Memakai teks dari Mushaf Standar Indonesia yang sudah bertashih adalah jalur paling aman
- Permohonan tashih diajukan sebelum v1.0 diedarkan luas, dengan pemohon yang ditetapkan di bagian pertanyaan terbuka
- Teks dikunci dengan checksum di repositori; pull request yang mengubah data teks wajib ditinjau minimal dua pemelihara
- Pada aplikasi dan README dicantumkan status tashih secara jujur, termasuk bila masih dalam proses

## Kebutuhan non-fungsional

| Aspek | Target |
| --- | --- |
| Platform | macOS 12+ (Apple Silicon dan Intel), Windows 10 dan 11 (x64) |
| Ukuran installer | Di bawah 40 MB tanpa audio; audio diunduh terpisah |
| Memori | Di bawah 200 MB saat memutar dengan panel Ambient terlihat |
| CPU | Rata-rata di bawah 3% saat memutar di mode Ambient |
| Waktu buka | Jendela utama siap dalam 2 detik pada laptop kelas menengah |
| Offline | Seluruh fitur inti berjalan tanpa internet setelah audio diunduh |
| Keamanan rilis | Build macOS ditandatangani Developer ID dan dinotarisasi; build Windows ditandatangani code signing; checksum SHA-256 untuk setiap rilis |
| Pembaruan | Auto-update lewat plugin updater Tauri dengan tanda tangan rilis; pengguna bisa menonaktifkan |
| Aksesibilitas | Seluruh fungsi bisa via keyboard; label pembaca layar; kontras teks memenuhi WCAG AA |
| Bahasa antarmuka | Indonesia (bawaan) dan Inggris; struktur i18n siap ditambah bahasa lain oleh kontributor |
| Kualitas kode | Uji otomatis untuk antrian audio, basmalah, penjadwalan murajaah, dan integritas teks; CI menjalankan build ketiga target di setiap pull request |

## Metrik keberhasilan

Karena aplikasi tidak memakai telemetri, metrik diambil dari sumber publik: unduhan rilis GitHub, aktivitas repositori, dan masukan pengguna.

| Metrik | Target 6 bulan setelah v0.1 | Target 12 bulan |
| --- | --- | --- |
| Unduhan rilis (GitHub Releases) | 5.000 | 25.000 |
| Bintang repositori GitHub | 500 | 2.000 |
| Kontributor yang pull request-nya digabung | 5 | 20 |
| Laporan bug kritis terbuka lebih dari 14 hari | 0 | 0 |
| Laporan kesalahan teks Qur'an | 0 terkonfirmasi | 0 terkonfirmasi |

Angka target adalah usulan awal untuk didiskusikan, bukan hasil riset pasar. Telemetri anonim opsional (opt-in) bisa dipertimbangkan setelah v1.0 bila komunitas setuju.

## Risiko dan pertanyaan terbuka

| Risiko | Dampak | Mitigasi |
| --- | --- | --- |
| Kesalahan teks Qur'an lolos ke rilis | Sangat tinggi: merusak kepercayaan dan melanggar amanah | Teks dari sumber bertashih, checksum di CI, tinjauan dua pemelihara, tombol lapor kesalahan di setiap ayat |
| Tashih LPMQ butuh waktu lama | Peredaran luas tertunda | Ajukan sejak Fase 2; rilis awal diberi label pratinjau dengan status tashih yang jujur |
| Lisensi terjemah atau font tidak mengizinkan redistribusi | Fitur harus diganti | Konfirmasi tertulis sebelum dibundel; siapkan sumber alternatif |
| Biaya tanda tangan kode (Apple Developer dan sertifikat Windows) | Tanpa itu muncul peringatan keamanan saat instal | Didanai pemelihara atau donasi terbuka; sementara itu sediakan checksum dan panduan instal |
| Panel transparan dan klik-tembus berperilaku berbeda di macOS dan Windows | Pengalaman mode Ambient tidak konsisten | Prototipe panel di kedua OS pada minggu pertama Fase 1, sebelum fitur lain |
| Server EveryAyah lambat atau tidak tersedia | Streaming gagal | Cache agresif, dorong unduhan offline, siapkan mirror alternatif |
| Proyek bergantung pada satu pemelihara | Pengembangan berhenti | Dokumentasi kontributor, label good first issue, minimal dua pemelihara aktif sebelum v1.0 |

**Pertanyaan terbuka**

- [ ] Domain, logo, dan identitas visual Ziyadah
- [ ] Siapa pemohon tashih: pribadi, VIC, atau yayasan khusus untuk proyek ini?
- [x] Pakai teks Mushaf Standar Indonesia atau Tanzil sebagai sumber utama? Tanzil (keputusan 0004)
- [x] Daftar qari bawaan untuk v0.1: Alafasy, Al-Husary, Minshawi, Abdul Basit (semuanya murattal, semuanya punya timing per kata). As-Sudais dikeluarkan karena temponya cepat dan data timing per katanya rusak di sumber. Bawaan saat pertama dibuka masih Alafasy; usulan Al-Husary sebagai bawaan belum diputuskan
- [ ] Algoritma murajaah: SM-2 (sederhana) atau FSRS (lebih akurat, lebih kompleks)?
- [ ] Apakah donasi diterima, dan lewat kanal apa?
- [ ] Sikap aplikasi soal adab menyimak (QS. Al-A'raf: 204): Qur'an yang diputar sebagai suara latar saat bekerja. Perlu ditanyakan ke ustadz atau dewan syariah, lalu dituangkan di halaman Tentang

## Roadmap dan estimasi waktu

Dengan pengerjaan paruh waktu, v1.0 diperkirakan rilis akhir Mei 2027. Urutan fase sengaja menaruh rilis Tilawah (target khatam) tepat sebelum Ramadan, momen ketika kebutuhan tilawah dan unduhan paling tinggi.

| Tahap | Mulai | Selesai |
| --- | --- | --- |
| Fase 1: inti + Ambient (v0.1) | 12 Okt 2026 | 29 Nov 2026 |
| Fase 2: Tilawah (v0.2) | 30 Nov 2026 | 31 Jan 2027 |
| Ajukan tashih LPMQ | 4 Jan 2027 | 4 Jan 2027 |
| Ramadan 1448 H (perkiraan) | 8 Feb 2027 | 9 Mar 2027 |
| Fase 3: Hafalan (v0.3) | 1 Feb 2027 | 11 Apr 2027 |
| Fase 4: pemolesan | 12 Apr 2027 | 23 Mei 2027 |
| Rilis v1.0 | 23 Mei 2027 | 23 Mei 2027 |

Tashih diajukan saat teks dan tampilan mushaf Fase 2 sudah stabil, agar copy master yang diserahkan sama dengan yang akan diedarkan.

**Gerbang tiap fase** (fase berikutnya dimulai hanya bila gerbang terpenuhi)

- [ ] Fase 1: panel transparan dan klik-tembus terbukti jalan di macOS dan Windows pada minggu pertama; pemutaran satu juz penuh tanpa jeda terdengar
- [ ] Fase 2: integritas teks lolos checksum; target khatam teruji 30 hari simulasi
- [ ] Fase 3: satu siklus murajaah 14 hari diuji minimal 5 penghafal dari komunitas
- [ ] Fase 4: build bertanda tangan untuk ketiga target, dokumentasi kontributor lengkap

Semua tanggal adalah estimasi awal dan akan disesuaikan setelah prototipe Fase 1.
