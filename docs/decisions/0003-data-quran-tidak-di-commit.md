# 0003. Data Qur'an tidak di-commit sampai izin dan tashih beres

Tanggal: 2026-10-06
Status: diterima

## Konteks

PMA Nomor 44 Tahun 2016 mewajibkan mushaf, termasuk media digital, yang diedarkan di Indonesia memperoleh Surat Tanda Tashih atau Surat Izin Edar dari LPMQ. Terjemah Kemenag dan font mushaf juga punya syarat redistribusi yang belum dikonfirmasi. Repo direncanakan publik sejak awal.

## Keputusan

Teks, terjemah, dan font Qur'an tidak di-commit ke repo. Data dibangun lewat script (`npm run fetch-data`) menjadi `src-tauri/resources/quran.db`, yang masuk `.gitignore`. Yang di-commit hanya script dan checksum sumbernya. Kode tetap MIT dan bisa dipublikasikan.

## Alasan dan alternatif

- Mem-bundel data sekarang berarti mengedarkan teks yang belum ditashih dan mungkin melanggar syarat lisensi.
- Menunda publikasi repo sampai izin beres memperlambat pengembangan dan menutup peluang kontributor.

## Konsekuensi

- Setiap pengembang harus menjalankan `npm run fetch-data` setelah clone.
- Sebelum rilis publik pertama, keputusan ini ditinjau ulang bersama hasil konfirmasi lisensi dan tashih (tugas F0-07 dan F2-09).
