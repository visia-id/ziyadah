import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { data, player, type PlayerState, type SurahDetail } from "../shared/player";
import { ArabicWords, useActiveWords } from "../shared/words";
import { BACKGROUNDS, backgroundById, type Background } from "../shared/backgrounds";
import { prefersReducedMotion, scrollBehavior, useManualScrollPause } from "../shared/follow";
import { safeGet, safeSet } from "../shared/storage";
import "./fullscreen.css";

// Mode Layar Penuh (F2-11): ayat besar dengan sorot per kata di atas latar yang bergerak pelan.
// Teks selalu di atas lapisan gelap supaya tetap terbaca (PRD: kontras WCAG AA).
// Keyboard: Esc keluar, Spasi putar/jeda, panah kanan/PageDown ayat berikutnya, panah kiri/PageUp sebelumnya,
// T terjemah, B ganti latar. PageUp/PageDown dipakai juga oleh remote presentasi.

const KEY_BG = "ziyadah.full.bg";
const KEY_TR = "ziyadah.full.tr";
const CONTROLS_HIDE_MS = 2500;
/** Ukuran huruf Arab (px): mulai dari AR_MAX relatif layar, dikecilkan sampai AR_MIN untuk ayat panjang. */
const AR_MIN = 26;

interface Props {
  state: PlayerState | null;
  /** Surah yang dipilih di jendela utama, dipakai saat belum memutar. */
  surahNo: number;
  onStart: () => void;
  onExit: () => void;
}

export function Fullscreen({ state, surahNo, onStart, onExit }: Props) {
  const [bg, setBg] = useState<Background>(() => backgroundById(safeGet(KEY_BG)));
  const [showTr, setShowTr] = useState(() => safeGet(KEY_TR) !== "0");
  const [controls, setControls] = useState(true);
  const [surah, setSurah] = useState<SurahDetail | null>(null);
  const hideTimer = useRef<number>();

  const idle = !state || state.status === "idle";
  const playing = state?.status === "playing";
  const surahToShow = !idle && state!.surah > 0 ? state!.surah : surahNo;

  useEffect(() => {
    data.surah(surahToShow).then(setSurah).catch(() => setSurah(null));
  }, [surahToShow]);

  useEffect(() => safeSet(KEY_BG, bg.id), [bg]);
  useEffect(() => safeSet(KEY_TR, showTr ? "1" : "0"), [showTr]);

  const wake = () => {
    setControls(true);
    window.clearTimeout(hideTimer.current);
    hideTimer.current = window.setTimeout(() => setControls(false), CONTROLS_HIDE_MS);
  };
  useEffect(() => {
    wake();
    return () => window.clearTimeout(hideTimer.current);
  }, []);

  const nextBg = () => setBg((b) => BACKGROUNDS[(BACKGROUNDS.indexOf(b) + 1) % BACKGROUNDS.length]);
  const togglePlay = () => {
    if (idle) onStart();
    else if (playing) player.pause();
    else player.resume();
  };

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const k = e.key;
      if (k === "Escape") onExit();
      else if (k === " ") togglePlay();
      else if (k === "ArrowRight" || k === "PageDown") player.next();
      else if (k === "ArrowLeft" || k === "PageUp") player.prev();
      else if (k === "t" || k === "T") setShowTr((v) => !v);
      else if (k === "b" || k === "B") nextBg();
      else return;
      e.preventDefault();
      wake();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  // Teks ayat aktif, sebelum, dan sesudah. Ayat 0 = basmalah.
  const n = idle ? -1 : state!.ayah;
  const isBasmalah = n === 0;
  const ayah = surah?.ayahs.find((a) => a.n === n);
  const ar = isBasmalah ? surah?.basmalah?.ar : ayah?.ar;
  const tr = isBasmalah ? surah?.basmalah?.id : ayah?.id;
  const prevAr = n > 1 ? surah?.ayahs.find((a) => a.n === n - 1)?.ar : undefined;
  const nextAr = n >= 0 ? surah?.ayahs.find((a) => a.n === (isBasmalah ? 1 : n + 1))?.ar : undefined;

  const active = useActiveWords(state?.reciter, state?.surah, playing);
  const range = active && active.ayah === n ? active.range : null;

  // Ayat panjang: huruf dikecilkan sampai AR_MIN, lalu kotak ayat di-scroll mengikuti kata yang dibaca.
  const boxRef = useRef<HTMLDivElement>(null);
  const manualScroll = useManualScrollPause(boxRef);
  useLayoutEffect(() => {
    const box = boxRef.current;
    if (!box) return;
    const fit = () => {
      const max = Math.min(76, Math.max(34, window.innerWidth * 0.046));
      let size = max;
      box.style.setProperty("--fs-ar", `${size}px`);
      while (box.scrollHeight > box.clientHeight + 1 && size > AR_MIN) {
        size -= 2;
        box.style.setProperty("--fs-ar", `${size}px`);
      }
      box.scrollTop = 0;
    };
    fit();
    window.addEventListener("resize", fit);
    return () => window.removeEventListener("resize", fit);
  }, [ar, showTr, tr]);
  useEffect(() => {
    const box = boxRef.current;
    const word = box?.querySelector<HTMLElement>(".word.on");
    if (!box || !word || box.scrollHeight <= box.clientHeight + 1 || manualScroll()) return;
    const h = box.clientHeight;
    if (word.offsetTop >= box.scrollTop + h * 0.15 && word.offsetTop + word.offsetHeight <= box.scrollTop + h * 0.75) return;
    box.scrollTo({ top: Math.max(0, word.offsetTop - h * 0.3), behavior: scrollBehavior() });
  }, [range?.[0], n]);

  return (
    <div className={`fs ${controls ? "" : "calm"}`} onMouseMove={wake}>
      <Backdrop bg={bg} />
      <div className="fs-shade" />

      <div className="fs-stage">
        {idle ? (
          <div className="fs-idle">
            <p className="fs-surah-ar" dir="rtl">{surah?.nameAr}</p>
            <p className="fs-hint">{surah ? `${surah.nameLatin} · ${surah.ayahCount} ayat` : ""}</p>
            <p className="fs-hint">Tekan Spasi atau tombol Putar untuk mulai</p>
          </div>
        ) : (
          <>
            <p className="fs-side" dir="rtl">{prevAr ?? ""}</p>
            <div className="fs-current" ref={boxRef}>
              <p key={`${state!.surah}:${n}`} className="fs-ar" dir="rtl">
                {ar && <ArabicWords text={ar} range={range} />}
                {ayah && <span className="fs-num"> ﴿{toArabicDigits(ayah.n)}﴾</span>}
              </p>
              {showTr && tr && <p className="fs-tr">{tr}</p>}
            </div>
            <p className="fs-side" dir="rtl">{nextAr ?? ""}</p>
          </>
        )}
      </div>

      {!idle && surah && (
        <div className="fs-meta">
          {surah.nameLatin} · {isBasmalah ? "Basmalah" : `${n}/${surah.ayahCount}`}
          {state!.status === "paused" && " · dijeda"}
          {state!.buffering && " · memuat audio"}
        </div>
      )}
      <div className="fs-credit">
        {bg.credit ? `Latar: ${bg.name} · ${bg.credit} · ${bg.license}` : `Latar: ${bg.name}`} · Belum ditashih LPMQ
      </div>

      <div className="fs-controls">
        <button onClick={() => player.prev()} title="Ayat sebelumnya (panah kiri)">⏮</button>
        <button className="main" onClick={togglePlay} title="Putar/jeda (Spasi)">
          {playing ? "⏸" : "▶"}
        </button>
        <button onClick={() => player.next()} title="Ayat berikutnya (panah kanan)">⏭</button>
        <button className={showTr ? "on" : ""} onClick={() => setShowTr((v) => !v)} title="Terjemah (T)">
          Terjemah
        </button>
        <button onClick={nextBg} title="Ganti latar (B)">Latar: {bg.name}</button>
        <button onClick={onExit} title="Keluar (Esc)">Keluar</button>
      </div>
    </div>
  );
}

