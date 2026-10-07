import { Fragment, useEffect, useState } from "react";
import { data, player, useWindowShown, type AyahTiming, type PlayerPos } from "./player";

// Sorot per kata (F1-17) memakai timing quran-align.
// Teks ayat tidak diubah: teks dipecah per spasi lalu disusun ulang dengan spasi yang sama.
// Indeks kata quran-align tidak menghitung token tanda waqaf (ۖ ۗ ۚ ۞ dll.), jadi token itu dilewati saat menghitung.

const MARK_TOKEN = /^[ۖ-ۭ]+$/u;

/** Rentang kata aktif `[awal, akhir)` pada ayat tertentu; ayat 0 = basmalah. */
export interface ActiveWords {
  surah: number;
  ayah: number;
  range: [number, number];
}

function segmentAt(timing: AyahTiming | undefined, posMs: number): [number, number] | null {
  if (!timing) return null;
  let found: [number, number] | null = null;
  for (const [ws, we, start] of timing.segments) {
    if (start > posMs) break;
    found = [ws, we];
  }
  return found;
}

/**
 * Kata yang sedang dibaca qari, atau null bila qari/ayat tidak punya timing (tampilan memakai sorot per ayat).
 * Basmalah diputar dari berkas Al-Fatihah 1, jadi timing-nya diambil dari ayat itu.
 * Posisi audio (tiap 100 ms) hanya didengar saat memutar dan jendela terlihat, supaya jendela di tray
 * tidak menggambar ulang tanpa guna (F1-20).
 */
export function useActiveWords(reciter: string | undefined, surah: number | undefined, playing: boolean) {
  const [timings, setTimings] = useState<Map<number, AyahTiming>>(new Map());
  const [basmalah, setBasmalah] = useState<AyahTiming | undefined>();
  const [pos, setPos] = useState<PlayerPos | null>(null);

  useEffect(() => {
    setTimings(new Map());
    if (!reciter || !surah) return;
    let alive = true;
    data
      .timing(reciter, surah)
      .then((list) => alive && setTimings(new Map(list.map((t) => [t.ayah, t]))))
      .catch(() => {});
    data
      .timing(reciter, 1)
      .then((list) => alive && setBasmalah(list.find((t) => t.ayah === 1)))
      .catch(() => {});
    return () => {
      alive = false;
    };
  }, [reciter, surah]);

  const shown = useWindowShown();
  const listening = playing && shown;
  useEffect(() => {
    setPos(null);
    if (!listening) return;
    const un = player.onPos(setPos);
    return () => {
      un.then((f) => f());
    };
  }, [listening]);

  if (!pos || pos.surah !== surah) return null;
  const timing = pos.ayah === 0 ? basmalah : timings.get(pos.ayah);
  const range = segmentAt(timing, pos.posMs);
  return range ? ({ surah: pos.surah, ayah: pos.ayah, range } satisfies ActiveWords) : null;
}

/** Teks Arab dengan penanda pada kata `range`; tanpa `range` teks tampil biasa. */
export function ArabicWords({ text, range }: { text: string; range: [number, number] | null }) {
  if (!range) return <>{text}</>;
  let word = -1;
  return (
    <>
      {text.split(" ").map((token, i) => {
        const isMark = MARK_TOKEN.test(token);
        if (!isMark) word++;
        const on = !isMark && word >= range[0] && word < range[1];
        return (
          <Fragment key={i}>
            {i > 0 && " "}
            <span className={on ? "word on" : "word"}>{token}</span>
          </Fragment>
        );
      })}
    </>
  );
}
