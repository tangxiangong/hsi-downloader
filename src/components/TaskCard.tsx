import { type Component, Show } from "solid-js";
import ChunkedProgressBar from "./ChunkedProgressBar";
import type { DownloadTask } from "../lib/types";
import {
  formatBytes,
  formatSpeed,
  formatEta,
  statusLabel,
  getFileIcon,
} from "../lib/format";
import { taskProgressPercent } from "../lib/progress";
import {
  pauseTask,
  resumeTask,
  cancelTask,
  retryTask,
  removeTask,
  removeTaskWithFile,
} from "../lib/commands";
import { config } from "../stores/config-store";
import { refreshTasks } from "../stores/task-store";
import PauseIcon from "../icons/pause.svg";
import PlayIcon from "../icons/play.svg";
import XIcon from "../icons/x.svg";
import RetryIcon from "../icons/retry.svg";
import TrashIcon from "../icons/trash.svg";

interface TaskCardProps {
  task: DownloadTask;
}

const TaskCard: Component<TaskCardProps> = (props) => {
  const t = () => props.task;
  const progress = () => taskProgressPercent(t());
  const filename = () => {
    const parts = t().dest.split(/[/\\]/);
    return parts[parts.length - 1] || t().url;
  };

  async function handlePause() {
    await pauseTask(t().id);
    await refreshTasks();
  }
  async function handleResume() {
    await resumeTask(t().id);
    await refreshTasks();
  }
  async function handleCancel() {
    await cancelTask(t().id);
    await refreshTasks();
  }
  async function handleRetry() {
    await retryTask(t().id);
    await refreshTasks();
  }
  async function handleRemove() {
    await removeTask(t().id);
    await refreshTasks();
  }
  async function handleRemoveWithFile() {
    await removeTaskWithFile(t().id);
    await refreshTasks();
  }

  return (
    <div class="task-card card bg-base-100 border border-base-300">
      <div class="card-body p-4">
        {/* Row 1: Icon + Info + Actions */}
        <div class="flex items-center gap-3">
          <div class="w-9 h-9 rounded-lg bg-base-300 flex items-center justify-center text-lg shrink-0">
            {getFileIcon(filename())}
          </div>
          <div class="flex-1 min-w-0">
            <h3 class="text-sm font-medium truncate">{filename()}</h3>
            <p class="text-xs text-base-content/40 mt-0.5 whitespace-nowrap overflow-hidden text-ellipsis">
              <Show
                when={t().status === "Downloading"}
                fallback={
                  <span>
                    {formatBytes(
                      t().total_size > 0 ? t().total_size : t().downloaded,
                    )}
                    {t().status !== "Pending" &&
                      ` · ${statusLabel(t().status)}`}
                  </span>
                }
              >
                <span>
                  {formatBytes(t().downloaded)} / {formatBytes(t().total_size)}
                  {t().bt_info ? (
                    <>
                      {" · ↓"}
                      {formatSpeed(t().speed)}
                      {" · ↑"}
                      {formatSpeed(t().bt_info!.upload_speed)}
                      {" · "}
                      {t().bt_info!.peers}
                      {"P"}
                      {t().eta != null &&
                        ` · 剩余 ${formatEta(t().eta)}`}
                    </>
                  ) : (
                    <>
                      {" · "}
                      {formatSpeed(t().speed)}
                      {t().eta != null && ` · ${formatEta(t().eta)}`}
                    </>
                  )}
                </span>
              </Show>
            </p>
          </div>
          <div class="flex items-center gap-1 shrink-0">
            <Show when={t().status === "Downloading"}>
              <button
                class="btn-icon"
                onClick={handlePause}
                title="暂停"
              >
                <PauseIcon class="w-4 h-4" />
              </button>
            </Show>
            <Show when={t().status === "Paused"}>
              <button
                class="btn-icon"
                onClick={handleResume}
                title="恢复"
              >
                <PlayIcon class="w-4 h-4" />
              </button>
            </Show>
            <Show
              when={
                t().status === "Downloading" ||
                t().status === "Paused" ||
                t().status === "Pending"
              }
            >
              <button
                class="btn-icon hover:!text-error"
                onClick={handleCancel}
                title="取消"
              >
                <XIcon class="w-4 h-4" />
              </button>
            </Show>
            <Show when={t().status === "Failed"}>
              <button
                class="btn-icon hover:!text-success"
                onClick={handleRetry}
                title="重试"
              >
                <RetryIcon class="w-4 h-4" />
              </button>
            </Show>
            <Show
              when={
                t().status === "Completed" ||
                t().status === "Failed" ||
                t().status === "Cancelled"
              }
            >
              <button
                class="btn-icon"
                onClick={handleRemove}
                title="移除"
              >
                <XIcon class="w-4 h-4" />
              </button>
              <button
                class="btn-icon hover:!text-error"
                onClick={handleRemoveWithFile}
                title="删除文件"
              >
                <TrashIcon class="w-4 h-4" />
              </button>
            </Show>
          </div>
        </div>

        {/* Row 2: Progress bar */}
        <Show when={t().total_size > 0 || t().status === "Downloading"}>
          <div class="flex items-center gap-2 mt-2">
            <ChunkedProgressBar
              task={t()}
              concurrency={config.max_concurrent_downloads}
            />
            <span class="text-xs font-medium text-base-content/60 w-10 text-right">
              {Math.round(progress())}%
            </span>
          </div>
        </Show>

        <Show when={t().error}>
          <p class="text-xs text-error mt-1">{t().error}</p>
        </Show>
      </div>
    </div>
  );
};

export default TaskCard;
