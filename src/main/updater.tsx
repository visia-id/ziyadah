import { useEffect, useState } from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

// Pembaruan otomatis (F4-03). Rilis ditandatangani dengan kunci milik pemelihara dan diambil dari
// GitHub Releases; pembaruan tanpa tanda tangan yang cocok ditolak oleh plugin.
// Pengecekan diam-diam: tanpa internet atau belum ada rilis, tidak ada pesan apa pun.

const FIRST_CHECK_MS = 5_000;
const RECHECK_MS = 6 * 60 * 60 * 1000;

type Phase = { kind: "idle" } | { kind: "ready"; update: Update } | { kind: "installing"; percent: number | null } | { kind: "error"; message: string };

export function UpdateBanner() {
  const [phase, setPhase] = useState<Phase>({ kind: "idle" });

  useEffect(() => {
    let alive = true;
    const run = () =>
      check()
        .then((update) => {
          if (alive && update) setPhase((p) => (p.kind === "idle" ? { kind: "ready", update } : p));
        })
        .catch(() => {});
    const first = setTimeout(run, FIRST_CHECK_MS);
    const every = setInterval(run, RECHECK_MS);
    return () => {
      alive = false;
      clearTimeout(first);
      clearInterval(every);
    };
  }, []);

  if (phase.kind === "idle") return null;

  const install = async (update: Update) => {
    let total = 0;
    let done = 0;
    setPhase({ kind: "installing", percent: null });
    try {
      await update.downloadAndInstall((e) => {
        if (e.event === "Started") total = e.data.contentLength ?? 0;
        if (e.event === "Progress") {
          done += e.data.chunkLength;
          setPhase({ kind: "installing", percent: total ? Math.round((done / total) * 100) : null });
        }
      });
      await relaunch();
    } catch (e) {
      setPhase({ kind: "error", message: String(e) });
    }
  };

  return (
    <section className="update">
      {phase.kind === "ready" && (
        <>
          <span>
            <strong>Ziyadah {phase.update.version} tersedia.</strong> Murottal akan berhenti sebentar saat aplikasi
            dimulai ulang.
          </span>
          <button className="primary" onClick={() => install(phase.update)}>
            Pasang dan mulai ulang
          </button>
        </>
      )}
      {phase.kind === "installing" && (
        <span>Mengunduh pembaruan{phase.percent !== null ? ` ${phase.percent}%` : "..."}</span>
      )}
      {phase.kind === "error" && (
        <span className="err">Pembaruan gagal: {phase.message}. Coba lagi nanti atau unduh dari situs Ziyadah.</span>
      )}
    </section>
  );
}
