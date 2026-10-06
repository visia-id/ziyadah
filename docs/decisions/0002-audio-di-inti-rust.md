# 0002. Audio dan status pemutaran di inti Rust

Tanggal: 2026-10-06
Status: diterima

## Konteks

Murottal harus tetap berjalan walau jendela utama ditutup dan panel disembunyikan. Jendela utama, panel Ambient, dan tray harus selalu menampilkan ayat yang sama.

## Keputusan

Audio diputar di inti Rust (crate rodio dengan decoder symphonia), bukan di elemen `<audio>` WebView. Inti Rust memegang status pemutaran dan menyiarkannya ke semua jendela lewat event `player://state`. Jendela hanya mengirim perintah (`player_play`, `player_pause`, dan seterusnya).

Satu file audio per ayat (format EveryAyah `SSSAAA.mp3`), diunduh ke cache aplikasi oleh thread terpisah. Satu ayat berikutnya selalu sudah diantrikan supaya pergantian ayat tanpa jeda.

## Alasan dan alternatif

- Audio di WebView akan terikat ke satu jendela; menutup jendela itu menghentikan suara, dan status harus disinkronkan manual antar jendela.
- Satu file per surah lebih sedikit unduhan, tetapi butuh data timing untuk tahu kapan ayat berganti dan menyulitkan loop per ayat untuk hafalan.

## Konsekuensi

- Satu sumber kebenaran untuk status pemutaran; semua jendela konsisten.
- Kontrol media OS (souvlaki) bisa dipasang langsung di inti.
- Logika antrian ada di Rust dan perlu tes unit (lihat `src-tauri/src/audio.rs`).
