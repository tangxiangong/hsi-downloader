import { type Component, Show } from "solid-js";
import ChunkedProgressBar from "./ChunkedProgressBar";
import type { DownloadTask } from "../lib/types";
import {
  formatBytes,
  formatEta,
  formatSpeed,
  statusLabel,
} from "../lib/format";
import { taskProgressPercent } from "../lib/progress";
import { cancelTask, pauseTask, resumeTask } from "../lib/commands";
import { config } from "../stores/config-store";
import { refreshTasks } from "../stores/task-store";

interface TrayTaskItemProps {
  task: DownloadTask;
}

const TrayTaskItem: Component<TrayTaskItemProps> = (props) => {
  const task = () => props.task;
  const percent = () => taskProgressPercent(task());
  const filename = () => {
    const parts = task().dest.split(/[/\\]/);
    return parts[parts.length - 1] || task().url;
  };

  async function handlePause() {
    await pauseTask(task().id);
    await refreshTasks();
  }

  async function handleResume() {
    await resumeTask(task().id);
    await refreshTasks();
  }

  async function handleCancel() {
    await cancelTask(task().id);
    await refreshTasks();
  }

  return (
    <div class="rounded-2xl border border-base-300/80 bg-base-100/90 p-3 shadow-sm">
      <div class="flex items-start gap-3">
        <div class="min-w-0 flex-1">
          <div class="flex items-center gap-2">
            <p class="truncate text-sm font-medium text-base-content">
              {filename()}
            </p>
            <span class="shrink-0 whitespace-nowrap rounded-full bg-base-200 px-2 py-0.5 text-[10px] font-medium text-base-content/60">
              {statusLabel(task().status)}
            </span>
          </div>
          <p class="mt-1 text-xs text-base-content/55 whitespace-nowrap overflow-hidden text-ellipsis">
            <Show
              when={task().total_size > 0}
              fallback={<span>{formatBytes(task().downloaded)}</span>}
            >
              <span>
                {formatBytes(task().downloaded)} /{" "}
                {formatBytes(task().total_size)}
              </span>
            </Show>
            <span>{` · ${formatSpeed(task().speed)}`}</span>
            <Show when={task().eta != null}>
              <span>{` · 剩余 ${formatEta(task().eta)}`}</span>
            </Show>
          </p>
        </div>
        <div class="flex shrink-0 items-center gap-1">
          <Show when={task().status === "Downloading"}>
            <button class="btn-icon btn-xs" onClick={handlePause} title="暂停">
              {"⏸"}
            </button>
          </Show>
          <Show when={task().status === "Paused"}>
            <button class="btn-icon btn-xs" onClick={handleResume} title="恢复">
              {"▶"}
            </button>
          </Show>
          <button
            class="btn-icon btn-xs hover:!text-error"
            onClick={handleCancel}
            title="取消"
          >
            {"✕"}
          </button>
        </div>
      </div>

      <div class="mt-2 flex items-center gap-2">
        <ChunkedProgressBar
          task={task()}
          compact
          concurrency={config.max_concurrent_downloads}
        />
        <span class="w-10 text-right text-[11px] font-medium text-base-content/55">
          {task().total_size > 0 ? `${Math.round(percent())}%` : "--"}
        </span>
      </div>
    </div>
  );
};

export default TrayTaskItem;
