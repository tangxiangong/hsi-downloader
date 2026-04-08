# TUI-GUI Parity Redesign

**Date:** 2026-04-08
**Goal:** Redesign the yushi-cli TUI to achieve full feature and visual parity with the SolidJS GUI.

---

## 1. Layout

Replace the current top-tab + 60/40 left-right split with a sidebar + full-width content layout:

```
┌──┬──────────────────────────────────────────────┐
│  │  Header: View Title + Stats          [Actions]│
│📥│──────────────────────────────────────────────│
│  │  Filter Tabs (Tasks view only)                │
│📋│  ┌─ 全部(5) ─┬─ 下载中(2) ─┬─ 已完成(3) ─┐  │
│  │  └───────────┴─────────────┴─────────────┘  │
│⚙ │──────────────────────────────────────────────│
│  │  Content Area (scrollable card list)          │
│  │  ┌──────────────────────────────────────┐    │
│  │  │ Task/History Card                    │    │
│  │  └──────────────────────────────────────┘    │
│  │  ┌──────────────────────────────────────┐    │
│  │  │ ...                                  │    │
│  │  └──────────────────────────────────────┘    │
│v0│──────────────────────────────────────────────│
│.1│  Help: context-sensitive keybindings          │
│.0│                                              │
└──┴──────────────────────────────────────────────┘
```

- **Sidebar:** 3 columns wide, icon-based navigation with active highlight, version at bottom.
- **Right detail panel removed.** All info is inline within cards.
- **Help bar:** 1 line at bottom, context-sensitive.

---

## 2. Task Cards

Each task renders as a 3-5 line card:

```
┌──────────────────────────────────────────────────┐
│ 📦 very-long-filename-that-gets-tru…  [⏸] [✕]   │
│ 250.0 MB / 500.0 MB · 5.2 MB/s · 剩余 3分45秒   │
│ ████████████████░░░░░░░░░░░░░░░░░░░░░░░░░ 50.0%  │
└──────────────────────────────────────────────────┘
```

**Status-dependent rendering:**

| Status | Progress bar color | Actions | Extra info |
|--------|-------------------|---------|------------|
| Pending | Gray | [✕] | "等待中" |
| Downloading | Cyan | [⏸] [✕] | speed + ETA |
| Paused | Yellow | [▶] [✕] | "已暂停" |
| Completed | Green | [🗑] [✕] | total size + "已完成" |
| Failed | Red | [🔄] [✕] | error message on extra line |
| Cancelled | DarkGray | [✕] | "已取消" |

**File-type icons:**
- 📦 archives (.zip, .tar, .gz)
- 🎬 video (.mp4, .mkv, .avi)
- 🎵 audio (.mp3, .flac, .wav)
- 🖼 images (.png, .jpg, .gif)
- 📄 documents (.pdf, .doc, .txt)
- 💿 disk images (.iso, .dmg)
- 📁 default/unknown

**Selected card:** Cyan border + background tint.

**Empty state:**
```
        ⬇
    暂无下载任务
  按 a 添加新任务
```

---

## 3. Filter Tabs (Tasks View)

Horizontal tabs below the header, Tasks view only:

```
┌─ 全部(5) ─┬─ 下载中(2) ─┬─ 已完成(3) ─┐
```

- Active tab: white text on cyan background.
- Inactive tabs: DarkGray text.
- `Tab` / `←` / `→` cycles tabs.
- Filters: `All` (all tasks), `Downloading` (Pending + Downloading + Paused), `Completed` (Completed + Failed + Cancelled).

---

## 4. Add Task Dialog

Multi-field centered popup overlay replacing the pipe-delimited input:

