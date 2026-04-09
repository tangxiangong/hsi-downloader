//! BitTorrent 下载模块
//!
//! 基于 librqbit 实现 BT 下载功能。

use crate::config::BtConfig;
use crate::types::{BtTaskInfo, DownloadSource, DownloaderEvent, ProgressEvent};
use crate::{Error, Result};
use librqbit::{
    AddTorrent, AddTorrentOptions, AddTorrentResponse, ManagedTorrent, Session, SessionOptions,
};
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, mpsc};

// ==================== Protocol Detection ====================

/// 根据 URL 自动检测下载来源类型（HTTP 或 BitTorrent）
pub fn detect_source(url: &str) -> DownloadSource {
    if url.starts_with("magnet:") {
        return DownloadSource::BitTorrent {
            uri: url.to_string(),
        };
    }
    // Strip query string and fragment for extension check
    let path = url.split(['?', '#']).next().unwrap_or(url);
    if path.ends_with(".torrent") {
        return DownloadSource::BitTorrent {
            uri: url.to_string(),
        };
    }
    DownloadSource::Http {
        url: url.to_string(),
    }
}

// ==================== BtEngine ====================

/// BitTorrent 下载引擎，封装 librqbit Session
pub struct BtEngine {
    session: Arc<Session>,
    handles: RwLock<HashMap<String, Arc<ManagedTorrent>>>,
}

impl BtEngine {
    /// 创建新的 BtEngine 实例
    pub async fn new(output_dir: PathBuf, config: &BtConfig) -> Result<Self> {
        let ratelimits = if let Some(upload_limit) = config.upload_limit
            && let Some(limit) = NonZeroU32::new(upload_limit as u32)
        {
            librqbit::limits::LimitsConfig {
                upload_bps: Some(limit),
                download_bps: None,
            }
        } else {
            Default::default()
        };

        let mut opts = SessionOptions {
            disable_dht: !config.dht_enabled,
            ratelimits,
            ..Default::default()
        };

        if let Some(port) = config.listen_port {
            opts.listen_port_range = Some(port..port.saturating_add(1));
        }

        let session = Session::new_with_opts(output_dir, opts)
            .await
            .map_err(|e| Error::BtError(format!("failed to create BT session: {e}")))?;

        Ok(Self {
            session,
            handles: RwLock::new(HashMap::new()),
        })
    }

    /// 添加种子任务，等待元数据初始化完成，返回 (total_size, name)
    pub async fn add_torrent(
        &self,
        task_id: &str,
        uri: &str,
        output_folder: Option<String>,
        selected_files: Option<Vec<usize>>,
    ) -> Result<(Option<u64>, Option<String>)> {
        let add = if uri.starts_with("magnet:")
            || uri.starts_with("http://")
            || uri.starts_with("https://")
        {
            AddTorrent::from_url(uri)
        } else {
            // Treat as local .torrent file path
            let bytes = fs_err::tokio::read(uri).await?;
            AddTorrent::TorrentFileBytes(bytes.into())
        };

        let mut opts = AddTorrentOptions {
            overwrite: true,
            ..Default::default()
        };
        if let Some(folder) = output_folder {
            opts.output_folder = Some(folder);
        }
        if let Some(files) = selected_files {
            opts.only_files = Some(files);
        }

        let response = self
            .session
            .add_torrent(add, Some(opts))
            .await
            .map_err(|e| Error::BtError(format!("failed to add torrent: {e}")))?;

        let handle = match response {
            AddTorrentResponse::Added(_, handle)
            | AddTorrentResponse::AlreadyManaged(_, handle) => handle,
            AddTorrentResponse::ListOnly(_) => {
                return Err(Error::BtError("torrent added in list-only mode".into()));
            }
        };

        // Wait for metadata initialization
        handle
            .wait_until_initialized()
            .await
            .map_err(|e| Error::BtError(format!("torrent initialization failed: {e}")))?;

        let stats = handle.stats();
        let total_size = if stats.total_bytes > 0 {
            Some(stats.total_bytes)
        } else {
            None
        };
        let name = handle.name();

        self.handles
            .write()
            .await
            .insert(task_id.to_string(), handle);

        Ok((total_size, name))
    }

