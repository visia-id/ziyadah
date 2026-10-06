// Pilihan kecil per pengguna disimpan di localStorage WebView.
// Sementara sampai penyimpanan preferensi di inti Rust ada (F1-18).
// Akses dibungkus try/catch: storage bisa tidak tersedia, dan kegagalan tidak boleh merusak aplikasi.

export function safeGet(key: string): string | null {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}

export function safeSet(key: string, value: string) {
  try {
    localStorage.setItem(key, value);
  } catch {
    /* abaikan */
  }
}
