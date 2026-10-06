# Catatan keputusan

Setiap keputusan teknis atau produk yang penting dan sulit dibalik dicatat sebagai satu file bernomor: `NNNN-judul-singkat.md`. Tujuannya supaya alasan di balik keputusan tidak hilang dan tidak dibahas ulang.

Kapan menulis catatan baru: memilih teknologi, sumber data, lisensi, algoritma, atau mengubah arsitektur.

Format:

```
# NNNN. Judul

Tanggal: YYYY-MM-DD
Status: diusulkan | diterima | diganti oleh NNNN

## Konteks
Masalah apa dan batasan apa yang ada.

## Keputusan
Apa yang dipilih.

## Alasan dan alternatif
Kenapa ini, apa yang ditolak dan kenapa.

## Konsekuensi
Apa yang jadi lebih mudah, apa yang jadi lebih sulit.
```

## Daftar

| No | Keputusan | Status |
| --- | --- | --- |
| 0001 | [Tauri v2 sebagai shell aplikasi](0001-tauri-v2.md) | diterima |
| 0002 | [Audio dan status pemutaran di inti Rust](0002-audio-di-inti-rust.md) | diterima |
| 0003 | [Data Qur'an tidak di-commit sampai izin dan tashih beres](0003-data-quran-tidak-di-commit.md) | diterima |
| 0004 | [Tanzil sebagai sumber teks Qur'an utama](0004-teks-tanzil.md) | diterima |
