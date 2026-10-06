# Ziyadah

Aplikasi Qur'an desktop gratis dan open source (MIT): tilawah, murottal sambil kerja, dan hafalan. Tauri v2 + React/Vite/TypeScript, inti Rust. Pemilik: Untung Kasirin.

- PRD lengkap: `docs/PRD.md`. Baca dulu sebelum menambah fitur.
- Daftar tugas dan status: `docs/ROADMAP.md`. Riwayat perubahan: `CHANGELOG.md`. Keputusan: `docs/decisions/`.
- Status saat ini: spike Fase 1 (mode Ambient). Tujuannya membuktikan panel transparan, selalu di atas, klik-tembus, dan tidak mencuri fokus berjalan di Windows dan macOS. Checklist uji ada di `README.md`.
- Platform pengembangan utama: Windows 11 x64.

## Cara kerja bertahap

1. Di awal sesi, baca `docs/ROADMAP.md` dan cari tugas pertama berstatus `[ ]` atau `[~]` di fase yang sedang berjalan. Kalau pengguna menyebut ID tugas, kerjakan yang itu.
2. Kerjakan satu tugas saja per sesi. Ubah statusnya jadi `[~]` saat mulai.
3. Sebelum menyatakan selesai: `npm run build` dan `cargo test` harus lolos.
4. Setelah selesai: ubah jadi `[x]` dengan tanggal, tambahkan baris di `CHANGELOG.md` bagian "Belum dirilis" dengan ID tugas, lalu buat satu commit dengan pesan `F1-08: <ringkasan>`.
5. Pekerjaan baru yang ditemukan di tengah jalan ditulis sebagai tugas baru di roadmap, jangan langsung dikerjakan.
6. Keputusan teknis atau produk yang sulit dibalik dicatat di `docs/decisions/` (format di README di folder itu).
7. Hasil uji dan temuan dicatat di bagian Catatan pada `docs/ROADMAP.md`.

## Perintah

- `npm install`
- `npm run fetch-data`: unduh teks Tanzil, cek checksum, bangun `src-tauri/resources/quran.db` (tidak di-commit; wajib sebelum `cargo test` dan `tauri dev`)
- `npm run tauri dev`: jalankan aplikasi
- `npm run build`: cek TypeScript + build frontend
- `cd src-tauri && cargo test`: tes Rust
- Build installer lokal: `npx tauri build` butuh `TAURI_SIGNING_PRIVATE_KEY` (isi kunci privat updater, disimpan pemilik di luar repo); di laptop 16 GB pakai `CARGO_BUILD_JOBS=1`
- Rilis: samakan versi di `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, lalu push tag `vX.Y.Z`; workflow `release.yml` membuat draf rilis yang diperiksa pemilik sebelum dipublikasikan
- Sebelum installer dibagikan, uji build release-nya, bukan hanya `tauri dev`

## Aturan

- Teks Qur'an adalah amanah: jangan pernah mengubah, merapikan, atau "memperbaiki" teks ayat secara manual. Data hanya berasal dari sumber lewat script.
- Jangan commit `quran.db`, `.cache/`, atau data Qur'an lain sampai lisensi dan tashih LPMQ beres.
- Checksum di `scripts/tanzil-sources.json` hanya boleh diganti setelah perubahan teks di Tanzil ditinjau manusia.
- Audio dan status pemutaran hanya hidup di inti Rust (`src-tauri/src/audio.rs`); jendela hanya mengirim perintah dan mendengar event `player://state`.
- Di antarmuka pakai istilah "Hafalan baru" dan "Murajaah". "Ziyadah" hanya nama aplikasi.
- Bahasa antarmuka bawaan: Indonesia. Komentar kode boleh Indonesia.
- Hindari em-dash dalam teks berbahasa Indonesia.
