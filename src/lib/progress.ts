import type { ChunkProgressInfo, DownloadTask } from "./types";

export interface ConcurrentProgressSegment {
  index: number;
  downloaded: number;
  size: number;
  percent: number;
  complete: boolean;
}

function clampPercent(value: number): number {
  if (!Number.isFinite(value)) return 0;
  return Math.max(0, Math.min(100, value));
}

function clampBytes(value: number, max: number): number {
  if (!Number.isFinite(value)) return 0;
  return Math.max(0, Math.min(max, value));
}

function normalizedChunks(chunks: ChunkProgressInfo[]): ChunkProgressInfo[] {
  return chunks
    .filter((chunk) => chunk && chunk.size > 0)
    .map((chunk) => ({
      ...chunk,
      downloaded: clampBytes(chunk.downloaded, chunk.size),
    }))
    .sort((left, right) => left.index - right.index);
}

export function chunkProgressPercent(
  chunks: ChunkProgressInfo[] | null | undefined,
): number | null {
  const list = normalizedChunks(chunks ?? []);
  if (list.length === 0) return null;

  const downloaded = list.reduce((sum, chunk) => sum + chunk.downloaded, 0);
  const size = list.reduce((sum, chunk) => sum + chunk.size, 0);
  return size > 0 ? clampPercent((downloaded / size) * 100) : null;
}

export function taskProgressPercent(task: DownloadTask): number {
  if (task.status === "Completed") return 100;

  const chunkPercent = chunkProgressPercent(task.chunk_progress);
  if (chunkPercent != null) return chunkPercent;

  if (task.total_size <= 0) return 0;
  return clampPercent((task.downloaded / task.total_size) * 100);
}

export function buildConcurrentProgressSegments(
  chunks: ChunkProgressInfo[] | null | undefined,
  concurrency: number,
): ConcurrentProgressSegment[] {
  const list = normalizedChunks(chunks ?? []);
  if (list.length === 0) return [];

  const laneCount = Math.max(
    1,
    Math.min(Math.floor(concurrency) || 1, list.length),
  );
  const segments: ConcurrentProgressSegment[] = Array.from(
    { length: laneCount },
    (_, index) => ({
      index,
      downloaded: 0,
      size: 0,
      percent: 0,
      complete: true,
    }),
  );

  for (const chunk of list) {
    const segment = segments[chunk.index % laneCount];
    segment.downloaded += chunk.downloaded;
    segment.size += chunk.size;
    segment.complete = segment.complete && chunk.complete;
  }

  return segments.map((segment) => ({
    ...segment,
    percent:
      segment.size > 0
        ? clampPercent((segment.downloaded / segment.size) * 100)
        : 0,
  }));
}
