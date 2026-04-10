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

export function getFileIcon(filename: string): string {
  const ext = filename.split(".").pop()?.toLowerCase() ?? "";
  const icons: Record<string, string> = {
    // Archives
    zip: "\ud83d\udce6",
    rar: "\ud83d\udce6",
    "7z": "\ud83d\udce6",
    tar: "\ud83d\udce6",
    gz: "\ud83d\udce6",
    bz2: "\ud83d\udce6",
    xz: "\ud83d\udce6",
    // Disk images
    iso: "\ud83d\udcbf",
    img: "\ud83d\udcbf",
    dmg: "\ud83d\udcbf",
    // Documents
    pdf: "\ud83d\udcc4",
    doc: "\ud83d\udcc4",
    docx: "\ud83d\udcc4",
    txt: "\ud83d\udcc4",
    md: "\ud83d\udcc4",
    // Media
    mp4: "\ud83c\udfac",
    mkv: "\ud83c\udfac",
    avi: "\ud83c\udfac",
    mov: "\ud83c\udfac",
    webm: "\ud83c\udfac",
    mp3: "\ud83c\udfb5",
    flac: "\ud83c\udfb5",
    wav: "\ud83c\udfb5",
    ogg: "\ud83c\udfb5",
    aac: "\ud83c\udfb5",
    // Images
    png: "\ud83d\uddbc\ufe0f",
    jpg: "\ud83d\uddbc\ufe0f",
    jpeg: "\ud83d\uddbc\ufe0f",
    gif: "\ud83d\uddbc\ufe0f",
    svg: "\ud83d\uddbc\ufe0f",
    webp: "\ud83d\uddbc\ufe0f",
    // Code
    js: "\ud83d\udcdd",
    ts: "\ud83d\udcdd",
    py: "\ud83d\udcdd",
    rs: "\ud83d\udcdd",
    go: "\ud83d\udcdd",
    c: "\ud83d\udcdd",
    cpp: "\ud83d\udcdd",
    // Executables
    exe: "\u2699\ufe0f",
    msi: "\u2699\ufe0f",
    deb: "\u2699\ufe0f",
    rpm: "\u2699\ufe0f",
    AppImage: "\u2699\ufe0f",
    appimage: "\u2699\ufe0f",
    // Data
    json: "\ud83d\udccb",
    csv: "\ud83d\udccb",
    xml: "\ud83d\udccb",
    sql: "\ud83d\udccb",
    db: "\ud83d\udccb",
  };
  return icons[ext] ?? "\ud83d\udcce";
}

export function progressClass(status: string): string {
  const classes: Record<string, string> = {
    Downloading: "progress-downloading",
    Completed: "progress-completed",
    Paused: "progress-paused",
    Failed: "progress-failed",
    Cancelled: "progress-failed",
    Pending: "progress-downloading",
  };
  return classes[status] ?? "";
}
