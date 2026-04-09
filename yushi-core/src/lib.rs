//! YuShi - 高性能异步下载库
//!
//! 提供统一的下载和队列管理功能，支持断点续传、并发下载等特性。

pub mod bt;
pub mod config;
pub mod downloader;
pub mod error;
pub mod history;
pub mod state;
pub mod storage;
pub mod types;
pub mod utils;

pub use error::*;

// 重新导出公共 API
pub use bt::{BtEngine, detect_source, spawn_bt_progress_poller};
pub use config::{AppConfig, BtConfig};
pub use downloader::YuShi;
pub use history::{CompletedTask, DownloadHistory};
pub use storage::{config_path, history_path, queue_state_path, storage_dir};
pub use types::{
    BtTaskInfo,
    ChecksumType,
    // 回调类型
    CompletionCallback,
    Config,
    DownloadCallback,
    DownloadConfig,
    DownloadSource,

    // 向后兼容别名
    DownloadTask,
    // 事件类型
    DownloaderEvent,
    Priority,
    ProgressEvent,
    QueueEvent,
    // 主要类型
    Task,
    TaskEvent,
    TaskPriority,
    // 枚举类型
    TaskStatus,
    TorrentFileInfo,
    VerificationEvent,
};
pub use utils::{SpeedCalculator, auto_rename, parse_speed_limit, verify_file};