    /// 获取任务的 BT 扩展信息
    pub async fn get_stats(&self, task_id: &str) -> Option<BtTaskInfo> {
        let handles = self.handles.read().await;
        let handle = handles.get(task_id)?;
        let stats = handle.stats();

        let (peers, upload_speed, uploaded) = if let Some(live) = &stats.live {
            let ps = &live.snapshot.peer_stats;
            let peers = (ps.queued + ps.connecting + ps.live) as u32;
            let upload_speed = (live.upload_speed.mbps * 1024.0 * 1024.0) as u64;
            let uploaded = stats.uploaded_bytes;
            (peers, upload_speed, uploaded)
        } else {
            (0, 0, stats.uploaded_bytes)
        };

        let selected_files = handle.only_files();

        Some(BtTaskInfo {
            peers,
            seeders: 0, // librqbit doesn't distinguish seeders from leechers
            upload_speed,
            uploaded,
            selected_files,
        })
    }

    /// 获取下载进度: (downloaded, total, finished)
    pub async fn get_progress(&self, task_id: &str) -> Option<(u64, u64, bool)> {
        let handles = self.handles.read().await;
        let handle = handles.get(task_id)?;
        let stats = handle.stats();
        Some((stats.progress_bytes, stats.total_bytes, stats.finished))
    }

    /// 获取下载速度（字节/秒）
    pub async fn get_speed(&self, task_id: &str) -> Option<u64> {
        let handles = self.handles.read().await;
        let handle = handles.get(task_id)?;
        let stats = handle.stats();
        stats
            .live
            .as_ref()
            .map(|live| (live.download_speed.mbps * 1024.0 * 1024.0) as u64)
    }

    /// 暂停任务
    pub async fn pause(&self, task_id: &str) -> Result<()> {
        let handles = self.handles.read().await;
        let handle = handles.get(task_id).ok_or(Error::TaskNotFound)?;
        self.session
            .pause(handle)
            .await
            .map_err(|e| Error::BtError(format!("failed to pause torrent: {e}")))
    }

    /// 恢复任务
    pub async fn resume(&self, task_id: &str) -> Result<()> {
        let handles = self.handles.read().await;
        let handle = handles.get(task_id).ok_or(Error::TaskNotFound)?;
        self.session
            .unpause(handle)
            .await
            .map_err(|e| Error::BtError(format!("failed to resume torrent: {e}")))
    }

    /// 取消任务
    pub async fn cancel(&self, task_id: &str, delete_files: bool) -> Result<()> {
        let handle = {
            let handles = self.handles.read().await;
            handles.get(task_id).ok_or(Error::TaskNotFound)?.clone()
        };
        let torrent_id = handle.id();
        self.session
            .delete(librqbit::api::TorrentIdOrHash::Id(torrent_id), delete_files)
            .await
            .map_err(|e| Error::BtError(format!("failed to cancel torrent: {e}")))?;
        self.handles.write().await.remove(task_id);
        Ok(())
    }

    /// 检查任务是否已暂停
    pub async fn is_paused(&self, task_id: &str) -> bool {
        let handles = self.handles.read().await;
        handles.get(task_id).map(|h| h.is_paused()).unwrap_or(false)
    }

    /// 移除 handle（完成后清理）
    pub async fn remove_handle(&self, task_id: &str) {
        self.handles.write().await.remove(task_id);
    }
}

// ==================== Progress Poller ====================

