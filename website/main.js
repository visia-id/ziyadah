// Situs Ziyadah: tanpa cookie, tanpa analitik.

// Contoh panel di hero: Ar-Ra'd 28 apa adanya dari Tanzil (text_display di quran.db), terjemah Kemenag,
// dan timing per kata Alafasy dari quran-align. Indeks kata tidak menghitung token tanda waqaf.
const AYAH = {
  ar: "ٱلَّذِينَ ءَامَنُوا۟ وَتَطْمَئِنُّ قُلُوبُهُم بِذِكْرِ ٱللَّهِ ۗ أَلَا بِذِكْرِ ٱللَّهِ تَطْمَئِنُّ ٱلْقُلُوبُ",
  id: "(yaitu) orang-orang yang beriman dan hati mereka manjadi tenteram dengan mengingat Allah. Ingatlah, hanya dengan mengingati Allah-lah hati menjadi tenteram.",
  segments: [[0, 1, 60, 1430], [1, 2, 1440, 2620], [2, 3, 2630, 4710], [3, 4, 4720, 6620], [4, 5, 6630, 7540], [5, 6, 7550, 10890], [6, 7, 10900, 11530], [7, 8, 11540, 12530], [8, 9, 12540, 13350], [9, 10, 13360, 15500], [10, 11, 15510, 17950]],
};
const MARK = /^[ۖ-ۭ]+$/u;
const PAUSE_MS = 2500;

function renderAyah() {
  const box = document.getElementById("hero-ayah");
  document.getElementById("hero-tr").textContent = AYAH.id;
  const words = [];
  AYAH.ar.split(" ").forEach((token, i) => {
    if (i > 0) box.append(" ");
    const span = document.createElement("span");
    span.textContent = token;
    if (!MARK.test(token)) {
      span.className = "w";
      words.push(span);
    }
    box.append(span);
  });
  return words;
}

function playHighlight(words) {
  if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
  const total = AYAH.segments[AYAH.segments.length - 1][3] + PAUSE_MS;
  let start = performance.now();
  let current = -1;
  const tick = (now) => {
    let t = now - start;
    if (t > total) {
      start = now;
      t = 0;
    }
    let active = -1;
    for (const [ws, , s, e] of AYAH.segments) if (t >= s && t <= e) active = ws;
    if (active !== current) {
      words.forEach((w, i) => w.classList.toggle("on", i === active));
      current = active;
    }
    requestAnimationFrame(tick);
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

playHighlight(renderAyah());
linkLatestInstaller();
