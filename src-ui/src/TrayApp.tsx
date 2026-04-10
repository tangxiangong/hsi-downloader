import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { type Component, For, Show, createMemo, onMount } from "solid-js";
import TrayTaskItem from "./components/TrayTaskItem";
import { formatBytes, formatSpeed } from "./lib/format";
import type { Page } from "./components/Sidebar";
import { loadConfig } from "./stores/config-store";
import {
  loadTasks,
  setupTaskEvents,
  taskCounts,
  taskSummary,
  trayTasks,
} from "./stores/task-store";

const currentWindow = getCurrentWindow();

const TrayApp: Component = () => {
  const visibleTasks = createMemo(() => trayTasks().slice(0, 4));
  const summary = taskSummary;
  const progressLabel = createMemo(() => {
    const progress = summary().progress;
    return progress == null ? "--" : `${Math.round(progress)}%`;
  });
  const speedDisplay = createMemo(() => {
    const formatted = formatSpeed(summary().totalSpeed);
    const [value = formatted, unit = ""] = formatted.split(" ");
    return { value, unit };
  });

  async function closeTraySurface() {
    try {
      await invoke("plugin:nspopover|hide_popover");
      return;
    } catch {
      // Non-macOS platforms use a regular tray window instead of NSPopover.
    }

    await currentWindow.hide();
  }

  onMount(async () => {
    await loadConfig();
    await loadTasks();
    setupTaskEvents();
  });

  async function openMainWindow(page?: Page) {
    const mainWindow = await WebviewWindow.getByLabel("main");
    if (mainWindow) {
      await mainWindow.emit("navigate-main", { page });
      await mainWindow.unminimize();
      await mainWindow.show();
      await mainWindow.setFocus();
    }

    await closeTraySurface();
  }

  async function handleOpenMainWindow() {
    await openMainWindow("tasks");
  }

  async function handleOpenSettings() {
    await openMainWindow("settings");
  }

  async function handleHide() {
    await closeTraySurface();
  }

  return (
    <div class="h-screen overflow-hidden bg-[radial-gradient(circle_at_top,_color-mix(in_oklch,_var(--color-primary)_18%,_transparent),_transparent_52%),linear-gradient(180deg,_var(--color-base-200),_var(--color-base-100))] text-base-content">
      <div class="flex h-full flex-col p-3">
        <div class="rounded-[28px] border border-base-300/70 bg-base-100/88 p-4 shadow-xl backdrop-blur">
          <div class="flex items-start justify-between gap-3">
            <div>
              <p class="text-xs font-semibold uppercase tracking-[0.28em] text-base-content/40">
                Hsi Tray
              </p>
              <h1 class="mt-1 text-lg font-semibold">下载概览</h1>
              <p class="mt-1 text-xs text-base-content/55">
                {taskSummary().active > 0
                  ? `当前 ${taskSummary().active} 个活动任务`
                  : "当前没有进行中的下载"}
              </p>
            </div>
            <div class="flex items-center gap-2">
              <button class="btn btn-ghost btn-sm rounded-full" onClick={handleHide}>
                隐藏
              </button>
              <button class="btn btn-ghost btn-sm rounded-full" onClick={handleOpenSettings}>
                设置
              </button>
              <button class="btn btn-primary btn-sm rounded-full" onClick={handleOpenMainWindow}>
                打开主窗口
              </button>
            </div>
          </div>

          <div class="mt-4 grid grid-cols-3 gap-2">
            <div class="rounded-2xl bg-base-200/70 p-3">
              <p class="text-[11px] uppercase tracking-[0.18em] text-base-content/45">
                总体进度
              </p>
              <p class="mt-2 text-lg font-semibold">
                {progressLabel()}
              </p>
              <p class="mt-1 text-xs text-base-content/50">
                {summary().totalSize > 0
                  ? `${formatBytes(summary().totalDownloaded)} / ${formatBytes(summary().totalSize)}`
                  : "等待获取大小"}
              </p>
            </div>
            <div class="rounded-2xl bg-base-200/70 p-3">
              <p class="text-[11px] uppercase tracking-[0.18em] text-base-content/45">
                汇总速率
              </p>
              <div class="mt-2 min-h-[3.25rem]">
                <p class="text-lg leading-none font-semibold tabular-nums">
                  {speedDisplay().value}
                </p>
                <p class="mt-1 text-xs font-medium uppercase tracking-[0.18em] text-base-content/45">
                  {speedDisplay().unit}
                </p>
              </div>
              <p class="mt-1 text-xs text-base-content/50">
                {summary().downloading} 下载中
              </p>
            </div>
            <div class="rounded-2xl bg-base-200/70 p-3">
              <p class="text-[11px] uppercase tracking-[0.18em] text-base-content/45">
                队列状态
              </p>
              <p class="mt-2 text-lg font-semibold">{taskCounts().all}</p>
              <p class="mt-1 text-xs text-base-content/50">
                {summary().pending} 等待 · {summary().paused} 暂停
              </p>
            </div>
          </div>
        </div>

        <div class="mt-3 min-h-0 flex-1 rounded-[28px] border border-base-300/70 bg-base-100/88 p-3 shadow-lg backdrop-blur">
          <div class="flex items-center justify-between px-1 pb-2">
            <div>
              <h2 class="text-sm font-semibold">活动任务</h2>
            </div>
            <span class="rounded-full bg-base-200 px-2.5 py-1 text-xs font-medium text-base-content/60">
              {visibleTasks().length} / {summary().active}
            </span>
          </div>

          <Show
            when={visibleTasks().length > 0}
            fallback={
              <div class="flex h-full flex-col items-center justify-center rounded-[24px] border border-dashed border-base-300 bg-base-200/45 px-6 text-center">
                <p class="text-sm font-medium">没有活动任务</p>
                <p class="mt-2 text-xs leading-5 text-base-content/55">
                  新任务开始后，这里会实时显示下载进度、速度和快捷控制。
                </p>
              </div>
            }
          >
            <div class="flex h-full flex-col gap-2 overflow-y-auto pr-1">
              <For each={visibleTasks()}>{(task) => <TrayTaskItem task={task} />}</For>
            </div>
          </Show>
        </div>
      </div>
    </div>
  );
};

export default TrayApp;
