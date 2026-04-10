use crate::{Result, storage, types::Task};
use fs_err::tokio as fs;
use serde::{Deserialize, Serialize};
use std::path::Path;

// ==================== 内部状态类型 ====================

/// 分块下载状态
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ChunkState {
    pub index: usize,
    pub start: u64,
    pub end: u64,
    pub current: u64,
    pub is_finished: bool,
}

/// 单文件下载状态
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct FileDownloadState {
    pub url: String,
    /// 文件总大小，None 表示未知（流式下载）
    pub total_size: Option<u64>,
    /// 已下载字节数（流式下载恢复用）
    #[serde(default)]
    pub downloaded: u64,
    pub chunks: Vec<ChunkState>,
    /// 是否为流式下载模式
    pub is_streaming: bool,
}

impl FileDownloadState {
    /// 保存状态到文件
    pub async fn save(&self, path: &Path) -> Result<()> {
        let data = serde_json::to_string(self)?;
        storage::atomic_write_string(path, &data).await?;
        Ok(())
    }

    /// 从文件加载状态
    pub async fn load(path: &Path) -> Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(path).await?;
        let state = serde_json::from_str(&content)?;
        Ok(Some(state))
    }
}

/// 下载器状态（队列状态）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DownloaderState {
    /// 版本号（用于状态文件兼容性）
    #[serde(default = "default_version")]
    pub version: String,
    /// 任务列表
    pub tasks: Vec<Task>,
    /// 创建时间戳
    #[serde(default = "current_timestamp")]
    pub created_at: u64,
    /// 最后更新时间戳
    #[serde(default = "current_timestamp")]
    pub updated_at: u64,
}

impl DownloaderState {
    /// 创建新的下载器状态
    pub fn new() -> Self {
        let now = current_timestamp();
        Self {
            version: default_version(),
            tasks: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// 保存下载器状态到文件
    pub async fn save(&self, path: &Path) -> Result<()> {
        let mut state = self.clone();
        state.updated_at = current_timestamp();

        let data = serde_json::to_string_pretty(&state)?;
        storage::atomic_write_string(path, &data).await?;
        Ok(())
    }

    /// 从文件加载下载器状态
    pub async fn load(path: &Path) -> Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(path).await?;
        let state = serde_json::from_str(&content)?;
        Ok(Some(state))
    }
}

impl Default for DownloaderState {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== 兼容性别名 ====================

/// 下载状态（向后兼容）
pub(crate) type DownloadState = FileDownloadState;
/// 队列状态（向后兼容）
pub(crate) type QueueState = DownloaderState;

// ==================== 辅助函数 ====================

fn default_version() -> String {
    "1.0".to_string()
}

pub(crate) fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
