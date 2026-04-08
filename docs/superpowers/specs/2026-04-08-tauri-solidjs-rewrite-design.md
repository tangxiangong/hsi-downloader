# YuShi GUI Rewrite: Tauri v2 + SolidJS

## Overview

Rewrite the desktop GUI from gpui to Tauri v2 + SolidJS + TailwindCSS + DaisyUI. The `yushi-core` and `yushi-cli` crates remain unchanged. The gpui code in `src/` is fully replaced.

## Decisions

| Decision | Choice |
|----------|--------|
| Frontend-backend interaction | Tauri Commands + Event System |
| View structure | 3 views: Tasks (with filter tabs), History, Settings |
| UI style | Clean modern, DaisyUI `light`/`dark` + System toggle |
| Tauri version | v2 |
| Frontend framework | SolidJS (TSX) |
| CSS framework | TailwindCSS + DaisyUI |
| Build tool | Vite |
| Package manager | Bun |
| Migration strategy | Full replacement (delete gpui code) |
| System tray | Community tray plugin with panel window |
| Auto-update | `tauri-plugin-updater`, source: GitHub Releases |

---

## 1. Project Structure

```
YuShi/
├── Cargo.toml              # workspace only: members = ["yushi-core", "yushi-cli", "src-tauri"]
├── yushi-core/             # unchanged
├── yushi-cli/              # unchanged
├── src-tauri/
│   ├── Cargo.toml          # package "yushi", depends on yushi-core, tauri, plugins
│   ├── build.rs            # tauri-build
│   ├── tauri.conf.json
│   ├── capabilities/       # Tauri v2 permission declarations
│   ├── icons/
│   └── src/
│       ├── main.rs         # Tauri entry point, setup hook initializes state & event loop
│       ├── state.rs        # AppState: Arc<YuShi>, RwLock<AppConfig>, RwLock<DownloadHistory>
│       ├── commands.rs     # #[tauri::command] functions
│       ├── events.rs       # DownloaderEvent → frontend event forwarding
│       └── tray.rs         # Tray icon + panel window setup
├── src-ui/
│   ├── package.json
│   ├── bun.lock
│   ├── vite.config.ts
│   ├── tsconfig.json
│   ├── tailwind.config.ts
│   ├── index.html
│   └── src/
│       ├── index.tsx        # SolidJS entry
│       ├── App.tsx          # Root component with router/layout
│       ├── stores/          # SolidJS stores (createStore + Context)
│       │   ├── task-store.ts
│       │   ├── history-store.ts
│       │   ├── config-store.ts
│       │   └── theme-store.ts
│       ├── pages/
│       │   ├── TasksPage.tsx
│       │   ├── HistoryPage.tsx
│       │   └── SettingsPage.tsx
│       ├── components/
│       │   ├── Layout.tsx
│       │   ├── Sidebar.tsx
│       │   ├── TaskCard.tsx
│       │   ├── HistoryCard.tsx
│       │   ├── AddTaskDialog.tsx
│       │   ├── FilterTabs.tsx
│       │   ├── ThemeToggle.tsx
│       │   └── UpdateNotice.tsx
│       ├── lib/
│       │   ├── commands.ts   # Typed wrappers around tauri invoke()
│       │   ├── events.ts     # Typed wrappers around tauri listen()
│       │   └── types.ts      # TypeScript types mirroring Rust types
│       └── styles/
│           └── app.css       # Tailwind directives + custom styles
└── .gitignore
```

### Files to Delete

- `src/main.rs`, `src/state.rs`, `src/utils.rs`
- `src/components/` (entire directory)
- `src/views/` (entire directory)

### Root Cargo.toml Changes

- Remove the `[package]` section (no longer a binary crate at root)
- Remove all gpui-related dependencies (`gpui`, `gpui-component`, `gpui_platform`, `tray-icon`, `gtk`)
- Update `members` to `["yushi-core", "yushi-cli", "src-tauri"]`
- Keep shared workspace dependencies used by `yushi-core` and `yushi-cli`

---

## 2. Tauri Backend (Rust)

### State

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use yushi_core::{YuShi, AppConfig, DownloadHistory};

pub struct AppState {
    pub queue: Arc<YuShi>,
    pub config: RwLock<AppConfig>,
    pub history: RwLock<DownloadHistory>,
}
```

Injected via `app.manage(app_state)`. Accessed in commands via `State<'_, AppState>`.

### Commands

| Command | Core API Call | Description |
|---------|--------------|-------------|
| `get_tasks` | `queue.get_all_tasks()` | Get all tasks |
| `add_task` | `queue.add_task_with_options(...)` | Add download task |
| `pause_task` | `queue.pause_task(id)` | Pause task |
| `resume_task` | `queue.resume_task(id)` | Resume task |
| `cancel_task` | `queue.cancel_task(id)` | Cancel task |
| `remove_task` | `queue.remove_task(id)` | Remove task |
| `get_history` | `history.read()` | Get download history |
| `remove_history` | `history.remove_from_file(id)` | Remove history entry |
| `get_config` | `config.read()` | Get config |
| `update_config` | `config.write() + apply_runtime_config()` | Update and hot-reload config |
| `pick_directory` | Tauri dialog plugin | Open native directory picker |

### Event Forwarding

In `setup`, spawn a tokio task that consumes the `DownloaderEvent` receiver channel and emits events to the frontend:

```rust
// Pseudocode
let event_rx = queue_event_receiver;
tauri::async_runtime::spawn(async move {
    while let Ok(event) = event_rx.recv().await {
        app_handle.emit("download-event", &event).unwrap();
    }
});
```

Events are serialized as JSON using serde. Frontend subscribes via `listen("download-event", callback)`.

