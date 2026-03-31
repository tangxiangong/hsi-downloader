# 驭时

YuShi (驭时) 是一个用 Rust 构建的下载管理器与下载核心库。当前仓库已经迁移为纯 Rust 工作区：

- `yushi-core` 提供下载引擎、队列、共享配置与下载历史
- `yushi-cli` 提供命令行接口和可选的 `ratatui` TUI
- `yushi-app` 提供基于 `gpui` + `gpui-component` 的桌面 GUI

GUI、CLI、TUI 都直接调用 `yushi-core`，不再经过 Tauri IPC 或 Web 前端。

## 工作区结构

```text
Cargo.toml
├── yushi-core/
├── yushi-cli/
└── yushi-app/
```

## 核心能力

- 并发分块下载和流式下载
- 断点续传与队列状态持久化
- 任务控制：添加、暂停、恢复、取消、删除
- 文件校验：MD5 / SHA256
- 共享配置：下载路径、并发数、分块大小、超时、User-Agent、主题
- 共享下载历史：完成记录、搜索、删除、清空

## 运行与构建

```bash
# GUI
cargo run -p yushi-app

# CLI
cargo run -p yushi-cli -- --help

# CLI + TUI
cargo run -p yushi-cli --features tui -- tui

# 检查 / 测试
cargo fmt --check
cargo clippy --workspace --all-targets --all-features
cargo test --workspace --all-features
```

## 共享数据位置

配置、历史和队列文件统一存储在 `dirs::config_dir()/yushi/` 下：

- `config.json`
- `history.json`
- `queue.json`

首次加载时会尝试兼容旧的 CLI 配置格式，并从旧的 Tauri 应用数据目录导入 `history.json` / `queue.json` / `config.json`（如果共享目录还不存在对应文件）。

## 桌面应用

`yushi-app` 使用 `gpui-component::Root` 作为窗口根视图，并通过：

- `Root::render_dialog_layer`
- `Root::render_notification_layer`

渲染对话框和通知层。应用状态由一个共享 `Entity<AppState>` 驱动，后台队列事件会刷新任务快照，设置与历史记录通过 `yushi-core` 直接持久化。
