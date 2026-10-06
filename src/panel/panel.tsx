import { StrictMode, useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import "../shared/theme.css";
import "./panel.css";
import { data, panel, player, type Basmalah, type PlayerState, type Surah } from "../shared/player";

// Panel Ambient: teks ayat yang sedang dibaca qari, melayang di atas semua jendela.
// Seret dari mana saja untuk memindah. Kontrol muncul saat kursor di atas panel.
// Tombol × menyembunyikan panel saja; murottal tetap jalan dan panel bisa dimunculkan lagi dari tray.

function Panel() {
  const [state, setState] = useState<PlayerState | null>(null);
  const [surah, setSurah] = useState<Surah | null>(null);
  const [basmalah, setBasmalah] = useState<Basmalah | null>(null);
  const [showTranslation, setShowTranslation] = useState(
    () => safeGet("ziyadah.panel.translation") === "1",
  );

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
    <div className="panel" data-tauri-drag-region>
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
