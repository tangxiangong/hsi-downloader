# AGENTS.md

This file provides guidance to Codex (Codex.ai/code) when working with code in this repository.

## Project Overview

Hsi is an async download manager built with a Tauri v2 + SolidJS desktop application and a shared Rust core library.

## Workspace Structure

- **Root** — Workspace-only `Cargo.toml` (no package at root). Coordinates the Rust crates below.
- **src-tauri/** — Tauri v2 desktop app binary. Uses `#[tauri::command]` to expose `hsi-core` API to the frontend. `DownloaderEvent`s are forwarded to the SolidJS frontend via the Tauri event system.
- **src-ui/** — SolidJS frontend. TailwindCSS + DaisyUI for styling. Built with Vite + Bun.
- **hsi-core** — Shared download library and data model. Owns the downloader engine (`Hsi`), queue/task types, checksum utils, shared `AppConfig`, shared `DownloadHistory`, and shared storage path helpers.
- **hsi-cli** — CLI binary. Uses `clap` for args and `indicatif` for progress bars. The `tui` feature (default) enables a `ratatui` terminal UI.

## Build & Dev Commands

```bash
# Desktop GUI (Tauri dev mode — starts both frontend and backend)
cargo tauri dev

# Production build
cargo tauri build

# Frontend only
cd src-ui && bun run dev

# CLI
cargo run -p hsi-cli -- --help

# CLI with TUI
cargo run -p hsi-cli --features tui -- tui

# Workspace validation
cargo fmt --check
cargo clippy --workspace --all-targets --all-features
cargo test --workspace --all-features

# Run a single test
cargo test -p hsi-core -- test_name
```

## Architecture Notes

- **Tauri Commands + Event System**: The frontend calls Rust functions via `#[tauri::command]` handlers. The backend pushes real-time updates to the frontend via `app_handle.emit("download-event", ...)`.
- **AppState**: Managed via `tauri::Manager::manage()` and accessed in command handlers via `State<'_, AppState>`.
- **SolidJS stores**: Reactive frontend state is organized into stores (`task-store`, `history-store`, `config-store`, `theme-store`) that listen for Tauri events and sync with the backend.
- **Theming**: DaisyUI themes controlled via the `data-theme` attribute on the root element. Supports Light/Dark/System toggle.
- **Tray icon**: The app uses a system tray icon and hides to tray on window close.
- **Shared persistence**: `config.json`, `history.json`, and `queue.json` live under `dirs::config_dir()/hsi/` (`~/.config/hsi/` on Linux/macOS). Atomic file writes (write-to-temp-then-rename) and file locking prevent corruption.
- **Event-driven updates**: `Hsi::with_config()` returns a `Receiver<DownloaderEvent>`. The Tauri backend consumes this channel and forwards events to the frontend via `app_handle.emit()`. The CLI uses it for progress bar updates.
- **Debounced state saves**: Queue state persistence is debounced (150ms debounce, 750ms interval) to avoid excessive disk I/O during rapid progress updates.
- **Per-task resume state**: Each downloading task saves a `.{filename}.json` sidecar file tracking chunk progress for resumability.
- **History tracking**: Both `hsi-cli` queue processing and the desktop app register `Hsi::set_on_complete` callbacks so successful downloads are appended into the shared `DownloadHistory`.

## Key Types & Entry Points

- `AppState` — `src-tauri/src/state.rs`: App state managed by Tauri, holding `Hsi` queue, config, history, and tasks.
- Commands — `src-tauri/src/commands.rs`: `#[tauri::command]` handlers exposing core functionality to the frontend.
- Event forwarding — `src-tauri/src/main.rs`: Setup hook that spawns the event-forwarding task from `DownloaderEvent` to Tauri events.
- Frontend entry — `src-ui/src/App.tsx`: SolidJS root component.
- `Hsi` — `hsi-core/src/downloader.rs`: Core downloader engine with task management, concurrency control via semaphore, and hot-reloadable config (`apply_runtime_config`).
- `AppConfig` — `hsi-core/src/config.rs`: Shared config with `validate()` and `downloader_config()` conversion.
- `DownloadTask` / `Task` — `hsi-core/src/types.rs`: Task model with status, priority, speed, ETA.
- `DownloaderEvent` — `hsi-core/src/types.rs`: Event enum (`TaskEvent`, `ProgressEvent`, `VerificationEvent`).

## Conventions

- Rust edition 2024, resolver v3. Workspace dependencies are declared in the root `Cargo.toml`.
- Frontend: SolidJS + TypeScript (TSX), TailwindCSS + DaisyUI for styling.
- Package manager: Bun.
- Build tool: Vite.
- Conventional commits (enforced by `git-cliff` config): `feat:`, `fix:`, `refactor:`, `chore:`, etc.
- Dual-licensed: MIT OR Apache-2.0.
- Keep config/history logic in `hsi-core`; do not reintroduce per-frontend copies.
- Tests use inline `#[cfg(test)]` modules with `#[tokio::test]` for async tests. Temp files use nonce suffixes for isolation.
