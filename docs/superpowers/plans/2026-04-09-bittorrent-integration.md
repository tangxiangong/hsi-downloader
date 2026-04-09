# BitTorrent Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add BitTorrent download support (magnet links + .torrent files) to YuShi via librqbit, alongside existing HTTP downloads.

**Architecture:** Composition delegation — `YuShi` holds a lazily-initialized `librqbit::Session` for BT tasks while HTTP code paths stay untouched. A new `bt.rs` module owns all BT logic. Task dispatch branches on `DownloadSource` enum at the `start_queue_task()` level.

**Tech Stack:** librqbit 8.x (tokio-based BT engine), serde for serialization, existing YuShi event system.

**Spec:** `docs/superpowers/specs/2026-04-09-bittorrent-integration-design.md`

---

## File Structure

| Action | File | Responsibility |
|--------|------|---------------|
| Modify | `Cargo.toml` (workspace) | Add `librqbit` workspace dependency |
| Modify | `yushi-core/Cargo.toml` | Add `librqbit` dependency |
| Modify | `yushi-core/src/types.rs` | Add `DownloadSource`, `BtTaskInfo`, `BtStatus` variant, extend `Task` |
| Modify | `yushi-core/src/config.rs` | Add `BtConfig`, extend `AppConfig` |
| Modify | `yushi-core/src/error.rs` | Add `BtError` variant |
| Modify | `yushi-core/src/lib.rs` | Add `pub mod bt`, export new types |
| Create | `yushi-core/src/bt.rs` | BT Session management, download logic, stats polling |
| Modify | `yushi-core/src/downloader.rs` | Add BT fields, dispatch branch, pause/resume/cancel for BT |
| Modify | `src-tauri/src/commands.rs` | Add `selected_files` to `AddTaskOptions` |
| Modify | `src-ui/src/lib/types.ts` | Add `DownloadSource`, `BtTaskInfo` types |

---

## Task 1: Add librqbit dependency

**Files:**
- Modify: `Cargo.toml` (workspace root)
- Modify: `yushi-core/Cargo.toml`

- [ ] **Step 1: Add librqbit to workspace dependencies**

In `Cargo.toml` (workspace root), add to `[workspace.dependencies]`:

```toml
librqbit = "8"
```

- [ ] **Step 2: Add librqbit to yushi-core dependencies**

In `yushi-core/Cargo.toml`, add to `[dependencies]`:

```toml
librqbit = { workspace = true }
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p yushi-core`
Expected: Compiles successfully (warnings OK, no errors).

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml yushi-core/Cargo.toml
git commit -m "$(cat <<'EOF'
chore: add librqbit dependency for BitTorrent support
EOF
)"
```

---

## Task 2: Extend data model — DownloadSource, BtTaskInfo, BtStatus

**Files:**
- Modify: `yushi-core/src/types.rs`

- [ ] **Step 1: Write tests for new types**

Add at the bottom of `yushi-core/src/types.rs`, inside a new `#[cfg(test)]` module:

```rust
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
        // Simulate old JSON without `source` or `bt_info` fields
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
            DownloadSource::Http { url } => assert_eq!(url, "https://example.com/file.zip"),
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
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p yushi-core -- types::tests`
Expected: FAIL — `DownloadSource`, `BtTaskInfo`, `BtStatus` not defined.

- [ ] **Step 3: Add DownloadSource enum**

Add after the `ChecksumType` enum in `yushi-core/src/types.rs`:

```rust
/// 下载来源
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DownloadSource {
    /// HTTP/HTTPS 下载
    Http { url: String },
    /// BitTorrent 下载（磁力链接或 .torrent 文件路径）
    BitTorrent { uri: String },
}
```

- [ ] **Step 4: Add BtTaskInfo struct**

Add after `DownloadSource` in `yushi-core/src/types.rs`:

```rust
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
```

- [ ] **Step 5: Add BtStatus variant to ProgressEvent**

Add the `BtStatus` variant to the `ProgressEvent` enum, after the existing `StreamDownloading` variant:

```rust
    /// BitTorrent 扩展状态
    BtStatus {
        task_id: String,
        peers: u32,
        seeders: u32,
        upload_speed: u64,
        uploaded: u64,
    },
```

- [ ] **Step 6: Extend Task struct with source and bt_info**

Add a helper function before the `Task` struct:

```rust
fn default_http_source() -> DownloadSource {
    DownloadSource::Http {
        url: String::new(),
    }
}
```

Add two new fields to the `Task` struct, after the `speed_limit` field:

```rust
    /// 下载来源（HTTP 或 BitTorrent）
    #[serde(default = "default_http_source")]
    pub source: DownloadSource,
    /// BitTorrent 扩展信息
    #[serde(default)]
    pub bt_info: Option<BtTaskInfo>,
```

- [ ] **Step 7: Fix Task construction in downloader.rs**

In `yushi-core/src/downloader.rs`, find the `add_task_with_options` method. Update the `Task` construction (around line 891) to include the new fields. Add after `speed_limit,`:

```rust
            source: DownloadSource::Http {
                url: url.clone(),
            },
            bt_info: None,
```

Add `DownloadSource` to the imports at the top of `downloader.rs`:

