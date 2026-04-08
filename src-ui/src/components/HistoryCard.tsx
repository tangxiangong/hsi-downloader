import type { Component } from "solid-js";
import type { CompletedTask } from "../lib/types";
import { formatBytes, formatSpeed, formatDate, getFileIcon } from "../lib/format";
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
    <div class="task-card card bg-base-100 border border-base-300">
      <div class="card-body p-4">
        <div class="flex items-center gap-3">
          <div class="w-9 h-9 rounded-lg bg-success/15 flex items-center justify-center text-lg shrink-0">
            {getFileIcon(filename())}
          </div>
          <div class="flex-1 min-w-0">
            <h3 class="text-sm font-medium truncate">{filename()}</h3>
            <p class="text-xs text-base-content/40 mt-0.5">
              {formatBytes(t().total_size)} \u00b7 \u5e73\u5747 {formatSpeed(t().avg_speed)} \u00b7 {formatDate(t().completed_at)}
            </p>
          </div>
          <div class="flex items-center gap-1 shrink-0">
            <button class="btn-icon" onClick={handleRemove} title="\u79fb\u9664\u8bb0\u5f55">{"\u2715"}</button>
            <button class="btn-icon hover:!text-error" onClick={handleRemoveWithFile} title="\u5220\u9664\u6587\u4ef6">{"\ud83d\uddd1"}</button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default HistoryCard;
