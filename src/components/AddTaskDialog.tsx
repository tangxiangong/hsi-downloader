import {
  type Component,
  createSignal,
  Show,
  For,
  createEffect,
} from "solid-js";
import { addTask, listTorrentFiles, inferDestination } from "../lib/commands";
import { refreshTasks } from "../stores/task-store";
import { config } from "../stores/config-store";
import { open } from "@tauri-apps/plugin-dialog";
import type { TorrentFileInfo, TaskPriority, ChecksumType } from "../lib/types";
import { formatBytes } from "../lib/format";

interface AddTaskDialogProps {
  onClose: () => void;
}

type DownloadType = "http" | "bt";

function isBtUrl(url: string): boolean {
  return (
    url.startsWith("magnet:") ||
    url.endsWith(".torrent") ||
    url.split(/[?#]/)[0].endsWith(".torrent")
  );
}

const AddTaskDialog: Component<AddTaskDialogProps> = (props) => {
  const [tab, setTab] = createSignal<DownloadType>("http");
  const [url, setUrl] = createSignal("");
  const [dest, setDest] = createSignal(config.default_download_path);
  const [priority, setPriority] = createSignal<TaskPriority>("Normal");
  const [speedLimit, setSpeedLimit] = createSignal("");
  const [error, setError] = createSignal("");
  const [loading, setLoading] = createSignal(false);

  // 高级选项
  const [showAdvanced, setShowAdvanced] = createSignal(false);

  // HTTP 高级：校验和
  const [checksumType, setChecksumType] = createSignal<
    "none" | "Md5" | "Sha256"
  >("none");
  const [checksumValue, setChecksumValue] = createSignal("");

  // HTTP 高级：自定义 Headers
  const [headers, setHeaders] = createSignal<{ key: string; value: string }[]>(
    [],
  );

  // BT：文件选择
  const [torrentFiles, setTorrentFiles] = createSignal<TorrentFileInfo[]>([]);
  const [selectedFiles, setSelectedFiles] = createSignal<Set<number>>(
    new Set(),
  );
  const [loadingFiles, setLoadingFiles] = createSignal(false);

  // URL 变更时自动检测类型
  createEffect(() => {
    const u = url();
    if (isBtUrl(u)) {
      setTab("bt");
    }
  });

  // Tab 切换时重置特有状态
  function switchTab(newTab: DownloadType) {
    setTab(newTab);
    setError("");
    if (newTab === "http") {
      setTorrentFiles([]);
      setSelectedFiles(new Set<number>());
    } else {
      setChecksumType("none");
      setChecksumValue("");
      setHeaders([]);
    }
  }

  async function pickDir() {
    const selected = await open({ directory: true, defaultPath: dest() });
    if (selected) setDest(selected);
  }

  // BT 文件操作
  async function fetchFiles() {
    setLoadingFiles(true);
    setError("");
    try {
      const files = await listTorrentFiles(url());
      setTorrentFiles(files);
      setSelectedFiles(new Set<number>(files.map((f) => f.index)));
    } catch (err) {
      setError(String(err));
    } finally {
      setLoadingFiles(false);
    }
  }

  function selectAll() {
    setSelectedFiles(new Set<number>(torrentFiles().map((f) => f.index)));
  }

  function selectNone() {
    setSelectedFiles(new Set<number>());
  }

  function toggleFile(index: number) {
    const current = new Set(selectedFiles());
    if (current.has(index)) current.delete(index);
    else current.add(index);
    setSelectedFiles(current);
  }

  // Headers 操作
  function addHeader() {
    setHeaders([...headers(), { key: "", value: "" }]);
  }

  function removeHeader(index: number) {
    setHeaders(headers().filter((_, i) => i !== index));
  }

  function updateHeader(index: number, field: "key" | "value", val: string) {
    setHeaders(
      headers().map((h, i) => (i === index ? { ...h, [field]: val } : h)),
    );
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
      const isBt = tab() === "bt";
      const finalDest = isBt ? dest() : await inferDestination(url(), dest());

      const options: Record<string, unknown> = {
        url: url(),
        dest: finalDest,
      };

      // 优先级
      if (priority() !== "Normal") {
        options.priority = priority();
      }

      // 速度限制（KB/s → bytes/s）
      const limit = parseInt(speedLimit(), 10);
      if (!isNaN(limit) && limit > 0) {
        options.speed_limit = limit * 1024;
      }

      if (isBt) {
        // BT 文件选择
        if (torrentFiles().length > 0) {
          options.selected_files = [...selectedFiles()];
        }
      } else {
        // HTTP 校验和
        if (checksumType() !== "none" && checksumValue().trim()) {
          options.checksum = { [checksumType()]: checksumValue().trim() };
        }

        // HTTP 自定义 Headers
        const validHeaders = headers().filter(
          (h) => h.key.trim() && h.value.trim(),
        );
        if (validHeaders.length > 0) {
          const headerMap: Record<string, string> = {};
          for (const h of validHeaders) {
            headerMap[h.key.trim()] = h.value.trim();
          }
          options.headers = headerMap;
        }
      }

      await addTask(options as any);
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
      <div class="modal-box bg-base-100 border border-base-300 max-w-lg">
        <h3 class="font-bold text-lg mb-3">添加下载任务</h3>

        {/* Tab 切换 */}
        <div role="tablist" class="tabs tabs-border mb-4">
          <button
            role="tab"
            class={`tab ${tab() === "http" ? "tab-active" : ""}`}
            onClick={() => switchTab("http")}
            type="button"
          >
            HTTP 下载
          </button>
          <button
            role="tab"
            class={`tab ${tab() === "bt" ? "tab-active" : ""}`}
            onClick={() => switchTab("bt")}
            type="button"
          >
            磁力下载
          </button>
        </div>

        <form onSubmit={handleSubmit} class="space-y-3">
          {/* URL */}
          <div class="form-control">
            <label class="label">
              <span class="label-text text-xs text-base-content/60">
                下载链接
              </span>
            </label>
            <input
              type="text"
              class="input input-bordered w-full"
              placeholder={
                tab() === "http"
                  ? "https://example.com/file.zip"
                  : "magnet:?xt=urn:btih:..."
              }
              value={url()}
              onInput={(e) => {
                setUrl(e.currentTarget.value);
                setTorrentFiles([]);
                setSelectedFiles(new Set<number>());
              }}
            />
          </div>

          {/* 保存位置 */}
          <div class="form-control">
            <label class="label">
              <span class="label-text text-xs text-base-content/60">
                保存位置
              </span>
            </label>
            <div class="flex gap-2">
              <input
                type="text"
                class="input input-bordered flex-1"
                value={dest()}
                onInput={(e) => setDest(e.currentTarget.value)}
              />
              <button type="button" class="btn btn-ghost" onClick={pickDir}>
                浏览
              </button>
            </div>
          </div>

          {/* 优先级 */}
          <div class="form-control">
            <label class="label">
              <span class="label-text text-xs text-base-content/60">
                优先级
              </span>
            </label>
            <select
              class="select select-bordered w-full"
              value={priority()}
              onChange={(e) =>
                setPriority(e.currentTarget.value as TaskPriority)
              }
            >
              <option value="Low">低</option>
              <option value="Normal">普通</option>
              <option value="High">高</option>
            </select>
          </div>

          {/* 速度限制 */}
          <div class="form-control">
            <label class="label">
              <span class="label-text text-xs text-base-content/60">
                速度限制 (KB/s，留空不限速)
              </span>
            </label>
            <input
              type="number"
              class="input input-bordered w-full"
              placeholder="不限速"
              value={speedLimit()}
              onInput={(e) => setSpeedLimit(e.currentTarget.value)}
              min="0"
            />
          </div>

          {/* 高级选项折叠区 */}
          <div class="collapse collapse-arrow bg-base-200/50 rounded-lg">
            <input
              type="checkbox"
              checked={showAdvanced()}
              onChange={(e) => setShowAdvanced(e.currentTarget.checked)}
            />
            <div class="collapse-title text-sm font-medium py-2 min-h-0">
              高级选项
            </div>
            <div class="collapse-content space-y-3">
              {/* HTTP 高级选项 */}
              <Show when={tab() === "http"}>
                {/* 校验和 */}
                <div class="form-control">
                  <label class="label">
                    <span class="label-text text-xs text-base-content/60">
                      校验和验证
                    </span>
                  </label>
                  <div class="flex gap-2">
                    <select
                      class="select select-bordered select-sm w-28"
                      value={checksumType()}
                      onChange={(e) =>
                        setChecksumType(
                          e.currentTarget.value as "none" | "Md5" | "Sha256",
                        )
                      }
                    >
                      <option value="none">无</option>
                      <option value="Md5">MD5</option>
                      <option value="Sha256">SHA256</option>
                    </select>
                    <Show when={checksumType() !== "none"}>
                      <input
                        type="text"
                        class="input input-bordered input-sm flex-1"
                        placeholder="输入校验和值"
                        value={checksumValue()}
                        onInput={(e) => setChecksumValue(e.currentTarget.value)}
                      />
                    </Show>
                  </div>
                </div>

                {/* 自定义 Headers */}
                <div class="form-control">
                  <div class="flex items-center justify-between">
                    <label class="label">
                      <span class="label-text text-xs text-base-content/60">
                        自定义 Headers
                      </span>
                    </label>
                    <button
                      type="button"
                      class="btn btn-ghost btn-xs"
                      onClick={addHeader}
                    >
                      + 添加
                    </button>
                  </div>
                  <div class="space-y-1">
                    <For each={headers()}>
                      {(header, index) => (
                        <div class="flex gap-1 items-center">
                          <input
                            type="text"
                            class="input input-bordered input-sm flex-1"
                            placeholder="Header 名称"
                            value={header.key}
                            onInput={(e) =>
                              updateHeader(
                                index(),
                                "key",
                                e.currentTarget.value,
                              )
                            }
                          />
                          <input
                            type="text"
                            class="input input-bordered input-sm flex-1"
                            placeholder="Header 值"
                            value={header.value}
                            onInput={(e) =>
                              updateHeader(
                                index(),
                                "value",
                                e.currentTarget.value,
                              )
                            }
                          />
                          <button
                            type="button"
                            class="btn btn-ghost btn-xs btn-square"
                            onClick={() => removeHeader(index())}
                          >
                            ✕
                          </button>
                        </div>
                      )}
                    </For>
                  </div>
                </div>
              </Show>

              {/* BT 高级选项：文件选择 */}
              <Show when={tab() === "bt"}>
                <Show
                  when={torrentFiles().length > 0}
                  fallback={
                    <button
                      type="button"
                      class="btn btn-sm btn-outline"
                      disabled={loadingFiles() || !url().trim()}
                      onClick={fetchFiles}
                    >
                      {loadingFiles() ? "获取中..." : "获取文件列表"}
                    </button>
                  }
                >
                  <div>
                    <div class="flex justify-between items-center mb-2">
                      <span class="label-text text-sm font-medium">
                        选择下载文件
                      </span>
                      <div class="flex gap-2">
                        <button
                          type="button"
                          class="btn btn-xs"
                          onClick={selectAll}
                        >
                          全选
                        </button>
                        <button
                          type="button"
                          class="btn btn-xs"
                          onClick={selectNone}
                        >
                          取消全选
                        </button>
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
                            <span class="text-sm flex-1 truncate">
                              {file.name}
                            </span>
                            <span class="text-xs text-base-content/50">
                              {formatBytes(file.size)}
                            </span>
                          </label>
                        )}
                      </For>
                    </div>
                  </div>
                </Show>
              </Show>
            </div>
          </div>

          {/* 错误信息 */}
          <Show when={error()}>
            <p class="text-error text-sm">{error()}</p>
          </Show>

          {/* 操作按钮 */}
          <div class="modal-action">
            <button type="button" class="btn btn-ghost" onClick={props.onClose}>
              取消
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
