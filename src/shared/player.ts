import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// Status pemutaran dipegang inti Rust dan disiarkan ke semua jendela.
export type PlayerStatus = "idle" | "loading" | "playing" | "paused";

export interface PlayerState {
  status: PlayerStatus;
  surah: number;
  /** 0 = basmalah, 1..n = nomor ayat */
  ayah: number;
  reciter: string;
  /** true bila ayat yang ditunggu sedang diunduh */
  buffering: boolean;
  error: string | null;
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

export const RECITERS = [
  { id: "Alafasy_128kbps", name: "Mishary Rashid Alafasy" },
  { id: "Husary_128kbps", name: "Mahmoud Khalil Al-Husary" },
  { id: "Minshawy_Murattal_128kbps", name: "Mohamed Siddiq Al-Minshawi" },
  { id: "Abdul_Basit_Murattal_192kbps", name: "Abdul Basit Abdus Samad" },
  { id: "Abdurrahmaan_As-Sudais_192kbps", name: "Abdurrahman As-Sudais" },
];

export const player = {
  play: (surah: number, ayahCount: number, startAyah: number, reciter: string) =>
    invoke<void>("player_play", { surah, ayahCount, startAyah, reciter }),
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
  setClickThrough: (on: boolean) => invoke<void>("panel_set_click_through", { on }),
};

const cache = new Map<string, unknown>();
async function loadJson<T>(path: string): Promise<T> {
  if (!cache.has(path)) {
    const res = await fetch(path);
    if (!res.ok) throw new Error(`Data ${path} tidak ditemukan. Jalankan: npm run fetch-data`);
    cache.set(path, await res.json());
  }
  return cache.get(path) as T;
}

export const data = {
  index: () => loadJson<SurahIndexItem[]>("/data/index.json"),
  surah: (n: number) => loadJson<Surah>(`/data/surah-${String(n).padStart(3, "0")}.json`),
  basmalah: () => loadJson<Basmalah>("/data/basmalah.json"),
};
