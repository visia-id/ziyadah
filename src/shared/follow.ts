import { useEffect, useRef, type RefObject } from "react";

// Auto-scroll yang mengikuti bacaan (F1-27), dipakai panel dan jendela utama.
// Bila pengguna menggulir sendiri, auto-scroll berhenti sebentar supaya tidak berebut.

const MANUAL_PAUSE_MS = 5000;

export function prefersReducedMotion() {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

export function scrollBehavior(): ScrollBehavior {
  return prefersReducedMotion() ? "auto" : "smooth";
}

/** Mengembalikan fungsi yang bernilai true selama pengguna baru saja menggulir sendiri. */
export function useManualScrollPause(target: RefObject<HTMLElement | null> | "window") {
  const until = useRef(0);
  const el: HTMLElement | Window | null = target === "window" ? window : target.current;
  useEffect(() => {
    if (!el) return;
    const mark = () => {
      until.current = Date.now() + MANUAL_PAUSE_MS;
    };
    // Di panel, menyeret scrollbar juga dihitung; di jendela utama klik biasa (Putar, pilih ayat) tidak.
    const events = target === "window" ? (["wheel", "touchmove", "keydown"] as const) : (["wheel", "touchmove", "keydown", "pointerdown"] as const);
    events.forEach((e) => el.addEventListener(e, mark, { passive: true }));
    return () => events.forEach((e) => el.removeEventListener(e, mark));
  }, [el]);
  return () => Date.now() < until.current;
}

/** Tandai sisi yang masih punya teks tersembunyi, untuk efek memudar di tepi atas/bawah. */
export function updateScrollEdges(box: HTMLElement | null) {
  if (!box) return;
  box.classList.toggle("can-up", box.scrollTop > 2);
  box.classList.toggle("can-down", box.scrollTop + box.clientHeight < box.scrollHeight - 2);
}