```rust
use crate::types::{
    ChecksumType, CompletionCallback, Config, DownloadSource, DownloaderEvent, ProgressEvent,
    Task, TaskEvent, TaskPriority, TaskStatus, VerificationEvent,
};
```

- [ ] **Step 8: Run tests to verify they pass**

Run: `cargo test -p yushi-core -- types::tests`
Expected: All 4 tests PASS.

Run: `cargo test -p yushi-core`
Expected: All existing tests also PASS (backward compat).

- [ ] **Step 9: Commit**

```bash
git add yushi-core/src/types.rs yushi-core/src/downloader.rs
git commit -m "$(cat <<'EOF'
feat(core): add DownloadSource, BtTaskInfo, and BtStatus types

Extend Task with source field (defaults to Http for backward compat)
and optional bt_info for BitTorrent-specific metadata.
EOF
)"
```

---

## Task 3: Add BtConfig and extend AppConfig

**Files:**
- Modify: `yushi-core/src/config.rs`

- [ ] **Step 1: Write tests for BtConfig**

Add to the existing `#[cfg(test)] mod tests` in `yushi-core/src/config.rs`:

```rust
    #[test]
    fn bt_config_default() {
        let config = BtConfig::default();
        assert!(config.dht_enabled);
        assert!(config.upload_limit.is_none());
        assert!(config.seed_ratio.is_none());
        assert!(config.listen_port.is_none());
    }

    #[test]
    fn bt_config_serde_roundtrip() {
        let config = BtConfig {
            dht_enabled: false,
            upload_limit: Some(1024 * 1024),
            seed_ratio: Some(2.0),
            listen_port: Some(6881),
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: BtConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn app_config_backward_compat_without_bt() {
        // Old config JSON without `bt` field should still deserialize
        let json = serde_json::to_string(&AppConfig::default()).unwrap();
        // Remove "bt" key if present to simulate old format
        let old_json = json.replace(r#","bt":{"dht_enabled":true,"upload_limit":null,"seed_ratio":null,"listen_port":null}"#, "");
        let config: AppConfig = serde_json::from_str(&old_json).unwrap();
        assert!(config.bt.dht_enabled); // should default
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p yushi-core -- config::tests`
Expected: FAIL — `BtConfig` not defined.

- [ ] **Step 3: Add BtConfig struct**

Add before `AppConfig` in `yushi-core/src/config.rs`:

```rust
/// BitTorrent 相关配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BtConfig {
    /// 是否启用 DHT（默认 true）
    pub dht_enabled: bool,
    /// 上传限速（字节/秒），None 表示不限制
    pub upload_limit: Option<u64>,
    /// 做种目标比例（如 2.0），达到后停止做种
    pub seed_ratio: Option<f64>,
    /// BT 监听端口，None 表示随机
    pub listen_port: Option<u16>,
}

impl Default for BtConfig {
    fn default() -> Self {
        Self {
            dht_enabled: true,
            upload_limit: None,
            seed_ratio: None,
            listen_port: None,
        }
    }
}
```

- [ ] **Step 4: Add bt field to AppConfig**

Add to `AppConfig` struct, after the `theme` field:

```rust
    /// BitTorrent 配置
    #[serde(default)]
    pub bt: BtConfig,
```

Also add `bt: BtConfig::default(),` to the `Default` impl for `AppConfig`, after `theme: AppTheme::default(),`.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p yushi-core -- config::tests`
Expected: All tests PASS.

Run: `cargo test -p yushi-core`
Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
git add yushi-core/src/config.rs
git commit -m "$(cat <<'EOF'
feat(core): add BtConfig with DHT, upload limit, seed ratio, listen port
EOF
)"
```

---

## Task 4: Add BtError variant

**Files:**
- Modify: `yushi-core/src/error.rs`

- [ ] **Step 1: Add BtError variant**

Add to the `Error` enum in `yushi-core/src/error.rs`, before `Unknown`:

```rust
    #[error("BitTorrent error: {0}")]
    BtError(String),
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p yushi-core`
Expected: Compiles successfully.

- [ ] **Step 3: Commit**

```bash
git add yushi-core/src/error.rs
git commit -m "$(cat <<'EOF'
feat(core): add BtError variant to Error enum
EOF
)"
```

---

## Task 5: Export new types from lib.rs

**Files:**
- Modify: `yushi-core/src/lib.rs`

- [ ] **Step 1: Add bt module and export new types**

Add `pub mod bt;` after `pub mod config;` in `yushi-core/src/lib.rs`.

Add to the `pub use types::` block:

```rust
    BtTaskInfo,
    DownloadSource,
```

Add to the top-level re-exports:

```rust
pub use config::BtConfig;
```

- [ ] **Step 2: Create empty bt.rs module**

Create `yushi-core/src/bt.rs` with:

```rust
//! BitTorrent 下载模块
//!
//! 基于 librqbit 实现 BT 下载功能，包括：
//! - Session 延迟初始化与生命周期管理
//! - 磁力链接和 .torrent 文件下载
//! - 进度轮询与事件转换
//! - 暂停/恢复/取消控制
//! - 做种管理
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p yushi-core`
Expected: Compiles successfully.

- [ ] **Step 4: Commit**

