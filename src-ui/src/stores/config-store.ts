import { createStore } from "solid-js/store";
import type { AppConfig } from "../lib/types";
import { getConfig, updateConfig as updateConfigCmd } from "../lib/commands";
import { setTheme } from "./theme-store";

const [config, setConfig] = createStore<AppConfig>({
  default_download_path: "",
  max_concurrent_downloads: 4,
  max_concurrent_tasks: 2,
  chunk_size: 10485760,
  timeout: 30,
  user_agent: "YuShi/1.0",
  proxy: null,
  speed_limit: null,
  theme: "system",
});

export async function loadConfig() {
  const cfg = await getConfig();
  setConfig(cfg);
  setTheme(cfg.theme);
}

export async function saveConfig(updates: Partial<AppConfig>) {
  const newConfig = { ...config, ...updates };
  await updateConfigCmd(newConfig);
  setConfig(newConfig);
  if (updates.theme) {
    setTheme(updates.theme);
  }
}

export { config };