```
┌─────────── 添加下载任务 ───────────┐
│                                    │
│  下载链接                          │
│  ┌────────────────────────────┐    │
│  │ https://example.com/file…  │    │
│  └────────────────────────────┘    │
│                                    │
│  保存位置                          │
│  ┌────────────────────────────┐    │
│  │ /Users/xiaoyu/Downloads    │    │
│  └────────────────────────────┘    │
│                                    │
│  优先级                            │
│  [ 低 ] [*正常*] [ 高 ]           │
│                                    │
│  限速 (留空不限)                   │
│  ┌────────────────────────────┐    │
│  │                            │    │
│  └────────────────────────────┘    │
│                                    │
│  [错误信息显示在这里]              │
│                                    │
│       [取消]    [添加]             │
│                                    │
└────────────────────────────────────┘
```

- `Tab` / `Shift+Tab` cycles fields.
- `←` / `→` toggles priority.
- Path pre-filled from config default.
- `Enter` on 添加 submits, `Esc` cancels.
- Background dimmed while dialog is open.
- Validation: URL required.

---

## 5. History View & Cards

**Header:**
```
历史                              [C 清空全部]
3 条记录
```

**History cards (2 lines):**
```
┌──────────────────────────────────────────────────┐
│ ✅ ubuntu-24.04.iso                         [✕]  │
│ 4.2 GB · 平均 12.5 MB/s · 用时 5分36秒 · 03-15  │
└──────────────────────────────────────────────────┘
```

- Green checkmark + filename.
- Total size, average speed, duration, completion date (MM-DD).
- [✕] to remove.

**Empty state:**
```
        📋
      暂无下载记录
```

**Clear all:** `C` opens confirmation dialog.

---

## 6. Settings View

Grouped card layout with 4 sections:

```
┌─────────────── 下载 ───────────────┐
│  默认下载路径    /Users/xiaoyu/Down │
│  单任务连接数    8                  │
│  最大并发任务    3                  │
└────────────────────────────────────┘

┌─────────────── 网络 ───────────────┐
│  代理地址                          │
│  User-Agent      YuShi/0.1.0      │
│  超时时间(秒)    30                │
│  分块大小(MB)    8                 │
│  限速             无               │
└────────────────────────────────────┘

┌─────────────── 外观 ───────────────┐
│  主题    [浅色] [*深色*] [系统]    │
└────────────────────────────────────┘

┌─────────────── 关于 ───────────────┐
│  YuShi v0.1.0                      │
│  驭时 - 异步下载管理器             │
└────────────────────────────────────┘
```

- `↑` / `↓` navigates editable fields across groups. Group headers are skipped.
- `Enter` enters inline edit mode.
- Theme field: `←` / `→` toggles options directly.
- `Esc` cancels, `Enter` saves immediately.

---

## 7. Confirmation Dialog

Generic reusable dialog for destructive actions:

```
┌────── 确认清空 ──────┐
│                      │
│  确定清空所有历史？  │
│                      │
│   [取消]  [确认]     │
│                      │
└──────────────────────┘
```

- `←` / `→` toggles selection.
- `Enter` confirms.
- `Esc` cancels.

---

## 8. Theme & Colors

**Dark theme (primary target):**

| Element | Color |
|---------|-------|
| Sidebar active icon | Cyan bg |
| Card border (selected) | Cyan |
| Card border (unselected) | Gray |
| Progress - downloading | Cyan |
| Progress - completed | Green |
| Progress - paused | Yellow |
| Progress - failed | Red |
| Progress - cancelled | DarkGray |
| Filter tab active | White on Cyan |
| Filter tab inactive | DarkGray |
| Help bar | DarkGray |
| Dialog overlay | Dimmed background |

**Light theme:** Inverted — dark text on light backgrounds, same accent colors.

**System theme:** Detect via `COLORFGBG` env var, default to dark.

Color palette defined in a `ThemeColors` struct for easy switching.

**Scrolling:** Content area scrolls vertically. The selected item is always kept visible (auto-scroll into view). `task_scroll` tracks the viewport offset.

**Delete-with-file (`D`):** Opens a confirmation dialog ("确定删除文件？") before removing the task and deleting the file from disk. This is a destructive action and must not be silent.

