import { invoke } from "@tauri-apps/api/core";
import type {
  AddTaskOptions,
  AppConfig,
  CompletedTask,
  DownloadTask,
  TorrentFileInfo,
} from "./types";

export async function getTasks(): Promise<DownloadTask[]> {
  return invoke("get_tasks");
}

export async function addTask(options: AddTaskOptions): Promise<string> {
  return invoke("add_task", { options });
}

export async function pauseTask(taskId: string): Promise<void> {
  return invoke("pause_task", { taskId });
}

export async function resumeTask(taskId: string): Promise<void> {
  return invoke("resume_task", { taskId });
}

export async function cancelTask(taskId: string): Promise<void> {
  return invoke("cancel_task", { taskId });
}

export async function removeTask(taskId: string): Promise<void> {
  return invoke("remove_task", { taskId });
}

export async function removeTaskWithFile(taskId: string): Promise<void> {
  return invoke("remove_task_with_file", { taskId });
}

export async function clearCompleted(): Promise<void> {
  return invoke("clear_completed");
}

export async function getHistory(): Promise<CompletedTask[]> {
  return invoke("get_history");
}

export async function removeHistory(taskId: string): Promise<void> {
  return invoke("remove_history", { taskId });
}

export async function removeHistoryWithFile(taskId: string): Promise<void> {
  return invoke("remove_history_with_file", { taskId });
}

export async function clearHistory(): Promise<void> {
  return invoke("clear_history");
}

export async function getConfig(): Promise<AppConfig> {
  return invoke("get_config");
}

export async function updateConfig(config: AppConfig): Promise<void> {
  return invoke("update_config", { config });
}

export async function listTorrentFiles(
  uri: string,
): Promise<TorrentFileInfo[]> {
  return invoke("list_torrent_files", { uri });
}

export async function inferDestination(
  url: string,
  directory: string,
): Promise<string> {
  return invoke("infer_destination", { url, directory });
}
