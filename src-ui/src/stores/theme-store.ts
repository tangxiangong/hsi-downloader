import { createSignal } from "solid-js";
import type { AppTheme } from "../lib/types";

const [theme, setThemeSignal] = createSignal<AppTheme>("system");

function applyTheme(t: AppTheme) {
  let resolved: "light" | "dark";
  if (t === "system") {
    resolved = window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  } else {
    resolved = t;
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
