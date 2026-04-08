import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { DownloaderEvent } from "./types";

export function onDownloadEvent(
  callback: (event: DownloaderEvent) => void,
): Promise<UnlistenFn> {
  return listen<DownloaderEvent>("download-event", (e) => callback(e.payload));
}
