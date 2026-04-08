# 驭时 (YuShi)

YuShi (驭时) 是一个用 Rust 构建的异步下载管理器，提供桌面 GUI 和命令行两种界面。

## 工作区结构

| 模块            | 说明                                                                                              |
| --------------- | ------------------------------------------------------------------------------------------------- |
| **src-tauri/**  | Tauri v2 桌面应用，通过 `#[tauri::command]` 暴露核心 API，并经 Tauri 事件系统将实时进度推送到前端 |
| **src-ui/**     | SolidJS + TailwindCSS + DaisyUI 前端，Vite 构建，Bun 包管理                                       |
| **yushi-core/** | 共享下载引擎、队列管理、配置、历史记录与存储路径                                                  |
| **yushi-cli/**  | 命令行接口（`clap`），默认启用 `ratatui` TUI                                                      |

## 核心能力

- 并发分块下载和流式下载
- 断点续传（按 chunk 记录进度）与队列状态持久化
- 任务控制：添加、暂停、恢复、取消、删除
- 文件校验：MD5 / SHA256
- 代理支持：HTTP / HTTPS / SOCKS5
- 共享配置：下载路径、并发数、分块大小、超时、User-Agent、主题
- 共享下载历史：完成记录、搜索、删除、清空
- 系统托盘：关闭窗口时最小化到托盘

## 运行与构建

```bash
# 桌面 GUI（Tauri 开发模式，同时启动前后端）
cargo tauri dev

# 生产构建
cargo tauri build

# 仅前端开发
cd src-ui && bun run dev

# CLI
cargo run -p yushi-cli -- --help

# CLI + TUI（tui 为默认 feature）
cargo run -p yushi-cli -- tui

# 检查 / 测试
cargo fmt --check
cargo clippy --workspace --all-targets --all-features
cargo test --workspace --all-features
```

## 架构概要

```
┌─────────────┐   Tauri Events    ┌──────────────┐
│  SolidJS    │ ◄──────────────── │  src-tauri   │
│  前端       │ ──────────────► │  后端        │
└─────────────┘   invoke()        └──────┬───────┘
                                         │
                                         ▼
                                  ┌──────────────┐
                                  │  yushi-core  │
                                  │  下载引擎     │
                                  └──────────────┘
```

- **前端 → 后端**：SolidJS 通过 `@tauri-apps/api` 的 `invoke()` 调用 `#[tauri::command]` 处理函数
- **后端 → 前端**：`yushi-core` 产生 `DownloaderEvent`，Tauri 后端通过 `app_handle.emit()` 推送到前端
- **状态管理**：前端使用响应式 Store（`task-store`、`history-store`、`config-store`、`theme-store`）监听事件并同步
- **主题**：DaisyUI 主题通过根元素 `data-theme` 属性切换，支持 亮色 / 暗色 / 跟随系统

## 共享数据位置

配置、历史和队列文件统一存储在 `dirs::config_dir()/yushi/`（macOS/Linux 下为 `~/.config/yushi/`）：

| 文件           | 用途                                 |
| -------------- | ------------------------------------ |
| `config.json`  | 应用配置（下载路径、并发数、代理等） |
| `history.json` | 已完成下载记录                       |
| `queue.json`   | 队列任务状态（支持断点续传）         |

## 许可证

MIT OR Apache-2.0
