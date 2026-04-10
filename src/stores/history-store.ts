import { createStore } from "solid-js/store";
import type { CompletedTask } from "../lib/types";
import { getHistory } from "../lib/commands";

const [history, setHistory] = createStore<CompletedTask[]>([]);

export async function loadHistory() {
  const list = await getHistory();
  setHistory(list);
}

export async function refreshHistory() {
  await loadHistory();
}

export { history };