```bash
git add yushi-core/src/lib.rs yushi-core/src/bt.rs
git commit -m "$(cat <<'EOF'
feat(core): add bt module and export new types
EOF
)"
```

---

## Task 6: Implement bt.rs — protocol detection helper

**Files:**
- Create: `yushi-core/src/bt.rs` (replace empty file)

- [ ] **Step 1: Write tests for protocol detection**

Add to `yushi-core/src/bt.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_magnet_link() {
        let source = detect_source("magnet:?xt=urn:btih:abc123&dn=test");
        assert!(matches!(source, DownloadSource::BitTorrent { .. }));
    }

    #[test]
    fn detect_torrent_file_path() {
        let source = detect_source("/tmp/test.torrent");
        assert!(matches!(source, DownloadSource::BitTorrent { .. }));
    }

    #[test]
    fn detect_torrent_url() {
        let source = detect_source("https://example.com/file.torrent");
        assert!(matches!(source, DownloadSource::BitTorrent { .. }));
    }

    #[test]
    fn detect_http_url() {
        let source = detect_source("https://example.com/file.zip");
        assert!(matches!(source, DownloadSource::Http { .. }));
    }

    #[test]
    fn detect_http_url_no_extension() {
        let source = detect_source("https://example.com/download");
        assert!(matches!(source, DownloadSource::Http { .. }));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p yushi-core -- bt::tests`
Expected: FAIL — `detect_source` not defined.

- [ ] **Step 3: Implement detect_source**

Add to `yushi-core/src/bt.rs` (before the tests module):

```rust
use crate::types::DownloadSource;

/// 根据 URL 自动判断下载协议
pub fn detect_source(url: &str) -> DownloadSource {
    if url.starts_with("magnet:") || url.ends_with(".torrent") {
        DownloadSource::BitTorrent {
            uri: url.to_string(),
        }
    } else {
        DownloadSource::Http {
            url: url.to_string(),
        }
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p yushi-core -- bt::tests`
Expected: All 5 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add yushi-core/src/bt.rs
git commit -m "$(cat <<'EOF'
feat(core): implement protocol auto-detection for HTTP vs BitTorrent
EOF
)"
```

---

## Task 7: Implement bt.rs — Session management and BT download

**Files:**
- Modify: `yushi-core/src/bt.rs`

This is the core BT implementation. Due to librqbit being a complete external engine, we cannot write isolated unit tests for session creation (it starts real network listeners). We'll implement the logic and test it via integration tests in Task 9.

- [ ] **Step 1: Add imports and BtEngine struct**

Replace the content of `yushi-core/src/bt.rs` (keep tests at bottom) with:

```rust
//! BitTorrent 下载模块

use crate::config::BtConfig;
use crate::types::{BtTaskInfo, DownloadSource, DownloaderEvent, ProgressEvent, TaskEvent};
use crate::{Error, Result};
use librqbit::{
    AddTorrent, AddTorrentOptions, AddTorrentResponse, ManagedTorrent, Session, SessionOptions,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, mpsc};

/// librqbit Session 的包装，管理 BT 下载生命周期
pub struct BtEngine {
    session: Session,
    /// task_id → ManagedTorrent 映射
    handles: RwLock<HashMap<String, Arc<ManagedTorrent>>>,
}

impl BtEngine {
    /// 创建 BT 引擎
    pub async fn new(output_dir: PathBuf, config: &BtConfig) -> Result<Self> {
        let opts = SessionOptions {
            disable_dht: !config.dht_enabled,
            listen_port_range: config
                .listen_port
                .map(|p| p..p.saturating_add(1)),
            ..Default::default()
        };

        let session = Session::new_with_opts(output_dir, opts)
            .await
            .map_err(|e| Error::BtError(e.to_string()))?;

        Ok(Self {
            session,
            handles: RwLock::new(HashMap::new()),
        })
    }

    /// 添加 BT 下载任务
    ///
    /// 返回文件总大小（如果已知）和任务名称
    pub async fn add_torrent(
        &self,
        task_id: &str,
        uri: &str,
        output_folder: PathBuf,
        selected_files: Option<Vec<usize>>,
    ) -> Result<(Option<u64>, Option<String>)> {
        let opts = AddTorrentOptions {
            output_folder: Some(output_folder.to_string_lossy().to_string()),
            only_files: selected_files.map(|files| {
                files.into_iter().map(|i| i as u32).collect()
            }),
            ..Default::default()
        };

        let response = self
            .session
            .add_torrent(AddTorrent::from_url(uri), Some(opts))
            .await
            .map_err(|e| Error::BtError(e.to_string()))?;

        let handle = response
            .into_handle()
            .ok_or_else(|| Error::BtError("failed to get torrent handle".into()))?;

        // Wait for metadata (magnet links need to resolve first)
        handle
            .wait_until_initialized()
            .await
            .map_err(|e| Error::BtError(e.to_string()))?;

        let name = handle.name();
        let stats = handle.stats();
        let total_size = if stats.total_bytes > 0 {
            Some(stats.total_bytes)
        } else {
            None
        };

        self.handles
            .write()
            .await
            .insert(task_id.to_string(), handle);

        Ok((total_size, name))
    }

