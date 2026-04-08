# Tauri v2 + SolidJS GUI Rewrite — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the gpui desktop GUI with a Tauri v2 + SolidJS + TailwindCSS + DaisyUI application while keeping `yushi-core` and `yushi-cli` unchanged.

**Architecture:** Tauri v2 backend wraps `yushi-core` via `#[tauri::command]` functions and forwards `DownloaderEvent` to the SolidJS frontend via Tauri's event system. The frontend uses SolidJS stores for reactive state and DaisyUI for styling with light/dark/system theme support.

**Tech Stack:** Tauri v2, SolidJS (TSX), TailwindCSS 4, DaisyUI 5, Vite, Bun

---

## File Structure

### Files to Delete
- `src/main.rs` — gpui entry point
- `src/state.rs` — gpui state management
- `src/utils.rs` — gpui UI helpers
- `src/components/` — all gpui components (header, nav_sidebar, content_panel, task_card, history_card, summary_row, settings_form, mod)
- `src/views/` — all gpui views (layout, tasks_page, history_page, settings_page, dialogs, task_list, mod)

### Files to Create — Rust (src-tauri/)
- `src-tauri/Cargo.toml` — Tauri binary crate, depends on `yushi-core`
- `src-tauri/build.rs` — `tauri_build::build()`
- `src-tauri/tauri.conf.json` — Tauri app configuration
- `src-tauri/capabilities/default.json` — Tauri v2 permissions
- `src-tauri/src/main.rs` — Tauri entry, setup hook, event loop
- `src-tauri/src/state.rs` — `AppState` struct with `Arc<YuShi>`, `RwLock<AppConfig>`, `RwLock<DownloadHistory>`
- `src-tauri/src/commands.rs` — all `#[tauri::command]` functions
- `src-tauri/src/events.rs` — `DownloaderEvent` → frontend event forwarding
- `src-tauri/src/tray.rs` — tray icon + panel window setup

### Files to Create — Frontend (src-ui/)
- `src-ui/package.json` — SolidJS + Tailwind + DaisyUI deps
- `src-ui/vite.config.ts` — Vite config for SolidJS
- `src-ui/tsconfig.json` — TypeScript config
- `src-ui/index.html` — HTML entry
- `src-ui/src/index.tsx` — SolidJS mount
- `src-ui/src/App.tsx` — root layout with sidebar + content
- `src-ui/src/styles/app.css` — Tailwind directives
- `src-ui/src/lib/types.ts` — TypeScript types mirroring Rust types
- `src-ui/src/lib/commands.ts` — typed `invoke()` wrappers
- `src-ui/src/lib/events.ts` — typed `listen()` wrappers
- `src-ui/src/stores/task-store.ts` — task list + filter state
- `src-ui/src/stores/config-store.ts` — app config state
- `src-ui/src/stores/history-store.ts` — history state
- `src-ui/src/stores/theme-store.ts` — theme switching
- `src-ui/src/components/Sidebar.tsx` — navigation sidebar
- `src-ui/src/components/FilterTabs.tsx` — all/downloading/completed tabs
- `src-ui/src/components/TaskCard.tsx` — task card with progress
- `src-ui/src/components/AddTaskDialog.tsx` — add task modal
- `src-ui/src/components/HistoryCard.tsx` — history entry card
- `src-ui/src/components/ThemeToggle.tsx` — light/dark/system toggle
- `src-ui/src/components/UpdateNotice.tsx` — update notification
- `src-ui/src/pages/TasksPage.tsx` — tasks view with filter
- `src-ui/src/pages/HistoryPage.tsx` — history view
- `src-ui/src/pages/SettingsPage.tsx` — settings form

### Files to Modify
- `Cargo.toml` (root) — remove `[package]` section and gpui deps, update workspace members
- `.gitignore` — add Tauri-specific ignores
- `CLAUDE.md` — update architecture docs and build commands

---

## Task 1: Clean Up Root Workspace & Delete gpui Code

**Files:**
- Modify: `Cargo.toml` (root)
- Modify: `.gitignore`
- Delete: `src/main.rs`, `src/state.rs`, `src/utils.rs`, `src/components/`, `src/views/`

- [ ] **Step 1: Delete all gpui source files**

```bash
rm -rf src/
```

- [ ] **Step 2: Update root Cargo.toml — remove package section and gpui deps**

Replace the entire `Cargo.toml` with:

```toml
[workspace]
resolver = "3"
members = ["yushi-core", "yushi-cli", "src-tauri"]

[workspace.package]
authors = ["tangxiangong <tangxiangong@gmail.com>"]
version = "0.1.0"
edition = "2024"
license = "MIT OR Apache-2.0"

[workspace.dependencies]
anyhow = "1"
dirs = "6"
fs-err = { version = "3.2", features = ["debug_tokio", "tokio"] }
futures = "0.3"
hex = "0.4"
md-5 = "0.11"
reqwest = { version = "0.13", features = ["stream", "socks"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.11"
thiserror = "2"
tokio = { version = "1", features = ["fs", "rt", "rt-multi-thread", "signal"] }
uuid = { version = "1", features = ["v4"] }
yushi-core = { path = "yushi-core" }

[profile.dev]
opt-level = 1
codegen-units = 16
debug = "limited"
split-debuginfo = "unpacked"
strip = "debuginfo"

[profile.release]
opt-level = "z"
lto = "fat"
codegen-units = 1
strip = true
panic = "abort"
```

- [ ] **Step 3: Update .gitignore — add Tauri-specific entries**

Append to `.gitignore`:

```
# Tauri
src-tauri/icons/

# Bun
bun.lock
```

- [ ] **Step 4: Verify workspace compiles without gpui**

