# 驭时

YuShi (驭时) 是一个用 Rust 构建的下载管理器与下载核心库，采用纯 Rust 工作区：

- **根包 (`yushi`)** — 基于 `gpui` + `gpui-component` 的桌面 GUI，源码位于 `src/`
- **`yushi-core`** — 下载引擎、队列、共享配置与下载历史
- **`yushi-cli`** — 命令行接口，默认启用 `ratatui` TUI

GUI、CLI、TUI 都直接调用 `yushi-core`，无 IPC 层。

## 核心能力

- 并发分块下载和流式下载
- 断点续传（按 chunk 记录进度）与队列状态持久化
- 任务控制：添加、暂停、恢复、取消、删除
- 文件校验：MD5 / SHA256
- 代理支持：HTTP / HTTPS / SOCKS5
- 共享配置：下载路径、并发数、分块大小、超时、User-Agent、主题
- 共享下载历史：完成记录、搜索、删除、清空

## 运行与构建

```bash
# 桌面 GUI（根包即桌面应用）
cargo run

# CLI
cargo run -p yushi-cli -- --help

# CLI + TUI（tui 为默认 feature）
cargo run -p yushi-cli -- tui

# 检查 / 测试
cargo fmt --check
cargo clippy --workspace --all-targets --all-features
cargo test --workspace --all-features
```

## 共享数据位置

配置、历史和队列文件统一存储在 `dirs::config_dir()/yushi/`（macOS/Linux 下为 `~/.config/yushi/`）：

| 文件 | 用途 |
|------|------|
| `config.json` | 应用配置（下载路径、并发数、代理等） |
| `history.json` | 已完成下载记录 |
| `queue.json` | 队列任务状态（支持断点续传） |


## 桌面应用

应用状态由一个共享 `Entity<AppState>` 驱动。`yushi-core` 通过 `tokio::sync::mpsc` 广播 `DownloaderEvent`，GUI 主循环轮询事件并触发 GPUI 响应式重渲染。

窗口根视图为 `gpui_component::Root`，通过 `render_dialog_layer` 和 `render_notification_layer` 渲染对话框与通知层。

## 许可证

MIT OR Apache-2.0
