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

// Jendela utama untuk spike: pilih surah dan qari, kendalikan pemutaran,
// tampilkan/sembunyikan panel Ambient. Mode Tilawah dan Hafalan menyusul di fase berikutnya.

function App() {
  const [index, setIndex] = useState<SurahIndexItem[]>([]);
  const [dataError, setDataError] = useState<string | null>(null);
  const [surahNo, setSurahNo] = useState(67);
  const [surah, setSurah] = useState<Surah | null>(null);
  const [reciter, setReciter] = useState(RECITERS[0].id);
  const [state, setState] = useState<PlayerState | null>(null);
  const [clickThrough, setClickThrough] = useState(false);
  const [mode, setMode] = useState<PlayMode>({ kind: "stop" });

  useEffect(() => {
    data
      .index()
      .then((items) => {
        setIndex(items);
        if (!items.some((s) => s.number === 67) && items[0]) setSurahNo(items[0].number);
      })
      .catch((e) => setDataError(String(e.message ?? e)));
    player.state().then(setState).catch(() => {});
    const un = player.onState(setState);
    return () => {
      un.then((f) => f());
    };
  }, []);

  useEffect(() => {
    data.surah(surahNo).then(setSurah).catch(() => setSurah(null));
  }, [surahNo]);

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

  const toggleClickThrough = async () => {
    const next = !clickThrough;
    await panel.setClickThrough(next);
    setClickThrough(next);
  };

  const playing = state?.status === "playing";
  const current = state && state.surah === surahNo ? state.ayah : -1;

  return (
    <main>
      <header>
        <div>
          <h1>Ziyadah</h1>
          <p className="sub">Prototipe mode Ambient · v0.0.1</p>
        </div>
        <div className="status">
          {state?.status === "loading" && "Memuat..."}
          {state?.buffering && " Mengunduh audio..."}
          {state?.error && <span className="err">{state.error}</span>}
        </div>
      </header>

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
                  <span className="ar" dir="rtl">{a.ar}</span>
                  <span className="id">{a.id}</span>
                </button>
              ))}
            </section>
          )}
        </>
      )}

      <footer className="muted">
        Teks: Tanzil Project (tanzil.net), Uthmani 1.1 · Terjemah: Kementerian Agama RI · Audio: EveryAyah.
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
