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
      setError("\u8bf7\u8f93\u5165\u4e0b\u8f7d\u94fe\u63a5");
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
      <div class="modal-box bg-base-100 border border-base-300">
        <h3 class="font-bold text-lg">{"\u6dfb\u52a0\u4e0b\u8f7d\u4efb\u52a1"}</h3>
        <form onSubmit={handleSubmit} class="mt-4 space-y-4">
          <div class="form-control">
            <label class="label"><span class="label-text text-xs text-base-content/60">{"\u4e0b\u8f7d\u94fe\u63a5"}</span></label>
            <input
              type="text"
              class="input input-bordered w-full"
              placeholder="https://example.com/file.zip"
              value={url()}
              onInput={(e) => setUrl(e.currentTarget.value)}
            />
          </div>
          <div class="form-control">
            <label class="label"><span class="label-text text-xs text-base-content/60">{"\u4fdd\u5b58\u4f4d\u7f6e"}</span></label>
            <div class="flex gap-2">
              <input
                type="text"
                class="input input-bordered flex-1"
                value={dest()}
                onInput={(e) => setDest(e.currentTarget.value)}
              />
              <button type="button" class="btn btn-ghost" onClick={pickDir}>
                {"\u6d4f\u89c8"}
              </button>
            </div>
          </div>
          {error() && <p class="text-error text-sm">{error()}</p>}
          <div class="modal-action">
            <button type="button" class="btn btn-ghost" onClick={props.onClose}>
              {"\u53d6\u6d88"}
            </button>
            <button type="submit" class="btn btn-primary" disabled={loading()}>
              {loading() ? "\u6dfb\u52a0\u4e2d..." : "\u6dfb\u52a0"}
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