/// 启动 BT 进度轮询任务
///
/// 每秒轮询一次，更新 Task 字段并发送 ProgressEvent。
/// 下载完成后根据做种比例决定是否继续做种。
pub fn spawn_bt_progress_poller(
    bt_engine: Arc<BtEngine>,
    task_id: String,
    event_tx: mpsc::Sender<DownloaderEvent>,
    tasks: Arc<RwLock<HashMap<String, crate::types::Task>>>,
    seed_ratio: Option<f64>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;

            let progress = bt_engine.get_progress(&task_id).await;
            let Some((downloaded, total, finished)) = progress else {
                // Handle removed, stop polling
                break;
            };

            // Get speed and BT stats
            let speed = bt_engine.get_speed(&task_id).await.unwrap_or(0);
            let eta = if speed > 0 && total > downloaded {
                Some((total - downloaded) / speed)
            } else {
                None
            };

            let bt_info = bt_engine.get_stats(&task_id).await;

            // Update the shared task map
            {
                let mut tasks_guard = tasks.write().await;
                if let Some(task) = tasks_guard.get_mut(&task_id) {
                    task.downloaded = downloaded;
                    task.total_size = total;
                    task.speed = speed;
                    task.eta = eta;
                    if let Some(ref info) = bt_info {
                        task.bt_info = Some(info.clone());
                    }
                }
            }

            // Emit progress event
            let _ = event_tx
                .send(DownloaderEvent::Progress(ProgressEvent::Updated {
                    task_id: task_id.clone(),
                    downloaded,
                    total,
                    speed,
                    eta,
                }))
                .await;

            // Emit BT status event
            if let Some(ref info) = bt_info {
                let _ = event_tx
                    .send(DownloaderEvent::Progress(ProgressEvent::BtStatus {
                        task_id: task_id.clone(),
                        peers: info.peers,
                        seeders: info.seeders,
                        upload_speed: info.upload_speed,
                        uploaded: info.uploaded,
                    }))
                    .await;
            }

            if finished {
                // Check seed ratio
                if let Some(ratio) = seed_ratio {
                    if total > 0 {
                        let uploaded = bt_info.as_ref().map(|i| i.uploaded).unwrap_or(0);
                        let current_ratio = uploaded as f64 / total as f64;
                        if current_ratio >= ratio {
                            break;
                        }
                        // Keep seeding, continue polling
                    } else {
                        break;
                    }
                } else {
                    // No seed ratio set, done
                    break;
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_source_magnet() {
        let source = detect_source("magnet:?xt=urn:btih:abc123&dn=test");
        assert!(
            matches!(source, DownloadSource::BitTorrent { .. }),
            "magnet link should be detected as BitTorrent"
        );
    }

    #[test]
    fn detect_source_torrent_file_path() {
        let source = detect_source("/tmp/test.torrent");
        assert!(
            matches!(source, DownloadSource::BitTorrent { .. }),
            "local .torrent file should be detected as BitTorrent"
        );
    }

    #[test]
    fn detect_source_torrent_url() {
        let source = detect_source("https://example.com/file.torrent");
        assert!(
            matches!(source, DownloadSource::BitTorrent { .. }),
            "URL ending in .torrent should be detected as BitTorrent"
        );
    }

    #[test]
    fn detect_source_torrent_url_with_query() {
        let source = detect_source("https://example.com/file.torrent?token=abc");
        assert!(
            matches!(source, DownloadSource::BitTorrent { .. }),
            ".torrent URL with query param should be detected as BitTorrent"
        );
    }

    #[test]
    fn detect_source_torrent_url_with_fragment() {
        let source = detect_source("https://example.com/file.torrent#section");
        assert!(
            matches!(source, DownloadSource::BitTorrent { .. }),
            ".torrent URL with fragment should be detected as BitTorrent"
        );
    }

    #[test]
    fn detect_source_http_zip() {
        let source = detect_source("https://example.com/file.zip");
        match source {
            DownloadSource::Http { url } => {
                assert_eq!(url, "https://example.com/file.zip");
            }
            _ => panic!("zip URL should be detected as Http"),
        }
    }

    #[test]
    fn detect_source_http_no_extension() {
        let source = detect_source("https://example.com/download");
        assert!(
            matches!(source, DownloadSource::Http { .. }),
            "URL without extension should be detected as Http"
        );
    }
}
