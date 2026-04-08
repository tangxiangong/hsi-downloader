import { createSignal } from "solid-js";
import type { AppTheme } from "../lib/types";

const [theme, setThemeSignal] = createSignal<AppTheme>("system");

function applyTheme(t: AppTheme) {
  let resolved: string;
  if (t === "system") {
    resolved = window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "yushi-dark"
      : "yushi-light";
  } else {
    resolved = t === "dark" ? "yushi-dark" : "yushi-light";
  }
  document.documentElement.setAttribute("data-theme", resolved);
}

if (typeof window !== "undefined") {
  window
    .matchMedia("(prefers-color-scheme: dark)")
    .addEventListener("change", () => {
      if (theme() === "system") {
        applyTheme("system");
      }
    });
}

export function setTheme(t: AppTheme) {
  setThemeSignal(t);
  applyTheme(t);
}

export { theme };
