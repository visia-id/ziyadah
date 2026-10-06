# 0004. Tanzil sebagai sumber teks Qur'an utama

Tanggal: 2026-10-06
Status: diterima

## Konteks

Ziyadah butuh satu sumber teks Rasm Utsmani yang terverifikasi untuk dibangun menjadi `quran.db` (F1-05). Dua kandidat: Mushaf Standar Indonesia (Kemenag, dikelola LPMQ) dan Tanzil (mengikuti Mushaf Madinah). Keduanya riwayat Hafs 'an 'Ashim dengan 6.236 ayat dan penomoran yang sama.

Syarat pemakaian data Mushaf Standar Indonesia di aplikasi pihak ketiga belum diketahui (F0-07). Data timing per kata (quran-align) dan audio per ayat (EveryAyah) disusun mengikuti teks Madinah.

## Keputusan

Memakai teks Tanzil edisi Uthmani sebagai sumber teks utama. `quran.db` dibangun dari berkas resmi unduhan tanzil.net, bukan dari API pihak ketiga.

## Alasan dan alternatif

- **Tanzil:** syarat pemakaian jelas (boleh disalin dan diedarkan secara verbatim, wajib menyebut Tanzil Project dan menautkan ke tanzil.net, teks tidak boleh diubah). Pemecahan kata cocok dengan data timing per kata dan audio EveryAyah, sehingga sorot per kata (F1-17) tidak perlu pemetaan ulang. Sudah dipakai luas dan teruji.
- **Mushaf Standar Indonesia:** lebih akrab bagi pembaca Indonesia dan paling aman untuk tashih LPMQ, tetapi izin pemakaian datanya belum ada, data digitalnya lebih terbatas, dan sorot per kata butuh pemetaan kata ke data timing berbasis Madinah. Ditolak untuk saat ini.

## Konsekuensi

- Pengembangan F1-05, F1-06, dan F1-17 bisa jalan tanpa menunggu jawaban Kemenag.
- Tampilan tanda baca mengikuti gaya Madinah (bentuk sukun, tanda mad, tanda waqaf), sedikit berbeda dari mushaf yang biasa dibaca di Indonesia.
- Aplikasi dan README wajib mencantumkan atribusi Tanzil Project beserta tautan ke tanzil.net, dan header lisensi dalam berkas Tanzil dipertahankan.
- Teks tidak boleh diubah sama sekali, termasuk "dirapikan". Ini sejalan dengan aturan amanah di CLAUDE.md.
- Tashih LPMQ tetap wajib sebelum diedarkan luas. Surat F0-07 perlu menanyakan apakah LPMQ mentashih aplikasi dengan teks Madinah. Keputusan ini ditinjau ulang bila jawabannya tidak.
- Font yang cocok adalah font Uthmani gaya Madinah (misalnya KFGQPC); syarat redistribusinya tetap perlu dicek sebelum dibundel.