Run: `cargo check --workspace`
Expected: SUCCESS (yushi-core and yushi-cli should compile fine)

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "refactor: remove gpui GUI code, prepare workspace for Tauri"
```

---

## Task 2: Scaffold Tauri v2 Backend

**Files:**
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/build.rs`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/capabilities/default.json`
- Create: `src-tauri/src/main.rs`

- [ ] **Step 1: Create src-tauri/Cargo.toml**

```toml
[package]
name = "yushi"
version.workspace = true
authors.workspace = true
edition.workspace = true
license.workspace = true

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
anyhow = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tokio = { workspace = true, features = ["macros"] }
tauri = { version = "2", features = [] }
tauri-plugin-dialog = "2"
tauri-plugin-updater = "2"
yushi-core = { workspace = true }
```

- [ ] **Step 2: Create src-tauri/build.rs**

```rust
fn main() {
    tauri_build::build();
}
```

- [ ] **Step 3: Create src-tauri/tauri.conf.json**

```json
{
  "$schema": "https://raw.githubusercontent.com/niceda/tauri/refs/heads/dev/crates/tauri-utils/schema.json",
  "productName": "YuShi",
  "version": "0.1.0",
  "identifier": "com.yushi.app",
  "build": {
    "devUrl": "http://localhost:5173",
    "frontendDist": "../src-ui/dist",
    "beforeDevCommand": "cd src-ui && bun run dev",
    "beforeBuildCommand": "cd src-ui && bun run build"
  },
  "app": {
    "windows": [
      {
        "title": "驭时 (YuShi)",
        "width": 1200,
        "height": 800,
        "minWidth": 800,
        "minHeight": 600
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  },
  "plugins": {
    "updater": {
      "endpoints": [
        "https://github.com/tangxiangong/YuShi/releases/latest/download/latest.json"
      ],
      "pubkey": ""
    }
  }
}
```

- [ ] **Step 4: Create src-tauri/capabilities/default.json**

```json
{
  "$schema": "https://raw.githubusercontent.com/niceda/tauri/refs/heads/dev/crates/tauri-utils/schema.json",
  "identifier": "default",
  "description": "Default capabilities for YuShi",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "dialog:default",
    "updater:default"
  ]
}
```

- [ ] **Step 5: Create minimal src-tauri/src/main.rs**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 6: Verify Rust side compiles**

Run: `cargo check -p yushi`
Expected: SUCCESS (Tauri binary compiles, no frontend yet)

- [ ] **Step 7: Commit**

```bash
git add src-tauri/
git commit -m "feat: scaffold Tauri v2 backend crate"
```

---

## Task 3: Scaffold SolidJS Frontend

**Files:**
- Create: `src-ui/package.json`
- Create: `src-ui/vite.config.ts`
- Create: `src-ui/tsconfig.json`
- Create: `src-ui/index.html`
- Create: `src-ui/src/index.tsx`
- Create: `src-ui/src/App.tsx`
- Create: `src-ui/src/styles/app.css`

- [ ] **Step 1: Create src-ui/package.json**

```json
{
  "name": "yushi-ui",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-dialog": "^2",
    "@tauri-apps/plugin-updater": "^2",
    "solid-js": "^1.9"
  },
  "devDependencies": {
    "@tailwindcss/vite": "^4",
    "daisyui": "^5",
    "tailwindcss": "^4",
    "typescript": "^5.7",
    "vite": "^6",
    "vite-plugin-solid": "^2"
  }
}
```

- [ ] **Step 2: Create src-ui/vite.config.ts**

```typescript
import { defineConfig } from "vite";
import solid from "vite-plugin-solid";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [solid(), tailwindcss()],
  server: {
    port: 5173,
    strictPort: true,
  },
  build: {
    target: "esnext",
  },
});
```

- [ ] **Step 3: Create src-ui/tsconfig.json**

```json
{
  "compilerOptions": {
    "strict": true,
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "noEmit": true,
    "jsx": "preserve",
    "jsxImportSource": "solid-js",
    "types": ["vite/client"],
    "isolatedModules": true,
    "resolveJsonModule": true,
    "skipLibCheck": true
  },
  "include": ["src"]
}
```

- [ ] **Step 4: Create src-ui/src/styles/app.css**

```css
@import "tailwindcss";
@plugin "daisyui";
```

- [ ] **Step 5: Create src-ui/index.html**

```html
<!doctype html>
<html data-theme="light">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>驭时 (YuShi)</title>
    <link rel="stylesheet" href="/src/styles/app.css" />
  </head>
  <body class="min-h-screen bg-base-200">
    <div id="app"></div>
    <script src="/src/index.tsx" type="module"></script>
  </body>
</html>
```

- [ ] **Step 6: Create src-ui/src/index.tsx**

```tsx
import { render } from "solid-js/web";
import App from "./App";

render(() => <App />, document.getElementById("app")!);
```

- [ ] **Step 7: Create src-ui/src/App.tsx**

```tsx
import type { Component } from "solid-js";

const App: Component = () => {
  return (
    <div class="flex h-screen">
      <aside class="w-56 bg-base-100 border-r border-base-300 p-4">
        <h1 class="text-xl font-bold mb-6">驭时</h1>
        <ul class="menu">
          <li><a class="active">任务</a></li>
          <li><a>历史</a></li>
          <li><a>设置</a></li>
        </ul>
      </aside>
      <main class="flex-1 p-6 overflow-y-auto">
        <p class="text-base-content">YuShi is running.</p>
      </main>
    </div>
  );
};

export default App;
```

- [ ] **Step 8: Install dependencies and verify frontend builds**

```bash
cd src-ui && bun install && bun run build
```

Expected: SUCCESS, `dist/` folder created with compiled assets

- [ ] **Step 9: Verify full Tauri dev mode works**

```bash
cargo tauri dev
```

Expected: Window opens showing "驭时" sidebar and "YuShi is running." content

- [ ] **Step 10: Commit**

```bash
git add src-ui/ Cargo.lock
git commit -m "feat: scaffold SolidJS frontend with TailwindCSS + DaisyUI"
```

---

## Task 4: Tauri State & Commands

**Files:**
- Create: `src-tauri/src/state.rs`
- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Create src-tauri/src/state.rs**

```rust
use anyhow::Result;
use std::path::PathBuf;
use tokio::sync::{RwLock, mpsc};
use yushi_core::{
    AppConfig, CompletedTask, DownloadHistory, DownloaderEvent, TaskStatus, YuShi,
    config_path, history_path, queue_state_path,
};

pub struct AppState {
    pub queue: YuShi,
    pub config: RwLock<AppConfig>,
    pub history: RwLock<DownloadHistory>,
    pub config_path: PathBuf,
    pub history_path: PathBuf,
}

