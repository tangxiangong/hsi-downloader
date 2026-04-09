import { type Component, createSignal, Show, For } from "solid-js";
import { addTask, listTorrentFiles } from "../lib/commands";
import { refreshTasks } from "../stores/task-store";
import { config } from "../stores/config-store";
import { open } from "@tauri-apps/plugin-dialog";
import type { TorrentFileInfo } from "../lib/types";
import { formatBytes } from "../lib/format";

interface AddTaskDialogProps {
  onClose: () => void;
}

function isBtUrl(url: string): boolean {
  return (
    url.startsWith("magnet:") ||
    url.endsWith(".torrent") ||
    url.split(/[?#]/)[0].endsWith(".torrent")
  );
}

const AddTaskDialog: Component<AddTaskDialogProps> = (props) => {
  const [url, setUrl] = createSignal("");
  const [dest, setDest] = createSignal(config.default_download_path);
  const [error, setError] = createSignal("");
  const [loading, setLoading] = createSignal(false);
  const [torrentFiles, setTorrentFiles] = createSignal<TorrentFileInfo[]>([]);
  const [selectedFiles, setSelectedFiles] = createSignal<Set<number>>(new Set());
  const [loadingFiles, setLoadingFiles] = createSignal(false);

  async function pickDir() {
    const selected = await open({ directory: true, defaultPath: dest() });
    if (selected) setDest(selected);
  }

  async function fetchFiles() {
    setLoadingFiles(true);
    setError("");
    try {
      const files = await listTorrentFiles(url());
      setTorrentFiles(files);
      setSelectedFiles(new Set(files.map((f) => f.index)));
    } catch (err) {
      setError(String(err));
    } finally {
      setLoadingFiles(false);
    }
  }

  function selectAll() {
    setSelectedFiles(new Set(torrentFiles().map((f) => f.index)));
  }

  function selectNone() {
    setSelectedFiles(new Set());
  }

  function toggleFile(index: number) {
    const current = new Set(selectedFiles());
    if (current.has(index)) {
      current.delete(index);
    } else {
      current.add(index);
    }
    setSelectedFiles(current);
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!url().trim()) {
      setError("请输入下载链接");
      return;
    }
    setLoading(true);
    setError("");
    try {
      const options: { url: string; dest: string; selected_files?: number[] } = {
        url: url(),
        dest: dest(),
      };
      if (torrentFiles().length > 0) {
        options.selected_files = [...selectedFiles()];
      }
      await addTask(options);
      await refreshTasks();
      props.onClose();
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }

  return (
    <dialog class="modal modal-open">
      <div class="modal-box bg-base-100 border border-base-300">
        <h3 class="font-bold text-lg">{"添加下载任务"}</h3>
        <form onSubmit={handleSubmit} class="mt-4 space-y-4">
          <div class="form-control">
            <label class="label"><span class="label-text text-xs text-base-content/60">{"下载链接"}</span></label>
            <input
              type="text"
              class="input input-bordered w-full"
              placeholder="https://example.com/file.zip"
              value={url()}
              onInput={(e) => {
                setUrl(e.currentTarget.value);
                // Reset torrent files when URL changes
                setTorrentFiles([]);
                setSelectedFiles(new Set());
              }}
            />
          </div>
          <div class="form-control">
            <label class="label"><span class="label-text text-xs text-base-content/60">{"保存位置"}</span></label>
            <div class="flex gap-2">
              <input
                type="text"
                class="input input-bordered flex-1"
                value={dest()}
                onInput={(e) => setDest(e.currentTarget.value)}
              />
              <button type="button" class="btn btn-ghost" onClick={pickDir}>
                {"浏览"}
              </button>
            </div>
          </div>

          <Show when={isBtUrl(url()) && torrentFiles().length === 0}>
            <button
              type="button"
              class="btn btn-sm btn-outline mt-2"
              disabled={loadingFiles()}
              onClick={fetchFiles}
            >
              {loadingFiles() ? "获取中..." : "获取文件列表"}
            </button>
          </Show>

          <Show when={torrentFiles().length > 0}>
            <div class="mt-3">
              <div class="flex justify-between items-center mb-2">
                <span class="label-text font-medium">选择下载文件</span>
                <div class="flex gap-2">
                  <button type="button" class="btn btn-xs" onClick={selectAll}>全选</button>
                  <button type="button" class="btn btn-xs" onClick={selectNone}>取消全选</button>
                </div>
              </div>
              <div class="max-h-48 overflow-y-auto border border-base-300 rounded-lg p-2 space-y-1">
                <For each={torrentFiles()}>
                  {(file) => (
                    <label class="flex items-center gap-2 cursor-pointer hover:bg-base-200 rounded px-1">
                      <input
                        type="checkbox"
                        class="checkbox checkbox-sm"
                        checked={selectedFiles().has(file.index)}
                        onChange={() => toggleFile(file.index)}
                      />
                      <span class="text-sm flex-1 truncate">{file.name}</span>
                      <span class="text-xs text-base-content/50">{formatBytes(file.size)}</span>
                    </label>
                  )}
                </For>
              </div>
            </div>
          </Show>

          {error() && <p class="text-error text-sm">{error()}</p>}
          <div class="modal-action">
            <button type="button" class="btn btn-ghost" onClick={props.onClose}>
              {"取消"}
            </button>
            <button type="submit" class="btn btn-primary" disabled={loading()}>
              {loading() ? "添加中..." : "添加"}
            </button>
          </div>
        </form>
      </div>
      <form method="dialog" class="modal-backdrop">
        <button onClick={props.onClose}>close</button>
      </form>
    </dialog>
  );
};

export default AddTaskDialog;
