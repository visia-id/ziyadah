// Membangun src-tauri/resources/quran.db dari berkas resmi Tanzil (keputusan 0004).
//
// - Teks: Tanzil Quran Text (Uthmani). Boleh disalin verbatim, TIDAK BOLEH DIUBAH,
//   wajib menyebut Tanzil Project dan menautkan ke tanzil.net. Header lisensinya disimpan di tabel `meta`.
// - Metadata juz, rub' hizb, halaman, sajdah: Tanzil quran-data.xml.
// - Terjemah: id.indonesian (Kementerian Agama RI) dari Tanzil.
//
// Integritas (F1-06): isi teks dicocokkan dengan checksum di scripts/tanzil-sources.json.
// Bila Tanzil memperbarui teks, script berhenti; perubahan wajib ditinjau manusia sebelum checksum diganti.
//
// Hasilnya TIDAK di-commit (lihat .gitignore dan keputusan 0003). Jalankan: npm run fetch-data

import { createHash } from "node:crypto";
import { existsSync } from "node:fs";
import { mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { DatabaseSync } from "node:sqlite";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const CACHE = join(ROOT, ".cache", "tanzil");
const OUT = join(ROOT, "src-tauri", "resources", "quran.db");
const SOURCES = JSON.parse(await readFile(join(ROOT, "scripts", "tanzil-sources.json"), "utf8"));

const TOTAL_AYAHS = 6236;

async function fetchSource(name) {
  const { url, file } = SOURCES[name];
  const path = join(CACHE, file);
  if (!existsSync(path)) {
    const res = await fetch(url);
    if (!res.ok) throw new Error(`Gagal mengunduh ${name}: HTTP ${res.status}`);
    await mkdir(CACHE, { recursive: true });
    await writeFile(path, Buffer.from(await res.arrayBuffer()));
    console.log(`Unduh ${file}`);
  }
  return readFile(path, "utf8");
}

/** Baris data `surah|ayah|teks` dan header komentar (`#`) dipisah. */
function parsePipe(raw) {
  const lines = raw.replace(/\r\n/g, "\n").split("\n");
  const data = lines.filter((l) => /^\d+\|\d+\|/.test(l));
  const header = lines.filter((l) => l.startsWith("#")).join("\n");
  return { data, header };
}

function sha256(s) {
  return createHash("sha256").update(s, "utf8").digest("hex");
}

/** Checksum dihitung dari baris data saja, supaya pergantian tahun di header lisensi tidak memicu alarm. */
function verify(name, dataLines) {
  const actual = sha256(dataLines.join("\n"));
  const expected = SOURCES[name].sha256;
  if (actual !== expected) {
    throw new Error(
      `Checksum ${name} berbeda.\n  diharapkan: ${expected}\n  didapat:    ${actual}\n` +
        `Teks sumber berubah. Tinjau perubahan di ${SOURCES[name].url} sebelum memperbarui scripts/tanzil-sources.json.`,
    );
  }
}

function attrs(tag) {
  return Object.fromEntries([...tag.matchAll(/(\w+)="([^"]*)"/g)].map((m) => [m[1], m[2]]));
}

function parseMeta(xml) {
  const pick = (el) => [...xml.matchAll(new RegExp(`<${el} [^>]*/>`, "g"))].map((m) => attrs(m[0]));
  return {
    suras: pick("sura"),
    juzs: pick("juz"),
    quarters: pick("quarter"),
    pages: pick("page"),
    sajdas: pick("sajda"),
  };
}

/** Untuk setiap ayat, nomor bagian (juz/rub'/halaman) = penanda terakhir yang posisinya <= ayat itu. */
function sectionIndex(marks, surahCounts) {
  const start = (s, a) => surahCounts.slice(0, s - 1).reduce((x, n) => x + n, 0) + a;
  const starts = marks.map((m) => ({ index: Number(m.index), pos: start(Number(m.sura), Number(m.aya)) }));
  starts.sort((a, b) => a.pos - b.pos);
  return (s, a) => {
    const pos = start(s, a);
    let idx = starts[0].index;
    for (const m of starts) {
      if (m.pos > pos) break;
      idx = m.index;
    }
    return idx;
  };
}

function fail(msg) {
  throw new Error(`Integritas gagal: ${msg}`);
}

async function main() {
  const text = parsePipe(await fetchSource("text"));
  const trans = parsePipe(await fetchSource("translation"));
  const metaXml = await fetchSource("metadata");

  verify("text", text.data);
  verify("translation", trans.data);
  if (sha256(metaXml.replace(/\r\n/g, "\n")) !== SOURCES.metadata.sha256) fail("checksum metadata berbeda");

  const meta = parseMeta(metaXml);
  if (meta.suras.length !== 114) fail(`jumlah surah ${meta.suras.length}`);
  const surahCounts = meta.suras.map((s) => Number(s.ayas));
  if (surahCounts.reduce((a, b) => a + b, 0) !== TOTAL_AYAHS) fail("jumlah ayat di metadata bukan 6.236");
  if (text.data.length !== TOTAL_AYAHS) fail(`jumlah ayat teks ${text.data.length}`);
  if (trans.data.length !== TOTAL_AYAHS) fail(`jumlah ayat terjemah ${trans.data.length}`);

  const rows = text.data.map((l, i) => {
    const [s, a, ...rest] = l.split("|");
    const [ts, ta, ...trest] = trans.data[i].split("|");
    if (s !== ts || a !== ta) fail(`urutan terjemah tidak sejajar di ${s}:${a}`);
    return { surah: Number(s), ayah: Number(a), text: rest.join("|"), tr: trest.join("|") };
  });

  // Basmalah di Tanzil menempel di awal ayat 1 (kecuali surah 1 dan 9). Dipisah untuk tampilan dan audio,
  // tanpa mengubah satu karakter pun: basmalah + " " + sisa ayat harus sama persis dengan teks asli.
  // Penulisannya tidak selalu sama dengan Al-Fatihah 1 (surah 95 dan 97 memakai tasydid pada ba),
  // jadi basmalah disimpan per surah apa adanya, bukan disalin dari Al-Fatihah.
  const basmalah = rows[0].text;
  const basmalahId = rows[0].tr;
  const noShadda = (t) => t.replace(/ّ/g, "");
  const surahBasmalah = new Map();
  for (const r of rows) {
    r.display = r.text;
    if (r.ayah !== 1 || r.surah === 1 || r.surah === 9) continue;
    const words = r.text.split(" ");
    const prefix = words.slice(0, 4).join(" ");
    if (noShadda(prefix) !== noShadda(basmalah)) fail(`ayat ${r.surah}:1 tidak diawali basmalah`);
    r.display = words.slice(4).join(" ");
    if (prefix + " " + r.display !== r.text) fail(`pemisahan basmalah ${r.surah}:1 mengubah teks`);
    surahBasmalah.set(r.surah, prefix);
  }
  if (rows.find((r) => r.surah === 9 && r.ayah === 1).text.startsWith(basmalah)) fail("At-Taubah diawali basmalah");

  const juzOf = sectionIndex(meta.juzs, surahCounts);
  const quarterOf = sectionIndex(meta.quarters, surahCounts);
  const pageOf = sectionIndex(meta.pages, surahCounts);
  const sajdah = new Map(meta.sajdas.map((x) => [`${x.sura}:${x.aya}`, x.type]));

  await mkdir(dirname(OUT), { recursive: true });
  await rm(OUT, { force: true });
  const db = new DatabaseSync(OUT);
  db.exec(`
    CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
    CREATE TABLE surah (
      number INTEGER PRIMARY KEY, name_ar TEXT NOT NULL, name_latin TEXT NOT NULL,
      ayah_count INTEGER NOT NULL, type TEXT NOT NULL, revelation_order INTEGER NOT NULL,
      basmalah TEXT  -- basmalah di awal surah persis seperti di Tanzil; NULL untuk surah 1 dan 9
    );
    CREATE TABLE ayah (
      surah INTEGER NOT NULL, ayah INTEGER NOT NULL,
      text TEXT NOT NULL,          -- teks Tanzil apa adanya
      text_display TEXT NOT NULL,  -- tanpa basmalah di awal ayat 1 (basmalah ditampilkan terpisah)
      juz INTEGER NOT NULL, rub INTEGER NOT NULL, page INTEGER NOT NULL, sajdah TEXT,
      PRIMARY KEY (surah, ayah)
    );
    CREATE TABLE translation (
      lang TEXT NOT NULL, surah INTEGER NOT NULL, ayah INTEGER NOT NULL, text TEXT NOT NULL,
      PRIMARY KEY (lang, surah, ayah)
    );
  `);

  const putMeta = db.prepare("INSERT INTO meta VALUES (?, ?)");
  const metaRows = {
    schema_version: "1",
    text_source: "Tanzil Quran Text (Uthmani, Version 1.1), https://tanzil.net",
    text_license: text.header,
    text_sha256: SOURCES.text.sha256,
    translation_id_source: "Kementerian Agama RI, via Tanzil (id.indonesian), https://tanzil.net",
    translation_id_header: trans.header,
    metadata_source: "Tanzil quran-data.xml, https://tanzil.net (CC BY)",
    basmalah: basmalah,
    basmalah_id: basmalahId,
    built_at: new Date().toISOString(),
  };
  for (const [k, v] of Object.entries(metaRows)) putMeta.run(k, v);

  const putSurah = db.prepare("INSERT INTO surah VALUES (?, ?, ?, ?, ?, ?, ?)");
  for (const s of meta.suras) {
    const n = Number(s.index);
    putSurah.run(n, s.name, s.tname, Number(s.ayas), s.type, Number(s.order), surahBasmalah.get(n) ?? null);
  }

  const putAyah = db.prepare("INSERT INTO ayah VALUES (?, ?, ?, ?, ?, ?, ?, ?)");
  const putTr = db.prepare("INSERT INTO translation VALUES ('id', ?, ?, ?)");
  db.exec("BEGIN");
  for (const r of rows) {
    putAyah.run(
      r.surah, r.ayah, r.text, r.display,
      juzOf(r.surah, r.ayah), quarterOf(r.surah, r.ayah), pageOf(r.surah, r.ayah),
      sajdah.get(`${r.surah}:${r.ayah}`) ?? null,
    );
    putTr.run(r.surah, r.ayah, r.tr);
  }
  db.exec("COMMIT");

  // Cek ulang dari database: tidak ada teks yang berubah saat disimpan.
  const stored = db.prepare("SELECT surah, ayah, text FROM ayah ORDER BY surah, ayah").all();
  const storedHash = sha256(stored.map((x) => `${x.surah}|${x.ayah}|${x.text}`).join("\n"));
  if (storedHash !== SOURCES.text.sha256) fail("teks di database berbeda dari sumber");
  const last = db.prepare("SELECT juz, page FROM ayah WHERE surah = 114 AND ayah = 6").get();
  if (last.juz !== 30 || last.page !== 604) fail(`An-Nas 6 harus juz 30 halaman 604, didapat ${last.juz}/${last.page}`);
  db.exec("VACUUM");
  db.close();

  console.log(`Selesai: ${OUT}`);
  console.log(`114 surah, ${rows.length} ayat, terjemah Indonesia, checksum teks cocok.`);
}

main().catch((e) => {
  console.error(e.message ?? e);
  process.exit(1);
});
