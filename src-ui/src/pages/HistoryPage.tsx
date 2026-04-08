import { type Component, For } from "solid-js";
import HistoryCard from "../components/HistoryCard";
import { history } from "../stores/history-store";
import { clearHistory } from "../lib/commands";
import { refreshHistory } from "../stores/history-store";

const HistoryPage: Component = () => {
  async function handleClear() {
    await clearHistory();
    await refreshHistory();
  }

  return (
    <div>
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-2xl font-bold">历史</h2>
        <button
          class="btn btn-ghost btn-sm text-error"
          onClick={handleClear}
          disabled={history.length === 0}
        >
          清空历史
        </button>
      </div>

      <div class="space-y-3">
        <For each={history} fallback={
          <div class="text-center text-base-content/50 py-12">暂无历史记录</div>
        }>
          {(task) => <HistoryCard task={task} />}
        </For>
      </div>
    </div>
  );
};

export default HistoryPage;
