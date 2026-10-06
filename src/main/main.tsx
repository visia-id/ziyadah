import { StrictMode, useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import "../shared/theme.css";
import "./main.css";
import {
  data,
  panel,
  player,
  RECITERS,
  type PlayMode,
  type PlayerState,
  type Surah,
  type SurahIndexItem,
} from "../shared/player";
import { safeGet, safeSet } from "../shared/storage";
import { ArabicWords, useActiveWords } from "../shared/words";
import { scrollBehavior, useManualScrollPause } from "../shared/follow";
import { UpdateBanner } from "./updater";
import { Fullscreen } from "./fullscreen";
import { getCurrentWindow } from "@tauri-apps/api/window";

// Pilihan terakhir diingat antar sesi; pertama kali dibuka mulai dari Al-Fatihah.
const KEY_SURAH = "ziyadah.main.surah";
const KEY_RECITER = "ziyadah.main.reciter";
const KEY_MODE = "ziyadah.main.mode";

function storedSurah() {
  const n = Number(safeGet(KEY_SURAH));
  return Number.isInteger(n) && n >= 1 && n <= 114 ? n : 1;
}

function storedReciter() {
  const r = safeGet(KEY_RECITER);
  return RECITERS.some((x) => x.id === r) ? r! : RECITERS[0].id;
}

function storedMode(): PlayMode {
  try {
    const m = JSON.parse(safeGet(KEY_MODE) ?? "null");
    if (m?.kind === "range" && Number.isInteger(m.from) && Number.isInteger(m.to)) return m;
    if (["stop", "continue", "repeatAyah", "repeatSurah"].includes(m?.kind)) return { kind: m.kind };
  } catch {
    /* data rusak: pakai bawaan */
  }
  return { kind: "stop" };
}

// Jendela utama untuk spike: pilih surah dan qari, kendalikan pemutaran,
// tampilkan/sembunyikan panel Ambient. Mode Tilawah dan Hafalan menyusul di fase berikutnya.

function App() {
  const [index, setIndex] = useState<SurahIndexItem[]>([]);
  const [dataError, setDataError] = useState<string | null>(null);
  const [surahNo, setSurahNo] = useState(storedSurah);
  const [surah, setSurah] = useState<Surah | null>(null);
  const [reciter, setReciter] = useState(storedReciter);
  const [state, setState] = useState<PlayerState | null>(null);
  const [clickThrough, setClickThrough] = useState(false);
  const [mode, setMode] = useState<PlayMode>(storedMode);

  useEffect(() => {
    data
      .index()
      .then(setIndex)
      .catch((e) => setDataError(String(e.message ?? e)));
    // Mode dipegang inti Rust; kirim mode terakhir sebelum membaca status supaya tidak tertimpa bawaan.
    player.setMode(storedMode()).catch(() => {});
    player.state().then(setState).catch(() => {});
    const un = player.onState(setState);
    return () => {
      un.then((f) => f());
    };
  }, []);

  useEffect(() => {
    data.surah(surahNo).then(setSurah).catch(() => setSurah(null));
    safeSet(KEY_SURAH, String(surahNo));
  }, [surahNo]);

  useEffect(() => safeSet(KEY_RECITER, reciter), [reciter]);
  useEffect(() => safeSet(KEY_MODE, JSON.stringify(mode)), [mode]);

  // Mode lanjut bisa pindah surah sendiri; jendela utama ikut menampilkan surah yang sedang diputar.
  useEffect(() => {
    if (state && state.status !== "idle" && state.surah > 0 && index.some((s) => s.number === state.surah)) {
      setSurahNo(state.surah);
    }
  }, [state?.surah, state?.status, index]);

  // Mode dipegang inti Rust; ikuti bila diubah dari jendela lain.
  useEffect(() => {
    if (state?.mode) setMode(state.mode);
  }, [state?.mode?.kind, state?.mode?.kind === "range" ? `${state.mode.from}-${state.mode.to}` : ""]);

  const changeMode = (next: PlayMode) => {
    setMode(next);
    player.setMode(next);
  };

  const play = (start = 1) => {
    if (!surah) return;
    const from = mode.kind === "range" && start === 1 ? mode.from : start;
    player.play(surah.number, from, reciter);
  };

  // Mode Layar Penuh (F2-11): jendela utama masuk layar penuh, panel Ambient disembunyikan selama itu.
  const [full, setFull] = useState(false);
  const enterFull = () => {
    setFull(true);
    panel.setSuppressed(true).catch(() => {});
    getCurrentWindow().setFullscreen(true).catch(() => {});
  };
  const exitFull = () => {
    setFull(false);
    getCurrentWindow().setFullscreen(false).catch(() => {});
    panel.setSuppressed(false).catch(() => {});
  };
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== "F11") return;
      e.preventDefault();
      if (full) exitFull();
      else enterFull();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  const toggleClickThrough = async () => {
    const next = !clickThrough;
    await panel.setClickThrough(next);
    setClickThrough(next);
  };

  const playing = state?.status === "playing";
  const current = state && state.surah === surahNo ? state.ayah : -1;
  const active = useActiveWords(state?.reciter, state?.surah, playing);

  // Daftar ayat ikut menampilkan ayat yang sedang dibaca (F1-27), kecuali pengguna baru saja menggulir sendiri.
  const manualScroll = useManualScrollPause("window");
  useEffect(() => {
    if (current < 0 || manualScroll()) return;
    const row = document.querySelector<HTMLElement>(".ayah.active");
    if (!row) return;
    const r = row.getBoundingClientRect();
    if (r.top >= 80 && r.bottom <= window.innerHeight - 40) return;
    row.scrollIntoView({ block: "center", behavior: scrollBehavior() });
  }, [current, surahNo]);

  if (full) {
    return <Fullscreen state={state} surahNo={surahNo} onStart={() => play(1)} onExit={exitFull} />;
  }

  return (
    <main>
      <header>
        <div>
          <h1>Ziyadah</h1>
          <p className="sub">Beta · v0.0.11</p>
        </div>
        <div className="status">
          {state?.status === "loading" && "Memuat..."}
          {state?.buffering && " Mengunduh audio..."}
          {state?.error && <span className="err">{state.error}</span>}
        </div>
      </header>

      <UpdateBanner />

      {/* Status tashih ditampilkan jujur sejak awal dan selalu terlihat (PRD: teks Qur'an sebagai amanah).
          Isinya sama dengan PERNYATAAN.txt yang tampil saat instal; perbarui keduanya bersamaan. */}
      <details className="tashih">
        <summary>
          <strong>Belum ditashih LPMQ.</strong> Teks dari Tanzil (Mushaf Madinah), tidak diubah. Selengkapnya
        </summary>
        <ul>
          <li>
            Ziyadah belum ditashih oleh Lajnah Pentashihan Mushaf Al-Qur'an (LPMQ) Kementerian Agama RI. Kami sedang
            mengurus komunikasi dengan LPMQ tentang proses tashih untuk aplikasi ini.
          </li>
          <li>
            Teks diambil apa adanya dari Tanzil Project (tanzil.net), Uthmani versi 1.1, dan dicocokkan dengan checksum
            sumbernya setiap kali aplikasi dibangun. Karena mengikuti Mushaf Madinah, sebagian tanda baca dan tanda
            waqaf bisa berbeda dari Mushaf Standar Indonesia.
          </li>
          <li>
            Ziyadah masih tahap uji coba. Gunakan mushaf yang sudah ditashih sebagai rujukan utama, terutama untuk
            menghafal.
          </li>
          <li>
            Temukan kesalahan? Laporkan dengan nama surah dan nomor ayat di github.com/visia-id/ziyadah/issues.
          </li>
        </ul>
      </details>

      {dataError ? (
        <section className="card notice">
          <strong>Data Qur'an belum ada.</strong>
          <p>Jalankan <code>npm run fetch-data</code> di folder project, lalu buka ulang aplikasi.</p>
          <p className="muted">{dataError}</p>
        </section>
      ) : (
        <>
          <section className="card controls">
            <label>
              Surah
              <select value={surahNo} onChange={(e) => setSurahNo(Number(e.target.value))}>
                {index.map((s) => (
                  <option key={s.number} value={s.number}>
                    {s.number}. {s.nameLatin} ({s.ayahCount} ayat)
                  </option>
                ))}
              </select>
            </label>
            <label>
              Qari
              <select value={reciter} onChange={(e) => setReciter(e.target.value)}>
                {RECITERS.map((r) => (
                  <option key={r.id} value={r.id}>
                    {r.name}
                  </option>
                ))}
              </select>
            </label>
            <label>
              Mode putar
              <select
                value={mode.kind}
                onChange={(e) => {
                  const kind = e.target.value as PlayMode["kind"];
                  const max = surah?.ayahCount ?? 1;
                  changeMode(kind === "range" ? { kind, from: 1, to: Math.min(5, max) } : { kind });
                }}
              >
                <option value="stop">Berhenti di akhir surah</option>
                <option value="continue">Lanjut ke surah berikutnya</option>
                <option value="repeatAyah">Ulang ayat</option>
                <option value="repeatSurah">Ulang surah</option>
                <option value="range">Ulang rentang ayat</option>
              </select>
            </label>
            {mode.kind === "range" && surah ? (
              <div className="range">
                <label>
                  Dari ayat
                  <input
                    type="number"
                    min={1}
                    max={surah.ayahCount}
                    value={mode.from}
                    onChange={(e) => {
                      const from = clamp(Number(e.target.value), 1, surah.ayahCount);
                      changeMode({ kind: "range", from, to: Math.max(from, mode.to) });
                    }}
                  />
                </label>
                <label>
                  Sampai ayat
                  <input
                    type="number"
                    min={mode.from}
                    max={surah.ayahCount}
                    value={mode.to}
                    onChange={(e) => {
                      const to = clamp(Number(e.target.value), mode.from, surah.ayahCount);
                      changeMode({ kind: "range", from: mode.from, to });
                    }}
                  />
                </label>
              </div>
            ) : (
              <div />
            )}
            <div className="buttons">
              <button onClick={() => player.prev()}>⏮</button>
              {playing ? (
                <button className="primary" onClick={() => player.pause()}>Jeda</button>
              ) : state?.status === "paused" ? (
                <button className="primary" onClick={() => player.resume()}>Lanjut</button>
              ) : (
                <button className="primary" onClick={() => play(1)}>Putar</button>
              )}
              <button onClick={() => player.next()}>⏭</button>
              <button onClick={() => player.stop()}>Stop</button>
            </div>
            <div className="buttons">
              <button className="primary" onClick={enterFull} title="Layar penuh (F11)">Layar penuh</button>
              <button onClick={() => panel.toggle()}>Tampilkan/sembunyikan panel</button>
              <button className={clickThrough ? "on" : ""} onClick={toggleClickThrough}>
                Klik-tembus: {clickThrough ? "aktif" : "mati"}
              </button>
            </div>
          </section>

          {surah && (
            <section className="card ayahs">
              <h2 dir="rtl">{surah.nameAr}</h2>
              {surah.ayahs.map((a) => (
                <button
                  key={a.n}
                  className={`ayah ${current === a.n ? "active" : ""}`}
                  onClick={() => play(a.n)}
                  title="Putar mulai ayat ini"
                >
                  <span className="n">{a.n}</span>
                  <span className="ar" dir="rtl">
                    <ArabicWords
                      text={a.ar}
                      range={active && active.surah === surahNo && active.ayah === a.n ? active.range : null}
                    />
                  </span>
                  <span className="id">{a.id}</span>
                </button>
              ))}
            </section>
          )}
        </>
      )}

      <footer className="muted">
        Teks: Tanzil Project (tanzil.net), Uthmani 1.1 · Terjemah: Kementerian Agama RI · Audio: EveryAyah ·
        Timing per kata: quran-align (Collin Fair, CC BY 4.0).
        Pratinjau: belum ditashih LPMQ, belum untuk diedarkan luas.
      </footer>
    </main>
  );
}

function clamp(n: number, min: number, max: number) {
  return Number.isFinite(n) ? Math.min(max, Math.max(min, Math.round(n))) : min;
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