    /// 获取任务统计信息
    pub async fn get_stats(&self, task_id: &str) -> Option<BtTaskInfo> {
        let handles = self.handles.read().await;
        let handle = handles.get(task_id)?;
        let stats = handle.stats();

        let (download_speed, upload_speed, peers, seeders) =
            if let Some(live) = &stats.live {
                (
                    live.download_speed.as_bytes(),
                    live.upload_speed.as_bytes(),
                    live.snapshot.peer_stats.queued as u32
                        + live.snapshot.peer_stats.connecting as u32
                        + live.snapshot.peer_stats.live as u32,
                    0u32, // librqbit doesn't distinguish seeders; approximate from context
                )
            } else {
                (0, 0, 0, 0)
            };

        Some(BtTaskInfo {
            peers,
            seeders,
            upload_speed,
            uploaded: stats.uploaded_bytes,
            selected_files: handle.only_files().map(|v| v.iter().map(|&i| i as usize).collect()),
        })
    }

    /// 获取已下载字节数和总大小
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
        stats.live.as_ref().map(|l| l.download_speed.as_bytes())
    }

    /// 暂停 BT 任务
    pub async fn pause(&self, task_id: &str) -> Result<()> {
        let handles = self.handles.read().await;
        let handle = handles
            .get(task_id)
            .ok_or_else(|| Error::BtError(format!("BT task not found: {task_id}")))?;
        self.session
            .pause(handle)
            .map_err(|e| Error::BtError(e.to_string()))?;
        Ok(())
    }

    /// 恢复 BT 任务
    pub async fn resume(&self, task_id: &str) -> Result<()> {
        let handles = self.handles.read().await;
        let handle = handles
            .get(task_id)
            .ok_or_else(|| Error::BtError(format!("BT task not found: {task_id}")))?;
        // librqbit uses start() to unpause — this is on Session, not handle
        // Use the torrent's id to unpause via session
        self.session
            .unpause(handle)
            .map_err(|e| Error::BtError(e.to_string()))?;
        Ok(())
    }

    /// 取消并删除 BT 任务
    pub async fn cancel(&self, task_id: &str, delete_files: bool) -> Result<()> {
        let handle = {
            let mut handles = self.handles.write().await;
            handles.remove(task_id)
        };
        if let Some(h) = handle {
            self.session
                .delete(h.id().into(), delete_files)
                .await
                .map_err(|e| Error::BtError(e.to_string()))?;
        }
        Ok(())
    }

    /// 检查任务是否已暂停
    pub async fn is_paused(&self, task_id: &str) -> bool {
        let handles = self.handles.read().await;
        handles
            .get(task_id)
            .map(|h| h.is_paused())
            .unwrap_or(false)
    }

    /// 移除句柄（不删除文件，用于任务完成后清理）
    pub async fn remove_handle(&self, task_id: &str) {
        self.handles.write().await.remove(task_id);
    }
}

/// 根据 URL 自动判断下载协议
pub fn detect_source(url: &str) -> DownloadSource {
    if url.starts_with("magnet:") || url.ends_with(".torrent") {
        DownloadSource::BitTorrent {
            uri: url.to_string(),
        }
    } else {
        DownloadSource::Http {
            url: url.to_string(),
        }
    }
}

