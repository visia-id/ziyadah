import { StrictMode, useEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import "../shared/theme.css";
import "./panel.css";
import { data, panel, player, type Basmalah, type PlayerState, type Surah } from "../shared/player";

// Panel Ambient: teks ayat yang sedang dibaca qari, melayang di atas semua jendela.
// Seret dari mana saja untuk memindah. Kontrol muncul saat kursor di atas panel.
// Tombol × menyembunyikan panel saja; murottal tetap jalan dan panel bisa dimunculkan lagi dari tray.
//
// Teks ayat tidak pernah dipotong: tinggi panel mengikuti isi sampai batas MAX_SCREEN_RATIO,
// lalu huruf Arab dikecilkan sampai AR_MIN, dan bila masih tidak muat teks Arab bisa di-scroll.

const AR_MAX = 28;
const AR_MIN = 18;
const AR_STEP = 2;
const MAX_SCREEN_RATIO = 0.4;

function Panel() {
  const [state, setState] = useState<PlayerState | null>(null);
  const [surah, setSurah] = useState<Surah | null>(null);
  const [basmalah, setBasmalah] = useState<Basmalah | null>(null);
  const [showTranslation, setShowTranslation] = useState(
    () => safeGet("ziyadah.panel.translation") === "1",
  );
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    // Terpicu setiap ukuran isi berubah: ayat berganti, terjemah dinyalakan, font selesai dimuat, lebar diubah.
    const ro = new ResizeObserver(() => fitToContent(el));
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  useEffect(() => {
    player.state().then(setState).catch(() => {});
    data.basmalah().then(setBasmalah).catch(() => {});
    const un = player.onState(setState);
    return () => {
      un.then((f) => f());
    };
  }, []);

  useEffect(() => {
    if (state && state.surah > 0 && state.surah !== surah?.number) {
      data.surah(state.surah).then(setSurah).catch(() => setSurah(null));
    }
  }, [state?.surah]);

  const toggleTranslation = () => {
    const next = !showTranslation;
    setShowTranslation(next);
    safeSet("ziyadah.panel.translation", next ? "1" : "0");
  };

  const idle = !state || state.status === "idle" || !surah;
  const isBasmalah = state?.ayah === 0;
  const ayah = !idle && !isBasmalah ? surah!.ayahs.find((a) => a.n === state!.ayah) : undefined;
  const ar = isBasmalah ? basmalah?.ar : ayah?.ar;
  const tr = isBasmalah ? basmalah?.id : ayah?.id;

  return (
    <div className="panel" ref={ref} data-tauri-drag-region>
      {idle ? (
        <p className="idle" data-tauri-drag-region>
          Ziyadah · pilih surah di jendela utama
        </p>
      ) : (
        <>
          <p key={`${state!.surah}:${state!.ayah}`} className="ar" dir="rtl" data-tauri-drag-region>
            {ar}
            {ayah && <span className="num"> ﴿{toArabicDigits(ayah.n)}﴾</span>}
          </p>
          {showTranslation && tr && (
            <p className="tr" dir="ltr" data-tauri-drag-region>
              {tr}
            </p>
          )}
          <div className="meta" dir="ltr" data-tauri-drag-region>
            {surah!.nameLatin} · {isBasmalah ? "Basmalah" : `${state!.ayah}/${surah!.ayahCount}`}
            {state!.buffering && " · memuat audio"}
            {state!.status === "paused" && " · dijeda"}
          </div>
        </>
      )}

      <div className="controls" dir="ltr">
        <button onClick={() => player.prev()} title="Ayat sebelumnya">⏮</button>
        {state?.status === "playing" ? (
          <button onClick={() => player.pause()} title="Jeda">⏸</button>
        ) : (
          <button onClick={() => player.resume()} title="Putar">▶</button>
        )}
        <button onClick={() => player.next()} title="Ayat berikutnya">⏭</button>
        <button className={showTranslation ? "on" : ""} onClick={toggleTranslation} title="Terjemah">
          ID
        </button>
        <button className="hide" onClick={() => panel.hide()} title="Sembunyikan panel (murottal tetap jalan)">
          ×
        </button>
      </div>
    </div>
  );
}

function fitToContent(el: HTMLDivElement) {
  const maxH = Math.round(window.screen.availHeight * MAX_SCREEN_RATIO);
  el.classList.remove("overflow");
  el.style.maxHeight = "";

  let size = AR_MAX;
  el.style.setProperty("--ar-size", `${size}px`);
  while (el.offsetHeight > maxH && size > AR_MIN) {
    size -= AR_STEP;
    el.style.setProperty("--ar-size", `${size}px`);
  }

  if (el.offsetHeight > maxH) {
    el.classList.add("overflow");
    el.style.maxHeight = `${maxH}px`;
  }
  panel.fit(el.offsetHeight).catch(() => {});
}

function toArabicDigits(n: number) {
  return String(n).replace(/\d/g, (d) => "٠١٢٣٤٥٦٧٨٩"[Number(d)]);
}

function safeGet(k: string) {
  try {
    return localStorage.getItem(k);
  } catch {
    return null;
  }
}
function safeSet(k: string, v: string) {
  try {
    localStorage.setItem(k, v);
  } catch {
    /* abaikan */
  }
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <Panel />
  </StrictMode>,
);