impl AppState {
    pub async fn bootstrap() -> Result<(Self, mpsc::Receiver<DownloaderEvent>)> {
        let cfg_path = config_path()?;
        let hist_path = history_path()?;
        let q_path = queue_state_path()?;

        let config = AppConfig::load(&cfg_path).await?;
        config.save(&cfg_path).await?;
        let history = DownloadHistory::load(&hist_path).await?;

        let (mut queue, event_rx) = YuShi::with_config(
            config.downloader_config(),
            config.max_concurrent_tasks,
            q_path,
        );

        // Install history tracking callback
        let queue_for_history = queue.clone();
        let hist_path_clone = hist_path.clone();
        queue.set_on_complete(move |task_id, result| {
            let queue = queue_for_history.clone();
            let history_path = hist_path_clone.clone();
            async move {
                if result.is_err() {
                    return;
                }
                let Some(task) = queue.get_task(&task_id).await else {
                    return;
                };
                if task.status != TaskStatus::Completed {
                    return;
                }
                let Some(completed) = CompletedTask::from_task(&task) else {
                    return;
                };
                let _ = DownloadHistory::append_completed_to_file(&history_path, completed).await;
            }
        });

        queue.load_queue_from_state().await?;

        Ok((
            Self {
                queue,
                config: RwLock::new(config),
                history: RwLock::new(history),
                config_path: cfg_path,
                history_path: hist_path,
            },
            event_rx,
        ))
    }
}
```

- [ ] **Step 2: Create src-tauri/src/commands.rs**

```rust
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::State;
use yushi_core::{
    AppConfig, CompletedTask, DownloadHistory, DownloadTask,
    config::AppTheme, ChecksumType, TaskPriority,
};

#[derive(Debug, Deserialize)]
pub struct AddTaskOptions {
    pub url: String,
    pub dest: PathBuf,
    pub headers: Option<HashMap<String, String>>,
    pub checksum: Option<ChecksumType>,
    pub priority: Option<TaskPriority>,
    pub speed_limit: Option<u64>,
}

#[tauri::command]
pub async fn get_tasks(state: State<'_, AppState>) -> Result<Vec<DownloadTask>, String> {
    Ok(state.queue.get_all_tasks().await)
}

#[tauri::command]
pub async fn add_task(state: State<'_, AppState>, options: AddTaskOptions) -> Result<String, String> {
    let task_id = state
        .queue
        .add_task_with_options(
            &options.url,
            options.dest,
            options.headers.unwrap_or_default(),
            options.checksum,
            options.priority.unwrap_or_default(),
            options.speed_limit,
        )
        .await
        .map_err(|e| e.to_string())?;
    Ok(task_id)
}

