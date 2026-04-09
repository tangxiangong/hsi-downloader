use crate::utils::{Unit, XByte};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Arc};

/// 下载完成回调类型
pub type CompletionCallback = Arc<
    dyn Fn(
            String,
            Result<(), String>,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
        + Send
        + Sync,
>;

// ==================== 枚举类型 ====================

/// 任务优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum TaskPriority {
    /// 低优先级
    Low = 0,
    #[default]
    /// 普通优先级
    Normal = 1,
    /// 高优先级
    High = 2,
}

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// 等待开始
    Pending,
    /// 正在下载
    Downloading,
    /// 已暂停
    Paused,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已取消
    Cancelled,
}

/// 文件校验类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChecksumType {
    /// MD5 校验
    Md5(String),
    /// SHA256 校验
    Sha256(String),
}

/// 下载来源
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DownloadSource {
    /// HTTP/HTTPS 下载
    Http { url: String },
    /// BitTorrent 下载（磁力链接或 .torrent 文件路径）
    BitTorrent { uri: String },
}

/// BitTorrent 任务扩展信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BtTaskInfo {
    /// 当前连接的 peers 数
    pub peers: u32,
    /// 当前连接的 seeders 数
    pub seeders: u32,
    /// 上传速度（字节/秒）
    pub upload_speed: u64,
    /// 已上传总量（字节）
    pub uploaded: u64,
    /// 选择下载的文件索引列表
    pub selected_files: Option<Vec<usize>>,
}

// ==================== 事件类型 ====================

/// 下载器事件（统一的事件类型）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum DownloaderEvent {
    /// 任务相关事件
    Task(TaskEvent),
    /// 进度相关事件
    Progress(ProgressEvent),
    /// 校验相关事件
    Verification(VerificationEvent),
}

/// 任务事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskEvent {
    /// 任务已添加
    Added { task_id: String },
    /// 任务开始下载
    Started { task_id: String },
    /// 任务完成
    Completed { task_id: String },
    /// 任务失败
    Failed { task_id: String, error: String },
    /// 任务暂停
    Paused { task_id: String },
    /// 任务恢复
    Resumed { task_id: String },
    /// 任务取消
    Cancelled { task_id: String },
}

/// 进度事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgressEvent {
    /// 初始化完成，获取到文件总大小
    Initialized {
        task_id: String,
        total_size: Option<u64>,
    },
    /// 进度更新
    Updated {
        task_id: String,
        downloaded: u64,
        total: u64,
        speed: u64,
        eta: Option<u64>,
    },
    /// 分块下载进度更新（内部使用）
    ChunkProgress {
        task_id: String,
        chunk_index: usize,
        delta: u64,
    },
    /// 流式下载进度更新（内部使用）
    StreamProgress { task_id: String, downloaded: u64 },
    /// 下载完成（内部使用）
    Finished { task_id: String },
    /// 下载失败（内部使用）
    Failed { task_id: String, error: String },

    // 向后兼容的变体
    /// 分块下载进度更新（向后兼容）
    ChunkDownloading { chunk_index: usize, delta: u64 },
    /// 流式下载进度更新（向后兼容）
    StreamDownloading { downloaded: u64 },
    /// BitTorrent 扩展状态
    BtStatus {
        task_id: String,
        peers: u32,
        seeders: u32,
        upload_speed: u64,
        uploaded: u64,
    },
}

/// 校验事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationEvent {
    /// 校验开始
    Started { task_id: String },
    /// 校验完成
    Completed { task_id: String, success: bool },
}

// ==================== 兼容性别名 ====================

/// 队列事件（向后兼容）
pub type QueueEvent = DownloaderEvent;
/// 任务优先级（向后兼容）
pub type Priority = TaskPriority;
/// 下载完成回调（向后兼容）
pub type DownloadCallback = CompletionCallback;

// ==================== 任务和配置类型 ====================

fn default_http_source() -> DownloadSource {
    DownloadSource::Http {
        url: String::new(),
    }
}

