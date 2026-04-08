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
    setUpdateStatus("检查中...");
    try {
      const update = await check();
      if (update) {
        setUpdateStatus(`发现新版本: ${update.version}`);
        if (confirm(`发现新版本 ${update.version}，是否立即更新？`)) {
          await update.downloadAndInstall();
        }
      } else {
        setUpdateStatus("已是最新版本");
      }
    } catch (e) {
      setUpdateStatus(`检查失败: ${e}`);
    }
  }

  return (
    <div>
      <h2 class="text-2xl font-bold mb-6">设置</h2>

      <div class="space-y-6 max-w-2xl">
        {/* Download Settings */}
        <div class="card bg-base-100 shadow-sm">
          <div class="card-body">
            <h3 class="card-title text-base">下载</h3>
            <div class="form-control">
              <label class="label"><span class="label-text">默认下载路径</span></label>
              <div class="flex gap-2">
                <input type="text" class="input input-bordered flex-1" value={config.default_download_path} readOnly />
                <button class="btn btn-ghost" onClick={pickDefaultDir}>浏览</button>
              </div>
            </div>
            <div class="grid grid-cols-2 gap-4">
              <div class="form-control">
                <label class="label"><span class="label-text">每任务最大连接数</span></label>
                <input
                  type="number" min="1" max="32"
                  class="input input-bordered"
                  value={config.max_concurrent_downloads}
                  onChange={(e) => handleSave("max_concurrent_downloads", parseInt(e.currentTarget.value))}
                />
              </div>
              <div class="form-control">
                <label class="label"><span class="label-text">最大同时任务数</span></label>
                <input
                  type="number" min="1" max="16"
                  class="input input-bordered"
                  value={config.max_concurrent_tasks}
                  onChange={(e) => handleSave("max_concurrent_tasks", parseInt(e.currentTarget.value))}
                />
              </div>
            </div>
          </div>
        </div>

        {/* Network Settings */}
        <div class="card bg-base-100 shadow-sm">
          <div class="card-body">
            <h3 class="card-title text-base">网络</h3>
            <div class="form-control">
              <label class="label"><span class="label-text">代理</span></label>
              <input
                type="text"
                class="input input-bordered"
                placeholder="socks5://127.0.0.1:1080"
                value={config.proxy ?? ""}
                onChange={(e) => handleSave("proxy", e.currentTarget.value || null)}
              />
            </div>
            <div class="form-control">
              <label class="label"><span class="label-text">User Agent</span></label>
              <input
                type="text"
                class="input input-bordered"
                value={config.user_agent}
                onChange={(e) => handleSave("user_agent", e.currentTarget.value)}
              />
            </div>
            <div class="grid grid-cols-2 gap-4">
              <div class="form-control">
                <label class="label"><span class="label-text">超时 (秒)</span></label>
                <input
                  type="number" min="5"
                  class="input input-bordered"
                  value={config.timeout}
                  onChange={(e) => handleSave("timeout", parseInt(e.currentTarget.value))}
                />
              </div>
              <div class="form-control">
                <label class="label"><span class="label-text">分块大小 (MB)</span></label>
                <input
                  type="number" min="1"
                  class="input input-bordered"
                  value={Math.round(config.chunk_size / 1048576)}
                  onChange={(e) => handleSave("chunk_size", parseInt(e.currentTarget.value) * 1048576)}
                />
              </div>
            </div>
          </div>
        </div>

        {/* Appearance */}
        <div class="card bg-base-100 shadow-sm">
          <div class="card-body">
            <h3 class="card-title text-base">外观</h3>
            <div class="form-control">
              <label class="label"><span class="label-text">主题</span></label>
              <ThemeToggle
                value={config.theme}
                onChange={(theme) => handleSave("theme", theme)}
              />
            </div>
          </div>
        </div>

        {/* About */}
        <div class="card bg-base-100 shadow-sm">
          <div class="card-body">
            <h3 class="card-title text-base">关于</h3>
            <p class="text-sm text-base-content/60">驭时 (YuShi) v0.1.0</p>
            <div class="flex items-center gap-4 mt-2">
              <button class="btn btn-sm btn-ghost" onClick={checkUpdate}>检查更新</button>
              {updateStatus() && <span class="text-sm text-base-content/60">{updateStatus()}</span>}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default SettingsPage;
