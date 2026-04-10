import { type Component, type JSX, For, Match, Switch, createMemo, createSignal } from "solid-js";
import { open } from "@tauri-apps/plugin-dialog";
import { check } from "@tauri-apps/plugin-updater";
import ThemeToggle from "../components/ThemeToggle";
import { config, saveConfig } from "../stores/config-store";
import DownloadIcon from "../icons/download.svg";
import GlobeIcon from "../icons/globe.svg";
import MagnetIcon from "../icons/magnet.svg";
import PaletteIcon from "../icons/palette.svg";
import HeartIcon from "../icons/heart.svg";

/* ── Types ───────────────────────────────────────────── */

type TabId = "transfer" | "network" | "bt" | "appearance" | "about";

interface Tab {
  id: TabId;
  label: string;
  icon: Component<JSX.SvgSVGAttributes<SVGSVGElement>>;
}

interface SettingsFieldProps {
  label: string;
  hint?: string;
  children: JSX.Element;
  span?: "full" | "half";
}

/* ── Constants ───────────────────────────────────────── */

const tabs: Tab[] = [
  { id: "transfer", label: "下载", icon: DownloadIcon },
  { id: "network", label: "网络", icon: GlobeIcon },
  { id: "bt", label: "BT", icon: MagnetIcon },
  { id: "appearance", label: "外观", icon: PaletteIcon },
  { id: "about", label: "关于", icon: HeartIcon },
];

const inputClass =
  "input input-bordered h-11 w-full rounded-2xl border-base-300/80 bg-base-100/90 px-4 text-sm shadow-sm transition-colors focus:border-primary/50 focus:outline-none";

/* ── Sub-components ──────────────────────────────────── */

const SettingsField: Component<SettingsFieldProps> = (props) => (
  <div
    class={`rounded-[1.4rem] border border-base-300/70 bg-base-200/60 p-4 ${
      props.span === "full" ? "md:col-span-2" : ""
    }`}
  >
    <div class="mb-3">
      <p class="text-sm font-medium text-base-content">{props.label}</p>
      {props.hint && (
        <p class="mt-1 text-xs leading-5 text-base-content/50">{props.hint}</p>
      )}
    </div>
    {props.children}
  </div>
);

/* ── Main page ───────────────────────────────────────── */

