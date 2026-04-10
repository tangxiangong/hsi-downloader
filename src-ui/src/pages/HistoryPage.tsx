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
      <div class="flex items-center justify-between mb-5">
        <div>
          <h2 class="text-xl font-bold">{"\u5386\u53f2"}</h2>
          <p class="text-xs text-base-content/40 mt-0.5">
            {history.length} {"\u6761\u8bb0\u5f55"}
          </p>
        </div>
        <button
          class="btn btn-ghost btn-sm text-error"
          onClick={handleClear}
          disabled={history.length === 0}
        >
          {"\u6e05\u7a7a"}
        </button>
      </div>

      <div class="space-y-2">
        <For
          each={history}
          fallback={
            <div class="text-center text-base-content/30 py-16">
              <div class="text-4xl mb-3">{"\u2630"}</div>
              <p class="text-sm">{"\u6682\u65e0\u5386\u53f2\u8bb0\u5f55"}</p>
            </div>
          }
        >
          {(task) => <HistoryCard task={task} />}
        </For>
      </div>
    </div>
  );
};

export default HistoryPage;