---

## 9. Module Architecture

```
yushi-cli/src/tui/
├── mod.rs                — terminal setup/teardown
├── app.rs                — App state, event handling, input dispatch
├── event.rs              — crossterm event loop (unchanged)
├── ui.rs                 — top-level layout orchestration
├── theme.rs              — ThemeColors for dark/light
└── widgets/
    ├── mod.rs            — re-exports
    ├── sidebar.rs        — icon sidebar with active indicator
    ├── task_card.rs      — task card with progress bar + actions
    ├── history_card.rs   — completed download card
    ├── filter_tabs.rs    — horizontal filter tabs
    ├── settings_group.rs — grouped settings with inline edit
    ├── add_task.rs       — multi-field add task popup
    ├── empty_state.rs    — centered icon + message
    ├── dialog.rs         — generic confirmation dialog
    └── help_bar.rs       — context-sensitive keybinding hints
```

---

## 10. State Changes in `app.rs`

```rust
pub struct App {
    // existing
    pub queue: YuShi,
    pub config: AppConfig,
    pub history: DownloadHistory,
    pub tasks: Vec<DownloadTask>,
    pub current_view: CurrentView,
    pub selected_index: usize,
    pub history_index: usize,
    pub setting_index: usize,
    pub input_mode: InputMode,
    pub status_message: String,
    event_rx: mpsc::Receiver<QueueEvent>,

    // new
    pub filter: TaskFilter,
    pub filtered_tasks: Vec<usize>,       // indices into tasks vec
    pub task_scroll: usize,
    pub add_task_state: AddTaskState,
    pub confirm_dialog: Option<ConfirmDialog>,
    pub theme_colors: ThemeColors,

    // removed
    // selected_panel — no left/right split
    // input_buffer — replaced by per-dialog state
}

pub enum TaskFilter { All, Downloading, Completed }
pub enum InputMode { Normal, AddTask, EditSetting, Confirm }

pub struct AddTaskState {
    pub url: String,
    pub path: String,
    pub priority: TaskPriority,
    pub speed_limit: String,
    pub focused_field: usize,  // 0=url, 1=path, 2=priority, 3=speed, 4=buttons
    pub error: Option<String>,
}

pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
    pub selected: bool,        // false=取消, true=确认
    pub on_confirm: ConfirmAction,
}

pub enum ConfirmAction {
    ClearHistory,
    // extensible for future use
}
```

---

## 11. Keyboard Shortcuts

### Global (Normal mode)
| Key | Action |
|-----|--------|
| `1` / `2` / `3` | Switch view |
| `q` / `Ctrl+C` | Quit |
| `r` / `F5` | Refresh |
| `↑` / `k` | Previous item |
| `↓` / `j` | Next item |
| `g` / `Home` | First item |
| `G` / `End` | Last item |

### Tasks view
| Key | Action |
|-----|--------|
| `a` | Open Add Task dialog |
| `p` | Pause/Resume |
| `c` | Cancel task |
| `d` | Remove task |
| `D` | Remove + delete file |
| `Tab` / `←` / `→` | Cycle filter tabs |

### History view
| Key | Action |
|-----|--------|
| `x` | Remove entry |
| `C` | Clear all (confirm dialog) |

### Settings view
| Key | Action |
|-----|--------|
| `Enter` / `e` | Edit field |
| `←` / `→` | Toggle theme (on theme field) |

### Add Task dialog
| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Cycle fields |
| `←` / `→` | Toggle priority |
| `Enter` | Submit / next field |
| `Esc` | Cancel |

### Edit Setting mode
| Key | Action |
|-----|--------|
| `Enter` | Save |
| `Esc` | Cancel |

### Confirm dialog
| Key | Action |
|-----|--------|
| `←` / `→` | Toggle selection |
| `Enter` | Confirm |
| `Esc` | Cancel |