const SettingsPage: Component = () => {
  const [activeTab, setActiveTab] = createSignal<TabId>("transfer");
  const [saving, setSaving] = createSignal(false);
  const [updateStatus, setUpdateStatus] = createSignal("");

  const chunkSizeMb = createMemo(() =>
    Math.max(1, Math.round(config.chunk_size / 1048576)),
  );

  const summaryCards = createMemo(() => [
    {
      label: "默认路径",
      value: config.default_download_path || "未设置",
      meta: "下载保存目录",
    },
    {
      label: "并发策略",
      value: `${config.max_concurrent_downloads} × ${config.max_concurrent_tasks}`,
      meta: "连接数 × 同时任务",
    },
    {
      label: "网络模式",
      value: config.proxy ? "代理已启用" : "直连",
      meta: config.proxy ?? "未配置代理地址",
    },
    {
      label: "主题",
      value:
        config.theme === "system"
          ? "跟随系统"
          : config.theme === "dark"
            ? "深色"
            : "浅色",
      meta: "界面显示模式",
    },
  ]);

  async function pickDefaultDir() {
    const selected = await open({
      directory: true,
      defaultPath: config.default_download_path,
    });
    if (selected) await saveConfig({ default_download_path: selected });
  }

  async function handleSave(field: string, value: unknown) {
    setSaving(true);
    try {
      await saveConfig({ [field]: value } as never);
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
    } catch (error) {
      setUpdateStatus(`检查失败: ${error}`);
    }
  }

  return (
    <div class="space-y-5">
      {/* ── Hero / Summary ──────────────────────────── */}
      <section class="relative overflow-hidden rounded-[2.2rem] border border-base-300/70 bg-[radial-gradient(circle_at_top_left,_color-mix(in_oklch,_var(--color-primary)_22%,_transparent),_transparent_38%),radial-gradient(circle_at_bottom_right,_color-mix(in_oklch,_var(--color-secondary)_18%,_transparent),_transparent_32%),linear-gradient(180deg,_var(--color-base-100),_color-mix(in_oklch,_var(--color-base-100)_76%,_var(--color-base-200)))] p-6 shadow-[0_24px_80px_-40px_color-mix(in_oklch,_var(--color-base-content)_18%,_transparent)]">
        <div class="absolute inset-y-0 right-0 hidden w-56 translate-x-10 rounded-full bg-primary/10 blur-3xl md:block" />
        <div class="relative flex flex-col gap-5 lg:flex-row lg:items-start lg:justify-between">
          <div class="max-w-2xl">
            <p class="text-[11px] font-semibold uppercase tracking-[0.28em] text-base-content/38">
              Control Deck
            </p>
            <div class="mt-3 flex flex-wrap items-center gap-3">
              <h2 class="text-3xl font-semibold tracking-tight text-base-content">设置中心</h2>
              <span class="rounded-full border border-base-300/80 bg-base-100/80 px-3 py-1 text-xs font-medium text-base-content/60">
                {saving() ? "正在保存" : "配置已同步"}
              </span>
            </div>
            <p class="mt-3 max-w-xl text-sm leading-7 text-base-content/58">
              把下载行为、网络参数、BitTorrent 策略和外观偏好放在同一页里，减少来回跳转和低效查找。
            </p>
          </div>

          <div class="grid gap-2 sm:grid-cols-2 lg:w-[24rem]">
            {summaryCards().map((card) => (
              <div class="rounded-[1.6rem] border border-base-300/70 bg-base-100/82 p-4 backdrop-blur">
                <p class="text-[11px] uppercase tracking-[0.18em] text-base-content/42">
                  {card.label}
                </p>
                <p class="mt-2 truncate text-base font-semibold text-base-content">
                  {card.value}
                </p>
                <p class="mt-1 truncate text-xs text-base-content/48">{card.meta}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* ── Sub-nav pill bar ────────────────────────── */}
      <nav class="flex items-center gap-1 rounded-2xl border border-base-300/60 bg-base-100/80 p-1.5 backdrop-blur-sm">
        <For each={tabs}>
          {(tab) => (
            <button
              class={`flex items-center gap-1.5 rounded-xl px-4 py-2 text-sm font-medium transition-all duration-200 ${
                activeTab() === tab.id
                  ? "bg-primary/15 text-primary shadow-sm"
                  : "text-base-content/55 hover:text-base-content hover:bg-base-200/80"
              }`}
              onClick={() => setActiveTab(tab.id)}
            >
              <tab.icon class="w-3.5 h-3.5 opacity-70" />
              {tab.label}
            </button>
          )}
        </For>
      </nav>

      {/* ── Tab content ─────────────────────────────── */}
      <Switch>
        {/* ---- Transfer ---- */}
        <Match when={activeTab() === "transfer"}>
          <section class="settings-panel rounded-[2rem] border border-base-300/70 bg-base-100/92 p-5 shadow-[0_18px_50px_-28px_color-mix(in_oklch,_var(--color-base-content)_18%,_transparent)] backdrop-blur">
            <div class="flex flex-col gap-1 border-b border-base-300/70 pb-4">
              <p class="text-[11px] font-semibold uppercase tracking-[0.22em] text-base-content/40">
                Transfer
              </p>
              <h3 class="text-lg font-semibold text-base-content">下载与并发</h3>
              <p class="max-w-2xl text-sm leading-6 text-base-content/55">
                定义默认保存路径、每个任务的连接数和队列同时执行规模。
              </p>
            </div>
            <div class="mt-5 grid gap-4 md:grid-cols-2">
              <SettingsField
                label="默认下载路径"
                hint="建议设置到稳定且空间充足的目录。修改后新任务会默认使用此位置。"
                span="full"
              >
                <div class="flex flex-col gap-3 sm:flex-row">
                  <input
                    type="text"
                    class={`${inputClass} flex-1`}
                    value={config.default_download_path}
                    readOnly
                  />
                  <button class="btn btn-primary h-11 rounded-2xl px-5" onClick={pickDefaultDir}>
                    浏览目录
                  </button>
                </div>
              </SettingsField>

              <SettingsField
                label="每任务最大连接数"
                hint="提高连接数能加快大文件下载，但也会增加服务器和网络压力。"
              >
                <input
                  type="number"
                  min="1"
                  max="32"
                  class={inputClass}
                  value={config.max_concurrent_downloads}
                  onChange={(event) =>
                    handleSave(
                      "max_concurrent_downloads",
                      parseInt(event.currentTarget.value, 10),
                    )}
                />
              </SettingsField>

              <SettingsField
                label="最大同时任务数"
                hint="控制队列一次并发跑几个任务，适合按带宽和磁盘吞吐来平衡。"
              >
                <input
                  type="number"
                  min="1"
                  max="16"
                  class={inputClass}
                  value={config.max_concurrent_tasks}
                  onChange={(event) =>
                    handleSave(
                      "max_concurrent_tasks",
                      parseInt(event.currentTarget.value, 10),
                    )}
                />
              </SettingsField>
            </div>
          </section>
        </Match>

        {/* ---- Network ---- */}
        <Match when={activeTab() === "network"}>
          <section class="settings-panel rounded-[2rem] border border-base-300/70 bg-base-100/92 p-5 shadow-[0_18px_50px_-28px_color-mix(in_oklch,_var(--color-base-content)_18%,_transparent)] backdrop-blur">
            <div class="flex flex-col gap-1 border-b border-base-300/70 pb-4">
              <p class="text-[11px] font-semibold uppercase tracking-[0.22em] text-base-content/40">
                Network
              </p>
              <h3 class="text-lg font-semibold text-base-content">网络与请求参数</h3>
              <p class="max-w-2xl text-sm leading-6 text-base-content/55">
                调整代理、请求超时、分块大小和 User-Agent，优化不同网络环境下的表现。
              </p>
            </div>
            <div class="mt-5 grid gap-4 md:grid-cols-2">
              <SettingsField
                label="代理地址"
                hint="支持例如 socks5://127.0.0.1:1080。留空则直接联网。"
                span="full"
              >
                <input
                  type="text"
                  class={inputClass}
                  placeholder="socks5://127.0.0.1:1080"
                  value={config.proxy ?? ""}
                  onChange={(event) => handleSave("proxy", event.currentTarget.value || null)}
                />
              </SettingsField>

              <SettingsField
                label="User Agent"
                hint="某些站点会根据 User Agent 返回不同资源或进行限制。"
                span="full"
              >
                <input
                  type="text"
                  class={inputClass}
                  value={config.user_agent}
                  onChange={(event) => handleSave("user_agent", event.currentTarget.value)}
                />
              </SettingsField>

              <SettingsField
                label="请求超时"
                hint="单位为秒。网络不稳定时可以适当调高。"
              >
                <div class="relative">
                  <input
                    type="number"
                    min="5"
                    class={`${inputClass} pr-14`}
                    value={config.timeout}
                    onChange={(event) =>
                      handleSave("timeout", parseInt(event.currentTarget.value, 10))}
                  />
                  <span class="pointer-events-none absolute inset-y-0 right-4 flex items-center text-xs font-medium text-base-content/45">
                    秒
                  </span>
                </div>
              </SettingsField>

              <SettingsField
                label="分块大小"
                hint="较大的分块适合高速稳定网络，较小的分块更利于弱网重试。"
              >
                <div class="relative">
                  <input
                    type="number"
                    min="1"
                    class={`${inputClass} pr-14`}
                    value={chunkSizeMb()}
                    onChange={(event) =>
                      handleSave(
                        "chunk_size",
                        parseInt(event.currentTarget.value, 10) * 1048576,
                      )}
                  />
                  <span class="pointer-events-none absolute inset-y-0 right-4 flex items-center text-xs font-medium text-base-content/45">
                    MB
                  </span>
                </div>
              </SettingsField>
            </div>
          </section>
        </Match>

        {/* ---- BitTorrent ---- */}
        <Match when={activeTab() === "bt"}>
          <section class="settings-panel rounded-[2rem] border border-base-300/70 bg-base-100/92 p-5 shadow-[0_18px_50px_-28px_color-mix(in_oklch,_var(--color-base-content)_18%,_transparent)] backdrop-blur">
            <div class="flex flex-col gap-1 border-b border-base-300/70 pb-4">
              <p class="text-[11px] font-semibold uppercase tracking-[0.22em] text-base-content/40">
                BitTorrent
              </p>
              <h3 class="text-lg font-semibold text-base-content">种子下载策略</h3>
              <p class="max-w-2xl text-sm leading-6 text-base-content/55">
                控制 DHT、做种、上传和监听端口，让 BT 任务行为更符合你的网络环境。
              </p>
            </div>
            <div class="mt-5 grid gap-4 md:grid-cols-2">
              <SettingsField
                label="DHT 网络"
                hint="开启后能更容易发现 peers。公网环境建议开启，受限网络可按需关闭。"
                span="full"
              >
                <label class="flex items-center justify-between rounded-2xl border border-base-300/70 bg-base-100/85 px-4 py-3">
                  <div>
                    <p class="text-sm font-medium text-base-content">启用 DHT</p>
                    <p class="mt-1 text-xs text-base-content/50">
                      提升节点发现能力和磁力链接启动速度
                    </p>
                  </div>
                  <input
                    type="checkbox"
                    class="toggle toggle-md"
                    checked={config.bt.dht_enabled}
                    onChange={(event) =>
                      handleSave("bt", {
                        ...config.bt,
                        dht_enabled: event.currentTarget.checked,
                      })}
                  />
                </label>
              </SettingsField>

              <SettingsField
                label="上传限速"
                hint="单位为 KB/s。留空表示不限速。"
              >
                <div class="relative">
                  <input
                    type="number"
                    class={`${inputClass} pr-16`}
                    placeholder="不限"
                    value={config.bt.upload_limit ? config.bt.upload_limit / 1024 : ""}
                    onChange={(event) => {
                      const value = event.currentTarget.value;
                      handleSave("bt", {
                        ...config.bt,
                        upload_limit: value ? parseInt(value, 10) * 1024 : null,
                      });
                    }}
                  />
                  <span class="pointer-events-none absolute inset-y-0 right-4 flex items-center text-xs font-medium text-base-content/45">
                    KB/s
                  </span>
                </div>
              </SettingsField>

              <SettingsField
                label="做种比例"
                hint="例如 2.0。留空表示下载完成后不继续做种。"
              >
                <input
                  type="number"
                  step="0.1"
                  class={inputClass}
                  placeholder="不做种"
                  value={config.bt.seed_ratio ?? ""}
                  onChange={(event) => {
                    const value = event.currentTarget.value;
                    handleSave("bt", {
                      ...config.bt,
                      seed_ratio: value ? parseFloat(value) : null,
                    });
                  }}
                />
              </SettingsField>

              <SettingsField
                label="监听端口"
                hint="留空则随机。固定端口更利于防火墙和端口映射配置。"
                span="full"
              >
                <input
                  type="number"
                  class={inputClass}
                  placeholder="随机"
                  value={config.bt.listen_port ?? ""}
                  onChange={(event) => {
                    const value = event.currentTarget.value;
                    handleSave("bt", {
                      ...config.bt,
                      listen_port: value ? parseInt(value, 10) : null,
                    });
                  }}
                />
              </SettingsField>
            </div>
          </section>
        </Match>

        {/* ---- Appearance ---- */}
        <Match when={activeTab() === "appearance"}>
          <section class="settings-panel rounded-[2rem] border border-base-300/70 bg-base-100/92 p-5 shadow-[0_18px_50px_-28px_color-mix(in_oklch,_var(--color-base-content)_18%,_transparent)] backdrop-blur">
            <div class="border-b border-base-300/70 pb-4">
              <p class="text-[11px] font-semibold uppercase tracking-[0.22em] text-base-content/40">
                Appearance
              </p>
              <h3 class="mt-1 text-lg font-semibold">界面风格</h3>
              <p class="mt-1 text-sm leading-6 text-base-content/55">
                选择浅色、深色或跟随系统，主题会立即应用到整个界面。
              </p>
            </div>

            <div class="mt-5 rounded-[1.5rem] border border-base-300/70 bg-base-200/60 p-4">
              <p class="text-sm font-medium text-base-content">主题模式</p>
              <p class="mt-1 text-xs leading-5 text-base-content/50">
                当前选项会同步影响主窗口与托盘视图。
              </p>
              <div class="mt-4">
                <ThemeToggle
                  value={config.theme}
                  onChange={(theme) => handleSave("theme", theme)}
                />
              </div>
            </div>
          </section>
        </Match>

        {/* ---- About ---- */}
        <Match when={activeTab() === "about"}>
          <section class="settings-panel rounded-[2rem] border border-base-300/70 bg-base-100/92 p-5 shadow-[0_18px_50px_-28px_color-mix(in_oklch,_var(--color-base-content)_18%,_transparent)] backdrop-blur">
            <div class="border-b border-base-300/70 pb-4">
              <p class="text-[11px] font-semibold uppercase tracking-[0.22em] text-base-content/40">
                Release
              </p>
              <h3 class="mt-1 text-lg font-semibold">版本与更新</h3>
              <p class="mt-1 text-sm leading-6 text-base-content/55">
                检查新版本并执行安装。更新状态会显示在下方，避免重复操作。
              </p>
            </div>

            <div class="mt-5 space-y-4">
              <div class="rounded-[1.5rem] border border-base-300/70 bg-base-200/60 p-4">
                <p class="text-sm font-medium text-base-content">Hsi v0.1.0</p>
                <p class="mt-1 text-xs leading-5 text-base-content/50">
                  当前桌面客户端版本。建议在网络稳定时执行更新检查。
                </p>
              </div>

              <button class="btn btn-primary h-11 w-full rounded-2xl" onClick={checkUpdate}>
                检查更新
              </button>

              <div class="rounded-[1.3rem] border border-dashed border-base-300/80 bg-base-200/45 p-4">
                <p class="text-[11px] font-semibold uppercase tracking-[0.18em] text-base-content/38">
                  Update Status
                </p>
                <p class="mt-2 min-h-[2.75rem] text-sm leading-6 text-base-content/58">
                  {updateStatus() || "尚未执行检查。"}
                </p>
              </div>
            </div>
          </section>
        </Match>
      </Switch>
    </div>
  );
};

export default SettingsPage;
