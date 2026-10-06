# 0001. Tauri v2 sebagai shell aplikasi

Tanggal: 2026-10-06
Status: diterima

## Konteks

Ziyadah harus jalan di Windows dan macOS, berjalan seharian di latar belakang, dan butuh beberapa jendela: jendela utama dan panel Ambient yang transparan, selalu di atas, dan bisa klik-tembus. Pemilik project terbiasa dengan React/Vite di frontend.

## Keputusan

Memakai Tauri v2 dengan frontend React + Vite + TypeScript.

## Alasan dan alternatif

- **Tauri v2:** installer kecil, memakai WebView bawaan OS, mendukung multi-jendela, jendela transparan, always-on-top, abaikan kursor (klik-tembus), dan tray. Frontend tetap React.
- **Wails (Go):** cocok dengan stack Go yang sudah dikuasai, tetapi dukungan multi-jendela yang dibutuhkan panel Ambient ada di v3 yang stabilitasnya belum pasti saat keputusan dibuat.
- **Electron:** paling mudah, tetapi ukuran dan pemakaian memori besar untuk aplikasi yang hidup seharian.

## Konsekuensi

- Inti aplikasi ditulis dalam Rust, bahasa yang perlu dipelajari lebih dalam.
- Tampilan bisa sedikit berbeda antar OS karena WebView berbeda (WebView2 di Windows, WKWebView di macOS), jadi panel harus diuji di kedua OS.
