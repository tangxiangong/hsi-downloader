import { Index, type Component, Show, createMemo } from "solid-js";
import type { ChunkProgressInfo, DownloadTask } from "../lib/types";
import { progressClass } from "../lib/format";

interface ChunkedProgressBarProps {
  task: DownloadTask;
  compact?: boolean;
}

const CHUNK_GAP_REM = 0.125;

function chunkFillStyle(
  chunk: ChunkProgressInfo,
  statusClass: string,
): Record<string, string> {
  const fill =
    chunk.size > 0 ? Math.min(100, (chunk.downloaded / chunk.size) * 100) : 0;
  const background =
    statusClass === "progress-paused"
      ? "var(--color-warning)"
      : statusClass === "progress-failed"
        ? "var(--color-error)"
        : statusClass === "progress-completed"
          ? "var(--color-success)"
          : "linear-gradient(90deg, var(--color-primary), var(--color-accent))";

  return {
    width: `${fill}%`,
    background,
  };
}

const ChunkedProgressBar: Component<ChunkedProgressBarProps> = (props) => {
  const chunks = createMemo(() => props.task.chunk_progress ?? []);
  const isSegmented = createMemo(
    () =>
      chunks().length > 1 &&
      props.task.status !== "Completed" &&
      chunks().some((chunk) => !chunk.complete),
  );
  const mergedProgress = createMemo(() =>
    props.task.total_size > 0
      ? Math.min(100, (props.task.downloaded / props.task.total_size) * 100)
      : 0,
  );
  const trackHeight = () => (props.compact ? "0.4rem" : "0.55rem");
  const statusClass = createMemo(() => progressClass(props.task.status));

  return (
    <Show
      when={isSegmented()}
      fallback={
        <progress
          class={`progress flex-1 ${props.compact ? "h-1.5" : "h-2"} ${statusClass()}`}
          value={mergedProgress()}
          max="100"
        />
      }
    >
      <div
        class="flex flex-1 items-stretch overflow-hidden rounded-full bg-base-300/80"
        style={{ height: trackHeight(), gap: `${CHUNK_GAP_REM}rem` }}
      >
        <Index each={chunks()}>
          {(chunk) => (
            <div
              class="relative min-w-[0.35rem] flex-1 overflow-hidden rounded-full bg-base-300"
              style={{ flex: `${Math.max(chunk().size, 1)} 1 0%` }}
            >
              <div
                class="absolute inset-y-0 left-0 rounded-full"
                style={chunkFillStyle(chunk(), statusClass())}
              />
            </div>
          )}
        </Index>
      </div>
    </Show>
  );
};

export default ChunkedProgressBar;
