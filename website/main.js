// Situs Ziyadah: tanpa cookie, tanpa analitik.

// Panel contoh di hero: Al-Insyirah 5 dan 6 apa adanya dari Tanzil (text_display di quran.db), terjemah Kemenag,
// dan timing per kata Alafasy dari quran-align. Diputar bergantian seperti murottal di aplikasi.
const AYAT = [
  {
    n: 5,
    ar: "فَإِنَّ مَعَ ٱلْعُسْرِ يُسْرًا",
    id: "Karena sesungguhnya sesudah kesulitan itu ada kemudahan,",
    segments: [[0, 1, 30, 1580], [1, 2, 1590, 2030], [2, 3, 2040, 2860], [3, 4, 2870, 3880]],
  },
  {
    n: 6,
    ar: "إِنَّ مَعَ ٱلْعُسْرِ يُسْرًا",
    id: "sesungguhnya sesudah kesulitan itu ada kemudahan.",
    segments: [[0, 1, 30, 1330], [1, 2, 1340, 1730], [2, 3, 1740, 2610], [3, 4, 2620, 3540]],
  },
];
const MARK = /^[\u06D6-\u06ED]+$/u;
const GAP_BETWEEN_MS = 700;
const GAP_LOOP_MS = 2600;
const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

const arEl = document.getElementById("hero-ayah");
const trEl = document.getElementById("hero-tr");
const metaEl = document.getElementById("hero-meta");

function render(ayah) {
  arEl.replaceChildren();
  const words = [];
  ayah.ar.split(" ").forEach((token, i) => {
    if (i > 0) arEl.append(" ");
    const span = document.createElement("span");
    span.textContent = token;
    if (!MARK.test(token)) {
      span.className = "w";
      words.push(span);
    }
    arEl.append(span);
  });
  arEl.append(" ", `﴿${String(ayah.n).replace(/\d/g, (d) => "٠١٢٣٤٥٦٧٨٩"[d])}﴾`);
  trEl.textContent = ayah.id;
  metaEl.textContent = `Al-Insyirah · ${ayah.n}/8`;
  if (!reduceMotion) {
    arEl.classList.remove("enter");
    void arEl.offsetWidth;
    arEl.classList.add("enter");
  }
  return words;
}

function play(index) {
  const ayah = AYAT[index];
  const words = render(ayah);
  if (reduceMotion) return;
  const end = ayah.segments[ayah.segments.length - 1][3];
  const start = performance.now();
  let current = -1;
  const tick = (now) => {
    const t = now - start;
    let active = -1;
    for (const [ws, , s, e] of ayah.segments) if (t >= s && t <= e) active = ws;
    if (active !== current) {
      words.forEach((w, i) => w.classList.toggle("on", i === active));
      current = active;
    }
    if (t <= end) requestAnimationFrame(tick);
    else {
      const last = index === AYAT.length - 1;
      setTimeout(() => play(last ? 0 : index + 1), last ? GAP_LOOP_MS : GAP_BETWEEN_MS);
    }
  };
  requestAnimationFrame(tick);
}

// Tombol unduh menunjuk langsung ke installer rilis terbaru; bila API GitHub tidak terjangkau,
// tetap mengarah ke halaman rilis terbaru.
async function linkLatestInstaller() {
  try {
    const res = await fetch("https://api.github.com/repos/visia-id/ziyadah/releases/latest");
    if (!res.ok) return;
    const release = await res.json();
    const asset = release.assets.find((a) => a.name.endsWith("-setup.exe"));
    if (!asset) return;
    document.querySelectorAll("a.btn-download").forEach((a) => (a.href = asset.browser_download_url));
    const mb = (asset.size / 1024 / 1024).toFixed(1).replace(".", ",");
    document.getElementById("download-meta").textContent =
      `Versi ${release.tag_name.replace(/^v/, "")} · Windows 10 dan 11 · ${mb} MB · pembaruan otomatis`;
  } catch {
    /* tetap memakai tautan ke halaman rilis */
  }
}

play(0);
linkLatestInstaller();
