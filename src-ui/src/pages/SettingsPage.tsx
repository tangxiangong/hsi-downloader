import { type Component, createSignal } from "solid-js";
import { config, saveConfig } from "../stores/config-store";
import ThemeToggle from "../components/ThemeToggle";
import type { AppTheme } from "../lib/types";
import { open } from "@tauri-apps/plugin-dialog";
import { check } from "@tauri-apps/plugin-updater";

const SettingsPage: Component = () => {
  const [saving, setSaving] = createSignal(false);
  const [updateStatus, setUpdateStatus] = createSignal("");

  async function pickDefaultDir() {
    const selected = await open({ directory: true, defaultPath: config.default_download_path });
    if (selected) await saveConfig({ default_download_path: selected });
  }

  async function handleSave(field: string, value: unknown) {
    setSaving(true);
    try {
      await saveConfig({ [field]: value } as any);
    } finally {
      setSaving(false);
    }
  }

  async function checkUpdate() {
    setUpdateStatus("\u68c0\u67e5\u4e2d...");
    try {
      const update = await check();
      if (update) {
        setUpdateStatus(`\u53d1\u73b0\u65b0\u7248\u672c: ${update.version}`);
        if (confirm(`\u53d1\u73b0\u65b0\u7248\u672c ${update.version}\uff0c\u662f\u5426\u7acb\u5373\u66f4\u65b0\uff1f`)) {
          await update.downloadAndInstall();
        }
      } else {
        setUpdateStatus("\u5df2\u662f\u6700\u65b0\u7248\u672c");
      }
    } catch (e) {
      setUpdateStatus(`\u68c0\u67e5\u5931\u8d25: ${e}`);
    }
  }

  return (
    <div>
      <div class="mb-5">
        <h2 class="text-xl font-bold">{"\u8bbe\u7f6e"}</h2>
        <p class="text-xs text-base-content/40 mt-0.5">{"\u5e94\u7528\u914d\u7f6e"}</p>
      </div>

      <div class="space-y-4">
        <div class="card bg-base-100 border border-base-300">
          <div class="card-body p-5">
            <h3 class="text-sm font-semibold mb-3">{"\u4e0b\u8f7d"}</h3>
            <div class="form-control mb-3">
              <label class="label"><span class="label-text text-xs text-base-content/60">{"\u9ed8\u8ba4\u4e0b\u8f7d\u8def\u5f84"}</span></label>
              <div class="flex gap-2">
                <input type="text" class="input input-bordered input-sm flex-1" value={config.default_download_path} readOnly />
                <button class="btn btn-ghost btn-sm" onClick={pickDefaultDir}>{"\u6d4f\u89c8"}</button>
              </div>
            </div>
            <div class="grid grid-cols-2 gap-3">
              <div class="form-control">
                <label class="label"><span class="label-text text-xs text-base-content/60">{"\u6bcf\u4efb\u52a1\u6700\u5927\u8fde\u63a5\u6570"}</span></label>
                <input
                  type="number" min="1" max="32"
                  class="input input-bordered input-sm"
                  value={config.max_concurrent_downloads}
                  onChange={(e) => handleSave("max_concurrent_downloads", parseInt(e.currentTarget.value))}
                />
              </div>
              <div class="form-control">
                <label class="label"><span class="label-text text-xs text-base-content/60">{"\u6700\u5927\u540c\u65f6\u4efb\u52a1\u6570"}</span></label>
                <input
                  type="number" min="1" max="16"
                  class="input input-bordered input-sm"
                  value={config.max_concurrent_tasks}
                  onChange={(e) => handleSave("max_concurrent_tasks", parseInt(e.currentTarget.value))}
                />
              </div>
            </div>
          </div>
        </div>

        <div class="card bg-base-100 border border-base-300">
          <div class="card-body p-5">
            <h3 class="text-sm font-semibold mb-3">{"\u7f51\u7edc"}</h3>
            <div class="form-control mb-3">
              <label class="label"><span class="label-text text-xs text-base-content/60">{"\u4ee3\u7406"}</span></label>
              <input
                type="text"
                class="input input-bordered input-sm"
                placeholder="socks5://127.0.0.1:1080"
                value={config.proxy ?? ""}
                onChange={(e) => handleSave("proxy", e.currentTarget.value || null)}
              />
            </div>
            <div class="form-control mb-3">
              <label class="label"><span class="label-text text-xs text-base-content/60">User Agent</span></label>
              <input
                type="text"
                class="input input-bordered input-sm"
                value={config.user_agent}
                onChange={(e) => handleSave("user_agent", e.currentTarget.value)}
              />
            </div>
            <div class="grid grid-cols-2 gap-3">
              <div class="form-control">
                <label class="label"><span class="label-text text-xs text-base-content/60">{"\u8d85\u65f6 (\u79d2)"}</span></label>
                <input
                  type="number" min="5"
                  class="input input-bordered input-sm"
                  value={config.timeout}
                  onChange={(e) => handleSave("timeout", parseInt(e.currentTarget.value))}
                />
              </div>
              <div class="form-control">
                <label class="label"><span class="label-text text-xs text-base-content/60">{"\u5206\u5757\u5927\u5c0f (MB)"}</span></label>
                <input
                  type="number" min="1"
                  class="input input-bordered input-sm"
                  value={Math.round(config.chunk_size / 1048576)}
                  onChange={(e) => handleSave("chunk_size", parseInt(e.currentTarget.value) * 1048576)}
                />
              </div>
            </div>
          </div>
        </div>

        <div class="card bg-base-100 border border-base-300">
          <div class="card-body p-5">
            <h3 class="text-sm font-semibold mb-3">{"\u5916\u89c2"}</h3>
            <div class="form-control">
              <label class="label"><span class="label-text text-xs text-base-content/60">{"\u4e3b\u9898"}</span></label>
              <ThemeToggle
                value={config.theme}
                onChange={(theme) => handleSave("theme", theme)}
              />
            </div>
          </div>
        </div>

        <div class="card bg-base-100 border border-base-300">
          <div class="card-body p-5">
            <h3 class="text-sm font-semibold mb-3">{"\u5173\u4e8e"}</h3>
            <p class="text-xs text-base-content/40">{"\u9a6d\u65f6 (YuShi) v0.1.0"}</p>
            <div class="flex items-center gap-3 mt-2">
              <button class="btn btn-ghost btn-sm" onClick={checkUpdate}>{"\u68c0\u67e5\u66f4\u65b0"}</button>
              {updateStatus() && <span class="text-xs text-base-content/50">{updateStatus()}</span>}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default SettingsPage;
