import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// Status pemutaran dipegang inti Rust dan disiarkan ke semua jendela.
export type PlayerStatus = "idle" | "loading" | "playing" | "paused";

/** Apa yang terjadi setelah sebuah ayat selesai. Rentang berlaku di surah yang sedang diputar. */
export type PlayMode =
  | { kind: "stop" }
  | { kind: "continue" }
  | { kind: "repeatAyah" }
  | { kind: "repeatSurah" }
  | { kind: "range"; from: number; to: number };

export interface PlayerState {
  status: PlayerStatus;
  surah: number;
  /** 0 = basmalah, 1..n = nomor ayat */
  ayah: number;
  reciter: string;
  /** true bila ayat yang ditunggu sedang diunduh */
  buffering: boolean;
  error: string | null;
  mode: PlayMode;
}

export interface SurahIndexItem {
  number: number;
  nameAr: string;
  nameLatin: string;
  ayahCount: number;
}

export interface Ayah {
  n: number;
  ar: string;
  id: string;
}

export interface Surah extends SurahIndexItem {
  ayahs: Ayah[];
}

export interface Basmalah {
  ar: string;
  id: string;
}

export interface SurahDetail extends Surah {
  /** Basmalah di awal surah persis seperti di Tanzil; null untuk Al-Fatihah dan At-Taubah. */
  basmalah: Basmalah | null;
}

export const RECITERS = [
  { id: "Alafasy_128kbps", name: "Mishary Rashid Alafasy" },
  { id: "Husary_128kbps", name: "Mahmoud Khalil Al-Husary" },
  { id: "Minshawy_Murattal_128kbps", name: "Mohamed Siddiq Al-Minshawi" },
  { id: "Abdul_Basit_Murattal_192kbps", name: "Abdul Basit Abdus Samad" },
  { id: "Abdurrahmaan_As-Sudais_192kbps", name: "Abdurrahman As-Sudais" },
];

export const player = {
  play: (surah: number, startAyah: number, reciter: string) =>
    invoke<void>("player_play", { surah, startAyah, reciter }),
  setMode: (mode: PlayMode) => invoke<void>("player_set_mode", { mode }),
  pause: () => invoke<void>("player_pause"),
  resume: () => invoke<void>("player_resume"),
  next: () => invoke<void>("player_next"),
  prev: () => invoke<void>("player_prev"),
  stop: () => invoke<void>("player_stop"),
  state: () => invoke<PlayerState>("player_state"),
  onState: (cb: (s: PlayerState) => void): Promise<UnlistenFn> =>
    listen<PlayerState>("player://state", (e) => cb(e.payload)),
};

export const panel = {
  toggle: () => invoke<boolean>("panel_toggle"),
  hide: () => invoke<void>("panel_hide"),
  /** Tinggi logis (CSS px) isi panel; jendela panel menyesuaikan dengan tepi bawah tetap. */
  fit: (height: number) => invoke<void>("panel_fit", { height }),
  setClickThrough: (on: boolean) => invoke<void>("panel_set_click_through", { on }),
};

// Data Qur'an dibaca inti Rust dari quran.db (sumber: Tanzil). Hasil disimpan per jendela.
const cache = new Map<string, Promise<unknown>>();
function cached<T>(key: string, load: () => Promise<T>): Promise<T> {
  if (!cache.has(key)) {
    const p = load();
    cache.set(key, p);
    p.catch(() => cache.delete(key));
  }
  return cache.get(key) as Promise<T>;
}

export const data = {
  index: () => cached("index", () => invoke<SurahIndexItem[]>("quran_index")),
  surah: (n: number) => cached(`surah:${n}`, () => invoke<SurahDetail>("quran_surah", { number: n })),
};
