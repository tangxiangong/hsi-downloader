import {
  buildConcurrentProgressSegments,
  taskProgressPercent,
} from "./progress";
import type { ChunkProgressInfo, DownloadTask } from "./types";

function assertEqual<T>(actual: T, expected: T, message: string) {
  if (actual !== expected) {
    throw new Error(`${message}: expected ${expected}, got ${actual}`);
  }
}

function assertClose(actual: number, expected: number, message: string) {
  if (Math.abs(actual - expected) > 0.0001) {
    throw new Error(`${message}: expected ${expected}, got ${actual}`);
  }
}

function chunk(
  index: number,
  downloaded: number,
  size = 100,
  complete = downloaded >= size,
): ChunkProgressInfo {
  return { index, downloaded, size, complete };
}

function taskWithChunks(
  chunks: ChunkProgressInfo[],
  downloaded = 999,
): DownloadTask {
  return {
    id: "task",
    url: "https://example.com/file.bin",
    dest: "/tmp/file.bin",
    status: "Downloading",
    total_size: 600,
    downloaded,
    created_at: 0,
    error: null,
    priority: "Normal",
    speed: 0,
    eta: null,
    headers: {},
    checksum: null,
    speed_limit: null,
    source: { type: "Http", url: "https://example.com/file.bin" },
    bt_info: null,
    chunk_progress: chunks,
  };
}

const chunks = [
  chunk(0, 100),
  chunk(1, 50, 100, false),
  chunk(2, 0, 100, false),
  chunk(3, 25, 100, false),
  chunk(4, 0, 100, false),
  chunk(5, 0, 100, false),
];

const segments = buildConcurrentProgressSegments(chunks, 3);
assertEqual(segments.length, 3, "segments should follow concurrency count");
assertEqual(segments[0].downloaded, 125, "lane 0 downloaded bytes");
assertEqual(segments[0].size, 200, "lane 0 total bytes");
assertClose(segments[0].percent, 62.5, "lane 0 percent");
assertEqual(segments[1].downloaded, 50, "lane 1 downloaded bytes");
assertEqual(segments[2].downloaded, 0, "lane 2 downloaded bytes");

assertClose(
  taskProgressPercent(taskWithChunks(chunks)),
  175 / 6,
  "task percent should come from chunk aggregation when chunks exist",
);

assertClose(
  taskProgressPercent({ ...taskWithChunks([]), downloaded: 700 }),
  100,
  "task percent should clamp total progress",
);

assertClose(
  taskProgressPercent({ ...taskWithChunks(chunks), status: "Completed" }),
  100,
  "completed tasks should display complete progress",
);
