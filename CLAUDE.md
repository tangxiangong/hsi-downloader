# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

YuShi (驭时) is an async download manager built as a pure Rust workspace. The old Tauri + React frontend has been removed and replaced by a `gpui` desktop application in `yushi-app`.

## Workspace Structure

- **yushi-core** — Shared download library and data model. Owns the downloader engine (`YuShi`), queue/task types, checksum utils, shared `AppConfig`, shared `DownloadHistory`, and shared storage path helpers.
- **yushi-cli** — CLI binary (`yushi`). Uses `clap` for args and `indicatif` for progress bars. Optional `tui` feature enables a `ratatui` terminal UI backed by the shared core config/history APIs.
- **yushi-app** — Desktop GUI built with `gpui` + `gpui-component`. Uses a shared `Entity<AppState)` model and calls `yushi-core` directly. No IPC layer.

## Build & Dev Commands

```bash
# Desktop GUI
cargo run -p yushi-app

# CLI
cargo run -p yushi-cli -- --help

# CLI with TUI
cargo run -p yushi-cli --features tui -- tui

# Workspace validation
cargo fmt --check
cargo clippy --workspace --all-targets --all-features
cargo test --workspace --all-features
```

## Architecture Notes

- **Shared persistence**: `config.json`, `history.json`, and `queue.json` live under the shared `dirs::config_dir()/yushi/` directory.
- **Compatibility loading**: `yushi-core::AppConfig` accepts the previous CLI config format and old Tauri config files that still contain the removed `window` field.
- **Queue events**: `yushi-core` emits `DownloaderEvent` through a `tokio::sync::mpsc` channel. `yushi-app` refreshes task snapshots from these events and `yushi-cli` uses them for progress display.
- **History tracking**: both `yushi-cli` queue processing and `yushi-app` register `YuShi::set_on_complete` callbacks so successful downloads are appended into the shared `DownloadHistory`.
- **UI root**: `yushi-app` must keep `gpui_component::Root` as the first view in the window to support dialogs and notifications.

## Conventions

- Rust edition 2024, resolver v3. Workspace dependencies are declared in the root `Cargo.toml`.
- Conventional commits (enforced by `git-cliff` config): `feat:`, `fix:`, `refactor:`, `chore:`, etc.
- Dual-licensed: MIT OR Apache-2.0.
- Keep config/history logic in `yushi-core`; do not reintroduce per-frontend copies.
