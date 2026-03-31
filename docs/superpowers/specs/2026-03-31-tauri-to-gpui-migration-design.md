# Migrate GUI from Tauri + React to gpui + gpui-component

**Date:** 2026-03-31
**Status:** Approved

## Goal

Replace the Tauri 2 (React + TypeScript) desktop frontend with a pure Rust GUI built on gpui and gpui-component. Simultaneously move `AppConfig` and `DownloadHistory` into `yushi-core` so that CLI, TUI, and GUI all share the same config/history logic.

## Scope

**In scope:**
- New `yushi-app/` crate — gpui desktop application
- Move `AppConfig` and `DownloadHistory` from `src-tauri/` into `yushi-core`
- Task management UI: list, add, pause, resume, cancel, remove
- Settings UI: download path, concurrency, chunk size, timeout, user agent, theme
- History UI: completed downloads list with search
- Theme support via gpui-component built-in themes (light/dark, follow system)
- Delete all Tauri and React files

**Out of scope:**
- Auto-update system (was `tauri-plugin-updater`)
- Custom theme design (use gpui-component built-in themes)

## Architecture

### Workspace Structure (after migration)

```
Cargo.toml              # workspace root
├── yushi-core/         # core library — download engine, config, history
├── yushi-app/          # NEW — gpui desktop GUI
└── yushi-cli/          # CLI + optional TUI (ratatui)
```

Both `yushi-app` and `yushi-cli` depend on `yushi-core`. There is no IPC layer — both call `yushi-core` directly as a Rust library.

### yushi-core Changes

Two modules move from `src-tauri/src/` into `yushi-core/src/`:

**config.rs** — `AppConfig` struct with JSON persistence. Changes from the Tauri version:
- Remove `WindowState` (Tauri-specific window state tracking)
- Keep all download-related fields: `default_download_path`, `max_concurrent_downloads`, `max_concurrent_tasks`, `chunk_size`, `timeout`, `user_agent`, `theme`
- Keep `load()`, `save()`, `validate()` methods — convert from `anyhow::Result` to `yushi-core`'s error type (core uses `thiserror`, not `anyhow`)
- Add `dirs` dependency to `yushi-core` for `dirs::download_dir()`

**history.rs** — `DownloadHistory` and `CompletedTask` structs with JSON persistence. No changes needed — this module has no Tauri dependencies.

**lib.rs** — Add `pub mod config; pub mod history;` and re-export `AppConfig`, `DownloadHistory`, `CompletedTask`.

### yushi-app Crate (NEW)

```
yushi-app/
├── Cargo.toml
└── src/
    ├── main.rs              # Application entry, window creation, event loop
    ├── app_state.rs         # AppState Entity — global state
    ├── views/
    │   ├── mod.rs
    │   ├── app_view.rs      # Root view: TitleBar + Sidebar + content routing
    │   ├── task_list.rs     # TaskListView: filtered task list
    │   ├── history.rs       # HistoryView: completed tasks with search
    │   └── settings.rs      # SettingsView: config form
    └── components/
        ├── mod.rs
        ├── task_item.rs     # TaskItem: task card with progress bar + controls
        └── add_task.rs      # AddTaskDialog: new download modal
```

**Dependencies:**
- `yushi-core` (workspace)
- `gpui` (git, from zed-industries/zed)
- `gpui-component` (git, from longbridge/gpui-component)
- `gpui-component-assets` (git, from longbridge/gpui-component — for bundled icons)
- `tokio` (workspace — for async runtime compatibility)
- `anyhow` (workspace)

### Global State: `AppState`

```rust
struct AppState {
    queue: Arc<YuShi>,              // yushi-core download engine
    config: AppConfig,              // from yushi-core
    history: DownloadHistory,       // from yushi-core
    tasks: Vec<DownloadTask>,       // cached task snapshot for rendering
    current_view: ViewKind,         // All | Downloading | Completed | History | Settings
    config_path: PathBuf,
    history_path: PathBuf,
}

enum ViewKind {
    AllTasks,
    Downloading,
    Completed,
    History,
    Settings,
}
```

