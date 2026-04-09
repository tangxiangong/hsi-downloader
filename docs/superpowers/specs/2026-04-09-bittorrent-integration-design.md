# BitTorrent Integration Design

## Overview

Integrate BitTorrent download support into YuShi via [librqbit](https://github.com/ikatson/rqbit), enabling magnet link and `.torrent` file downloads alongside existing HTTP downloads.

**Scope (standard feature set):**
- Magnet link + `.torrent` file parsing and download
- Pause / resume / cancel
- Progress display (peers, seeders, speed, ETA)
- Seeding with configurable ratio
- Upload speed limiting
- DHT toggle
- Selective file download within a torrent

**Out of scope (first version):**
- UPnP port mapping
- IPv6
- Private tracker support
- Peer blacklist
- RPC interface

## Architecture: Composition Delegation

HTTP code paths remain untouched. BitTorrent logic lives in a new `bt.rs` module. The `YuShi` struct holds a `librqbit::Session` alongside the existing `reqwest::Client`. Task dispatch branches on `DownloadSource` at the `start_queue_task()` level.

```
YuShi {
    client: reqwest::Client,        // HTTP — unchanged
    bt_session: librqbit::Session,  // BT — new, lazily initialized
    tasks: HashMap<String, Task>,   // unified task map
}
```

### Why not a Protocol Trait?

librqbit's `Session` is already a complete, well-tested BT engine. Wrapping it in a trait adds abstraction without value. A trait can be introduced later if a third protocol (FTP) is added — YAGNI for now.

## Data Model Changes

### DownloadSource (new)

```rust
/// types.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DownloadSource {
    Http { url: String },
    BitTorrent { uri: String },  // magnet: URI or .torrent file path
}
```

### BtTaskInfo (new)

```rust
/// types.rs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BtTaskInfo {
    pub peers: u32,
    pub seeders: u32,
    pub upload_speed: u64,
    pub uploaded: u64,
    pub selected_files: Option<Vec<usize>>,
}
```

### Task (extended)

Two new fields added to `Task`:

```rust
pub struct Task {
    // --- all existing fields unchanged ---
    pub id: String,
    pub url: String,  // kept for backward compat; BT tasks store magnet URI here
    pub dest: PathBuf,
    pub status: TaskStatus,
    pub total_size: u64,
    pub downloaded: u64,
    // ...

    // --- new ---
    #[serde(default = "default_http_source")]
    pub source: DownloadSource,
    #[serde(default)]
    pub bt_info: Option<BtTaskInfo>,
}
```

`source` defaults to `DownloadSource::Http { url }` for backward compatibility with existing persisted queue state.

### BtConfig (new)

```rust
/// config.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BtConfig {
    /// Enable DHT for peer discovery (default: true)
    pub dht_enabled: bool,
    /// Upload speed limit in bytes/sec (None = unlimited)
    pub upload_limit: Option<u64>,
    /// Seed ratio target — stop seeding after reaching this ratio (e.g. 2.0)
    pub seed_ratio: Option<f64>,
    /// BT listen port (None = random)
    pub listen_port: Option<u16>,
}
```

Added to `AppConfig`:

```rust
pub struct AppConfig {
    // --- existing fields unchanged ---
    // ...

    #[serde(default)]
    pub bt: BtConfig,
}
```

### Error (extended)

```rust
/// error.rs
pub enum Error {
    // --- existing variants unchanged ---
    // ...

    #[error("BitTorrent error: {0}")]
    BtError(String),
}
```

## YuShi Struct Changes

### New Fields

```rust
pub struct YuShi {
    // --- existing fields unchanged ---
    client: Arc<std::sync::RwLock<Client>>,
    config: Arc<std::sync::RwLock<Config>>,
    tasks: Arc<RwLock<HashMap<String, Task>>>,
    active_downloads: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
    max_concurrent_tasks: Arc<AtomicUsize>,
    queue_state_path: PathBuf,
    queue_state_save_tx: mpsc::Sender<QueueStateSaveSignal>,
    queue_state_write_lock: Arc<Mutex<()>>,
    queue_event_tx: mpsc::Sender<DownloaderEvent>,
    on_complete: Option<CompletionCallback>,

    // --- new ---
    bt_session: Arc<RwLock<Option<librqbit::Session>>>,
    bt_config: Arc<std::sync::RwLock<BtConfig>>,
    bt_handles: Arc<RwLock<HashMap<String, ManagedTorrentHandle>>>,
}
```

### Session Lifecycle

- **Lazy initialization:** `bt_session` starts as `None`. Created on first BT task via `ensure_bt_session()`.
- **Session config:** `SessionOptions` configured from `BtConfig` (DHT toggle, listen port).
- **Output directory:** Each torrent's output is set per-task via `AddTorrentOptions`, not the Session-level default.
- **Shutdown:** Session dropped when `YuShi` is dropped (graceful).

### Task Dispatch

In `start_queue_task()`:

```rust
match &task.source {
    DownloadSource::Http { .. } => self.start_http_download(task).await,
    DownloadSource::BitTorrent { uri } => self.start_bt_download(task, uri).await,
}
```

HTTP path is completely unchanged.

### BT Download Flow (`start_bt_download`)

1. Call `ensure_bt_session()` to lazily create the Session if needed.
2. Build `AddTorrentOptions` with:
   - `output_folder`: task's `dest` parent directory
   - `selected_files`: from `bt_info.selected_files` if specified
   - Speed limits from task or global config
3. Call `session.add_torrent(AddTorrent::from_url(uri), Some(options))`.
4. Store the `ManagedTorrentHandle` in `bt_handles`.
5. Spawn a progress polling task (interval ~1s):
   - Read `TorrentStats` from the handle
   - Emit `ProgressEvent::Updated { downloaded, total, speed, eta }` — reuses existing variant
   - Emit `ProgressEvent::BtStatus { peers, seeders, upload_speed, uploaded }` — new variant
   - Update `Task` fields in `tasks` map
6. On download completion:
   - If `seed_ratio` is set, continue seeding until ratio reached, then stop
   - If no `seed_ratio`, stop immediately after download completes
   - Emit `TaskEvent::Completed`
   - Trigger `on_complete` callback (for history tracking)

### Pause / Resume / Cancel

- **Pause:** Call `handle.pause()` on the `ManagedTorrentHandle`. Update task status.
- **Resume:** Call `handle.start()`. Update task status.
- **Cancel:** Call `handle.delete()` or equivalent. Remove from `bt_handles`. Update task status.

## Event System

Existing events untouched. One new variant added:

```rust
pub enum ProgressEvent {
    // --- all existing variants unchanged ---

    /// BT-specific extended status
    BtStatus {
        task_id: String,
        peers: u32,
        seeders: u32,
        upload_speed: u64,
        uploaded: u64,
    },
}
```

BT download progress reuses `ProgressEvent::Updated` — the frontend progress bar works without changes. `BtStatus` is supplementary; old frontends ignore it via serde tagged enum compatibility.

## Tauri Command Layer

### AddTaskOptions (extended)

```rust
pub struct AddTaskOptions {
    pub url: String,
    pub dest: PathBuf,
    pub checksum: Option<ChecksumType>,
    pub priority: Option<TaskPriority>,
    pub speed_limit: Option<u64>,
    #[serde(default)]
    pub auto_rename_on_conflict: bool,

    // --- new ---
    pub selected_files: Option<Vec<usize>>,
}
```

### Protocol Auto-Detection

Inside `add_task` command handler, protocol is inferred from the URL:

- Starts with `magnet:` → `DownloadSource::BitTorrent`
- Ends with `.torrent` or is a local file path to a `.torrent` → `DownloadSource::BitTorrent`
- Otherwise → `DownloadSource::Http`

No explicit `source` field needed in the frontend API.

### Config Commands

`update_config` already handles `AppConfig` — adding `bt: BtConfig` field just works. Runtime config changes to BT (upload limit, seed ratio) applied via `bt_session` reconfiguration.

## Frontend Types

### TypeScript Extensions

```typescript
// types.ts

type DownloadSource =
  | { type: "Http"; url: string }
  | { type: "BitTorrent"; uri: string };

interface BtTaskInfo {
  peers: number;
  seeders: number;
  upload_speed: number;
  uploaded: number;
  selected_files: number[] | null;
}

interface DownloadTask {
  // --- all existing fields unchanged ---
  // ...

  // --- new ---
  source: DownloadSource;
  bt_info: BtTaskInfo | null;
}
```

Frontend can use `task.source.type` to conditionally render BT-specific info (peers/seeders badge, upload speed, etc.). Existing progress bar, speed, ETA display all work unchanged.

## File Structure

```
yushi-core/src/
├── bt.rs          ← NEW: BT Session mgmt, download logic, stats polling
├── downloader.rs  ← MINOR: add bt fields, start_queue_task dispatch branch
├── types.rs       ← MINOR: add DownloadSource, BtTaskInfo, BtStatus variant
├── config.rs      ← MINOR: add BtConfig, extend AppConfig
├── error.rs       ← MINOR: add BtError variant
├── lib.rs         ← MINOR: pub mod bt, export new types
└── (all other files unchanged)

src-tauri/src/
├── commands.rs    ← MINOR: add selected_files to AddTaskOptions
└── (all other files unchanged)

src-ui/src/lib/
├── types.ts       ← MINOR: add DownloadSource, BtTaskInfo types
└── (all other files unchanged)
```

## Dependencies

Add to `yushi-core/Cargo.toml`:

```toml
[dependencies]
librqbit = { version = "...", default-features = false, features = ["..."] }
```

Feature flags TBD based on librqbit's available features (likely: `dht`, `default`).

## Backward Compatibility

- **Queue state:** `source` field defaults to `DownloadSource::Http { url: task.url }` on deserialization. Existing queue.json files load without migration.
- **Events:** New `BtStatus` variant ignored by old frontends (tagged enum).
- **Config:** `bt` field defaults via `#[serde(default)]`. Existing config.json files load without migration.
- **API:** `AddTaskOptions.url` field unchanged. Protocol auto-detected.

## Testing Strategy

- Unit tests for protocol detection logic (magnet URI, .torrent path, HTTP URL)
- Unit tests for `BtConfig` validation and serialization
- Integration test: add a magnet link, verify task created with correct `DownloadSource`
- Integration test: BT session lazy initialization (no session until first BT task)
- Manual test: download a known Linux distro torrent end-to-end