/// 下载任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 任务唯一标识符
    pub id: String,
    /// 下载 URL
    pub url: String,
    /// 目标文件路径
    pub dest: PathBuf,
    /// 当前状态
    pub status: TaskStatus,
    /// 文件总大小（字节）
    pub total_size: u64,
    /// 已下载大小（字节）
    pub downloaded: u64,
    /// 创建时间戳（Unix 时间）
    pub created_at: u64,
    /// 错误信息（如果失败）
    pub error: Option<String>,
    /// 任务优先级
    #[serde(default)]
    pub priority: TaskPriority,
    /// 当前下载速度（字节/秒）
    #[serde(default)]
    pub speed: u64,
    /// 预计剩余时间（秒）
    #[serde(default)]
    pub eta: Option<u64>,
    /// 自定义 HTTP 头
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// 文件校验
    #[serde(default)]
    pub checksum: Option<ChecksumType>,
    /// 任务速度限制（字节/秒）
    #[serde(default)]
    pub speed_limit: Option<u64>,
    /// 下载来源（HTTP 或 BitTorrent）
    #[serde(default = "default_http_source")]
    pub source: DownloadSource,
    /// BitTorrent 扩展信息
    #[serde(default)]
    pub bt_info: Option<BtTaskInfo>,
}

/// 下载任务（向后兼容）
pub type DownloadTask = Task;

/// 下载器配置
#[derive(Debug, Clone)]
pub struct Config {
    /// 最大并发连接数
    pub max_concurrent: usize,
    /// 分块大小（字节）
    pub chunk_size: u64,
    /// 速度限制（字节/秒），None 表示不限速
    pub speed_limit: Option<u64>,
    /// 自定义 HTTP 头
    pub headers: HashMap<String, String>,
    /// 代理 URL
    pub proxy: Option<String>,
    /// 连接超时（秒）
    pub timeout: u64,
    /// 用户代理
    pub user_agent: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_concurrent: 4,
            chunk_size: XByte::new(10, 0, Unit::MB).to_bytes(),
            speed_limit: None,
            headers: HashMap::new(),
            proxy: None,
            timeout: 30,
            user_agent: Some("YuShi/1.0".to_string()),
        }
    }
}

/// 下载配置（向后兼容）
pub type DownloadConfig = Config;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_source_serde_http() {
        let source = DownloadSource::Http {
            url: "https://example.com/file.zip".to_string(),
        };
        let json = serde_json::to_string(&source).unwrap();
        assert!(json.contains("\"type\":\"Http\""));
        let deserialized: DownloadSource = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, DownloadSource::Http { .. }));
    }

    #[test]
    fn download_source_serde_bittorrent() {
        let source = DownloadSource::BitTorrent {
            uri: "magnet:?xt=urn:btih:abc123".to_string(),
        };
        let json = serde_json::to_string(&source).unwrap();
        assert!(json.contains("\"type\":\"BitTorrent\""));
        let deserialized: DownloadSource = serde_json::from_str(&json).unwrap();
        match deserialized {
            DownloadSource::BitTorrent { uri } => {
                assert_eq!(uri, "magnet:?xt=urn:btih:abc123");
            }
            _ => panic!("expected BitTorrent variant"),
        }
    }

    #[test]
    fn bt_task_info_default() {
        let info = BtTaskInfo::default();
        assert_eq!(info.peers, 0);
        assert_eq!(info.seeders, 0);
        assert_eq!(info.upload_speed, 0);
        assert_eq!(info.uploaded, 0);
        assert!(info.selected_files.is_none());
    }

    #[test]
    fn task_backward_compat_deserialize() {
        let json = r#"{
            "id": "test-id",
            "url": "https://example.com/file.zip",
            "dest": "/tmp/file.zip",
            "status": "Pending",
            "total_size": 0,
            "downloaded": 0,
            "created_at": 1700000000,
            "error": null,
            "priority": "Normal",
            "speed": 0,
            "eta": null,
            "headers": {},
            "checksum": null,
            "speed_limit": null
        }"#;
        let task: Task = serde_json::from_str(json).unwrap();
        match &task.source {
            DownloadSource::Http { url } => assert_eq!(url, ""),
            _ => panic!("expected Http source for backward compat"),
        }
        assert!(task.bt_info.is_none());
    }

    #[test]
    fn progress_event_bt_status_serde() {
        let event = ProgressEvent::BtStatus {
            task_id: "task-1".to_string(),
            peers: 10,
            seeders: 5,
            upload_speed: 1024,
            uploaded: 5000,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("BtStatus"));
        let deserialized: ProgressEvent = serde_json::from_str(&json).unwrap();
        match deserialized {
            ProgressEvent::BtStatus { peers, seeders, .. } => {
                assert_eq!(peers, 10);
                assert_eq!(seeders, 5);
            }
            _ => panic!("expected BtStatus"),
        }
    }
}
