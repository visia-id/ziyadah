// Latar Mode Layar Penuh (F2-11). Aturan (PRD): tanpa gambar makhluk bernyawa, lisensi jelas, kredit
// dicantumkan, dan latar hanya suasana, tidak dipasangkan dengan ayat tertentu.
// Kredit juga tercantum di public/backgrounds/KREDIT.md dan README; perbarui bersamaan.

export type Motion = "rotate" | "pan" | "zoom";

export interface Background {
  id: string;
  name: string;
  /** Berkas di public/backgrounds; tanpa berkas = digambar aplikasi. */
  file?: string;
  motion?: Motion;
  /** Kind khusus yang digambar aplikasi. */
  kind?: "stars" | "gradient";
  credit?: string;
  license?: string;
  source?: string;
}

export const BACKGROUNDS: Background[] = [
  {
    id: "whirlpool",
    name: "Galaksi Whirlpool",
    file: "/backgrounds/whirlpool.jpg",
    motion: "rotate",
    credit: "NASA, ESA, S. Beckwith (STScI), Hubble Heritage Team (STScI/AURA)",
    license: "CC BY 4.0 (ESA), domain publik (NASA)",
    source: "https://images.nasa.gov/details/GSFC_20171208_Archive_e001925",
  },
  {
    id: "cosmic-cliffs",
    name: "Tebing Kosmik",
    file: "/backgrounds/cosmic-cliffs.jpg",
    motion: "pan",
    credit: "NASA, ESA, CSA, STScI, Webb ERO Production Team",
    license: "Domain publik",
    source: "https://images.nasa.gov/details/carina_nebula",
  },
  {
    id: "blue-marble",
    name: "Bumi",
    file: "/backgrounds/blue-marble.jpg",
    motion: "zoom",
    credit: "NASA, kru Apollo 17",
    license: "Domain publik",
    source: "https://images.nasa.gov/details/as17-148-22727",
  },
  { id: "stars", name: "Bintang", kind: "stars" },
  { id: "gradient", name: "Gradasi", kind: "gradient" },
];

export function backgroundById(id: string | null): Background {
  return BACKGROUNDS.find((b) => b.id === id) ?? BACKGROUNDS[0];
}
