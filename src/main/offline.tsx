import { useEffect, useRef, useState } from "react";
import {
  offline,
  RECITERS,
  type OfflineProgress,
  type OfflineScope,
  type OfflineUsage,
  type SurahIndexItem,
} from "../shared/player";

// Unduh audio untuk didengar tanpa internet (F1-11). Pekerjaan berjalan di inti Rust; di sini hanya kendali
// dan tampilan kemajuan. Satu pekerjaan berjalan pada satu waktu.

/** Jumlah berkas ayat yang sudah tersimpan per surah untuk qari ini; diperbarui saat unduhan berjalan dan selesai. */
export function useOfflineCounts(reciter: string, progress: OfflineProgress | null, version = 0) {
  const [counts, setCounts] = useState<number[]>([]);
  const refresh = () => offline.counts(reciter).then(setCounts).catch(() => {});
  useEffect(() => {
    refresh();
  }, [reciter, progress?.status, version]);
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

/** Folder qari yang tidak lagi ada di daftar (misalnya As-Sudais) tetap ditampilkan supaya bisa dibersihkan. */
const folderName = (id: string) =>
  RECITERS.some((r) => r.id === id) ? reciterName(id) : `${id.replace(/_\d+kbps$/, "").replace(/_/g, " ")} (tidak dipakai lagi)`;

export function formatBytes(bytes: number) {
  if (bytes >= 1024 ** 3) return `${(bytes / 1024 ** 3).toLocaleString("id-ID", { maximumFractionDigits: 1 })} GB`;
  return `${Math.max(1, Math.round(bytes / 1024 ** 2)).toLocaleString("id-ID")} MB`;
}

/** Ruang disk audio per qari dan tombol hapus (F1-12). */
function DiskUsage({ progress, onChanged }: { progress: OfflineProgress | null; onChanged: () => void }) {
  const [usage, setUsage] = useState<OfflineUsage[]>([]);
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const refresh = () => offline.usage().then(setUsage).catch(() => {});
  useEffect(() => {
    refresh();
  }, [progress?.status]);
  useEffect(() => {
    if (progress?.status !== "running") return;
    const t = setInterval(refresh, 5000);
    return () => clearInterval(t);
  }, [progress?.status]);

  const remove = async (u: OfflineUsage) => {
    const ok = window.confirm(
      `Hapus audio ${folderName(u.reciter)} (${u.files.toLocaleString("id-ID")} berkas, ${formatBytes(u.bytes)})?\n\n` +
        "Ayat qari ini akan diunduh lagi saat diputar, jadi butuh internet.",
    );
    if (!ok) return;
    setBusy(u.reciter);
    try {
      const d = await offline.remove(u.reciter);
      setMessage(
        `Terhapus ${d.files.toLocaleString("id-ID")} berkas (${formatBytes(d.bytes)}).` +
          (d.skipped > 0 ? ` ${d.skipped} berkas sedang dipakai dan tidak terhapus; coba lagi setelah murottal berhenti.` : ""),
      );
    } catch (e) {
      setMessage(String(e));
    }
    setBusy(null);
    refresh();
    onChanged();
  };

  const total = usage.reduce((sum, u) => sum + u.bytes, 0);
  return (
    <div className="disk">
      <div className="disk-head">
        <strong>Ruang disk</strong>
        <span className="muted">{usage.length ? `Total ${formatBytes(total)}` : "Belum ada audio tersimpan"}</span>
      </div>
      {usage.map((u) => (
        <div key={u.reciter} className="disk-row">
          <span>{folderName(u.reciter)}</span>
          <span className="muted">
            {u.files.toLocaleString("id-ID")} berkas · {formatBytes(u.bytes)}
          </span>
          <button disabled={busy !== null} onClick={() => remove(u)}>
            {busy === u.reciter ? "Menghapus..." : "Hapus"}
          </button>
        </div>
      ))}
      {message && <p className="muted">{message}</p>}
    </div>
  );
}

export function OfflinePanel({
  reciter,
  surah,
  index,
  counts,
  progress,
  onChanged,
}: {
  reciter: string;
  surah: SurahIndexItem | undefined;
  index: SurahIndexItem[];
  counts: number[];
  progress: OfflineProgress | null;
  /** Audio terhapus: hitungan per surah perlu dibaca ulang. */
  onChanged: () => void;
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
      <DiskUsage progress={progress} onChanged={onChanged} />
    </details>
  );
}
