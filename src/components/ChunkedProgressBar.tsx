import { Index, type Component, Show, createMemo } from "solid-js";
import type { DownloadTask } from "../lib/types";
import { progressClass } from "../lib/format";
import {
  buildConcurrentProgressSegments,
  type ConcurrentProgressSegment,
  taskProgressPercent,
} from "../lib/progress";

interface ChunkedProgressBarProps {
  task: DownloadTask;
  compact?: boolean;
  concurrency: number;
}

const SEGMENT_GAP_REM = 0.125;

function segmentFillStyle(
  segment: ConcurrentProgressSegment,
  statusClass: string,
): Record<string, string> {
  const background =
    statusClass === "progress-paused"
      ? "var(--color-warning)"
      : statusClass === "progress-failed"
        ? "var(--color-error)"
        : statusClass === "progress-completed"
          ? "var(--color-success)"
          : "linear-gradient(90deg, var(--color-primary), var(--color-accent))";

  return {
    width: `${segment.percent}%`,
    background,
  };
}

const ChunkedProgressBar: Component<ChunkedProgressBarProps> = (props) => {
  const chunks = createMemo(() => props.task.chunk_progress ?? []);
  const segments = createMemo(() =>
    buildConcurrentProgressSegments(chunks(), props.concurrency),
  );
  const isSegmented = createMemo(
    () =>
      segments().length > 1 &&
      props.task.status !== "Completed" &&
      chunks().some((chunk) => !chunk.complete),
  );
  const mergedProgress = createMemo(() => taskProgressPercent(props.task));
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
        style={{ height: trackHeight(), gap: `${SEGMENT_GAP_REM}rem` }}
      >
        <Index each={segments()}>
          {(segment) => (
            <div
              class="relative min-w-[1.75rem] flex-1 overflow-hidden rounded-full bg-base-300"
              style={{ flex: `${Math.max(segment().size, 1)} 1 0%` }}
            >
              <div
                class="absolute inset-y-0 left-0 rounded-full"
                style={segmentFillStyle(segment(), statusClass())}
              />
            </div>
          )}
        </Index>
      </div>
    </Show>
  );
};

export default ChunkedProgressBar;
