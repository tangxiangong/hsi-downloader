export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const val = bytes / Math.pow(1024, i);
  return `${val.toFixed(i === 0 ? 0 : 2)} ${units[i]}`;
}

export function formatSpeed(bytesPerSec: number): string {
  return `${formatBytes(bytesPerSec)}/s`;
}

export function formatEta(seconds: number | null): string {
  if (seconds == null || seconds <= 0) return "--";
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = seconds % 60;
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m ${s}s`;
  return `${s}s`;
}

export function formatDate(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString();
}

export function statusLabel(status: string): string {
  const labels: Record<string, string> = {
    Pending: "等待中",
    Downloading: "下载中",
    Paused: "已暂停",
    Completed: "已完成",
    Failed: "失败",
    Cancelled: "已取消",
  };
  return labels[status] ?? status;
}

export function statusBadgeClass(status: string): string {
  const classes: Record<string, string> = {
    Pending: "badge-ghost",
    Downloading: "badge-info",
    Paused: "badge-warning",
    Completed: "badge-success",
    Failed: "badge-error",
    Cancelled: "badge-ghost",
  };
  return classes[status] ?? "badge-ghost";
}