/// 启动 BT 进度轮询任务
///
/// 每秒读取 BT 任务统计信息，转换为 YuShi 事件发送到事件通道
pub fn spawn_bt_progress_poller(
    bt_engine: Arc<BtEngine>,
    task_id: String,
    event_tx: mpsc::Sender<DownloaderEvent>,
    tasks: Arc<RwLock<HashMap<String, crate::types::Task>>>,
    seed_ratio: Option<f64>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        let mut was_finished = false;

        loop {
            interval.tick().await;

            let Some((downloaded, total, finished)) =
                bt_engine.get_progress(&task_id).await
            else {
                break; // handle removed
            };

            let speed = bt_engine.get_speed(&task_id).await.unwrap_or(0);
            let eta = if speed > 0 && total > downloaded {
                Some((total - downloaded) / speed)
            } else {
                None
            };

            // Update task in shared map
            {
                let mut tasks = tasks.write().await;
                if let Some(task) = tasks.get_mut(&task_id) {
                    task.downloaded = downloaded;
                    task.total_size = total;
                    task.speed = speed;
                    task.eta = eta;

                    if let Some(bt_info) = bt_engine.get_stats(&task_id).await {
                        task.bt_info = Some(bt_info);
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

            // Emit BT-specific status
            if let Some(bt_info) = bt_engine.get_stats(&task_id).await {
                let _ = event_tx
                    .send(DownloaderEvent::Progress(ProgressEvent::BtStatus {
                        task_id: task_id.clone(),
                        peers: bt_info.peers,
                        seeders: bt_info.seeders,
                        upload_speed: bt_info.upload_speed,
                        uploaded: bt_info.uploaded,
                    }))
                    .await;
            }

            // Handle completion and seeding
            if finished && !was_finished {
                was_finished = true;

                // Check seed ratio
                if let Some(ratio) = seed_ratio {
                    if total > 0 {
                        let stats = bt_engine.get_stats(&task_id).await;
                        let uploaded = stats.map(|s| s.uploaded).unwrap_or(0);
                        let current_ratio = uploaded as f64 / total as f64;
                        if current_ratio >= ratio {
                            break; // done seeding
                        }
                        // else: continue polling until ratio reached
                    } else {
                        break;
                    }
                } else {
                    break; // no seeding configured, done
                }
            }

            // If already finished and seeding, check ratio each tick
            if was_finished {
                if let Some(ratio) = seed_ratio {
                    let stats = bt_engine.get_stats(&task_id).await;
                    let uploaded = stats.map(|s| s.uploaded).unwrap_or(0);
                    if total > 0 {
                        let current_ratio = uploaded as f64 / total as f64;
                        if current_ratio >= ratio {
                            break;
                        }
                    }
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_magnet_link() {
        let source = detect_source("magnet:?xt=urn:btih:abc123&dn=test");
        assert!(matches!(source, DownloadSource::BitTorrent { .. }));
    }

    #[test]
    fn detect_torrent_file_path() {
        let source = detect_source("/tmp/test.torrent");
        assert!(matches!(source, DownloadSource::BitTorrent { .. }));
    }

    #[test]
    fn detect_torrent_url() {
        let source = detect_source("https://example.com/file.torrent");
        assert!(matches!(source, DownloadSource::BitTorrent { .. }));
    }

    #[test]
    fn detect_http_url() {
        let source = detect_source("https://example.com/file.zip");
        assert!(matches!(source, DownloadSource::Http { .. }));
    }

    #[test]
    fn detect_http_url_no_extension() {
        let source = detect_source("https://example.com/download");
        assert!(matches!(source, DownloadSource::Http { .. }));
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p yushi-core`
Expected: Compiles (some warnings about unused methods OK at this stage).

Note: The exact librqbit API may differ slightly from what's documented. If compilation fails due to API mismatches (field names, method signatures), consult `cargo doc -p librqbit --open` and adjust accordingly. The key types to verify are:
- `SessionOptions::disable_dht` field name
- `AddTorrentOptions::output_folder` type (may be `String` or `PathBuf`)
- `AddTorrentOptions::only_files` type (may be `Vec<u32>` or different)
- `Session::pause()` and `Session::unpause()` method signatures
- `ManagedTorrent::stats()` return type fields
- `LiveStats::download_speed.as_bytes()` method existence
- `StatsSnapshot::peer_stats` field structure

Fix any compilation errors based on actual API.

- [ ] **Step 3: Run tests**

Run: `cargo test -p yushi-core -- bt::tests`
Expected: All 5 detect_source tests PASS.

- [ ] **Step 4: Commit**

```bash
git add yushi-core/src/bt.rs
git commit -m "$(cat <<'EOF'
feat(core): implement BtEngine with session management, download, and progress polling
EOF
)"
```

---

## Task 8: Integrate BT into YuShi downloader

**Files:**
- Modify: `yushi-core/src/downloader.rs`

- [ ] **Step 1: Add BT fields to YuShi struct**

Add imports at the top of `downloader.rs`:

```rust
use crate::bt::{BtEngine, detect_source, spawn_bt_progress_poller};
use crate::config::BtConfig;
```

Add new fields to the `YuShi` struct (after `on_complete`):

```rust
    bt_engine: Arc<RwLock<Option<Arc<BtEngine>>>>,
    bt_config: Arc<std::sync::RwLock<BtConfig>>,
```

- [ ] **Step 2: Initialize BT fields in with_config**

Update the `with_config` method. Change the function signature to accept `BtConfig`:

```rust
    pub fn with_config(
        config: Config,
        max_concurrent_tasks: usize,
        queue_state_path: PathBuf,
        bt_config: BtConfig,
    ) -> (Self, mpsc::Receiver<DownloaderEvent>) {
```

Add the new fields to the `Self` constructor inside `with_config`:

```rust
            bt_engine: Arc::new(RwLock::new(None)),
            bt_config: Arc::new(std::sync::RwLock::new(bt_config)),
```

Update the `new` method to pass `BtConfig::default()`:

```rust
    pub fn new(
        max_concurrent_downloads: usize,
        max_concurrent_tasks: usize,
        queue_state_path: PathBuf,
    ) -> (Self, mpsc::Receiver<DownloaderEvent>) {
        let config = Config {
            max_concurrent: max_concurrent_downloads,
            ..Default::default()
        };
        Self::with_config(config, max_concurrent_tasks, queue_state_path, BtConfig::default())
    }
```

- [ ] **Step 3: Add ensure_bt_engine method**

Add to `impl YuShi`:

```rust
    /// 确保 BT 引擎已初始化（延迟初始化）
    async fn ensure_bt_engine(&self) -> Result<Arc<BtEngine>> {
        {
            let engine = self.bt_engine.read().await;
            if let Some(ref e) = *engine {
                return Ok(Arc::clone(e));
            }
        }

        let bt_config = self
            .bt_config
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone();

        // Use a temp dir for session-level default; actual output is per-torrent
        let output_dir = dirs::download_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        let engine = Arc::new(BtEngine::new(output_dir, &bt_config).await?);

        let mut guard = self.bt_engine.write().await;
        *guard = Some(Arc::clone(&engine));
        Ok(engine)
    }
```

- [ ] **Step 4: Update add_task_with_options for protocol detection**

Modify `add_task_with_options` to detect source and store it in the task. Replace the `Task` construction block:

```rust
        let source = detect_source(&url);

        let task = Task {
            id: task_id.clone(),
            url: url.clone(),
            dest,
            status: TaskStatus::Pending,
            total_size: 0,
            downloaded: 0,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            error: None,
            priority,
            speed: 0,
            eta: None,
            headers: HashMap::new(),
            checksum,
            speed_limit,
            source,
            bt_info: None,
        };
```

- [ ] **Step 5: Add start_bt_download method**

Add to `impl YuShi`:

```rust
    /// 启动 BT 下载任务
    async fn start_bt_download(
        &self,
        task_id: &str,
        uri: &str,
        dest: &Path,
        selected_files: Option<Vec<usize>>,
    ) -> Result<()> {
        let bt_engine = self.ensure_bt_engine().await?;

        let output_folder = dest.parent().unwrap_or(dest).to_path_buf();
        let (total_size, _name) = bt_engine
            .add_torrent(task_id, uri, output_folder, selected_files)
            .await?;

        // Update task with resolved total size
        if let Some(total) = total_size {
            let mut tasks = self.tasks.write().await;
            if let Some(task) = tasks.get_mut(task_id) {
                task.total_size = total;
            }
        }

        // Emit initialized event
        let _ = self
            .queue_event_tx
            .send(DownloaderEvent::Progress(ProgressEvent::Initialized {
                task_id: task_id.to_string(),
                total_size,
            }))
            .await;

        Ok(())
    }
```

- [ ] **Step 6: Update start_queue_task to dispatch BT tasks**

In `start_queue_task`, the main download is spawned in a `tokio::spawn` block. We need to branch on `task.source` inside that spawn. Find the line:

```rust
            let result = downloader
                .download_internal(&task.url, task.dest.to_str().unwrap(), tx, task.speed_limit)
                .await;
```

Replace with:

```rust
            let result = match &task.source {
                DownloadSource::BitTorrent { uri } => {
                    let uri = uri.clone();
                    let selected_files = task.bt_info.as_ref().and_then(|b| b.selected_files.clone());

                    async {
                        let engine = downloader.ensure_bt_engine().await?;

                        downloader
                            .start_bt_download(
                                &task_id_owned,
                                &uri,
                                &task.dest,
                                selected_files,
                            )
                            .await?;

                        let seed_ratio = downloader
                            .bt_config
                            .read()
                            .unwrap_or_else(|e| e.into_inner())
                            .seed_ratio;

                        // Spawn progress poller — blocks until download complete (+ seeding)
                        let poller = spawn_bt_progress_poller(
                            engine,
                            task_id_owned.clone(),
                            queue_event_tx.clone(),
                            Arc::clone(&tasks),
                            seed_ratio,
                        );
                        let _ = poller.await;
                        Ok(())
                    }
                    .await
                }
                DownloadSource::Http { .. } => {
                    downloader
                        .download_internal(
                            &task.url,
                            task.dest.to_str().unwrap(),
                            tx,
                            task.speed_limit,
                        )
                        .await
                }
            };
```

Note: For HTTP tasks, the existing progress listener (`rx.recv()` loop) continues to work. For BT tasks, the progress poller in `bt.rs` handles progress events directly, so the internal `rx` channel is not used — the `tx` sender simply drops at the end of the BT branch.

- [ ] **Step 7: Update pause_task for BT**

In `pause_task`, after aborting the JoinHandle, add BT-specific pause logic:

```rust
    pub async fn pause_task(&self, task_id: &str) -> Result<()> {
        let is_bt = {
            let tasks = self.tasks.read().await;
            let task = tasks.get(task_id).ok_or(Error::TaskNotFound)?;
            matches!(task.source, DownloadSource::BitTorrent { .. })
        };

        let mut tasks = self.tasks.write().await;
        let task = tasks.get_mut(task_id).ok_or(Error::TaskNotFound)?;

        if task.status == TaskStatus::Downloading {
            if is_bt {
                // Pause via BT engine
                if let Some(engine) = self.bt_engine.read().await.as_ref() {
                    engine.pause(task_id).await?;
                }
            } else {
                // Cancel HTTP download handle
                let mut active = self.active_downloads.write().await;
                if let Some(handle) = active.remove(task_id) {
                    handle.abort();
                }
            }

            task.status = TaskStatus::Paused;
            drop(tasks);

            self.save_queue_state().await?;
            let _ = self
                .queue_event_tx
                .send(DownloaderEvent::Task(TaskEvent::Paused {
                    task_id: task_id.to_string(),
                }))
                .await;

            if !is_bt {
                self.process_queue().await?;
            }
        }

        Ok(())
    }
```

- [ ] **Step 8: Update resume_task for BT**

```rust
    pub async fn resume_task(&self, task_id: &str) -> Result<()> {
        let is_bt = {
            let tasks = self.tasks.read().await;
            let task = tasks.get(task_id).ok_or(Error::TaskNotFound)?;
            matches!(task.source, DownloadSource::BitTorrent { .. })
        };

        {
            let mut tasks = self.tasks.write().await;
            let task = tasks.get_mut(task_id).ok_or(Error::TaskNotFound)?;

            if task.status == TaskStatus::Paused {
                if is_bt {
                    // Resume via BT engine
                    if let Some(engine) = self.bt_engine.read().await.as_ref() {
                        engine.resume(task_id).await?;
                    }
                    task.status = TaskStatus::Downloading;
                } else {
                    task.status = TaskStatus::Pending;
                }
                drop(tasks);

                self.save_queue_state().await?;
                let _ = self
                    .queue_event_tx
                    .send(DownloaderEvent::Task(TaskEvent::Resumed {
                        task_id: task_id.to_string(),
                    }))
                    .await;
            }
        }

        if !is_bt {
            self.process_queue().await?;
        }
        Ok(())
    }
```

- [ ] **Step 9: Update cancel_task for BT**

```rust
    pub async fn cancel_task(&self, task_id: &str) -> Result<()> {
        let is_bt = {
            let tasks = self.tasks.read().await;
            tasks
                .get(task_id)
                .map(|t| matches!(t.source, DownloadSource::BitTorrent { .. }))
                .unwrap_or(false)
        };

        if is_bt {
            // Cancel via BT engine (deletes files)
            if let Some(engine) = self.bt_engine.read().await.as_ref() {
                let _ = engine.cancel(task_id, true).await;
            }
        } else {
            // Cancel HTTP download handle
            let mut active = self.active_downloads.write().await;
            if let Some(handle) = active.remove(task_id) {
                handle.abort();
            }
            drop(active);
        }

        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.status = TaskStatus::Cancelled;

            if !is_bt {
                let _ = fs::remove_file(&task.dest).await;
                let state_path = task.dest.with_extension("json");
                let _ = fs::remove_file(state_path).await;
            }
        }
        drop(tasks);

        self.save_queue_state().await?;
        let _ = self
            .queue_event_tx
            .send(DownloaderEvent::Task(TaskEvent::Cancelled {
                task_id: task_id.to_string(),
            }))
            .await;

        self.process_queue().await?;
        Ok(())
    }
```

- [ ] **Step 10: Verify compilation**

Run: `cargo check -p yushi-core`
Expected: Compiles. Fix any API mismatches with librqbit.

- [ ] **Step 11: Run all existing tests**

Run: `cargo test -p yushi-core`
Expected: All existing tests PASS. No regressions.

- [ ] **Step 12: Commit**

```bash
git add yushi-core/src/downloader.rs
git commit -m "$(cat <<'EOF'
feat(core): integrate BtEngine into YuShi with lazy session init and task dispatch
EOF
)"
```

---

## Task 9: Update Tauri layer and callers

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/state.rs`

- [ ] **Step 1: Update AddTaskOptions in commands.rs**

Add `selected_files` field to `AddTaskOptions` in `src-tauri/src/commands.rs`:

```rust
#[derive(Debug, Deserialize)]
pub struct AddTaskOptions {
    pub url: String,
    pub dest: PathBuf,
    pub checksum: Option<ChecksumType>,
    pub priority: Option<TaskPriority>,
    pub speed_limit: Option<u64>,
    #[serde(default)]
    pub auto_rename_on_conflict: bool,
    /// BT 任务：选择下载的文件索引列表
    pub selected_files: Option<Vec<usize>>,
}
```

- [ ] **Step 2: Update AppState bootstrap to pass BtConfig**

Read `src-tauri/src/state.rs` and find where `YuShi::with_config()` is called. Update to pass `config.bt.clone()` as the new fourth argument. For example:

```rust
let (queue, event_rx) = YuShi::with_config(
    config.downloader_config(),
    config.max_concurrent_tasks,
    queue_state_path,
    config.bt.clone(),
);
```

- [ ] **Step 3: Update CLI callers**

Check `yushi-cli/src/main.rs` or wherever `YuShi::new()` or `YuShi::with_config()` is called. The `new()` method signature is unchanged (passes `BtConfig::default()` internally), so CLI should compile without changes. Verify:

Run: `cargo check -p yushi-cli`
Expected: Compiles.

- [ ] **Step 4: Verify full workspace compiles**

Run: `cargo check --workspace`
Expected: Compiles.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/state.rs
git commit -m "$(cat <<'EOF'
feat(tauri): pass BtConfig to YuShi and add selected_files to AddTaskOptions
EOF
)"
```

---

## Task 10: Update frontend types

**Files:**
- Modify: `src-ui/src/lib/types.ts`

- [ ] **Step 1: Add DownloadSource and BtTaskInfo types**

Add to `src-ui/src/lib/types.ts`:

```typescript
export type DownloadSource =
  | { type: "Http"; url: string }
  | { type: "BitTorrent"; uri: string };

export interface BtTaskInfo {
  peers: number;
  seeders: number;
  upload_speed: number;
  uploaded: number;
  selected_files: number[] | null;
}

export interface BtConfig {
  dht_enabled: boolean;
  upload_limit: number | null;
  seed_ratio: number | null;
  listen_port: number | null;
}
```

- [ ] **Step 2: Extend DownloadTask interface**

Add two fields to the `DownloadTask` interface:

```typescript
export interface DownloadTask {
  // ... existing fields ...
  source: DownloadSource;
  bt_info: BtTaskInfo | null;
}
```

- [ ] **Step 3: Extend AppConfig interface**

Add to `AppConfig`:

```typescript
export interface AppConfig {
  // ... existing fields ...
  bt: BtConfig;
}
```

- [ ] **Step 4: Add BtStatus to ProgressEvent**

Add to the `ProgressEvent` union type:

```typescript
  | {
      BtStatus: {
        task_id: string;
        peers: number;
        seeders: number;
        upload_speed: number;
        uploaded: number;
      };
    }
```

- [ ] **Step 5: Extend AddTaskOptions**

Add to `AddTaskOptions`:

```typescript
export interface AddTaskOptions {
  // ... existing fields ...
  selected_files?: number[];
}
```

- [ ] **Step 6: Verify frontend builds**

Run: `cd src-ui && bun run build`
Expected: Build succeeds (TypeScript type errors in components that reference new fields are OK — the types file itself should be valid).

- [ ] **Step 7: Commit**

```bash
git add src-ui/src/lib/types.ts
git commit -m "$(cat <<'EOF'
feat(ui): add BitTorrent types (DownloadSource, BtTaskInfo, BtConfig)
EOF
)"
```

---

## Task 11: Compilation verification and API adjustment

**Files:**
- Potentially all modified files

This task is a catch-all for fixing compilation errors due to librqbit API differences from documentation.

- [ ] **Step 1: Full workspace build**

Run: `cargo build --workspace`

If there are compilation errors in `bt.rs` related to librqbit API:

1. Run `cargo doc -p librqbit --no-deps --open` to browse actual API
2. Fix type mismatches, field names, method signatures in `bt.rs`
3. Common issues to check:
   - `SessionOptions` field names (`disable_dht` vs `dht` vs something else)
   - `AddTorrentOptions::output_folder` may be `Option<String>` or `Option<PathBuf>`
   - `AddTorrentOptions::only_files` element type
   - `Session::pause()`/`unpause()` method signatures (may take `&ManagedTorrent` or torrent id)
   - `Speed::as_bytes()` may not exist — check if it's `mbps * 1024.0 * 1024.0 / 8.0`
   - `StatsSnapshot::peer_stats` field structure

- [ ] **Step 2: Run full test suite**

Run: `cargo test --workspace`
Expected: All tests PASS.

- [ ] **Step 3: Run clippy**

Run: `cargo clippy --workspace --all-targets`
Expected: No errors (warnings OK).

- [ ] **Step 4: Run fmt check**

Run: `cargo fmt --check`
Expected: No formatting issues. If any, run `cargo fmt` and commit.

- [ ] **Step 5: Commit any fixes**

```bash
git add -A
git commit -m "$(cat <<'EOF'
fix(core): adjust librqbit API usage to match actual v8 interface
EOF
)"
```

---

## Task 12: Integration test — add BT task and verify state

**Files:**
- Modify: `yushi-core/src/bt.rs` (add integration test)

- [ ] **Step 1: Write integration test**

Add to the `#[cfg(test)] mod tests` in `yushi-core/src/bt.rs`:

```rust
    #[test]
    fn detect_source_edge_cases() {
        // Case-insensitive magnet
        let source = detect_source("MAGNET:?xt=urn:btih:abc");
        // magnet: is case-sensitive per spec, so this should be Http
        assert!(matches!(source, DownloadSource::Http { .. }));

        // .torrent with query params
        let source = detect_source("https://example.com/file.torrent?token=abc");
        // ends_with(".torrent") won't match due to query string
        assert!(matches!(source, DownloadSource::Http { .. }));
    }
```

Wait — the `.torrent?token=abc` case reveals a bug in `detect_source`. Let's fix it.

- [ ] **Step 2: Update detect_source to handle query params**

```rust
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
```

- [ ] **Step 3: Update tests for new behavior**

Update the `detect_torrent_url` test expectation and add the new edge cases:

```rust
    #[test]
    fn detect_torrent_url_with_query() {
        let source = detect_source("https://example.com/file.torrent?token=abc");
        assert!(matches!(source, DownloadSource::BitTorrent { .. }));
    }

    #[test]
    fn detect_torrent_url_with_fragment() {
        let source = detect_source("https://example.com/file.torrent#section");
        assert!(matches!(source, DownloadSource::BitTorrent { .. }));
    }
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p yushi-core -- bt::tests`
Expected: All tests PASS.

- [ ] **Step 5: Commit**

```bash
git add yushi-core/src/bt.rs
git commit -m "$(cat <<'EOF'
fix(core): handle query params and fragments in torrent URL detection
EOF
)"
```

---

## Task 13: Final verification

- [ ] **Step 1: Full workspace build**

Run: `cargo build --workspace`
Expected: Clean build.

- [ ] **Step 2: Full test suite**

Run: `cargo test --workspace --all-features`
Expected: All tests PASS.

- [ ] **Step 3: Clippy**

Run: `cargo clippy --workspace --all-targets --all-features`
Expected: No errors.

- [ ] **Step 4: Format check**

Run: `cargo fmt --check`
Expected: Clean.

- [ ] **Step 5: Commit any final fixes and verify git log**

Run: `git log --oneline -15`
Expected: Clean commit history following conventional commits.
