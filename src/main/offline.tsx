import { useEffect, useRef, useState } from "react";
import { offline, RECITERS, type OfflineProgress, type OfflineScope, type SurahIndexItem } from "../shared/player";

// Unduh audio untuk didengar tanpa internet (F1-11). Pekerjaan berjalan di inti Rust; di sini hanya kendali
// dan tampilan kemajuan. Satu pekerjaan berjalan pada satu waktu.

/** Jumlah berkas ayat yang sudah tersimpan per surah untuk qari ini; diperbarui saat unduhan berjalan dan selesai. */
export function useOfflineCounts(reciter: string, progress: OfflineProgress | null) {
  const [counts, setCounts] = useState<number[]>([]);
  const refresh = () => offline.counts(reciter).then(setCounts).catch(() => {});
  useEffect(() => {
    refresh();
  }, [reciter, progress?.status]);
  // Selama berjalan, cukup beberapa detik sekali: membaca folder ribuan berkas tidak perlu tiap event.
  useEffect(() => {
    if (progress?.status !== "running") return;
    const t = setInterval(refresh, 3000);
    return () => clearInterval(t);
  }, [reciter, progress?.status]);
  return counts;
}

export function useOfflineProgress() {
  const [progress, setProgress] = useState<OfflineProgress | null>(null);
  useEffect(() => {
    offline.status().then(setProgress).catch(() => {});
    const un = offline.onProgress(setProgress);
    return () => {
      un.then((f) => f());
    };
  }, []);
  return progress;
}

function scopeLabel(scope: OfflineScope, index: SurahIndexItem[]) {
  if (scope.kind === "surah") return index.find((s) => s.number === scope.surah)?.nameLatin ?? `Surah ${scope.surah}`;
  if (scope.kind === "juz") return `Juz ${scope.juz}`;
  return "Seluruh mushaf";
}

const reciterName = (id: string) => RECITERS.find((r) => r.id === id)?.name ?? id;

export function OfflinePanel({
  reciter,
  surah,
  index,
  counts,
  progress,
}: {
  reciter: string;
  surah: SurahIndexItem | undefined;
  index: SurahIndexItem[];
  counts: number[];
  progress: OfflineProgress | null;
}) {
  const [juz, setJuz] = useState(30);
  const [error, setError] = useState<string | null>(null);
  const start = (scope: OfflineScope) => {
    setError(null);
    offline.start(reciter, scope).catch((e) => setError(String(e)));
  };
  const startAll = () => {
    const ok = window.confirm(
      `Unduh seluruh mushaf (6.236 berkas) untuk ${reciterName(reciter)}?\n\n` +
        "Ukurannya sekitar 1 sampai 1,5 GB tergantung qari, dan bisa makan waktu lama. " +
        "Unduhan bisa dibatalkan dan dilanjutkan kapan saja.",
    );
    if (ok) start({ kind: "all" });
  };

  const saved = surah ? Math.min(counts[surah.number - 1] ?? 0, surah.ayahCount) : 0;
  const running = progress?.status === "running";
  const job = progress?.job;
  const what = job ? `${scopeLabel(job.scope, index)} · ${reciterName(job.reciter)}` : "";
  const percent = progress && progress.total > 0 ? Math.round((progress.done / progress.total) * 100) : 0;

  // Terbuka sendiri saat unduhan mulai atau ada unduhan tertunda; setelah itu bebas dilipat pengguna.
  const box = useRef<HTMLDetailsElement>(null);
  useEffect(() => {
    if (box.current && (running || progress?.status === "pending")) box.current.open = true;
  }, [running, progress?.status]);

  return (
    <details className="card offline" ref={box}>
      <summary>
        <strong>Unduh untuk offline</strong>
        {running && ` · ${percent}%`}
      </summary>
      <p className="muted">
        Audio yang sudah diunduh diputar tanpa internet. Qari: {reciterName(reciter)}.
        {surah && ` ${surah.nameLatin}: ${saved} dari ${surah.ayahCount} ayat sudah tersimpan.`}
      </p>

      {progress && progress.status !== "idle" && job && (
        <div className={`offline-status ${progress.status}`}>
          {running && (
            <>
              <div>
                Mengunduh {what}: {progress.done} dari {progress.total} berkas
                {progress.failed > 0 && `, ${progress.failed} gagal`}
              </div>
              <progress max={progress.total} value={progress.done} />
            </>
          )}
          {progress.status === "pending" && <div>Unduhan sebelumnya belum selesai: {what}.</div>}
          {progress.status === "interrupted" && (
            <div>
              {progress.error} Tersimpan {progress.done} dari {progress.total} berkas ({what}).
            </div>
          )}
          {progress.status === "failed" && (
            <div>
              Selesai dengan {progress.failed} berkas gagal ({what}). Coba lagi untuk mengunduh yang gagal saja.
            </div>
          )}
          {progress.status === "done" && <div>Selesai: {what} tersimpan untuk didengar tanpa internet.</div>}
          {progress.status === "cancelled" && <div>Unduhan {what} dibatalkan.</div>}
          <div className="buttons">
            {running && <button onClick={() => offline.cancel()}>Batalkan</button>}
            {["pending", "interrupted", "failed"].includes(progress.status) && (
              <>
                <button className="primary" onClick={() => offline.resume().catch((e) => setError(String(e)))}>
                  {progress.status === "failed" ? "Coba lagi" : "Lanjutkan"}
                </button>
                <button onClick={() => offline.cancel()}>Batalkan</button>
              </>
            )}
          </div>
        </div>
      )}

      <div className="buttons">
        <button disabled={!surah || running} onClick={() => surah && start({ kind: "surah", surah: surah.number })}>
          Unduh {surah?.nameLatin ?? "surah ini"}
        </button>
        <span className="juz">
          <select value={juz} disabled={running} onChange={(e) => setJuz(Number(e.target.value))}>
            {Array.from({ length: 30 }, (_, i) => (
              <option key={i + 1} value={i + 1}>
                Juz {i + 1}
              </option>
            ))}
          </select>
          <button disabled={running} onClick={() => start({ kind: "juz", juz })}>
            Unduh juz
          </button>
        </span>
        <button disabled={running} onClick={startAll}>
          Unduh seluruh mushaf
        </button>
      </div>
      {error && <p className="err">{error}</p>}
    </details>
  );
}