function Backdrop({ bg }: { bg: Background }) {
  if (bg.kind === "stars") return <Stars />;
  if (bg.kind === "gradient") return <div className="fs-bg fs-gradient" />;
  return (
    <div className="fs-bg fs-photo-frame">
      <div className={`fs-photo ${bg.motion ?? ""}`} style={{ backgroundImage: `url(${bg.file})` }} />
    </div>
  );
}

/** Bintang berkelip yang digambar aplikasi; sekitar 30 bingkai per detik, diam bila "kurangi gerakan" aktif. */
function Stars() {
  const ref = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const canvas = ref.current!;
    const ctx = canvas.getContext("2d")!;
    let stars: { x: number; y: number; r: number; a: number; p: number; s: number }[] = [];
    const init = () => {
      const dpr = window.devicePixelRatio || 1;
      canvas.width = window.innerWidth * dpr;
      canvas.height = window.innerHeight * dpr;
      const count = Math.round((window.innerWidth * window.innerHeight) / 5000);
      stars = Array.from({ length: count }, () => ({
        x: Math.random() * canvas.width,
        y: Math.random() * canvas.height,
        r: (Math.random() * 1.2 + 0.3) * dpr,
        a: Math.random() * 0.6 + 0.3,
        p: Math.random() * Math.PI * 2,
        s: Math.random() * 0.8 + 0.2,
      }));
    };
    const draw = (t: number) => {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      for (const st of stars) {
        ctx.globalAlpha = st.a * (0.6 + 0.4 * Math.sin(st.p + (t / 1000) * st.s));
        ctx.beginPath();
        ctx.arc(st.x, st.y, st.r, 0, Math.PI * 2);
        ctx.fillStyle = "#fff";
        ctx.fill();
      }
    };
    init();
    let raf = 0;
    let last = 0;
    const loop = (t: number) => {
      raf = requestAnimationFrame(loop);
      if (t - last < 33) return;
      last = t;
      draw(t);
    };
    if (prefersReducedMotion()) draw(0);
    else raf = requestAnimationFrame(loop);
    const onResize = () => {
      init();
      draw(performance.now());
    };
    window.addEventListener("resize", onResize);
    return () => {
      cancelAnimationFrame(raf);
      window.removeEventListener("resize", onResize);
    };
  }, []);
  return <canvas ref={ref} className="fs-bg fs-stars" />;
}

function toArabicDigits(n: number) {
  return String(n).replace(/\d/g, (d) => "٠١٢٣٤٥٦٧٨٩"[Number(d)]);
}
