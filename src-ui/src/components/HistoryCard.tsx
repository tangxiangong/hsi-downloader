import type { Component } from "solid-js";
import type { CompletedTask } from "../lib/types";
import { formatBytes, formatSpeed, formatDate } from "../lib/format";
import { removeHistory, removeHistoryWithFile } from "../lib/commands";
import { refreshHistory } from "../stores/history-store";

interface HistoryCardProps {
  task: CompletedTask;
}

const HistoryCard: Component<HistoryCardProps> = (props) => {
  const t = () => props.task;
  const filename = () => {
    const parts = t().dest.split(/[/\\]/);
    return parts[parts.length - 1] || t().url;
  };

  async function handleRemove() {
    await removeHistory(t().id);
    await refreshHistory();
  }

  async function handleRemoveWithFile() {
    await removeHistoryWithFile(t().id);
    await refreshHistory();
  }

  return (
    <div class="card bg-base-100 shadow-sm">
      <div class="card-body p-4">
        <div class="flex items-center justify-between">
          <div class="flex-1 min-w-0">
            <h3 class="font-medium truncate">{filename()}</h3>
            <p class="text-xs text-base-content/50 truncate">{t().url}</p>
          </div>
          <span class="badge badge-success ml-2">已完成</span>
        </div>
        <div class="flex gap-4 text-xs text-base-content/60 mt-2">
          <span>{formatBytes(t().total_size)}</span>
          <span>平均 {formatSpeed(t().avg_speed)}</span>
          <span>{formatDate(t().completed_at)}</span>
        </div>
        <div class="card-actions justify-end mt-2">
          <button class="btn btn-sm btn-ghost" onClick={handleRemove}>移除记录</button>
          <button class="btn btn-sm btn-ghost text-error" onClick={handleRemoveWithFile}>删除文件</button>
        </div>
      </div>
    </div>
  );
};

export default HistoryCard;
