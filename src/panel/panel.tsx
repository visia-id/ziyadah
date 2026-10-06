import { StrictMode, useEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import "../shared/theme.css";
import "./panel.css";
import { data, panel, player, type PlayerState, type SurahDetail } from "../shared/player";
import { safeGet, safeSet } from "../shared/storage";
import { ArabicWords, useActiveWords } from "../shared/words";
import { scrollBehavior, updateScrollEdges, useManualScrollPause } from "../shared/follow";

// Panel Ambient: teks ayat yang sedang dibaca qari, melayang di atas semua jendela.
// Seret dari mana saja untuk memindah. Kontrol muncul saat kursor di atas panel.
// Tombol × menyembunyikan panel saja; murottal tetap jalan dan panel bisa dimunculkan lagi dari tray.
//
// Teks ayat dan terjemahnya tidak pernah dipotong: tinggi panel mengikuti isi sampai batas
// MAX_SCREEN_RATIO, lalu huruf Arab dikecilkan sampai AR_MIN, dan bila masih tidak muat
// teks ayat beserta terjemahnya bisa di-scroll.

const AR_MAX = 28;
const AR_MIN = 18;
const AR_STEP = 2;
const MAX_SCREEN_RATIO = 0.4;

function Panel() {
  const [state, setState] = useState<PlayerState | null>(null);
  const [surah, setSurah] = useState<SurahDetail | null>(null);
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

  const idle = !state || state.status === "idle";
  // Mode lanjut bisa masuk ke surah yang teksnya belum diambil (data uji coba hanya beberapa surah).
  const missingText = !idle && (!surah || surah.number !== state!.surah);
  const isBasmalah = state?.ayah === 0;
  // Data surah bisa belum selesai dimuat saat pemutaran baru mulai; jangan anggap sudah ada.
  const ayah = !idle && !isBasmalah ? surah?.ayahs.find((a) => a.n === state!.ayah) : undefined;
  const ar = isBasmalah ? surah?.basmalah?.ar : ayah?.ar;
  const tr = isBasmalah ? surah?.basmalah?.id : ayah?.id;
  const active = useActiveWords(state?.reciter, state?.surah, state?.status === "playing");
  const range = active && active.ayah === state?.ayah ? active.range : null;

  // Ayat panjang yang di-scroll (F1-27): ganti ayat kembali ke atas; selama dibaca, baris aktif dijaga
  // di sekitar sepertiga atas. Panel hanya bergeser saat qari pindah baris, dan berhenti bila pengguna menggulir.
  const textRef = useRef<HTMLDivElement>(null);
  const manualScroll = useManualScrollPause(textRef);
  useEffect(() => {
    const box = textRef.current;
    if (!box) return;
    box.scrollTop = 0;
    updateScrollEdges(box);
  }, [state?.surah, state?.ayah]);
  useEffect(() => {
    const box = textRef.current;
    const word = box?.querySelector<HTMLElement>(".word.on");
    if (!box || !word || box.scrollHeight <= box.clientHeight + 1 || manualScroll()) return;
    const h = box.clientHeight;
    const top = word.offsetTop;
    if (top >= box.scrollTop + h * 0.15 && top + word.offsetHeight <= box.scrollTop + h * 0.75) return;
    box.scrollTo({ top: Math.max(0, top - h * 0.3), behavior: scrollBehavior() });
  }, [range?.[0], state?.ayah]);

  return (
    <div className="panel" ref={ref} data-tauri-drag-region>
      {idle ? (
        <p className="idle" data-tauri-drag-region>
          Ziyadah · pilih surah di jendela utama
        </p>
      ) : missingText ? (
        <p className="idle" data-tauri-drag-region>
          Surah {state!.surah} · {isBasmalah ? "Basmalah" : `ayat ${state!.ayah}`} · teks belum tersedia
        </p>
      ) : (
        <>
          <div className="text" ref={textRef} onScroll={(e) => updateScrollEdges(e.currentTarget)} data-tauri-drag-region>
            <p key={`${state!.surah}:${state!.ayah}`} className="ar" dir="rtl" data-tauri-drag-region>
              {ar && <ArabicWords text={ar} range={range} />}
              {ayah && <span className="num"> ﴿{toArabicDigits(ayah.n)}﴾</span>}
            </p>
            {showTranslation && tr && (
              <p className="tr" dir="ltr" data-tauri-drag-region>
                {tr}
              </p>
            )}
          </div>
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
  updateScrollEdges(el.querySelector<HTMLElement>(".text"));
  panel.fit(el.offsetHeight).catch(() => {});
}

function toArabicDigits(n: number) {
  return String(n).replace(/\d/g, (d) => "٠١٢٣٤٥٦٧٨٩"[Number(d)]);
}


createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <Panel />
  </StrictMode>,
);