#[tauri::command]
pub async fn pause_task(state: State<'_, AppState>, task_id: String) -> Result<(), String> {
    state.queue.pause_task(&task_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resume_task(state: State<'_, AppState>, task_id: String) -> Result<(), String> {
    state.queue.resume_task(&task_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cancel_task(state: State<'_, AppState>, task_id: String) -> Result<(), String> {
    state.queue.cancel_task(&task_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_task(state: State<'_, AppState>, task_id: String) -> Result<(), String> {
    state.queue.remove_task(&task_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_task_with_file(state: State<'_, AppState>, task_id: String) -> Result<(), String> {
    state.queue.remove_task_with_file(&task_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_completed(state: State<'_, AppState>) -> Result<(), String> {
    state.queue.clear_completed().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_history(state: State<'_, AppState>) -> Result<Vec<CompletedTask>, String> {
    let history = state.history.read().await;
    Ok(history.get_all().to_vec())
}

#[tauri::command]
pub async fn remove_history(state: State<'_, AppState>, task_id: String) -> Result<(), String> {
    let (new_history, _) = DownloadHistory::remove_from_file(&state.history_path, &task_id)
        .await
        .map_err(|e| e.to_string())?;
    *state.history.write().await = new_history;
    Ok(())
}

#[tauri::command]
pub async fn remove_history_with_file(state: State<'_, AppState>, task_id: String) -> Result<(), String> {
    let (new_history, _) = DownloadHistory::remove_entry_and_file_from_file(&state.history_path, &task_id)
        .await
        .map_err(|e| e.to_string())?;
    *state.history.write().await = new_history;
    Ok(())
}

#[tauri::command]
pub async fn clear_history(state: State<'_, AppState>) -> Result<(), String> {
    let new_history = DownloadHistory::clear_file(&state.history_path)
        .await
        .map_err(|e| e.to_string())?;
    *state.history.write().await = new_history;
    Ok(())
}

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    Ok(state.config.read().await.clone())
}

#[tauri::command]
pub async fn update_config(state: State<'_, AppState>, config: AppConfig) -> Result<(), String> {
    config.validate().map_err(|e| e.to_string())?;
    config.save(&state.config_path).await.map_err(|e| e.to_string())?;

    state
        .queue
        .apply_runtime_config(config.downloader_config(), config.max_concurrent_tasks)
        .await;

    *state.config.write().await = config;
    Ok(())
}

#[tauri::command]
pub async fn infer_destination(
    state: State<'_, AppState>,
    url: String,
    directory: PathBuf,
) -> Result<PathBuf, String> {
    state
        .queue
        .infer_destination_in_dir(&url, &directory)
        .await
        .map_err(|e| e.to_string())
}
```

- [ ] **Step 3: Update src-tauri/src/main.rs to wire state and commands**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;

use state::AppState;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let handle = app.handle().clone();

            tauri::async_runtime::block_on(async {
                let (app_state, mut event_rx) = AppState::bootstrap()
                    .await
                    .expect("failed to bootstrap app state");

                handle.manage(app_state);

                // Forward DownloaderEvent to frontend
                tauri::async_runtime::spawn(async move {
                    while let Some(event) = event_rx.recv().await {
                        let _ = handle.emit("download-event", &event);
                    }
                });
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_tasks,
            commands::add_task,
            commands::pause_task,
            commands::resume_task,
            commands::cancel_task,
            commands::remove_task,
            commands::remove_task_with_file,
            commands::clear_completed,
            commands::get_history,
            commands::remove_history,
            commands::remove_history_with_file,
            commands::clear_history,
            commands::get_config,
            commands::update_config,
            commands::infer_destination,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 4: Verify compilation**

Run: `cargo check -p yushi`
Expected: SUCCESS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/
git commit -m "feat: implement Tauri state management and commands"
```

---

## Task 5: Frontend TypeScript Types & Tauri Bindings

**Files:**
- Create: `src-ui/src/lib/types.ts`
- Create: `src-ui/src/lib/commands.ts`
- Create: `src-ui/src/lib/events.ts`

- [ ] **Step 1: Create src-ui/src/lib/types.ts**

```typescript
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
  headers?: Record<string, string>;
  checksum?: ChecksumType;
  priority?: TaskPriority;
  speed_limit?: number;
}

// Event types matching DownloaderEvent
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
  | { Updated: { task_id: string; downloaded: number; total: number; speed: number; eta: number | null } }
  | { Finished: { task_id: string } }
  | { Failed: { task_id: string; error: string } };

export type VerificationEvent =
  | { Started: { task_id: string } }
  | { Completed: { task_id: string; success: boolean } };
```

- [ ] **Step 2: Create src-ui/src/lib/commands.ts**

```typescript
import { invoke } from "@tauri-apps/api/core";
import type { AddTaskOptions, AppConfig, CompletedTask, DownloadTask } from "./types";

export async function getTasks(): Promise<DownloadTask[]> {
  return invoke("get_tasks");
}

export async function addTask(options: AddTaskOptions): Promise<string> {
  return invoke("add_task", { options });
}

export async function pauseTask(taskId: string): Promise<void> {
  return invoke("pause_task", { taskId });
}

export async function resumeTask(taskId: string): Promise<void> {
  return invoke("resume_task", { taskId });
}

export async function cancelTask(taskId: string): Promise<void> {
  return invoke("cancel_task", { taskId });
}

export async function removeTask(taskId: string): Promise<void> {
  return invoke("remove_task", { taskId });
}

export async function removeTaskWithFile(taskId: string): Promise<void> {
  return invoke("remove_task_with_file", { taskId });
}

export async function clearCompleted(): Promise<void> {
  return invoke("clear_completed");
}

export async function getHistory(): Promise<CompletedTask[]> {
  return invoke("get_history");
}

export async function removeHistory(taskId: string): Promise<void> {
  return invoke("remove_history", { taskId });
}

export async function removeHistoryWithFile(taskId: string): Promise<void> {
  return invoke("remove_history_with_file", { taskId });
}

export async function clearHistory(): Promise<void> {
  return invoke("clear_history");
}

export async function getConfig(): Promise<AppConfig> {
  return invoke("get_config");
}

export async function updateConfig(config: AppConfig): Promise<void> {
  return invoke("update_config", { config });
}

export async function inferDestination(url: string, directory: string): Promise<string> {
  return invoke("infer_destination", { url, directory });
}
```

- [ ] **Step 3: Create src-ui/src/lib/events.ts**

```typescript
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { DownloaderEvent } from "./types";

export function onDownloadEvent(
  callback: (event: DownloaderEvent) => void,
): Promise<UnlistenFn> {
  return listen<DownloaderEvent>("download-event", (e) => callback(e.payload));
}
```

- [ ] **Step 4: Verify frontend builds with new types**

Run: `cd src-ui && bun run build`
Expected: SUCCESS

- [ ] **Step 5: Commit**

```bash
git add src-ui/src/lib/
git commit -m "feat: add TypeScript types and Tauri command/event bindings"
```

---

## Task 6: SolidJS Stores

**Files:**
- Create: `src-ui/src/stores/theme-store.ts`
- Create: `src-ui/src/stores/config-store.ts`
- Create: `src-ui/src/stores/task-store.ts`
- Create: `src-ui/src/stores/history-store.ts`

- [ ] **Step 1: Create src-ui/src/stores/theme-store.ts**

```typescript
import { createSignal, createEffect, onMount } from "solid-js";
import type { AppTheme } from "../lib/types";

const [theme, setThemeSignal] = createSignal<AppTheme>("system");

function applyTheme(t: AppTheme) {
  let resolved: "light" | "dark";
  if (t === "system") {
    resolved = window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  } else {
    resolved = t;
  }
  document.documentElement.setAttribute("data-theme", resolved);
}

// Watch system preference changes
if (typeof window !== "undefined") {
  window
    .matchMedia("(prefers-color-scheme: dark)")
    .addEventListener("change", () => {
      if (theme() === "system") {
        applyTheme("system");
      }
    });
}

export function setTheme(t: AppTheme) {
  setThemeSignal(t);
  applyTheme(t);
}

export { theme };
```

- [ ] **Step 2: Create src-ui/src/stores/config-store.ts**

```typescript
import { createStore } from "solid-js/store";
import type { AppConfig } from "../lib/types";
import { getConfig, updateConfig as updateConfigCmd } from "../lib/commands";
import { setTheme } from "./theme-store";

const [config, setConfig] = createStore<AppConfig>({
  default_download_path: "",
  max_concurrent_downloads: 4,
  max_concurrent_tasks: 2,
  chunk_size: 10485760,
  timeout: 30,
  user_agent: "YuShi/1.0",
  proxy: null,
  speed_limit: null,
  theme: "system",
});

export async function loadConfig() {
  const cfg = await getConfig();
  setConfig(cfg);
  setTheme(cfg.theme);
}

export async function saveConfig(updates: Partial<AppConfig>) {
  const newConfig = { ...config, ...updates };
  await updateConfigCmd(newConfig);
  setConfig(newConfig);
  if (updates.theme) {
    setTheme(updates.theme);
  }
}

export { config };
```

- [ ] **Step 3: Create src-ui/src/stores/task-store.ts**

```typescript
import { createSignal, createMemo } from "solid-js";
import { createStore, produce } from "solid-js/store";
import type { DownloadTask, DownloaderEvent, TaskStatus } from "../lib/types";
import { getTasks } from "../lib/commands";
import { onDownloadEvent } from "../lib/events";

export type TaskFilter = "all" | "downloading" | "completed";

const [tasks, setTasks] = createStore<DownloadTask[]>([]);
const [filter, setFilter] = createSignal<TaskFilter>("all");

export const filteredTasks = createMemo(() => {
  const f = filter();
  if (f === "all") return tasks;
  if (f === "downloading")
    return tasks.filter(
      (t) => t.status === "Downloading" || t.status === "Pending" || t.status === "Paused",
    );
  return tasks.filter(
    (t) => t.status === "Completed" || t.status === "Failed" || t.status === "Cancelled",
  );
});

export const taskCounts = createMemo(() => ({
  all: tasks.length,
  downloading: tasks.filter(
    (t) => t.status === "Downloading" || t.status === "Pending" || t.status === "Paused",
  ).length,
  completed: tasks.filter(
    (t) => t.status === "Completed" || t.status === "Failed" || t.status === "Cancelled",
  ).length,
}));

export async function loadTasks() {
  const list = await getTasks();
  setTasks(list);
}

export async function refreshTasks() {
  await loadTasks();
}

export function setupTaskEvents() {
  onDownloadEvent(async (event: DownloaderEvent) => {
    // Refresh full task list on any event for simplicity and correctness
    await refreshTasks();
  });
}

export { tasks, filter, setFilter };
```

- [ ] **Step 4: Create src-ui/src/stores/history-store.ts**

```typescript
import { createStore } from "solid-js/store";
import type { CompletedTask } from "../lib/types";
import { getHistory } from "../lib/commands";

const [history, setHistory] = createStore<CompletedTask[]>([]);

export async function loadHistory() {
  const list = await getHistory();
  setHistory(list);
}

export async function refreshHistory() {
  await loadHistory();
}

export { history };
```

- [ ] **Step 5: Verify frontend builds**

Run: `cd src-ui && bun run build`
Expected: SUCCESS

- [ ] **Step 6: Commit**

```bash
git add src-ui/src/stores/
git commit -m "feat: add SolidJS stores for tasks, history, config, and theme"
```

---

## Task 7: Layout, Sidebar & Page Navigation

**Files:**
- Create: `src-ui/src/components/Sidebar.tsx`
- Modify: `src-ui/src/App.tsx`
- Create: `src-ui/src/pages/TasksPage.tsx` (placeholder)
- Create: `src-ui/src/pages/HistoryPage.tsx` (placeholder)
- Create: `src-ui/src/pages/SettingsPage.tsx` (placeholder)

- [ ] **Step 1: Create src-ui/src/components/Sidebar.tsx**

```tsx
import type { Component } from "solid-js";

export type Page = "tasks" | "history" | "settings";

interface SidebarProps {
  current: Page;
  onChange: (page: Page) => void;
}

const Sidebar: Component<SidebarProps> = (props) => {
  const items: { page: Page; label: string }[] = [
    { page: "tasks", label: "任务" },
    { page: "history", label: "历史" },
    { page: "settings", label: "设置" },
  ];

  return (
    <aside class="w-56 bg-base-100 border-r border-base-300 flex flex-col h-screen">
      <div class="p-4 border-b border-base-300">
        <h1 class="text-xl font-bold">驭时</h1>
        <p class="text-xs text-base-content/50 mt-1">YuShi Download Manager</p>
      </div>
      <ul class="menu flex-1 p-2">
        {items.map((item) => (
          <li>
            <a
              class={props.current === item.page ? "active" : ""}
              onClick={() => props.onChange(item.page)}
            >
              {item.label}
            </a>
          </li>
        ))}
      </ul>
      <div class="p-4 text-xs text-base-content/40">
        v0.1.0
      </div>
    </aside>
  );
};

export default Sidebar;
```

- [ ] **Step 2: Create placeholder pages**

`src-ui/src/pages/TasksPage.tsx`:
```tsx
import type { Component } from "solid-js";

const TasksPage: Component = () => {
  return <div><h2 class="text-2xl font-bold mb-4">任务</h2></div>;
};

export default TasksPage;
```

`src-ui/src/pages/HistoryPage.tsx`:
```tsx
import type { Component } from "solid-js";

const HistoryPage: Component = () => {
  return <div><h2 class="text-2xl font-bold mb-4">历史</h2></div>;
};

export default HistoryPage;
```

`src-ui/src/pages/SettingsPage.tsx`:
```tsx
import type { Component } from "solid-js";

const SettingsPage: Component = () => {
  return <div><h2 class="text-2xl font-bold mb-4">设置</h2></div>;
};

export default SettingsPage;
```

- [ ] **Step 3: Update App.tsx with navigation and initialization**

```tsx
import { type Component, createSignal, onMount, Match, Switch } from "solid-js";
import Sidebar, { type Page } from "./components/Sidebar";
import TasksPage from "./pages/TasksPage";
import HistoryPage from "./pages/HistoryPage";
import SettingsPage from "./pages/SettingsPage";
import { loadConfig } from "./stores/config-store";
import { loadTasks, setupTaskEvents } from "./stores/task-store";
import { loadHistory } from "./stores/history-store";

const App: Component = () => {
  const [page, setPage] = createSignal<Page>("tasks");

  onMount(async () => {
    await loadConfig();
    await loadTasks();
    await loadHistory();
    setupTaskEvents();
  });

  return (
    <div class="flex h-screen">
      <Sidebar current={page()} onChange={setPage} />
      <main class="flex-1 p-6 overflow-y-auto bg-base-200">
        <Switch>
          <Match when={page() === "tasks"}>
            <TasksPage />
          </Match>
          <Match when={page() === "history"}>
            <HistoryPage />
          </Match>
          <Match when={page() === "settings"}>
            <SettingsPage />
          </Match>
        </Switch>
      </main>
    </div>
  );
};

export default App;
```

- [ ] **Step 4: Verify with `cargo tauri dev`**

Expected: Window opens, sidebar navigation works between 3 placeholder pages

- [ ] **Step 5: Commit**

```bash
git add src-ui/src/
git commit -m "feat: add sidebar navigation and page routing"
```

---

## Task 8: TasksPage — Filter Tabs + Task Cards

**Files:**
- Create: `src-ui/src/components/FilterTabs.tsx`
- Create: `src-ui/src/components/TaskCard.tsx`
- Create: `src-ui/src/components/AddTaskDialog.tsx`
- Modify: `src-ui/src/pages/TasksPage.tsx`

- [ ] **Step 1: Create src-ui/src/components/FilterTabs.tsx**

```tsx
import type { Component } from "solid-js";
import { type TaskFilter, taskCounts } from "../stores/task-store";

interface FilterTabsProps {
  current: TaskFilter;
  onChange: (filter: TaskFilter) => void;
}

const FilterTabs: Component<FilterTabsProps> = (props) => {
  const tabs: { filter: TaskFilter; label: string; countKey: TaskFilter }[] = [
    { filter: "all", label: "全部", countKey: "all" },
    { filter: "downloading", label: "下载中", countKey: "downloading" },
    { filter: "completed", label: "已完成", countKey: "completed" },
  ];

  return (
    <div role="tablist" class="tabs tabs-bordered">
      {tabs.map((tab) => (
        <a
          role="tab"
          class={`tab ${props.current === tab.filter ? "tab-active" : ""}`}
          onClick={() => props.onChange(tab.filter)}
        >
          {tab.label}
          <span class="badge badge-sm ml-2">{taskCounts()[tab.countKey]}</span>
        </a>
      ))}
    </div>
  );
};

export default FilterTabs;
```

- [ ] **Step 2: Create src-ui/src/lib/format.ts — shared formatting helpers**

```typescript
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
```

- [ ] **Step 3: Create src-ui/src/components/TaskCard.tsx**

```tsx
import { type Component, Show } from "solid-js";
import type { DownloadTask } from "../lib/types";
import { formatBytes, formatSpeed, formatEta, statusLabel, statusBadgeClass } from "../lib/format";
import { pauseTask, resumeTask, cancelTask, removeTask, removeTaskWithFile } from "../lib/commands";
import { refreshTasks } from "../stores/task-store";

interface TaskCardProps {
  task: DownloadTask;
}

const TaskCard: Component<TaskCardProps> = (props) => {
  const t = () => props.task;
  const progress = () =>
    t().total_size > 0 ? (t().downloaded / t().total_size) * 100 : 0;
  const filename = () => {
    const parts = t().dest.split(/[/\\]/);
    return parts[parts.length - 1] || t().url;
  };

  async function handlePause() {
    await pauseTask(t().id);
    await refreshTasks();
  }
  async function handleResume() {
    await resumeTask(t().id);
    await refreshTasks();
  }
  async function handleCancel() {
    await cancelTask(t().id);
    await refreshTasks();
  }
  async function handleRemove() {
    await removeTask(t().id);
    await refreshTasks();
  }
  async function handleRemoveWithFile() {
    await removeTaskWithFile(t().id);
    await refreshTasks();
  }

  return (
    <div class="card bg-base-100 shadow-sm">
      <div class="card-body p-4">
        <div class="flex items-center justify-between">
          <div class="flex-1 min-w-0">
            <h3 class="font-medium truncate">{filename()}</h3>
            <p class="text-xs text-base-content/50 truncate">{t().url}</p>
          </div>
          <span class={`badge ${statusBadgeClass(t().status)} ml-2`}>
            {statusLabel(t().status)}
          </span>
        </div>

        <Show when={t().status === "Downloading" || t().total_size > 0}>
          <div class="mt-2">
            <progress
              class="progress progress-info w-full"
              value={progress()}
              max="100"
            />
            <div class="flex justify-between text-xs text-base-content/60 mt-1">
              <span>
                {formatBytes(t().downloaded)} / {formatBytes(t().total_size)}
              </span>
              <Show when={t().status === "Downloading"}>
                <span>
                  {formatSpeed(t().speed)} · {formatEta(t().eta)}
                </span>
              </Show>
            </div>
          </div>
        </Show>

        <Show when={t().error}>
          <p class="text-xs text-error mt-1">{t().error}</p>
        </Show>

        <div class="card-actions justify-end mt-2">
          <Show when={t().status === "Downloading"}>
            <button class="btn btn-sm btn-ghost" onClick={handlePause}>暂停</button>
          </Show>
          <Show when={t().status === "Paused"}>
            <button class="btn btn-sm btn-ghost" onClick={handleResume}>恢复</button>
          </Show>
          <Show when={t().status === "Downloading" || t().status === "Paused" || t().status === "Pending"}>
            <button class="btn btn-sm btn-ghost text-error" onClick={handleCancel}>取消</button>
          </Show>
          <Show when={t().status === "Completed" || t().status === "Failed" || t().status === "Cancelled"}>
            <button class="btn btn-sm btn-ghost" onClick={handleRemove}>移除</button>
            <button class="btn btn-sm btn-ghost text-error" onClick={handleRemoveWithFile}>删除文件</button>
          </Show>
        </div>
      </div>
    </div>
  );
};

export default TaskCard;
```

- [ ] **Step 4: Create src-ui/src/components/AddTaskDialog.tsx**

```tsx
import { type Component, createSignal } from "solid-js";
import { addTask } from "../lib/commands";
import { refreshTasks } from "../stores/task-store";
import { config } from "../stores/config-store";
import { open } from "@tauri-apps/plugin-dialog";

interface AddTaskDialogProps {
  onClose: () => void;
}

const AddTaskDialog: Component<AddTaskDialogProps> = (props) => {
  const [url, setUrl] = createSignal("");
  const [dest, setDest] = createSignal(config.default_download_path);
  const [error, setError] = createSignal("");
  const [loading, setLoading] = createSignal(false);

  async function pickDir() {
    const selected = await open({ directory: true, defaultPath: dest() });
    if (selected) setDest(selected);
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!url().trim()) {
      setError("请输入下载链接");
      return;
    }
    setLoading(true);
    setError("");
    try {
      await addTask({ url: url(), dest: dest() });
      await refreshTasks();
      props.onClose();
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }

  return (
    <dialog class="modal modal-open">
      <div class="modal-box">
        <h3 class="font-bold text-lg">添加下载任务</h3>
        <form onSubmit={handleSubmit} class="mt-4 space-y-4">
          <div class="form-control">
            <label class="label"><span class="label-text">下载链接</span></label>
            <input
              type="text"
              class="input input-bordered w-full"
              placeholder="https://example.com/file.zip"
              value={url()}
              onInput={(e) => setUrl(e.currentTarget.value)}
            />
          </div>
          <div class="form-control">
            <label class="label"><span class="label-text">保存位置</span></label>
            <div class="flex gap-2">
              <input
                type="text"
                class="input input-bordered flex-1"
                value={dest()}
                onInput={(e) => setDest(e.currentTarget.value)}
              />
              <button type="button" class="btn btn-ghost" onClick={pickDir}>
                浏览
              </button>
            </div>
          </div>
          {error() && <p class="text-error text-sm">{error()}</p>}
          <div class="modal-action">
            <button type="button" class="btn btn-ghost" onClick={props.onClose}>
              取消
            </button>
            <button type="submit" class="btn btn-primary" disabled={loading()}>
              {loading() ? "添加中..." : "添加"}
            </button>
          </div>
        </form>
      </div>
      <form method="dialog" class="modal-backdrop">
        <button onClick={props.onClose}>close</button>
      </form>
    </dialog>
  );
};

export default AddTaskDialog;
```

- [ ] **Step 5: Update TasksPage.tsx**

```tsx
import { type Component, createSignal, For, Show } from "solid-js";
import FilterTabs from "../components/FilterTabs";
import TaskCard from "../components/TaskCard";
import AddTaskDialog from "../components/AddTaskDialog";
import { filteredTasks, filter, setFilter } from "../stores/task-store";

const TasksPage: Component = () => {
  const [showAddDialog, setShowAddDialog] = createSignal(false);

  return (
    <div>
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-2xl font-bold">任务</h2>
        <button class="btn btn-primary btn-sm" onClick={() => setShowAddDialog(true)}>
          添加任务
        </button>
      </div>

      <FilterTabs current={filter()} onChange={setFilter} />

      <div class="mt-4 space-y-3">
        <For each={filteredTasks()} fallback={
          <div class="text-center text-base-content/50 py-12">暂无任务</div>
        }>
          {(task) => <TaskCard task={task} />}
        </For>
      </div>

      <Show when={showAddDialog()}>
        <AddTaskDialog onClose={() => setShowAddDialog(false)} />
      </Show>
    </div>
  );
};

export default TasksPage;
```

- [ ] **Step 6: Verify with `cargo tauri dev`**

Expected: Tasks page shows filter tabs, "添加任务" button opens dialog, task cards render with status/progress

- [ ] **Step 7: Commit**

```bash
git add src-ui/src/
git commit -m "feat: implement tasks page with filter tabs, task cards, and add dialog"
```

---

## Task 9: HistoryPage

**Files:**
- Create: `src-ui/src/components/HistoryCard.tsx`
- Modify: `src-ui/src/pages/HistoryPage.tsx`

- [ ] **Step 1: Create src-ui/src/components/HistoryCard.tsx**

```tsx
import type { Component } from "solid-js";
import type { CompletedTask } from "../lib/types";
import { formatBytes, formatSpeed, formatDate } from "../lib/format";
import { removeHistory, removeHistoryWithFile } from "../lib/commands";
import { refreshHistory } from "../stores/history-store";

interface HistoryCardProps {
  task: CompletedTask;
}

const HistoryCard: Component<HistoryCardProps> = (props) => {
  const t = () => props.task;
  const filename = () => {
    const parts = t().dest.split(/[/\\]/);
    return parts[parts.length - 1] || t().url;
  };

  async function handleRemove() {
    await removeHistory(t().id);
    await refreshHistory();
  }

  async function handleRemoveWithFile() {
    await removeHistoryWithFile(t().id);
    await refreshHistory();
  }

  return (
    <div class="card bg-base-100 shadow-sm">
      <div class="card-body p-4">
        <div class="flex items-center justify-between">
          <div class="flex-1 min-w-0">
            <h3 class="font-medium truncate">{filename()}</h3>
            <p class="text-xs text-base-content/50 truncate">{t().url}</p>
          </div>
          <span class="badge badge-success ml-2">已完成</span>
        </div>
        <div class="flex gap-4 text-xs text-base-content/60 mt-2">
          <span>{formatBytes(t().total_size)}</span>
          <span>平均 {formatSpeed(t().avg_speed)}</span>
          <span>{formatDate(t().completed_at)}</span>
        </div>
        <div class="card-actions justify-end mt-2">
          <button class="btn btn-sm btn-ghost" onClick={handleRemove}>移除记录</button>
          <button class="btn btn-sm btn-ghost text-error" onClick={handleRemoveWithFile}>删除文件</button>
        </div>
      </div>
    </div>
  );
};

export default HistoryCard;
```

- [ ] **Step 2: Update HistoryPage.tsx**

```tsx
import { type Component, For } from "solid-js";
import HistoryCard from "../components/HistoryCard";
import { history } from "../stores/history-store";
import { clearHistory } from "../lib/commands";
import { refreshHistory } from "../stores/history-store";

const HistoryPage: Component = () => {
  async function handleClear() {
    await clearHistory();
    await refreshHistory();
  }

  return (
    <div>
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-2xl font-bold">历史</h2>
        <button
          class="btn btn-ghost btn-sm text-error"
          onClick={handleClear}
          disabled={history.length === 0}
        >
          清空历史
        </button>
      </div>

      <div class="space-y-3">
        <For each={history} fallback={
          <div class="text-center text-base-content/50 py-12">暂无历史记录</div>
        }>
          {(task) => <HistoryCard task={task} />}
        </For>
      </div>
    </div>
  );
};

export default HistoryPage;
```

- [ ] **Step 3: Wire history refresh on task completion — update task-store.ts event handler**

In `src-ui/src/stores/task-store.ts`, update the `setupTaskEvents` function:

```typescript
import { refreshHistory } from "./history-store";

export function setupTaskEvents() {
  onDownloadEvent(async (event: DownloaderEvent) => {
    await refreshTasks();
    // Refresh history when a task completes
    if (event.type === "Task" && "Completed" in event.data) {
      await refreshHistory();
    }
  });
}
```

- [ ] **Step 4: Verify with `cargo tauri dev`**

Expected: History page lists completed downloads, clear/remove actions work

- [ ] **Step 5: Commit**

```bash
git add src-ui/src/
git commit -m "feat: implement history page with history cards"
```

---

## Task 10: SettingsPage

**Files:**
- Create: `src-ui/src/components/ThemeToggle.tsx`
- Modify: `src-ui/src/pages/SettingsPage.tsx`

- [ ] **Step 1: Create src-ui/src/components/ThemeToggle.tsx**

```tsx
import type { Component } from "solid-js";
import type { AppTheme } from "../lib/types";

interface ThemeToggleProps {
  value: AppTheme;
  onChange: (theme: AppTheme) => void;
}

const ThemeToggle: Component<ThemeToggleProps> = (props) => {
  const options: { value: AppTheme; label: string }[] = [
    { value: "light", label: "浅色" },
    { value: "dark", label: "深色" },
    { value: "system", label: "跟随系统" },
  ];

  return (
    <div class="flex gap-2">
      {options.map((opt) => (
        <button
          class={`btn btn-sm ${props.value === opt.value ? "btn-primary" : "btn-ghost"}`}
          onClick={() => props.onChange(opt.value)}
        >
          {opt.label}
        </button>
      ))}
    </div>
  );
};

export default ThemeToggle;
```

- [ ] **Step 2: Update SettingsPage.tsx**

```tsx
import { type Component, createSignal } from "solid-js";
import { config, saveConfig } from "../stores/config-store";
import ThemeToggle from "../components/ThemeToggle";
import type { AppTheme } from "../lib/types";
import { open } from "@tauri-apps/plugin-dialog";
import { check } from "@tauri-apps/plugin-updater";

const SettingsPage: Component = () => {
  const [saving, setSaving] = createSignal(false);
  const [updateStatus, setUpdateStatus] = createSignal("");

  async function pickDefaultDir() {
    const selected = await open({ directory: true, defaultPath: config.default_download_path });
    if (selected) await saveConfig({ default_download_path: selected });
  }

  async function handleSave(field: string, value: unknown) {
    setSaving(true);
    try {
      await saveConfig({ [field]: value } as any);
    } finally {
      setSaving(false);
    }
  }

  async function checkUpdate() {
    setUpdateStatus("检查中...");
    try {
      const update = await check();
      if (update) {
        setUpdateStatus(`发现新版本: ${update.version}`);
        if (confirm(`发现新版本 ${update.version}，是否立即更新？`)) {
          await update.downloadAndInstall();
        }
      } else {
        setUpdateStatus("已是最新版本");
      }
    } catch (e) {
      setUpdateStatus(`检查失败: ${e}`);
    }
  }

  return (
    <div>
      <h2 class="text-2xl font-bold mb-6">设置</h2>

      <div class="space-y-6 max-w-2xl">
        {/* Download Settings */}
        <div class="card bg-base-100 shadow-sm">
          <div class="card-body">
            <h3 class="card-title text-base">下载</h3>
            <div class="form-control">
              <label class="label"><span class="label-text">默认下载路径</span></label>
              <div class="flex gap-2">
                <input type="text" class="input input-bordered flex-1" value={config.default_download_path} readOnly />
                <button class="btn btn-ghost" onClick={pickDefaultDir}>浏览</button>
              </div>
            </div>
            <div class="grid grid-cols-2 gap-4">
              <div class="form-control">
                <label class="label"><span class="label-text">每任务最大连接数</span></label>
                <input
                  type="number" min="1" max="32"
                  class="input input-bordered"
                  value={config.max_concurrent_downloads}
                  onChange={(e) => handleSave("max_concurrent_downloads", parseInt(e.currentTarget.value))}
                />
              </div>
              <div class="form-control">
                <label class="label"><span class="label-text">最大同时任务数</span></label>
                <input
                  type="number" min="1" max="16"
                  class="input input-bordered"
                  value={config.max_concurrent_tasks}
                  onChange={(e) => handleSave("max_concurrent_tasks", parseInt(e.currentTarget.value))}
                />
              </div>
            </div>
          </div>
        </div>

        {/* Network Settings */}
        <div class="card bg-base-100 shadow-sm">
          <div class="card-body">
            <h3 class="card-title text-base">网络</h3>
            <div class="form-control">
              <label class="label"><span class="label-text">代理</span></label>
              <input
                type="text"
                class="input input-bordered"
                placeholder="socks5://127.0.0.1:1080"
                value={config.proxy ?? ""}
                onChange={(e) => handleSave("proxy", e.currentTarget.value || null)}
              />
            </div>
            <div class="form-control">
              <label class="label"><span class="label-text">User Agent</span></label>
              <input
                type="text"
                class="input input-bordered"
                value={config.user_agent}
                onChange={(e) => handleSave("user_agent", e.currentTarget.value)}
              />
            </div>
            <div class="grid grid-cols-2 gap-4">
              <div class="form-control">
                <label class="label"><span class="label-text">超时 (秒)</span></label>
                <input
                  type="number" min="5"
                  class="input input-bordered"
                  value={config.timeout}
                  onChange={(e) => handleSave("timeout", parseInt(e.currentTarget.value))}
                />
              </div>
              <div class="form-control">
                <label class="label"><span class="label-text">分块大小 (MB)</span></label>
                <input
                  type="number" min="1"
                  class="input input-bordered"
                  value={Math.round(config.chunk_size / 1048576)}
                  onChange={(e) => handleSave("chunk_size", parseInt(e.currentTarget.value) * 1048576)}
                />
              </div>
            </div>
          </div>
        </div>

        {/* Appearance */}
        <div class="card bg-base-100 shadow-sm">
          <div class="card-body">
            <h3 class="card-title text-base">外观</h3>
            <div class="form-control">
              <label class="label"><span class="label-text">主题</span></label>
              <ThemeToggle
                value={config.theme}
                onChange={(theme) => handleSave("theme", theme)}
              />
            </div>
          </div>
        </div>

        {/* About */}
        <div class="card bg-base-100 shadow-sm">
          <div class="card-body">
            <h3 class="card-title text-base">关于</h3>
            <p class="text-sm text-base-content/60">驭时 (YuShi) v0.1.0</p>
            <div class="flex items-center gap-4 mt-2">
              <button class="btn btn-sm btn-ghost" onClick={checkUpdate}>检查更新</button>
              {updateStatus() && <span class="text-sm text-base-content/60">{updateStatus()}</span>}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default SettingsPage;
```

- [ ] **Step 3: Verify with `cargo tauri dev`**

Expected: Settings page shows all config fields, theme toggle switches themes, directory picker works

- [ ] **Step 4: Commit**

```bash
git add src-ui/src/
git commit -m "feat: implement settings page with theme toggle and update check"
```

---

## Task 11: Tray Icon with Panel Window

**Files:**
- Create: `src-tauri/src/tray.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Research the community tray plugin**

Search for `tauri-plugin-tray-panel` or similar community plugin on crates.io / GitHub. Identify the correct crate name and version. If no suitable community plugin with panel window is available, fall back to Tauri's built-in `TrayIconBuilder` with a separate small webview window positioned near the tray icon.

Note: This step requires runtime research. The implementer should check which community tray plugin supports panel windows and adapt accordingly.

- [ ] **Step 2: Create src-tauri/src/tray.rs**

Using Tauri's built-in tray API (fallback approach if no community plugin fits):

```rust
use tauri::{
    AppHandle, Manager,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItemBuilder::with_id("show", "显示主窗口").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;
    let menu = MenuBuilder::new(app).items(&[&show, &quit]).build()?;

    TrayIconBuilder::new()
        .tooltip("驭时 (YuShi)")
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}
```

- [ ] **Step 3: Update src-tauri/src/main.rs — add tray setup and window close handling**

Add `mod tray;` at the top, then in the `setup` closure, after managing state:

```rust
mod tray;

// Inside setup, after handle.manage(app_state):
tray::setup_tray(&handle)?;

// Handle window close — hide to tray instead of quitting
let main_window = app.get_webview_window("main").unwrap();
main_window.on_window_event(move |event| {
    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        let _ = main_window_clone.hide();
    }
});
```

Adjust the full main.rs to integrate properly (the implementer will need to get the window handle correctly within the setup closure).

- [ ] **Step 4: Verify tray icon appears and window hides on close**

Run: `cargo tauri dev`
Expected: Tray icon appears, closing window hides it, tray menu "显示主窗口" restores it, "退出" quits

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/
git commit -m "feat: add system tray with hide-to-tray on close"
```

---

## Task 12: Update CLAUDE.md

**Files:**
- Modify: `CLAUDE.md`

- [ ] **Step 1: Update CLAUDE.md to reflect new architecture**

Replace the relevant sections:

- **Project Overview**: Update to mention Tauri v2 + SolidJS
- **Workspace Structure**: Update root package → `src-tauri`, add `src-ui` description
- **Build & Dev Commands**: Replace with `cargo tauri dev` / `cargo tauri build` / `cd src-ui && bun run dev`
- **Architecture Notes**: Replace gpui references with Tauri Commands + Event System
- **Key Types & Entry Points**: Update file paths to `src-tauri/src/`
- **Conventions**: Add frontend conventions (SolidJS, TailwindCSS, DaisyUI)

- [ ] **Step 2: Verify the entire app works end-to-end**

```bash
cargo tauri dev
```

Expected: Full app works — task management, history, settings, theme switching, tray icon

- [ ] **Step 3: Run workspace checks**

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features
cargo test --workspace --all-features
```

Expected: All pass

- [ ] **Step 4: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: update CLAUDE.md for Tauri + SolidJS architecture"
```
