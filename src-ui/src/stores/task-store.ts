import { createSignal, createMemo } from "solid-js";
import { createStore } from "solid-js/store";
import type { DownloadTask, DownloaderEvent } from "../lib/types";
import { getTasks } from "../lib/commands";
import { onDownloadEvent } from "../lib/events";
import { refreshHistory } from "./history-store";

export type TaskFilter = "all" | "downloading" | "completed";

const [tasks, setTasks] = createStore<DownloadTask[]>([]);
const [filter, setFilter] = createSignal<TaskFilter>("all");

export const filteredTasks = createMemo(() => {
  const f = filter();
  if (f === "all") return tasks;
  if (f === "downloading")
    return tasks.filter(
      (t) =>
        t.status === "Downloading" ||
        t.status === "Pending" ||
        t.status === "Paused",
    );
  return tasks.filter(
    (t) =>
      t.status === "Completed" ||
      t.status === "Failed" ||
      t.status === "Cancelled",
  );
});

export const taskCounts = createMemo(() => ({
  all: tasks.length,
  downloading: tasks.filter(
    (t) =>
      t.status === "Downloading" ||
      t.status === "Pending" ||
      t.status === "Paused",
  ).length,
  completed: tasks.filter(
    (t) =>
      t.status === "Completed" ||
      t.status === "Failed" ||
      t.status === "Cancelled",
  ).length,
}));

export async function loadTasks() {
  const list = await getTasks();
  setTasks(list);
}

export async function refreshTasks() {
  await loadTasks();
}

export function setupTaskEvents() {
  onDownloadEvent(async (event: DownloaderEvent) => {
    await refreshTasks();
    if (event.type === "Task" && "Completed" in event.data) {
      await refreshHistory();
    }
  });
}

export { tasks, filter, setFilter };
