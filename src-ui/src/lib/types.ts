export type TaskStatus =
  | "Pending"
  | "Downloading"
  | "Paused"
  | "Completed"
  | "Failed"
  | "Cancelled";

export type TaskPriority = "Low" | "Normal" | "High";

export type AppTheme = "light" | "dark" | "system";

export interface ChecksumType {
  Md5?: string;
  Sha256?: string;
}

export interface DownloadTask {
  id: string;
  url: string;
  dest: string;
  status: TaskStatus;
  total_size: number;
  downloaded: number;
  created_at: number;
  error: string | null;
  priority: TaskPriority;
  speed: number;
  eta: number | null;
  headers: Record<string, string>;
  checksum: ChecksumType | null;
  speed_limit: number | null;
}

export interface CompletedTask {
  id: string;
  url: string;
  dest: string;
  total_size: number;
  completed_at: number;
  duration: number;
  avg_speed: number;
}

export interface AppConfig {
  default_download_path: string;
  max_concurrent_downloads: number;
  max_concurrent_tasks: number;
  chunk_size: number;
  timeout: number;
  user_agent: string;
  proxy: string | null;
  speed_limit: number | null;
  theme: AppTheme;
}

export interface AddTaskOptions {
  url: string;
  dest: string;
  checksum?: ChecksumType;
  priority?: TaskPriority;
  speed_limit?: number;
  auto_rename_on_conflict?: boolean;
}

export type DownloaderEvent =
  | { type: "Task"; data: TaskEvent }
  | { type: "Progress"; data: ProgressEvent }
  | { type: "Verification"; data: VerificationEvent };

export type TaskEvent =
  | { Added: { task_id: string } }
  | { Started: { task_id: string } }
  | { Completed: { task_id: string } }
  | { Failed: { task_id: string; error: string } }
  | { Paused: { task_id: string } }
  | { Resumed: { task_id: string } }
  | { Cancelled: { task_id: string } };

export type ProgressEvent =
  | { Initialized: { task_id: string; total_size: number | null } }
  | {
      Updated: {
        task_id: string;
        downloaded: number;
        total: number;
        speed: number;
        eta: number | null;
      };
    }
  | { Finished: { task_id: string } }
  | { Failed: { task_id: string; error: string } };

export type VerificationEvent =
  | { Started: { task_id: string } }
  | { Completed: { task_id: string; success: boolean } };
