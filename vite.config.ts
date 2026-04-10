import { defineConfig } from "vite";
import solid from "vite-plugin-solid";
import solidSvg from "vite-plugin-solid-svg";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [solid(), solidSvg(), tailwindcss()],
  server: {
    port: 5173,
    strictPort: true,
  },
  build: {
    target: "esnext",
  },
});
