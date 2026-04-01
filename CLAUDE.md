# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

YuShi (驭时) is an async download manager built as a pure Rust workspace. The old Tauri + React frontend has been removed and replaced by a `gpui` desktop application.

## Workspace Structure

- **Root package (`yushi`)** — This is the desktop GUI app. Its source lives in `src/` at the repo root. Built with `gpui` + `gpui-component`. Uses a shared `Entity<AppState>` model and calls `yushi-core` directly. No IPC layer.
- **yushi-core** — Shared download library and data model. Owns the downloader engine (`YuShi`), queue/task types, checksum utils, shared `AppConfig`, shared `DownloadHistory`, and shared storage path helpers.
- **yushi-cli** — CLI binary. Uses `clap` for args and `indicatif` for progress bars. The `tui` feature (default) enables a `ratatui` terminal UI.

## Build & Dev Commands

```bash
# Desktop GUI (root package is the app)
cargo run

# CLI
cargo run -p yushi-cli -- --help

# CLI with TUI
cargo run -p yushi-cli --features tui -- tui

# Workspace validation
cargo fmt --check
cargo clippy --workspace --all-targets --all-features
cargo test --workspace --all-features

# Run a single test
cargo test -p yushi-core -- test_name
```

## Architecture Notes

- **Root package = GUI app**: The root `Cargo.toml` defines the `yushi` package which *is* the desktop app. Its code is in `src/` (not a separate `yushi-app/` directory).
- **Shared persistence**: `config.json`, `history.json`, and `queue.json` live under `dirs::config_dir()/yushi/` (`~/.config/yushi/` on Linux/macOS). Atomic file writes (write-to-temp-then-rename) and file locking prevent corruption.
- **Event-driven updates**: `YuShi::with_config()` returns a `Receiver<DownloaderEvent>`. The desktop app polls this channel in its main loop and calls `cx.notify()` to trigger GPUI re-renders. The CLI uses it for progress bar updates.
- **Debounced state saves**: Queue state persistence is debounced (150ms debounce, 750ms interval) to avoid excessive disk I/O during rapid progress updates.
- **Per-task resume state**: Each downloading task saves a `.{filename}.json` sidecar file tracking chunk progress for resumability.
- **History tracking**: Both `yushi-cli` queue processing and the desktop app register `YuShi::set_on_complete` callbacks so successful downloads are appended into the shared `DownloadHistory`.
- **UI root**: The desktop app must keep `gpui_component::Root` as the first view in the window to support dialogs and notifications.

## Key Types & Entry Points

- `AppState` — `src/state.rs`: GUI state entity holding `YuShi` queue, config, history, tasks, and current view.
- `YuShi` — `yushi-core/src/downloader.rs`: Core downloader engine with task management, concurrency control via semaphore, and hot-reloadable config (`apply_runtime_config`).
- `AppConfig` — `yushi-core/src/config.rs`: Shared config with `validate()` and `downloader_config()` conversion.
- `DownloadTask` / `Task` — `yushi-core/src/types.rs`: Task model with status, priority, speed, ETA.
- `DownloaderEvent` — `yushi-core/src/types.rs`: Event enum (`TaskEvent`, `ProgressEvent`, `VerificationEvent`).

## Conventions

- Rust edition 2024, resolver v3. Workspace dependencies are declared in the root `Cargo.toml`.
- Conventional commits (enforced by `git-cliff` config): `feat:`, `fix:`, `refactor:`, `chore:`, etc.
- Dual-licensed: MIT OR Apache-2.0.
- Keep config/history logic in `yushi-core`; do not reintroduce per-frontend copies.
- Tests use inline `#[cfg(test)]` modules with `#[tokio::test]` for async tests. Temp files use nonce suffixes for isolation.
