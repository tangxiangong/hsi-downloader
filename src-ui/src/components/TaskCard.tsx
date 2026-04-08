import { type Component, Show } from "solid-js";
import type { DownloadTask } from "../lib/types";
import { formatBytes, formatSpeed, formatEta, statusLabel, statusBadgeClass } from "../lib/format";
import { pauseTask, resumeTask, cancelTask, removeTask, removeTaskWithFile } from "../lib/commands";
import { refreshTasks } from "../stores/task-store";

interface TaskCardProps {
  task: DownloadTask;
}

const TaskCard: Component<TaskCardProps> = (props) => {
  const t = () => props.task;
  const progress = () =>
    t().total_size > 0 ? (t().downloaded / t().total_size) * 100 : 0;
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
  async function handleRemove() {
    await removeTask(t().id);
    await refreshTasks();
  }
  async function handleRemoveWithFile() {
    await removeTaskWithFile(t().id);
    await refreshTasks();
  }

  return (
    <div class="card bg-base-100 shadow-sm">
      <div class="card-body p-4">
        <div class="flex items-center justify-between">
          <div class="flex-1 min-w-0">
            <h3 class="font-medium truncate">{filename()}</h3>
            <p class="text-xs text-base-content/50 truncate">{t().url}</p>
          </div>
          <span class={`badge ${statusBadgeClass(t().status)} ml-2`}>
            {statusLabel(t().status)}
          </span>
        </div>

        <Show when={t().status === "Downloading" || t().total_size > 0}>
          <div class="mt-2">
            <progress
              class="progress progress-info w-full"
              value={progress()}
              max="100"
            />
            <div class="flex justify-between text-xs text-base-content/60 mt-1">
              <span>
                {formatBytes(t().downloaded)} / {formatBytes(t().total_size)}
              </span>
              <Show when={t().status === "Downloading"}>
                <span>
                  {formatSpeed(t().speed)} · {formatEta(t().eta)}
                </span>
              </Show>
            </div>
          </div>
        </Show>

        <Show when={t().error}>
          <p class="text-xs text-error mt-1">{t().error}</p>
        </Show>

        <div class="card-actions justify-end mt-2">
          <Show when={t().status === "Downloading"}>
            <button class="btn btn-sm btn-ghost" onClick={handlePause}>暂停</button>
          </Show>
          <Show when={t().status === "Paused"}>
            <button class="btn btn-sm btn-ghost" onClick={handleResume}>恢复</button>
          </Show>
          <Show when={t().status === "Downloading" || t().status === "Paused" || t().status === "Pending"}>
            <button class="btn btn-sm btn-ghost text-error" onClick={handleCancel}>取消</button>
          </Show>
          <Show when={t().status === "Completed" || t().status === "Failed" || t().status === "Cancelled"}>
            <button class="btn btn-sm btn-ghost" onClick={handleRemove}>移除</button>
            <button class="btn btn-sm btn-ghost text-error" onClick={handleRemoveWithFile}>删除文件</button>
          </Show>
        </div>
      </div>
    </div>
  );
};

export default TaskCard;
