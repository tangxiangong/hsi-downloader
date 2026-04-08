import { type Component, createSignal } from "solid-js";
import { addTask } from "../lib/commands";
import { refreshTasks } from "../stores/task-store";
import { config } from "../stores/config-store";
import { open } from "@tauri-apps/plugin-dialog";

interface AddTaskDialogProps {
  onClose: () => void;
}

const AddTaskDialog: Component<AddTaskDialogProps> = (props) => {
  const [url, setUrl] = createSignal("");
  const [dest, setDest] = createSignal(config.default_download_path);
  const [error, setError] = createSignal("");
  const [loading, setLoading] = createSignal(false);

  async function pickDir() {
    const selected = await open({ directory: true, defaultPath: dest() });
    if (selected) setDest(selected);
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
      await addTask({ url: url(), dest: dest() });
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
      <div class="modal-box">
        <h3 class="font-bold text-lg">添加下载任务</h3>
        <form onSubmit={handleSubmit} class="mt-4 space-y-4">
          <div class="form-control">
            <label class="label"><span class="label-text">下载链接</span></label>
            <input
              type="text"
              class="input input-bordered w-full"
              placeholder="https://example.com/file.zip"
              value={url()}
              onInput={(e) => setUrl(e.currentTarget.value)}
            />
          </div>
          <div class="form-control">
            <label class="label"><span class="label-text">保存位置</span></label>
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
          {error() && <p class="text-error text-sm">{error()}</p>}
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