`AppState` is held as an `Entity<AppState>` — gpui's reference-counted reactive state container. All views receive the same `Entity<AppState>` handle and read from it. Mutations go through `entity.update(cx, |state, cx| { ... ; cx.notify(); })`.

### Data Flow

1. **User action** (e.g., click "Add Task") calls a method on `AppState` via `cx.listener()`
2. **AppState** calls `self.queue.add_task(url, dest).await` — direct Rust function call
3. **yushi-core** starts the download and sends `QueueEvent` variants through an `mpsc::Receiver`
4. **Background task** (spawned at app startup) receives events from the channel
5. **Background task** calls `entity.update(cx, |state, cx| { state.tasks = queue.get_all_tasks().await; cx.notify(); })`
6. **gpui** re-renders affected views via GPU

This replaces the Tauri flow (6 hops, 2 language boundaries, JSON serialization) with 3 hops in pure Rust.

### Event Bridge (main.rs)

At startup, `main.rs`:
1. Creates `Application::new().with_assets(gpui_component_assets::Assets)`
2. Calls `gpui_component::init(cx)`
3. Loads `AppConfig` and `DownloadHistory` from disk
4. Creates `YuShi::new()` which returns `(YuShi, Receiver<QueueEvent>)`
5. Wraps everything in `Entity<AppState>`
6. Spawns a background task that loops on the `Receiver<QueueEvent>`, updating the entity on each event
7. Opens the window with `Root::new(AppView, window, cx)`

### GUI Layout

```
+------------------------------------------+
|  TitleBar: [YuShi v0.1.0]    [+ New Task]|
+--------+---------------------------------+
| Sidebar| Content Area                     |
|        |                                  |
| All    |  [TaskItem: file.iso]            |
| Down.. |  [====67%====] 12.5 MB/s         |
| Done   |                                  |
|        |  [TaskItem: setup.exe]           |
|--------|  [==23%] Paused                  |
| History|                                  |
| Setting|  [TaskItem: node.pkg]            |
|        |  Completed ✓                     |
+--------+---------------------------------+
```

- **TitleBar**: gpui-component `TitleBar` with app name and "New Task" button
- **Sidebar**: gpui-component `Sidebar` with nav items, collapsible
- **Content Area**: renders the current `ViewKind`
- **TaskItem**: card with filename, URL, progress bar (`Progress` component), speed/ETA text, action buttons (`Button` component)
- **AddTaskDialog**: gpui-component `Dialog` with URL input and destination path input
- **SettingsView**: form with `Input`, `NumberInput`, theme `Select`
- **HistoryView**: search `Input` + list of `CompletedTask` entries

### Theme

Use gpui-component's built-in theme system:
- `ThemeRegistry` with bundled light/dark themes
- Respect system preference for light/dark
- User can override in Settings (stored in `AppConfig.theme` as `"light"` | `"dark"` | `"system"`)

## Files to Delete

Everything related to Tauri and React:

- `src-tauri/` — entire directory
- `src/` — entire directory
- `index.html`
- `package.json`
- `deno.json`, `deno.lock`
- `vite.config.ts`
- `tsconfig.json`, `tsconfig.node.json`

## Workspace Cargo.toml Changes

- Replace `src-tauri` with `yushi-app` in `members`
- Add workspace dependencies: `gpui`, `gpui-component`, `gpui-component-assets` (all git-based)
- Remove Tauri-specific workspace dependencies that are no longer used by any crate
- Add `dirs` to workspace dependencies (needed by `yushi-core` for config)

## CLAUDE.md Update

After migration, update CLAUDE.md to reflect:
- New workspace structure (replace `src-tauri` description with `yushi-app`)
- Remove all React/frontend references
- New build commands (`cargo run -p yushi-app`)
- New architecture notes (gpui Entity model, no IPC)

## Testing Strategy

- `yushi-core`: existing tests continue to pass; add unit tests for `AppConfig::validate()` and `DownloadHistory` methods
- `yushi-app`: manual testing of the GUI (gpui does not have a standard UI test framework)
- `yushi-cli`: verify it compiles and works with the new `yushi-core` config/history API
