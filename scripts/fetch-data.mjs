// Mengambil data teks untuk prototipe dari api.alquran.cloud.
// Teks: edisi quran-uthmani. Terjemah: id.indonesian (Kemenag).
//
// Data TIDAK di-commit ke repo (lihat .gitignore) sampai urusan lisensi
// dan tashih beres. Jalankan: npm run fetch-data
//
// Untuk prototipe hanya beberapa surah pendek. Tambah nomor surah di SURAHS bila perlu.

import { mkdir, writeFile } from "node:fs/promises";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const SURAHS = [1, 67, 112, 113, 114];
const OUT = join(dirname(fileURLToPath(import.meta.url)), "..", "public", "data");
const API = "https://api.alquran.cloud/v1/surah";

// Di edisi quran-uthmani, ayat 1 surah 2 sampai 114 (kecuali 9) diawali basmalah.
// Basmalah dipisah karena diputar dan ditampilkan sebagai item tersendiri.
function stripBasmalah(text, basmalah) {
  if (text.startsWith(basmalah)) return text.slice(basmalah.length).trim();
  // Cadangan: buang 4 kata pertama bila diawali "بِسْمِ".
  if (text.startsWith("بِسْمِ")) return text.split(" ").slice(4).join(" ").trim();
  return text;
}

async function getSurah(n) {
  const res = await fetch(`${API}/${n}/editions/quran-uthmani,id.indonesian`);
  if (!res.ok) throw new Error(`Gagal mengambil surah ${n}: HTTP ${res.status}`);
  const json = await res.json();
  const [ar, id] = json.data;
  return { ar, id };
}

async function main() {
  await mkdir(OUT, { recursive: true });

  const fatihah = await getSurah(1);
  const basmalah = fatihah.ar.ayahs[0].text.trim();
  await writeFile(
    join(OUT, "basmalah.json"),
    JSON.stringify({ ar: basmalah, id: fatihah.id.ayahs[0].text }, null, 2),
  );

  const index = [];
  for (const n of SURAHS) {
    const { ar, id } = n === 1 ? fatihah : await getSurah(n);
    const ayahs = ar.ayahs.map((a, i) => {
      let text = a.text.trim();
      if (i === 0 && n !== 1 && n !== 9) text = stripBasmalah(text, basmalah);
      return { n: a.numberInSurah, ar: text, id: id.ayahs[i].text };
    });
    const surah = {
      number: n,
      nameAr: ar.name,
      nameLatin: ar.englishName,
      ayahCount: ayahs.length,
      ayahs,
    };
    await writeFile(join(OUT, `surah-${String(n).padStart(3, "0")}.json`), JSON.stringify(surah, null, 2));
    index.push({ number: n, nameAr: ar.name, nameLatin: ar.englishName, ayahCount: ayahs.length });
    console.log(`OK  surah ${n} ${ar.englishName} (${ayahs.length} ayat)`);
  }
  await writeFile(join(OUT, "index.json"), JSON.stringify(index, null, 2));
  console.log(`Selesai. Data tersimpan di ${OUT}`);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