Event categories (matching existing `DownloaderEvent` enum):
- `TaskEvent`: Added, Started, Completed, Failed, Paused, Resumed, Cancelled
- `ProgressEvent`: Initialized, Updated, ChunkProgress, StreamProgress, Finished, Failed
- `VerificationEvent`: Started, Completed

### Tray Icon

Use the community tray plugin with panel window support:
- Tray icon always visible when app is running
- Clicking the tray icon opens a small panel window showing:
  - Active download count and overall progress
  - Quick actions (pause all, resume all)
- Closing the main window hides it to tray (app keeps running)
- Tray context menu: "Show Window", "Quit"

### Auto-Update

Use `tauri-plugin-updater` with GitHub Releases as the update source:
- Check for updates on app startup
- Manual "Check for Updates" button in Settings page
- When a new version is available, show a notification with release notes
- User confirms before downloading and installing
- Update endpoint: GitHub Releases `latest.json` (generated by Tauri's build/release workflow)

---

## 3. Frontend (SolidJS)

### State Management

SolidJS `createStore` + Context pattern. No external state library.

#### task-store.ts
- `tasks: DownloadTask[]` — full task list
- `filter: "all" | "downloading" | "completed"` — active filter
- `filteredTasks` — derived signal based on filter
- Initialized via `invoke("get_tasks")`
- Updated reactively via `listen("download-event")`
- Per-task progress uses fine-grained signals to avoid full-list re-renders

#### history-store.ts
- `history: CompletedTask[]`
- Loaded via `invoke("get_history")`
- Updated when `TaskEvent::Completed` is received

#### config-store.ts
- `config: AppConfig`
- Loaded via `invoke("get_config")`
- Saved via `invoke("update_config")`

#### theme-store.ts
- `theme: "light" | "dark" | "system"`
- Applies `data-theme` attribute to `<html>`
- System mode uses `matchMedia("(prefers-color-scheme: dark)")` listener
- Persisted as part of AppConfig

### Pages

#### TasksPage.tsx
- Top: `FilterTabs` (All / Downloading / Completed)
- Action bar: "Add Task" button, search/filter controls
- Content: scrollable list of `TaskCard` components
- Empty state when no tasks match filter

#### HistoryPage.tsx
- Scrollable list of `HistoryCard` components
- Each card shows: filename, URL, size, completion time, average speed
- Actions: open file, open containing folder, remove from history

#### SettingsPage.tsx
- Form sections:
  - Download: default path (with directory picker), max concurrent downloads, max concurrent tasks, chunk size
  - Network: proxy, user agent, speed limit, timeout
  - Appearance: theme toggle (Light / Dark / System)
  - About: version info, "Check for Updates" button

### Components

#### Layout.tsx
- Flexbox layout: fixed-width sidebar + flexible content area
- Sidebar: app logo/title + navigation menu + version info

#### TaskCard.tsx
- Card showing: filename, URL (truncated), status badge, progress bar, speed, ETA
- Actions: pause/resume, cancel, remove (with confirmation)
- Status-dependent rendering (different colors/icons per status)

#### AddTaskDialog.tsx
- DaisyUI modal
- Fields: URL, destination directory (with picker), optional headers, optional checksum, priority select, speed limit

#### FilterTabs.tsx
- DaisyUI `tabs` component
- Three tabs: All, Downloading, Completed
- Shows count badge per tab

#### ThemeToggle.tsx
- Dropdown or segmented control: Light / Dark / System

#### UpdateNotice.tsx
- Toast/alert notification when update is available
- Shows version number and release notes summary
- Confirm/dismiss actions

### DaisyUI Component Mapping

| Feature | DaisyUI Component |
|---------|------------------|
| Sidebar nav | `menu` |
| Task card | `card` |
| Progress bar | `progress` |
| Filter tabs | `tabs` + `tab` |
| Add task dialog | `modal` |
| Settings form | `form-control` + `input` + `select` + `toggle` |
| Theme switch | `dropdown` or `swap` |
| Action buttons | `btn` with variants |
| Status badge | `badge` |
| Update notice | `alert` or `toast` |
| Confirmation | `modal` (compact) |

---

## 4. Build & Dev

### tauri.conf.json Key Settings

- `build.devUrl`: `http://localhost:5173`
- `build.frontendDist`: `../src-ui/dist`
- `build.beforeDevCommand`: `cd src-ui && bun run dev`
- `build.beforeBuildCommand`: `cd src-ui && bun run build`
- `app.windows[0].width`: 1200
- `app.windows[0].height`: 800
- `app.windows[0].title`: "驭时 (YuShi)"
- `bundle.identifier`: `com.yushi.app`

### Tauri Plugins

- `tauri-plugin-dialog` — native directory picker
- `tauri-plugin-updater` — auto-update from GitHub Releases
- Community tray plugin — tray icon with panel window

### Commands

```bash
# Dev mode (starts frontend dev server + Rust with hot reload)
cargo tauri dev

# Production build
cargo tauri build

# Frontend only
cd src-ui && bun run dev

# Workspace checks (unchanged)
cargo fmt --check
cargo clippy --workspace --all-targets --all-features
cargo test --workspace --all-features
```

### Dependency Cleanup

Remove from root workspace:
- `gpui`, `gpui-component`, `gpui_platform`
- `tray-icon`, `gtk`

Add to `src-tauri/Cargo.toml`:
- `tauri`, `tauri-build`
- `tauri-plugin-dialog`, `tauri-plugin-updater`
- Community tray plugin
- `serde`, `serde_json`, `tokio`
- `yushi-core` (path dependency)

### CLAUDE.md

Update to reflect new architecture:
- Root package description → workspace-only
- Build commands → `cargo tauri dev` / `cargo tauri build`
- Architecture notes → Tauri Commands + Event System
- Frontend stack → SolidJS + TailwindCSS + DaisyUI
