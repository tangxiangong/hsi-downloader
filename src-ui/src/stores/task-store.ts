import { createSignal, createMemo } from "solid-js";
import { createStore } from "solid-js/store";
import type { DownloadTask, DownloaderEvent, TaskStatus } from "../lib/types";
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

export const activeTasks = createMemo(() =>
  tasks.filter(
    (t) =>
      t.status === "Downloading" ||
      t.status === "Pending" ||
      t.status === "Paused",
  ),
);

export const trayTasks = createMemo(() => {
  const statusRank: Record<TaskStatus, number> = {
    Downloading: 0,
    Paused: 1,
    Pending: 2,
    Failed: 3,
    Completed: 4,
    Cancelled: 5,
  };

  return [...activeTasks()].sort((left, right) => {
    const byStatus = statusRank[left.status] - statusRank[right.status];
    if (byStatus !== 0) return byStatus;
    return right.created_at - left.created_at;
  });
});

export const taskSummary = createMemo(() => {
  const list = activeTasks();
  const knownSizeTasks = list.filter((task) => task.total_size > 0);

  const totalDownloaded = knownSizeTasks.reduce(
    (sum, task) => sum + task.downloaded,
    0,
  );
  const totalSize = knownSizeTasks.reduce((sum, task) => sum + task.total_size, 0);
  const totalSpeed = list.reduce((sum, task) => sum + task.speed, 0);

  return {
    active: list.length,
    downloading: list.filter((task) => task.status === "Downloading").length,
    paused: list.filter((task) => task.status === "Paused").length,
    pending: list.filter((task) => task.status === "Pending").length,
    totalSpeed,
    totalDownloaded,
    totalSize,
    progress:
      totalSize > 0 ? Math.min(100, (totalDownloaded / totalSize) * 100) : null,
  };
});

export async function loadTasks() {
  const list = await getTasks();
  setTasks(
    list.map((task) => {
      const existing = tasks.find((current) => current.id === task.id);
      return {
        ...task,
        chunk_progress: task.chunk_progress ?? existing?.chunk_progress ?? null,
      };
    }),
  );
}

export async function refreshTasks() {
  await loadTasks();
}

export function setupTaskEvents() {
  onDownloadEvent(async (event: DownloaderEvent) => {
    if (event.type === "Task") {
      if ("Started" in event.data) {
        const { task_id } = event.data.Started;
        setTasks((t) => t.id === task_id, {
          status: "Downloading" as TaskStatus,
          error: null,
        });
      }
      if ("Paused" in event.data) {
        const { task_id } = event.data.Paused;
        setTasks((t) => t.id === task_id, {
          status: "Paused" as TaskStatus,
          speed: 0,
          eta: null,
        });
      }
      if ("Resumed" in event.data) {
        const { task_id } = event.data.Resumed;
        setTasks((t) => t.id === task_id, {
          status: "Pending" as TaskStatus,
          error: null,
        });
      }
      if ("Cancelled" in event.data) {
        const { task_id } = event.data.Cancelled;
        setTasks((t) => t.id === task_id, {
          status: "Cancelled" as TaskStatus,
          speed: 0,
          eta: null,
        });
      }
      if ("Completed" in event.data) {
        const { task_id } = event.data.Completed;
        setTasks((t) => t.id === task_id, {
          status: "Completed" as TaskStatus,
          speed: 0,
          eta: 0,
        });
      }
      if ("Failed" in event.data) {
        const { task_id, error } = event.data.Failed;
        setTasks((t) => t.id === task_id, {
          status: "Failed" as TaskStatus,
          error,
          speed: 0,
          eta: null,
        });
      }

      await refreshTasks();
      if ("Completed" in event.data) {
        await refreshHistory();
      }
      return;
    }

    if (event.type === "Progress" && "Initialized" in event.data) {
      const { task_id, total_size, chunks } = event.data.Initialized;
      setTasks((t) => t.id === task_id, {
        total_size: total_size ?? 0,
        chunk_progress: chunks ?? null,
      });
      return;
    }

    if (event.type === "Progress" && "ChunkProgress" in event.data) {
      const { task_id, chunk_index, downloaded, size, complete } = event.data.ChunkProgress;
      const currentTask = tasks.find((task) => task.id === task_id);
      if (!currentTask) return;

      if (!currentTask.chunk_progress || currentTask.chunk_progress.length <= chunk_index) {
        const nextChunks = [...(currentTask.chunk_progress ?? [])];
        nextChunks[chunk_index] = { index: chunk_index, downloaded, size, complete };
        setTasks((t) => t.id === task_id, {
          chunk_progress: nextChunks,
        });
        return;
      }

      setTasks((t) => t.id === task_id, "chunk_progress", chunk_index, {
        index: chunk_index,
        downloaded,
        size,
        complete,
      });
      return;
    }

    if (event.type === "Progress" && "Updated" in event.data) {
      const { task_id, downloaded, total, speed, eta } = event.data.Updated;
      const currentTask = tasks.find((task) => task.id === task_id);
      if (
        currentTask &&
        (currentTask.status === "Paused" ||
          currentTask.status === "Completed" ||
          currentTask.status === "Failed" ||
          currentTask.status === "Cancelled")
      ) {
        return;
      }

      setTasks((t) => t.id === task_id, {
        downloaded,
        total_size: total,
        speed,
        eta,
        status: "Downloading" as TaskStatus,
      });
    }
    if (event.type === "Progress" && "BtStatus" in event.data) {
      const { task_id, peers, seeders, upload_speed, uploaded } = event.data.BtStatus;
      const currentTask = tasks.find((task) => task.id === task_id);
      if (
        currentTask &&
        (currentTask.status === "Paused" ||
          currentTask.status === "Completed" ||
          currentTask.status === "Failed" ||
          currentTask.status === "Cancelled")
      ) {
        return;
      }

      setTasks((t) => t.id === task_id, "bt_info", {
        peers,
        seeders,
        upload_speed,
        uploaded,
        selected_files: null,
      });
    }
  });
}

export { tasks, filter, setFilter };
