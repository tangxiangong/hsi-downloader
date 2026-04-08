# TUI-GUI Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign the yushi-cli TUI to achieve full feature and visual parity with the SolidJS GUI, using a component-based modular architecture.

**Architecture:** Replace the monolithic 60/40 split layout with a sidebar + full-width card list. Extract each visual element into its own widget module under `widgets/`. State management gains filter, add-task dialog, and confirmation dialog support.

**Tech Stack:** Rust, ratatui 0.30, crossterm 0.29, yushi-core types

**Spec:** `docs/superpowers/specs/2026-04-08-tui-gui-parity-design.md`

---

## File Structure

```
yushi-cli/src/tui/
├── mod.rs              — terminal setup (minor changes to import new modules)
├── app.rs              — rewritten: new state fields, input modes, filter logic
├── event.rs            — unchanged
├── ui.rs               — rewritten: sidebar + header + content + help layout
├── theme.rs            — new: ThemeColors struct for dark/light palettes
└── widgets/
    ├── mod.rs          — new: re-exports all widgets
    ├── sidebar.rs      — new: 3-col icon sidebar
    ├── task_card.rs    — new: rich task card with progress bar
    ├── history_card.rs — new: completed download card
    ├── filter_tabs.rs  — new: horizontal filter tabs
    ├── settings_group.rs — new: grouped settings cards
    ├── add_task.rs     — new: multi-field popup dialog
    ├── empty_state.rs  — new: centered icon + message
    ├── dialog.rs       — new: generic confirmation dialog
    └── help_bar.rs     — new: context-sensitive help line
```

---

### Task 1: Create `theme.rs` — Color Palette

**Files:**
- Create: `yushi-cli/src/tui/theme.rs`

This is a foundational module that all widgets depend on.

- [ ] **Step 1: Create `theme.rs` with `ThemeColors` struct**

```rust
// yushi-cli/src/tui/theme.rs
use ratatui::style::Color;
use yushi_core::AppTheme;

#[derive(Debug, Clone)]
pub struct ThemeColors {
    /// Primary accent color (sidebar active, selected borders, downloading progress)
    pub primary: Color,
    /// Success color (completed tasks, history checkmarks)
    pub success: Color,
    /// Warning color (paused tasks)
    pub warning: Color,
    /// Error color (failed tasks)
    pub error: Color,
    /// Muted/disabled color (cancelled tasks, inactive elements)
    pub muted: Color,
    /// Card border (unselected)
    pub border: Color,
    /// Card border (selected)
    pub border_active: Color,
    /// Background for selected items
    pub selection_bg: Color,
    /// Primary text
    pub text: Color,
    /// Secondary/dimmed text
    pub text_secondary: Color,
    /// Help bar text
    pub text_help: Color,
    /// Background
    pub bg: Color,
    /// Dialog overlay dimmed background
    pub overlay_bg: Color,
}

impl ThemeColors {
    pub fn dark() -> Self {
        Self {
            primary: Color::Cyan,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            muted: Color::DarkGray,
            border: Color::Gray,
            border_active: Color::Cyan,
            selection_bg: Color::DarkGray,
            text: Color::White,
            text_secondary: Color::Gray,
            text_help: Color::DarkGray,
            bg: Color::Reset,
            overlay_bg: Color::DarkGray,
        }
    }

    pub fn light() -> Self {
        Self {
            primary: Color::Blue,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            muted: Color::Gray,
            border: Color::Gray,
            border_active: Color::Blue,
            selection_bg: Color::Indexed(254), // light gray
            text: Color::Black,
            text_secondary: Color::DarkGray,
            text_help: Color::Gray,
            bg: Color::Reset,
            overlay_bg: Color::Indexed(250),
        }
    }

    pub fn from_app_theme(theme: AppTheme) -> Self {
        match theme {
            AppTheme::Dark => Self::dark(),
            AppTheme::Light => Self::light(),
            AppTheme::System => {
                // Detect from COLORFGBG env var: "15;0" means light-on-dark (dark theme)
                if let Ok(val) = std::env::var("COLORFGBG") {
                    if let Some(bg) = val.split(';').last() {
                        if let Ok(bg_num) = bg.parse::<u8>() {
                            if bg_num > 8 {
                                return Self::light();
                            }
                        }
                    }
                }
                Self::dark()
            }
        }
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p yushi-cli --features tui`

This won't compile yet since nothing imports it. Just save the file — it will be wired in Task 3.

- [ ] **Step 3: Commit**

```bash
git add yushi-cli/src/tui/theme.rs
git commit -m "feat(tui): add ThemeColors palette for dark/light themes"
```

---

### Task 2: Create `widgets/` Module Skeleton

**Files:**
- Create: `yushi-cli/src/tui/widgets/mod.rs`
- Create: `yushi-cli/src/tui/widgets/sidebar.rs`
- Create: `yushi-cli/src/tui/widgets/task_card.rs`
- Create: `yushi-cli/src/tui/widgets/history_card.rs`
- Create: `yushi-cli/src/tui/widgets/filter_tabs.rs`
- Create: `yushi-cli/src/tui/widgets/settings_group.rs`
- Create: `yushi-cli/src/tui/widgets/add_task.rs`
- Create: `yushi-cli/src/tui/widgets/empty_state.rs`
- Create: `yushi-cli/src/tui/widgets/dialog.rs`
- Create: `yushi-cli/src/tui/widgets/help_bar.rs`

Create all widget files with minimal placeholder public functions. Each will be filled in subsequent tasks.

- [ ] **Step 1: Create `widgets/mod.rs`**

```rust
// yushi-cli/src/tui/widgets/mod.rs
pub mod add_task;
pub mod dialog;
pub mod empty_state;
pub mod filter_tabs;
pub mod help_bar;
pub mod history_card;
pub mod settings_group;
pub mod sidebar;
pub mod task_card;
```

- [ ] **Step 2: Create each widget file with a stub `draw` function**

Each file gets a minimal stub so the project compiles. Example for all files:

```rust
// yushi-cli/src/tui/widgets/sidebar.rs
use ratatui::{Frame, layout::Rect};
use crate::tui::app::App;
use crate::tui::theme::ThemeColors;

pub fn draw(_f: &mut Frame, _app: &App, _theme: &ThemeColors, _area: Rect) {}
```

```rust
// yushi-cli/src/tui/widgets/task_card.rs
use ratatui::{Frame, layout::Rect};
use crate::tui::app::App;
use crate::tui::theme::ThemeColors;

pub fn draw(_f: &mut Frame, _app: &App, _theme: &ThemeColors, _area: Rect) {}
```

```rust
// yushi-cli/src/tui/widgets/history_card.rs
use ratatui::{Frame, layout::Rect};
use crate::tui::app::App;
use crate::tui::theme::ThemeColors;

pub fn draw(_f: &mut Frame, _app: &App, _theme: &ThemeColors, _area: Rect) {}
```

```rust
// yushi-cli/src/tui/widgets/filter_tabs.rs
use ratatui::{Frame, layout::Rect};
use crate::tui::app::App;
use crate::tui::theme::ThemeColors;

pub fn draw(_f: &mut Frame, _app: &App, _theme: &ThemeColors, _area: Rect) {}
```

```rust
// yushi-cli/src/tui/widgets/settings_group.rs
use ratatui::{Frame, layout::Rect};
use crate::tui::app::App;
use crate::tui::theme::ThemeColors;

pub fn draw(_f: &mut Frame, _app: &App, _theme: &ThemeColors, _area: Rect) {}
```

```rust
// yushi-cli/src/tui/widgets/add_task.rs
use ratatui::{Frame, layout::Rect};
use crate::tui::app::App;
use crate::tui::theme::ThemeColors;

pub fn draw(_f: &mut Frame, _app: &App, _theme: &ThemeColors, _area: Rect) {}
```

```rust
// yushi-cli/src/tui/widgets/empty_state.rs
use ratatui::{Frame, layout::Rect};
use crate::tui::theme::ThemeColors;

pub fn draw(_f: &mut Frame, _icon: &str, _message: &str, _hint: &str, _theme: &ThemeColors, _area: Rect) {}
```

```rust
// yushi-cli/src/tui/widgets/dialog.rs
use ratatui::{Frame, layout::Rect};
use crate::tui::app::ConfirmDialog;
use crate::tui::theme::ThemeColors;

pub fn draw(_f: &mut Frame, _dialog: &ConfirmDialog, _theme: &ThemeColors, _area: Rect) {}
```

```rust
// yushi-cli/src/tui/widgets/help_bar.rs
use ratatui::{Frame, layout::Rect};
use crate::tui::app::App;
use crate::tui::theme::ThemeColors;

pub fn draw(_f: &mut Frame, _app: &App, _theme: &ThemeColors, _area: Rect) {}
```

- [ ] **Step 3: Commit**

```bash
git add yushi-cli/src/tui/widgets/
git commit -m "feat(tui): add widget module skeleton with stubs"
```

---

### Task 3: Rewrite `app.rs` — New State Model

**Files:**
- Modify: `yushi-cli/src/tui/app.rs`
- Modify: `yushi-cli/src/tui/mod.rs`

Replace the old state model with new fields for filters, add-task dialog, confirmation dialog, and theme. Keep all existing business logic (add_task_from_input, apply_setting_value, on_tick, etc.) but restructure input handling.

- [ ] **Step 1: Rewrite `app.rs` with new enums, structs, and state**

Replace the entire file. Key changes:
- Remove `InputMode::AddUrl` → `InputMode::AddTask` (multi-field)
- Remove `InputMode::EditSetting` stays but `input_buffer` moves to inline
- Add `InputMode::Confirm`
- Remove `SelectedPanel` (no more left/right split)
- Add `TaskFilter`, `AddTaskState`, `ConfirmDialog`, `ConfirmAction`
- Add `theme_colors: ThemeColors` field
- Add `filter: TaskFilter` and `filtered_indices: Vec<usize>` fields
- Add `task_scroll: usize` for viewport offset
- Keep `SettingField` and its `label()`/`current_value()`/`apply()` methods unchanged

```rust
// yushi-cli/src/tui/app.rs
use anyhow::{Result, anyhow};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;
use yushi_core::{
    AppConfig, AppTheme, DownloadHistory, DownloadTask, DownloaderEvent, Priority,
    ProgressEvent, QueueEvent, TaskEvent, TaskStatus, YuShi, parse_speed_limit,
};

use crate::config::ConfigStore;
use crate::tui::theme::ThemeColors;

// ── Enums ──────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    AddTask,
    EditSetting,
    Confirm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentView {
    Tasks,
    History,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskFilter {
    All,
    Downloading,
    Completed,
}

impl TaskFilter {
    pub fn label(self) -> &'static str {
        match self {
            Self::All => "全部",
            Self::Downloading => "下载中",
            Self::Completed => "已完成",
        }
    }

    pub fn matches(self, status: TaskStatus) -> bool {
        match self {
            Self::All => true,
            Self::Downloading => matches!(
                status,
                TaskStatus::Pending | TaskStatus::Downloading | TaskStatus::Paused
            ),
            Self::Completed => matches!(
                status,
                TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
            ),
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::All => Self::Downloading,
            Self::Downloading => Self::Completed,
            Self::Completed => Self::All,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::All => Self::Completed,
            Self::Downloading => Self::All,
            Self::Completed => Self::Downloading,
        }
    }
}

// ── Add Task Dialog State ──────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddTaskField {
    Url,
    Path,
    Priority,
    SpeedLimit,
    Buttons,
}

const ADD_TASK_FIELDS: [AddTaskField; 5] = [
    AddTaskField::Url,
    AddTaskField::Path,
    AddTaskField::Priority,
    AddTaskField::SpeedLimit,
    AddTaskField::Buttons,
];

#[derive(Debug, Clone)]
pub struct AddTaskState {
    pub url: String,
    pub path: String,
    pub priority: Priority,
    pub speed_limit: String,
    pub focused_field: usize,
    pub error: Option<String>,
    pub button_confirm: bool, // false = 取消, true = 添加
}

impl AddTaskState {
    pub fn new(default_path: &str) -> Self {
        Self {
            url: String::new(),
            path: default_path.to_string(),
            priority: Priority::Normal,
            speed_limit: String::new(),
            focused_field: 0,
            error: None,
            button_confirm: true,
        }
    }

    pub fn focused(&self) -> AddTaskField {
        ADD_TASK_FIELDS[self.focused_field]
    }

    pub fn focus_next(&mut self) {
        self.focused_field = (self.focused_field + 1) % ADD_TASK_FIELDS.len();
    }

    pub fn focus_prev(&mut self) {
        self.focused_field = if self.focused_field == 0 {
            ADD_TASK_FIELDS.len() - 1
        } else {
            self.focused_field - 1
        };
    }
}

// ── Confirm Dialog ─────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
    pub selected_confirm: bool, // false = 取消, true = 确认
    pub action: ConfirmAction,
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    ClearHistory,
    DeleteTaskWithFile { task_id: String },
}

// ── Settings ───────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingField {
    OutputDir,
    Connections,
    MaxTasks,
    ChunkSize,
    Timeout,
    UserAgent,
    Proxy,
    SpeedLimit,
    Theme,
}

/// Settings organized by group for rendering
pub struct SettingsGroup {
    pub title: &'static str,
    pub fields: &'static [SettingField],
}

pub const SETTINGS_GROUPS: &[SettingsGroup] = &[
    SettingsGroup {
        title: "下载",
        fields: &[
            SettingField::OutputDir,
            SettingField::Connections,
            SettingField::MaxTasks,
        ],
    },
    SettingsGroup {
        title: "网络",
        fields: &[
            SettingField::Proxy,
            SettingField::UserAgent,
            SettingField::Timeout,
            SettingField::ChunkSize,
            SettingField::SpeedLimit,
        ],
    },
    SettingsGroup {
        title: "外观",
        fields: &[SettingField::Theme],
    },
];

pub const SETTINGS_FIELDS: [SettingField; 9] = [
    SettingField::OutputDir,
    SettingField::Connections,
    SettingField::MaxTasks,
    SettingField::Proxy,
    SettingField::UserAgent,
    SettingField::Timeout,
    SettingField::ChunkSize,
    SettingField::SpeedLimit,
    SettingField::Theme,
];

// ── App ────────────────────────────────────────────────

pub struct App {
    // Core data
    pub queue: YuShi,
    pub config: AppConfig,
    pub history: DownloadHistory,
    pub tasks: Vec<DownloadTask>,
    event_rx: mpsc::Receiver<QueueEvent>,

    // View state
    pub current_view: CurrentView,
    pub input_mode: InputMode,
    pub status_message: String,

    // Tasks view
    pub selected_index: usize,
    pub filter: TaskFilter,
    pub filtered_indices: Vec<usize>,
    pub task_scroll: usize,

    // History view
    pub history_index: usize,

    // Settings view
    pub setting_index: usize,
    pub edit_buffer: String,

    // Dialogs
    pub add_task_state: Option<AddTaskState>,
    pub confirm_dialog: Option<ConfirmDialog>,

    // Theme
    pub theme: ThemeColors,
}

impl App {
    pub async fn new() -> Result<Self> {
        let config = ConfigStore::load().await?;
        let history_path = ConfigStore::history_path()?;
        let history = DownloadHistory::load(&history_path).await?;
        let (queue, event_rx) = ConfigStore::build_queue(&config, None, None).await?;
        queue.load_queue_from_state().await?;
        let tasks = queue.get_all_tasks().await;
        let theme = ThemeColors::from_app_theme(config.theme);

        let mut app = Self {
            queue,
            config,
            history,
            tasks,
            event_rx,

            current_view: CurrentView::Tasks,
            input_mode: InputMode::Normal,
            status_message: "就绪".to_string(),

            selected_index: 0,
            filter: TaskFilter::All,
            filtered_indices: Vec::new(),
            task_scroll: 0,

            history_index: 0,

            setting_index: 0,
            edit_buffer: String::new(),

            add_task_state: None,
            confirm_dialog: None,

            theme,
        };
        app.recompute_filtered();
        Ok(app)
    }

    // ── Filter logic ───────────────────────────────

    pub fn recompute_filtered(&mut self) {
        self.filtered_indices = self
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, t)| self.filter.matches(t.status))
            .map(|(i, _)| i)
            .collect();
        // Clamp selection
        if !self.filtered_indices.is_empty() {
            if self.selected_index >= self.filtered_indices.len() {
                self.selected_index = self.filtered_indices.len() - 1;
            }
        } else {
            self.selected_index = 0;
        }
    }

    pub fn selected_task(&self) -> Option<&DownloadTask> {
        self.filtered_indices
            .get(self.selected_index)
            .and_then(|&i| self.tasks.get(i))
    }

    pub fn filter_count(&self, filter: TaskFilter) -> usize {
        self.tasks.iter().filter(|t| filter.matches(t.status)).count()
    }

    // ── Persistence ────────────────────────────────

    pub async fn persist_on_exit(&self) -> Result<()> {
        self.queue.persist_queue_state().await?;
        Ok(())
    }

    // ── Key handling dispatch ──────────────────────

    pub async fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key).await,
            InputMode::AddTask => {
                self.handle_add_task_key(key).await?;
                Ok(true)
            }
            InputMode::EditSetting => {
                self.handle_edit_setting_key(key).await?;
                Ok(true)
            }
            InputMode::Confirm => {
                self.handle_confirm_key(key).await?;
                Ok(true)
            }
        }
    }

    async fn handle_normal_key(&mut self, key: KeyEvent) -> Result<bool> {
        match (key.code, key.modifiers) {
            // Quit
            (KeyCode::Char('q'), KeyModifiers::NONE)
            | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                return Ok(false);
            }
            // View switching
            (KeyCode::Char('1'), KeyModifiers::NONE) => {
                self.current_view = CurrentView::Tasks;
                self.status_message = "任务视图".to_string();
            }
            (KeyCode::Char('2'), KeyModifiers::NONE) => {
                self.current_view = CurrentView::History;
                self.status_message = "历史视图".to_string();
            }
            (KeyCode::Char('3'), KeyModifiers::NONE) => {
                self.current_view = CurrentView::Settings;
                self.status_message = "设置视图".to_string();
            }
            // Delegate to view-specific handler
            _ => match self.current_view {
                CurrentView::Tasks => self.handle_tasks_key(key).await?,
                CurrentView::History => self.handle_history_key(key).await?,
                CurrentView::Settings => self.handle_settings_key(key).await?,
            },
        }
        Ok(true)
    }

    // ── Tasks view keys ────────────────────────────

    async fn handle_tasks_key(&mut self, key: KeyEvent) -> Result<()> {
        match (key.code, key.modifiers) {
            // Navigation
            (KeyCode::Up | KeyCode::Char('k'), KeyModifiers::NONE) => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            (KeyCode::Down | KeyCode::Char('j'), KeyModifiers::NONE) => {
                if self.selected_index < self.filtered_indices.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
            }
            (KeyCode::Home | KeyCode::Char('g'), KeyModifiers::NONE) => {
                self.selected_index = 0;
            }
            (KeyCode::End, KeyModifiers::NONE) => {
                self.selected_index = self.filtered_indices.len().saturating_sub(1);
            }
            (KeyCode::Char('G'), KeyModifiers::SHIFT) => {
                self.selected_index = self.filtered_indices.len().saturating_sub(1);
            }
            // Filter tabs
            (KeyCode::Tab, KeyModifiers::NONE) | (KeyCode::Right, KeyModifiers::NONE) => {
                self.filter = self.filter.next();
                self.recompute_filtered();
            }
            (KeyCode::BackTab, KeyModifiers::SHIFT) | (KeyCode::Left, KeyModifiers::NONE) => {
                self.filter = self.filter.prev();
                self.recompute_filtered();
            }
            // Add task
            (KeyCode::Char('a'), KeyModifiers::NONE) => {
                let default_path = self.config.default_download_path.display().to_string();
                self.add_task_state = Some(AddTaskState::new(&default_path));
                self.input_mode = InputMode::AddTask;
            }
            // Pause/Resume
            (KeyCode::Char('p'), KeyModifiers::NONE) => {
                if let Some(task) = self.selected_task().cloned() {
                    match task.status {
                        TaskStatus::Downloading => {
                            self.queue.pause_task(&task.id).await?;
                            self.status_message = format!("已暂停: {}", &task.id[..8]);
                        }
                        TaskStatus::Paused => {
                            self.queue.resume_task(&task.id).await?;
                            self.status_message = format!("已恢复: {}", &task.id[..8]);
                        }
                        _ => {}
                    }
                }
            }
            // Cancel
            (KeyCode::Char('c'), KeyModifiers::NONE) => {
                if let Some(task) = self.selected_task().cloned() {
                    if matches!(
                        task.status,
                        TaskStatus::Pending | TaskStatus::Downloading | TaskStatus::Paused
                    ) {
                        self.queue.cancel_task(&task.id).await?;
                        self.status_message = format!("已取消: {}", &task.id[..8]);
                    }
                }
            }
            // Remove task
            (KeyCode::Char('d'), KeyModifiers::NONE) => {
                if let Some(task) = self.selected_task().cloned() {
                    if matches!(
                        task.status,
                        TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
                    ) {
                        self.queue.remove_task(&task.id).await?;
                        self.status_message = format!("已删除: {}", &task.id[..8]);
                        self.refresh_tasks().await?;
                    }
                }
            }
            // Remove task + delete file (with confirmation)
            (KeyCode::Char('D'), KeyModifiers::SHIFT) => {
                if let Some(task) = self.selected_task().cloned() {
                    if matches!(
                        task.status,
                        TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
                    ) {
                        self.confirm_dialog = Some(ConfirmDialog {
                            title: "确认删除".to_string(),
                            message: "确定删除任务并删除文件？".to_string(),
                            selected_confirm: false,
                            action: ConfirmAction::DeleteTaskWithFile {
                                task_id: task.id.clone(),
                            },
                        });
                        self.input_mode = InputMode::Confirm;
                    }
                }
            }
            // Refresh
            (KeyCode::Char('r'), KeyModifiers::NONE) | (KeyCode::F(5), KeyModifiers::NONE) => {
                self.refresh_tasks().await?;
                self.status_message = "已刷新".to_string();
            }
            _ => {}
        }
        Ok(())
    }

    // ── History view keys ──────────────────────────

    async fn handle_history_key(&mut self, key: KeyEvent) -> Result<()> {
        match (key.code, key.modifiers) {
            (KeyCode::Up | KeyCode::Char('k'), KeyModifiers::NONE) => {
                if self.history_index > 0 {
                    self.history_index -= 1;
                }
            }
            (KeyCode::Down | KeyCode::Char('j'), KeyModifiers::NONE) => {
                if self.history_index < self.history.completed_tasks.len().saturating_sub(1) {
                    self.history_index += 1;
                }
            }
            (KeyCode::Char('r'), KeyModifiers::NONE) | (KeyCode::F(5), KeyModifiers::NONE) => {
                self.refresh_history().await?;
                self.status_message = "历史已刷新".to_string();
            }
            (KeyCode::Char('x'), KeyModifiers::NONE) => {
                if let Some(task) = self.history.completed_tasks.get(self.history_index).cloned() {
                    let path = ConfigStore::history_path()?;
                    let (history, removed) =
                        DownloadHistory::remove_from_file(&path, &task.id).await?;
                    if removed {
                        self.history = history;
                        if self.history_index > 0
                            && self.history_index >= self.history.completed_tasks.len()
                        {
                            self.history_index -= 1;
                        }
                        self.status_message = format!("已删除记录: {}", &task.id[..8]);
                    }
                }
            }
            // Clear all history (with confirmation)
            (KeyCode::Char('C'), KeyModifiers::SHIFT) => {
                if !self.history.completed_tasks.is_empty() {
                    self.confirm_dialog = Some(ConfirmDialog {
                        title: "确认清空".to_string(),
                        message: "确定清空所有历史记录？".to_string(),
                        selected_confirm: false,
                        action: ConfirmAction::ClearHistory,
                    });
                    self.input_mode = InputMode::Confirm;
                }
            }
            _ => {}
        }
        Ok(())
    }

    // ── Settings view keys ─────────────────────────

    async fn handle_settings_key(&mut self, key: KeyEvent) -> Result<()> {
        match (key.code, key.modifiers) {
            (KeyCode::Up | KeyCode::Char('k'), KeyModifiers::NONE) => {
                if self.setting_index > 0 {
                    self.setting_index -= 1;
                }
            }
            (KeyCode::Down | KeyCode::Char('j'), KeyModifiers::NONE) => {
                if self.setting_index < SETTINGS_FIELDS.len().saturating_sub(1) {
                    self.setting_index += 1;
                }
            }
            (KeyCode::Enter, KeyModifiers::NONE) | (KeyCode::Char('e'), KeyModifiers::NONE) => {
                let field = self.selected_setting();
                if field == SettingField::Theme {
                    // Theme uses left/right toggle, not text edit
                    self.cycle_theme().await?;
                } else {
                    self.input_mode = InputMode::EditSetting;
                    self.edit_buffer = field
                        .current_value(&self.config)
                        .unwrap_or_default();
                    self.status_message = format!(
                        "编辑: {} — Enter 保存, Esc 取消",
                        field.label()
                    );
                }
            }
            (KeyCode::Left, KeyModifiers::NONE) | (KeyCode::Right, KeyModifiers::NONE) => {
                if self.selected_setting() == SettingField::Theme {
                    self.cycle_theme().await?;
                }
            }
            (KeyCode::Char('r'), KeyModifiers::NONE) | (KeyCode::F(5), KeyModifiers::NONE) => {
                self.config = ConfigStore::load().await?;
                self.theme = ThemeColors::from_app_theme(self.config.theme);
                self.status_message = "配置已重载".to_string();
            }
            _ => {}
        }
        Ok(())
    }

    async fn cycle_theme(&mut self) -> Result<()> {
        self.config.theme = match self.config.theme {
            AppTheme::Light => AppTheme::Dark,
            AppTheme::Dark => AppTheme::System,
            AppTheme::System => AppTheme::Light,
        };
        self.theme = ThemeColors::from_app_theme(self.config.theme);
        ConfigStore::save(&self.config).await?;
        self.status_message = format!("主题: {}", self.config.theme);
        Ok(())
    }

    // ── Add Task dialog keys ───────────────────────

    async fn handle_add_task_key(&mut self, key: KeyEvent) -> Result<()> {
        let Some(state) = &mut self.add_task_state else {
            self.input_mode = InputMode::Normal;
            return Ok(());
        };

        match key.code {
            KeyCode::Esc => {
                self.add_task_state = None;
                self.input_mode = InputMode::Normal;
                self.status_message = "已取消".to_string();
            }
            KeyCode::Tab => {
                state.focus_next();
            }
            KeyCode::BackTab => {
                state.focus_prev();
            }
            KeyCode::Enter => {
                match state.focused() {
                    AddTaskField::Buttons => {
                        if state.button_confirm {
                            // Submit
                            self.submit_add_task().await?;
                        } else {
                            // Cancel
                            self.add_task_state = None;
                            self.input_mode = InputMode::Normal;
                            self.status_message = "已取消".to_string();
                        }
                    }
                    _ => {
                        // Enter advances to next field
                        state.focus_next();
                    }
                }
            }
            KeyCode::Left => {
                match state.focused() {
                    AddTaskField::Priority => {
                        state.priority = match state.priority {
                            Priority::Normal => Priority::Low,
                            Priority::High => Priority::Normal,
                            Priority::Low => Priority::High,
                        };
                    }
                    AddTaskField::Buttons => {
                        state.button_confirm = !state.button_confirm;
                    }
                    _ => {}
                }
            }
            KeyCode::Right => {
                match state.focused() {
                    AddTaskField::Priority => {
                        state.priority = match state.priority {
                            Priority::Low => Priority::Normal,
                            Priority::Normal => Priority::High,
                            Priority::High => Priority::Low,
                        };
                    }
                    AddTaskField::Buttons => {
                        state.button_confirm = !state.button_confirm;
                    }
                    _ => {}
                }
            }
            KeyCode::Char(c) => {
                match state.focused() {
                    AddTaskField::Url => state.url.push(c),
                    AddTaskField::Path => state.path.push(c),
                    AddTaskField::SpeedLimit => state.speed_limit.push(c),
                    _ => {}
                }
            }
            KeyCode::Backspace => {
                match state.focused() {
                    AddTaskField::Url => { state.url.pop(); }
                    AddTaskField::Path => { state.path.pop(); }
                    AddTaskField::SpeedLimit => { state.speed_limit.pop(); }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn submit_add_task(&mut self) -> Result<()> {
        let Some(state) = &self.add_task_state else {
            return Ok(());
        };

        if state.url.trim().is_empty() {
            if let Some(s) = &mut self.add_task_state {
                s.error = Some("URL 不能为空".to_string());
            }
            return Ok(());
        }

        let url = state.url.trim().to_string();
        let output = if state.path.trim().is_empty() {
            self.queue
                .infer_destination_in_dir(&url, self.config.default_download_path.clone())
                .await
        } else {
            std::path::PathBuf::from(state.path.trim())
        };
        let priority = state.priority;
        let speed_limit = if state.speed_limit.trim().is_empty() {
            None
        } else {
            match parse_speed_limit(state.speed_limit.trim()) {
                Some(limit) => Some(limit),
                None => {
                    if let Some(s) = &mut self.add_task_state {
                        s.error = Some(format!("无效限速: {}", state.speed_limit.trim()));
                    }
                    return Ok(());
                }
            }
        };

        match self
            .queue
            .add_task_with_options(url, output, priority, None, speed_limit, false)
            .await
        {
            Ok(task_id) => {
                self.status_message = format!("已添加: {}", &task_id[..8]);
                self.add_task_state = None;
                self.input_mode = InputMode::Normal;
                self.refresh_tasks().await?;
            }
            Err(e) => {
                if let Some(s) = &mut self.add_task_state {
                    s.error = Some(format!("添加失败: {e}"));
                }
            }
        }
        Ok(())
    }

    // ── Edit Setting keys ──────────────────────────

    async fn handle_edit_setting_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                let value = self.edit_buffer.clone();
                self.edit_buffer.clear();
                self.apply_setting_value(value).await?;
            }
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.edit_buffer.clear();
                self.status_message = "已取消".to_string();
            }
            KeyCode::Char(c) => {
                self.edit_buffer.push(c);
            }
            KeyCode::Backspace => {
                self.edit_buffer.pop();
            }
            _ => {}
        }
        Ok(())
    }

    // ── Confirm dialog keys ────────────────────────

    async fn handle_confirm_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Left | KeyCode::Right => {
                if let Some(dialog) = &mut self.confirm_dialog {
                    dialog.selected_confirm = !dialog.selected_confirm;
                }
            }
            KeyCode::Enter => {
                if let Some(dialog) = self.confirm_dialog.take() {
                    self.input_mode = InputMode::Normal;
                    if dialog.selected_confirm {
                        self.execute_confirm_action(dialog.action).await?;
                    } else {
                        self.status_message = "已取消".to_string();
                    }
                }
            }
            KeyCode::Esc => {
                self.confirm_dialog = None;
                self.input_mode = InputMode::Normal;
                self.status_message = "已取消".to_string();
            }
            _ => {}
        }
        Ok(())
    }

    async fn execute_confirm_action(&mut self, action: ConfirmAction) -> Result<()> {
        match action {
            ConfirmAction::ClearHistory => {
                let path = ConfigStore::history_path()?;
                self.history = DownloadHistory::default();
                self.history.save(&path).await?;
                self.history_index = 0;
                self.status_message = "已清空历史".to_string();
            }
            ConfirmAction::DeleteTaskWithFile { task_id } => {
                self.queue.remove_task_with_file(&task_id).await?;
                self.status_message = format!("已删除任务和文件: {}", &task_id[..8]);
                self.refresh_tasks().await?;
            }
        }
        Ok(())
    }

    // ── Settings helpers ───────────────────────────

    async fn apply_setting_value(&mut self, value: String) -> Result<()> {
        let field = self.selected_setting();
        field.apply(&mut self.config, &value)?;
        self.config.validate()?;
        ConfigStore::save(&self.config).await?;
        self.queue
            .apply_runtime_config(
                self.config.downloader_config(),
                self.config.max_concurrent_tasks,
            )
            .await?;
        self.status_message = format!("已保存: {}", field.label());
        Ok(())
    }

    pub fn selected_setting(&self) -> SettingField {
        SETTINGS_FIELDS[self.setting_index]
    }

    // ── Tick / events ──────────────────────────────

    pub async fn on_tick(&mut self) -> Result<()> {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                DownloaderEvent::Progress(ProgressEvent::Updated { task_id, .. }) => {
                    if let Some(task) = self.queue.get_task(&task_id).await {
                        if let Some(idx) = self.tasks.iter().position(|t| t.id == task_id) {
                            self.tasks[idx] = task;
                        }
                    }
                }
                DownloaderEvent::Task(TaskEvent::Completed { task_id }) => {
                    self.status_message = format!("完成: {}", &task_id[..8]);
                    self.refresh_tasks().await?;
                    self.refresh_history().await?;
                }
                DownloaderEvent::Task(TaskEvent::Failed { task_id, error }) => {
                    self.status_message = format!("失败: {} - {}", &task_id[..8], error);
                    self.refresh_tasks().await?;
                }
                DownloaderEvent::Task(
                    TaskEvent::Added { .. }
                    | TaskEvent::Paused { .. }
                    | TaskEvent::Resumed { .. }
                    | TaskEvent::Cancelled { .. }
                    | TaskEvent::Started { .. },
                ) => {
                    self.refresh_tasks().await?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    // ── Refresh helpers ────────────────────────────

    pub async fn refresh_tasks(&mut self) -> Result<()> {
        self.tasks = self.queue.get_all_tasks().await;
        self.recompute_filtered();
        Ok(())
    }

    pub async fn refresh_history(&mut self) -> Result<()> {
        let path = ConfigStore::history_path()?;
        self.history = DownloadHistory::load(&path).await?;
        if self.history_index >= self.history.completed_tasks.len()
            && !self.history.completed_tasks.is_empty()
        {
            self.history_index = self.history.completed_tasks.len() - 1;
        }
        Ok(())
    }

    // ── Accessor for history ───────────────────────

    pub fn get_selected_history(&self) -> Option<&yushi_core::CompletedTask> {
        self.history.completed_tasks.get(self.history_index)
    }
}

// ── SettingField methods (unchanged from original) ─────

impl SettingField {
    pub fn label(self) -> &'static str {
        match self {
            Self::OutputDir => "默认下载路径",
            Self::Connections => "单任务连接数",
            Self::MaxTasks => "最大并发任务",
            Self::ChunkSize => "分块大小",
            Self::Timeout => "超时(秒)",
            Self::UserAgent => "User-Agent",
            Self::Proxy => "代理地址",
            Self::SpeedLimit => "限速",
            Self::Theme => "主题",
        }
    }

    pub fn current_value(self, config: &AppConfig) -> Option<String> {
        Some(match self {
            Self::OutputDir => config.default_download_path.display().to_string(),
            Self::Connections => config.max_concurrent_downloads.to_string(),
            Self::MaxTasks => config.max_concurrent_tasks.to_string(),
            Self::ChunkSize => config.chunk_size.to_string(),
            Self::Timeout => config.timeout.to_string(),
            Self::UserAgent => config.user_agent.clone(),
            Self::Proxy => config.proxy.clone().unwrap_or_default(),
            Self::SpeedLimit => config
                .speed_limit
                .map(|limit| limit.to_string())
                .unwrap_or_default(),
            Self::Theme => config.theme.to_string(),
        })
    }

    pub fn apply(self, config: &mut AppConfig, value: &str) -> Result<()> {
        match self {
            Self::OutputDir => {
                config.default_download_path = value.trim().into();
            }
            Self::Connections => {
                config.max_concurrent_downloads = value.trim().parse()?;
            }
            Self::MaxTasks => {
                config.max_concurrent_tasks = value.trim().parse()?;
            }
            Self::ChunkSize => {
                config.chunk_size = value.trim().parse()?;
            }
            Self::Timeout => {
                config.timeout = value.trim().parse()?;
            }
            Self::UserAgent => {
                config.user_agent = value.to_string();
            }
            Self::Proxy => {
                config.proxy = if value.trim().is_empty() {
                    None
                } else {
                    Some(value.trim().to_string())
                };
            }
            Self::SpeedLimit => {
                config.speed_limit = if value.trim().is_empty() {
                    None
                } else {
                    Some(
                        parse_speed_limit(value.trim())
                            .ok_or_else(|| anyhow!("无效限速: {}", value.trim()))?,
                    )
                };
            }
            Self::Theme => {
                config.theme = value.trim().parse()?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yushi_core::AppConfig;

    #[test]
    fn applies_speed_limit_setting() {
        let mut config = AppConfig::default();
        SettingField::SpeedLimit
            .apply(&mut config, "2M")
            .expect("apply speed limit");
        assert_eq!(config.speed_limit, Some(2 * 1024 * 1024));
    }

    #[test]
    fn clears_proxy_setting() {
        let mut config = AppConfig {
            proxy: Some("http://127.0.0.1:8080".into()),
            ..AppConfig::default()
        };
        SettingField::Proxy
            .apply(&mut config, "")
            .expect("clear proxy");
        assert_eq!(config.proxy, None);
    }

    #[test]
    fn task_filter_matches() {
        assert!(TaskFilter::All.matches(TaskStatus::Downloading));
        assert!(TaskFilter::Downloading.matches(TaskStatus::Pending));
        assert!(TaskFilter::Downloading.matches(TaskStatus::Downloading));
        assert!(TaskFilter::Downloading.matches(TaskStatus::Paused));
        assert!(!TaskFilter::Downloading.matches(TaskStatus::Completed));
        assert!(TaskFilter::Completed.matches(TaskStatus::Completed));
        assert!(TaskFilter::Completed.matches(TaskStatus::Failed));
        assert!(TaskFilter::Completed.matches(TaskStatus::Cancelled));
        assert!(!TaskFilter::Completed.matches(TaskStatus::Downloading));
    }

    #[test]
    fn task_filter_cycle() {
        assert_eq!(TaskFilter::All.next(), TaskFilter::Downloading);
        assert_eq!(TaskFilter::Downloading.next(), TaskFilter::Completed);
        assert_eq!(TaskFilter::Completed.next(), TaskFilter::All);
        assert_eq!(TaskFilter::All.prev(), TaskFilter::Completed);
    }

    #[test]
    fn add_task_state_focus_cycle() {
        let mut state = AddTaskState::new("/tmp");
        assert_eq!(state.focused(), AddTaskField::Url);
        state.focus_next();
        assert_eq!(state.focused(), AddTaskField::Path);
        state.focus_next();
        assert_eq!(state.focused(), AddTaskField::Priority);
        state.focus_next();
        assert_eq!(state.focused(), AddTaskField::SpeedLimit);
        state.focus_next();
        assert_eq!(state.focused(), AddTaskField::Buttons);
        state.focus_next();
        assert_eq!(state.focused(), AddTaskField::Url); // wraps
    }
}
```

- [ ] **Step 2: Update `mod.rs` to declare the new modules**

In `yushi-cli/src/tui/mod.rs`, add the new module declarations:

```rust
// yushi-cli/src/tui/mod.rs
mod app;
mod event;
pub mod theme;
mod ui;
pub mod widgets;

// ... rest stays the same
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p yushi-cli --features tui`
Expected: compiles (widget stubs reference the new types from app.rs)

- [ ] **Step 4: Run existing tests**

Run: `cargo test -p yushi-cli`
Expected: all tests pass (existing + new filter/focus tests)

- [ ] **Step 5: Commit**

```bash
git add yushi-cli/src/tui/app.rs yushi-cli/src/tui/mod.rs
git commit -m "feat(tui): rewrite app state with filters, add-task dialog, confirm dialog"
```

---

### Task 4: Implement `widgets/sidebar.rs`

**Files:**
- Modify: `yushi-cli/src/tui/widgets/sidebar.rs`

- [ ] **Step 1: Implement sidebar rendering**

```rust
// yushi-cli/src/tui/widgets/sidebar.rs
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::tui::app::{App, CurrentView};
use crate::tui::theme::ThemeColors;

pub fn draw(f: &mut Frame, app: &App, theme: &ThemeColors, area: Rect) {
    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(theme.border));
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Layout: logo (2), spacer (1), nav items (3 * 3 = 9), flex spacer, version
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // logo
            Constraint::Length(1),  // spacer
            Constraint::Length(3),  // tasks icon
            Constraint::Length(3),  // history icon
            Constraint::Length(3),  // settings icon
            Constraint::Min(0),    // flex spacer
            Constraint::Length(3),  // version
        ])
        .split(inner);

    // Logo
    let logo = Paragraph::new("驭")
        .style(
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(logo, chunks[0]);

    // Nav items
    let nav_items = [
        ("📥", CurrentView::Tasks),
        ("📋", CurrentView::History),
        ("⚙", CurrentView::Settings),
    ];

    for (i, (icon, view)) in nav_items.iter().enumerate() {
        let is_active = app.current_view == *view;
        let style = if is_active {
            Style::default()
                .fg(theme.text)
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.muted)
        };

        let item = Paragraph::new(Line::from(Span::styled(*icon, style)))
            .alignment(Alignment::Center);
        f.render_widget(item, chunks[2 + i]);
    }

    // Version
    let version = Paragraph::new(vec![
        Line::from("v0"),
        Line::from(".1"),
        Line::from(".0"),
    ])
    .style(Style::default().fg(theme.text_help))
    .alignment(Alignment::Center);
    f.render_widget(version, chunks[6]);
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p yushi-cli --features tui`

- [ ] **Step 3: Commit**

```bash
git add yushi-cli/src/tui/widgets/sidebar.rs
git commit -m "feat(tui): implement sidebar widget with nav icons and version"
```

---

### Task 5: Implement `widgets/filter_tabs.rs`

**Files:**
- Modify: `yushi-cli/src/tui/widgets/filter_tabs.rs`

- [ ] **Step 1: Implement filter tabs rendering**

```rust
// yushi-cli/src/tui/widgets/filter_tabs.rs
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::tui::app::{App, TaskFilter};
use crate::tui::theme::ThemeColors;

pub fn draw(f: &mut Frame, app: &App, theme: &ThemeColors, area: Rect) {
    let filters = [TaskFilter::All, TaskFilter::Downloading, TaskFilter::Completed];
    let tab_count = filters.len() as u16;

    let constraints: Vec<Constraint> = filters
        .iter()
        .map(|_| Constraint::Ratio(1, tab_count as u32))
        .collect();

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    for (i, filter) in filters.iter().enumerate() {
        let count = app.filter_count(*filter);
        let is_active = app.filter == *filter;

        let label = format!(" {}({}) ", filter.label(), count);

        let style = if is_active {
            Style::default()
                .fg(theme.text)
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_secondary)
        };

        let tab = Paragraph::new(Line::from(Span::styled(label, style)));
        f.render_widget(tab, chunks[i]);
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p yushi-cli --features tui`

- [ ] **Step 3: Commit**

```bash
git add yushi-cli/src/tui/widgets/filter_tabs.rs
git commit -m "feat(tui): implement filter tabs widget for task filtering"
```

---

### Task 6: Implement `widgets/task_card.rs`

**Files:**
- Modify: `yushi-cli/src/tui/widgets/task_card.rs`

- [ ] **Step 1: Implement task card rendering**

```rust
// yushi-cli/src/tui/widgets/task_card.rs
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
};
use yushi_core::{DownloadTask, TaskStatus};

use crate::tui::theme::ThemeColors;
use crate::ui::format_size;

pub fn draw(f: &mut Frame, task: &DownloadTask, selected: bool, theme: &ThemeColors, area: Rect) {
    let border_style = if selected {
        Style::default().fg(theme.border_active)
    } else {
        Style::default().fg(theme.border)
    };

    let bg = if selected {
        theme.selection_bg
    } else {
        theme.bg
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(Style::default().bg(bg));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // filename + actions
            Constraint::Length(1), // stats line
            Constraint::Length(1), // progress bar
        ])
        .split(inner);

    // Line 1: icon + filename + action hints
    let filename = task
        .dest
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let icon = file_icon(filename);
    let actions = action_hints(task, theme);

    let available_width = chunks[0].width as usize;
    let icon_len = 3; // emoji + space
    let actions_len: usize = actions.iter().map(|s| s.width()).sum();
    let max_name_len = available_width.saturating_sub(icon_len + actions_len + 1);
    let display_name = truncate_str(filename, max_name_len);

    let mut line1_spans = vec![
        Span::styled(
            format!("{} ", icon),
            Style::default().fg(status_color(task.status, theme)),
        ),
        Span::styled(
            display_name,
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ];
    // Right-align actions
    let name_width = icon_len + filename.len().min(max_name_len);
    let padding = available_width.saturating_sub(name_width + actions_len);
    if padding > 0 {
        line1_spans.push(Span::raw(" ".repeat(padding)));
    }
    line1_spans.extend(actions);

    f.render_widget(Paragraph::new(Line::from(line1_spans)), chunks[0]);

    // Line 2: stats
    let stats = stats_line(task, theme);
    f.render_widget(Paragraph::new(Line::from(stats)), chunks[1]);

    // Line 3: progress gauge
    let progress = if task.total_size > 0 {
        ((task.downloaded as f64 / task.total_size as f64) * 100.0) as u16
    } else {
        0
    };

    let gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(progress_color(task.status, theme))
                .add_modifier(Modifier::BOLD),
        )
        .percent(progress.min(100))
        .label(format!("{}%", progress));

    f.render_widget(gauge, chunks[2]);
}

/// Returns the card height (including borders)
pub fn card_height(task: &DownloadTask) -> u16 {
    if task.error.is_some() && matches!(task.status, TaskStatus::Failed) {
        5 + 1 // 3 lines + error line + 2 borders
    } else {
        5 // 3 lines + 2 borders
    }
}

fn file_icon(filename: &str) -> &'static str {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" => "📦",
        "iso" | "img" | "dmg" => "💿",
        "pdf" | "doc" | "docx" | "txt" | "md" => "📄",
        "mp4" | "mkv" | "avi" | "mov" | "webm" => "🎬",
        "mp3" | "flac" | "wav" | "ogg" | "aac" => "🎵",
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" => "🖼",
        "exe" | "msi" | "deb" | "rpm" | "appimage" => "⚙",
        "js" | "ts" | "py" | "rs" | "go" | "c" | "cpp" => "📝",
        "json" | "csv" | "xml" | "sql" | "db" => "📋",
        _ => "📎",
    }
}

fn status_color(status: TaskStatus, theme: &ThemeColors) -> ratatui::style::Color {
    match status {
        TaskStatus::Pending => theme.warning,
        TaskStatus::Downloading => theme.primary,
        TaskStatus::Paused => theme.warning,
        TaskStatus::Completed => theme.success,
        TaskStatus::Failed => theme.error,
        TaskStatus::Cancelled => theme.muted,
    }
}

fn progress_color(status: TaskStatus, theme: &ThemeColors) -> ratatui::style::Color {
    match status {
        TaskStatus::Downloading | TaskStatus::Pending => theme.primary,
        TaskStatus::Completed => theme.success,
        TaskStatus::Paused => theme.warning,
        TaskStatus::Failed | TaskStatus::Cancelled => theme.error,
    }
}

fn action_hints<'a>(task: &DownloadTask, theme: &ThemeColors) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    match task.status {
        TaskStatus::Downloading => {
            spans.push(Span::styled("[⏸]", Style::default().fg(theme.warning)));
            spans.push(Span::styled("[✕]", Style::default().fg(theme.error)));
        }
        TaskStatus::Paused => {
            spans.push(Span::styled("[▶]", Style::default().fg(theme.success)));
            spans.push(Span::styled("[✕]", Style::default().fg(theme.error)));
        }
        TaskStatus::Pending => {
            spans.push(Span::styled("[✕]", Style::default().fg(theme.error)));
        }
        TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled => {
            spans.push(Span::styled("[✕]", Style::default().fg(theme.muted)));
        }
    }
    spans
}

fn stats_line<'a>(task: &DownloadTask, theme: &ThemeColors) -> Vec<Span<'a>> {
    let mut spans = Vec::new();

    match task.status {
        TaskStatus::Downloading => {
            let size_str = if task.total_size > 0 {
                format!("{} / {}", format_size(task.downloaded), format_size(task.total_size))
            } else {
                format_size(task.downloaded)
            };
            spans.push(Span::styled(size_str, Style::default().fg(theme.text_secondary)));
            if task.speed > 0 {
                spans.push(Span::styled(
                    format!(" · {}/s", format_size(task.speed)),
                    Style::default().fg(theme.primary),
                ));
            }
            if let Some(eta) = task.eta {
                spans.push(Span::styled(
                    format!(" · 剩余 {}", format_eta(eta)),
                    Style::default().fg(theme.text_secondary),
                ));
            }
        }
        TaskStatus::Pending => {
            spans.push(Span::styled("等待中", Style::default().fg(theme.warning)));
        }
        TaskStatus::Paused => {
            let size_str = if task.total_size > 0 {
                format!("{} / {}", format_size(task.downloaded), format_size(task.total_size))
            } else {
                format_size(task.downloaded)
            };
            spans.push(Span::styled(size_str, Style::default().fg(theme.text_secondary)));
            spans.push(Span::styled(" · 已暂停", Style::default().fg(theme.warning)));
        }
        TaskStatus::Completed => {
            spans.push(Span::styled(
                format_size(task.total_size),
                Style::default().fg(theme.text_secondary),
            ));
            spans.push(Span::styled(
                " · 已完成",
                Style::default().fg(theme.success),
            ));
        }
        TaskStatus::Failed => {
            let msg = task.error.as_deref().unwrap_or("未知错误");
            spans.push(Span::styled(
                format!("失败: {}", msg),
                Style::default().fg(theme.error),
            ));
        }
        TaskStatus::Cancelled => {
            spans.push(Span::styled("已取消", Style::default().fg(theme.muted)));
        }
    }

    spans
}

fn format_eta(seconds: u64) -> String {
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;
    let s = seconds % 60;
    if h > 0 {
        format!("{}h{}m", h, m)
    } else if m > 0 {
        format!("{}m{}s", m, s)
    } else {
        format!("{}s", s)
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len > 3 {
        format!("{}…", &s[..max_len - 1])
    } else {
        s[..max_len].to_string()
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p yushi-cli --features tui`

- [ ] **Step 3: Commit**

```bash
git add yushi-cli/src/tui/widgets/task_card.rs
git commit -m "feat(tui): implement task card widget with progress bars and file icons"
```

---

### Task 7: Implement `widgets/history_card.rs`

**Files:**
- Modify: `yushi-cli/src/tui/widgets/history_card.rs`

- [ ] **Step 1: Implement history card rendering**

```rust
// yushi-cli/src/tui/widgets/history_card.rs
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use yushi_core::CompletedTask;

use crate::tui::theme::ThemeColors;
use crate::ui::format_size;

pub fn draw(
    f: &mut Frame,
    task: &CompletedTask,
    selected: bool,
    theme: &ThemeColors,
    area: Rect,
) {
    let border_style = if selected {
        Style::default().fg(theme.border_active)
    } else {
        Style::default().fg(theme.border)
    };

    let bg = if selected {
        theme.selection_bg
    } else {
        theme.bg
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(Style::default().bg(bg));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let filename = task
        .dest
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    // Line 1: checkmark + filename + [✕]
    let available = inner.width as usize;
    let prefix_len = 3; // "✅ "
    let suffix_len = 4; // " [✕]"
    let max_name = available.saturating_sub(prefix_len + suffix_len);
    let display_name = if filename.len() > max_name && max_name > 3 {
        format!("{}…", &filename[..max_name - 1])
    } else {
        filename.to_string()
    };
    let padding = available.saturating_sub(prefix_len + display_name.len() + suffix_len);

    let line1 = Line::from(vec![
        Span::styled("✅ ", Style::default().fg(theme.success)),
        Span::styled(display_name, Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" ".repeat(padding)),
        Span::styled("[✕]", Style::default().fg(theme.muted)),
    ]);

    // Line 2: size · avg speed · duration · date
    let duration = format_duration(task.duration);
    let date = format_date(task.completed_at);
    let line2 = Line::from(vec![
        Span::styled(
            format_size(task.total_size),
            Style::default().fg(theme.text_secondary),
        ),
        Span::styled(" · ", Style::default().fg(theme.text_help)),
        Span::styled(
            format!("平均 {}/s", format_size(task.avg_speed)),
            Style::default().fg(theme.primary),
        ),
        Span::styled(" · ", Style::default().fg(theme.text_help)),
        Span::styled(
            format!("用时 {}", duration),
            Style::default().fg(theme.text_secondary),
        ),
        Span::styled(" · ", Style::default().fg(theme.text_help)),
        Span::styled(date, Style::default().fg(theme.text_secondary)),
    ]);

    if inner.height >= 2 {
        f.render_widget(Paragraph::new(line1), Rect { height: 1, ..inner });
        f.render_widget(
            Paragraph::new(line2),
            Rect {
                y: inner.y + 1,
                height: 1,
                ..inner
            },
        );
    }
}

/// Card height including borders: 2 content lines + 2 border lines = 4
pub fn card_height() -> u16 {
    4
}

fn format_duration(seconds: u64) -> String {
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;
    let s = seconds % 60;
    if h > 0 {
        format!("{}时{}分{}秒", h, m, s)
    } else if m > 0 {
        format!("{}分{}秒", m, s)
    } else {
        format!("{}秒", s)
    }
}

fn format_date(timestamp: u64) -> String {
    // Simple MM-DD format from unix timestamp
    let secs = timestamp as i64;
    // Use chrono-free approach: calculate from epoch
    // For simplicity, format as the raw timestamp if chrono is not available
    // But we can do basic date math
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;

    // Zeller-like calculation for month-day from days since epoch
    // Simplified: just show HH:MM for recent, or use a basic conversion
    let (year, month, day) = days_to_ymd(days_since_epoch);
    let _ = year; // we only show MM-DD
    format!("{:02}-{:02} {:02}:{:02}", month, day, hours, minutes)
}

fn days_to_ymd(days: i64) -> (i64, u32, u32) {
    // Algorithm from https://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p yushi-cli --features tui`

- [ ] **Step 3: Commit**

```bash
git add yushi-cli/src/tui/widgets/history_card.rs
git commit -m "feat(tui): implement history card widget with date formatting"
```

---

### Task 8: Implement `widgets/empty_state.rs`

**Files:**
- Modify: `yushi-cli/src/tui/widgets/empty_state.rs`

- [ ] **Step 1: Implement empty state rendering**

```rust
// yushi-cli/src/tui/widgets/empty_state.rs
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::Paragraph,
};

use crate::tui::theme::ThemeColors;

pub fn draw(
    f: &mut Frame,
    icon: &str,
    message: &str,
    hint: &str,
    theme: &ThemeColors,
    area: Rect,
) {
    // Center vertically
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(4),
            Constraint::Min(0),
        ])
        .split(area);

    let content = Paragraph::new(vec![
        Line::from(""),
        Line::from(icon).alignment(Alignment::Center),
        Line::styled(
            message,
            Style::default()
                .fg(theme.text_secondary)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center),
        Line::styled(hint, Style::default().fg(theme.text_help)).alignment(Alignment::Center),
    ]);

    f.render_widget(content, v_chunks[1]);
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p yushi-cli --features tui`

- [ ] **Step 3: Commit**

```bash
git add yushi-cli/src/tui/widgets/empty_state.rs
git commit -m "feat(tui): implement empty state widget"
```

---

### Task 9: Implement `widgets/settings_group.rs`

**Files:**
- Modify: `yushi-cli/src/tui/widgets/settings_group.rs`

- [ ] **Step 1: Implement grouped settings rendering**

```rust
// yushi-cli/src/tui/widgets/settings_group.rs
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use yushi_core::AppTheme;

use crate::tui::app::{App, InputMode, SettingField, SETTINGS_FIELDS, SETTINGS_GROUPS};
use crate::tui::theme::ThemeColors;

pub fn draw(f: &mut Frame, app: &App, theme: &ThemeColors, area: Rect) {
    // Calculate heights for each group: title border (2) + fields (1 each) + bottom border (1)
    let constraints: Vec<Constraint> = SETTINGS_GROUPS
        .iter()
        .map(|group| Constraint::Length(group.fields.len() as u16 + 2))
        .chain(std::iter::once(Constraint::Length(3))) // About section
        .chain(std::iter::once(Constraint::Min(0)))    // flex spacer
        .collect();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    for (gi, group) in SETTINGS_GROUPS.iter().enumerate() {
        let block = Block::default()
            .title(format!(" {} ", group.title))
            .title_style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border));
        let inner = block.inner(chunks[gi]);
        f.render_widget(block, chunks[gi]);

        let field_constraints: Vec<Constraint> =
            group.fields.iter().map(|_| Constraint::Length(1)).collect();
        let field_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(field_constraints)
            .split(inner);

        for (fi, field) in group.fields.iter().enumerate() {
            let global_index = SETTINGS_FIELDS
                .iter()
                .position(|f| f == field)
                .unwrap_or(0);
            let is_selected = global_index == app.setting_index;
            let is_editing =
                is_selected && app.input_mode == InputMode::EditSetting;

            let line = render_field(*field, &app.config, is_selected, is_editing, &app.edit_buffer, theme);
            f.render_widget(Paragraph::new(line), field_chunks[fi]);
        }
    }

    // About section
    let about_idx = SETTINGS_GROUPS.len();
    let about_block = Block::default()
        .title(" 关于 ")
        .title_style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border));
    let about_inner = about_block.inner(chunks[about_idx]);
    f.render_widget(about_block, chunks[about_idx]);

    let about = Paragraph::new(Line::from(vec![
        Span::styled(
            "YuShi v0.1.0",
            Style::default()
                .fg(theme.text)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "  驭时 - 异步下载管理器",
            Style::default().fg(theme.text_secondary),
        ),
    ]));
    f.render_widget(about, about_inner);
}

fn render_field<'a>(
    field: SettingField,
    config: &yushi_core::AppConfig,
    selected: bool,
    editing: bool,
    edit_buffer: &str,
    theme: &ThemeColors,
) -> Line<'a> {
    let label = field.label();

    if field == SettingField::Theme {
        return render_theme_field(label, config.theme, selected, theme);
    }

    let value = if editing {
        format!("{}▎", edit_buffer)
    } else {
        field.current_value(config).unwrap_or_default()
    };

    let label_style = if selected {
        Style::default()
            .fg(theme.text)
            .bg(theme.selection_bg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text)
    };

    let value_style = if selected {
        Style::default()
            .fg(theme.primary)
            .bg(theme.selection_bg)
    } else {
        Style::default().fg(theme.text_secondary)
    };

    // Pad label to fixed width for alignment
    let padded_label = format!("{:<14}", label);

    Line::from(vec![
        Span::styled(format!("  {}", padded_label), label_style),
        Span::styled(value, value_style),
    ])
}

fn render_theme_field<'a>(
    label: &str,
    current: AppTheme,
    selected: bool,
    theme: &ThemeColors,
) -> Line<'a> {
    let label_style = if selected {
        Style::default()
            .fg(theme.text)
            .bg(theme.selection_bg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text)
    };

    let padded_label = format!("{:<14}", label);

    let options = [
        ("浅色", AppTheme::Light),
        ("深色", AppTheme::Dark),
        ("系统", AppTheme::System),
    ];

    let mut spans = vec![Span::styled(format!("  {}", padded_label), label_style)];

    for (name, value) in &options {
        let style = if *value == current {
            Style::default()
                .fg(theme.text)
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_secondary)
        };
        spans.push(Span::styled(format!(" {} ", name), style));
        spans.push(Span::raw(" "));
    }

    Line::from(spans)
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p yushi-cli --features tui`

- [ ] **Step 3: Commit**

```bash
git add yushi-cli/src/tui/widgets/settings_group.rs
git commit -m "feat(tui): implement grouped settings widget with theme toggle"
```

---

### Task 10: Implement `widgets/add_task.rs`

**Files:**
- Modify: `yushi-cli/src/tui/widgets/add_task.rs`

- [ ] **Step 1: Implement add task dialog rendering**

```rust
// yushi-cli/src/tui/widgets/add_task.rs
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use yushi_core::Priority;

use crate::tui::app::{AddTaskField, AddTaskState};
use crate::tui::theme::ThemeColors;

pub fn draw(f: &mut Frame, state: &AddTaskState, theme: &ThemeColors, area: Rect) {
    // Center the dialog: 50 wide, 18 tall
    let dialog_width = 50u16.min(area.width.saturating_sub(4));
    let dialog_height = 18u16.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
    let y = area.y + (area.height.saturating_sub(dialog_height)) / 2;
    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    // Dim background
    f.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(" 添加下载任务 ")
        .title_style(
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary));
    let inner = block.inner(dialog_area);
    f.render_widget(block, dialog_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // "下载链接" label
            Constraint::Length(1), // url input
            Constraint::Length(1), // spacer
            Constraint::Length(1), // "保存位置" label
            Constraint::Length(1), // path input
            Constraint::Length(1), // spacer
            Constraint::Length(1), // "优先级" label
            Constraint::Length(1), // priority toggle
            Constraint::Length(1), // spacer
            Constraint::Length(1), // "限速" label
            Constraint::Length(1), // speed input
            Constraint::Length(1), // spacer
            Constraint::Length(1), // error message
            Constraint::Length(1), // spacer
            Constraint::Length(1), // buttons
        ])
        .split(inner);

    let focused = state.focused();

    // URL field
    draw_label(f, "下载链接", chunks[0], theme);
    draw_input(
        f,
        &state.url,
        focused == AddTaskField::Url,
        chunks[1],
        theme,
    );

    // Path field
    draw_label(f, "保存位置", chunks[3], theme);
    draw_input(
        f,
        &state.path,
        focused == AddTaskField::Path,
        chunks[4],
        theme,
    );

    // Priority field
    draw_label(f, "优先级", chunks[6], theme);
    draw_priority(f, state.priority, focused == AddTaskField::Priority, chunks[7], theme);

    // Speed limit field
    draw_label(f, "限速 (留空不限)", chunks[9], theme);
    draw_input(
        f,
        &state.speed_limit,
        focused == AddTaskField::SpeedLimit,
        chunks[10],
        theme,
    );

    // Error message
    if let Some(error) = &state.error {
        let err = Paragraph::new(Line::from(Span::styled(
            error.as_str(),
            Style::default().fg(theme.error),
        )))
        .alignment(Alignment::Center);
        f.render_widget(err, chunks[12]);
    }

    // Buttons
    draw_buttons(f, state.button_confirm, focused == AddTaskField::Buttons, chunks[14], theme);
}

fn draw_label(f: &mut Frame, text: &str, area: Rect, theme: &ThemeColors) {
    let p = Paragraph::new(Span::styled(
        format!("  {}", text),
        Style::default().fg(theme.text_secondary),
    ));
    f.render_widget(p, area);
}

fn draw_input(f: &mut Frame, value: &str, focused: bool, area: Rect, theme: &ThemeColors) {
    let border_color = if focused {
        theme.border_active
    } else {
        theme.border
    };

    let display = if focused {
        format!(" {}▎", value)
    } else {
        format!(" {}", value)
    };

    let input = Paragraph::new(display)
        .style(Style::default().fg(theme.text))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        );
    f.render_widget(input, area);
}

fn draw_priority(
    f: &mut Frame,
    current: Priority,
    focused: bool,
    area: Rect,
    theme: &ThemeColors,
) {
    let options = [
        (" 低 ", Priority::Low),
        ("正常", Priority::Normal),
        (" 高 ", Priority::High),
    ];

    let mut spans = vec![Span::raw("  ")];
    for (label, value) in &options {
        let is_active = *value == current;
        let style = if is_active {
            Style::default()
                .fg(theme.text)
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD)
        } else if focused {
            Style::default().fg(theme.text)
        } else {
            Style::default().fg(theme.text_secondary)
        };
        spans.push(Span::styled(format!("[{}]", label), style));
        spans.push(Span::raw(" "));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_buttons(
    f: &mut Frame,
    confirm_selected: bool,
    focused: bool,
    area: Rect,
    theme: &ThemeColors,
) {
    let cancel_style = if focused && !confirm_selected {
        Style::default()
            .fg(theme.text)
            .bg(theme.muted)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_secondary)
    };

    let confirm_style = if focused && confirm_selected {
        Style::default()
            .fg(theme.text)
            .bg(theme.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_secondary)
    };

    let buttons = Paragraph::new(Line::from(vec![
        Span::raw("        "),
        Span::styled(" 取消 ", cancel_style),
        Span::raw("    "),
        Span::styled(" 添加 ", confirm_style),
    ]))
    .alignment(Alignment::Center);

    f.render_widget(buttons, area);
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p yushi-cli --features tui`

- [ ] **Step 3: Commit**

```bash
git add yushi-cli/src/tui/widgets/add_task.rs
git commit -m "feat(tui): implement add task dialog widget with multi-field input"
```

---

### Task 11: Implement `widgets/dialog.rs`

**Files:**
- Modify: `yushi-cli/src/tui/widgets/dialog.rs`

- [ ] **Step 1: Implement confirmation dialog rendering**

```rust
// yushi-cli/src/tui/widgets/dialog.rs
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::tui::app::ConfirmDialog;
use crate::tui::theme::ThemeColors;

pub fn draw(f: &mut Frame, dialog: &ConfirmDialog, theme: &ThemeColors, area: Rect) {
    let dialog_width = 30u16.min(area.width.saturating_sub(4));
    let dialog_height = 7u16;
    let x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
    let y = area.y + (area.height.saturating_sub(dialog_height)) / 2;
    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    f.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(format!(" {} ", dialog.title))
        .title_style(
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.warning));
    let inner = block.inner(dialog_area);
    f.render_widget(block, dialog_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // spacer
            Constraint::Length(1), // message
            Constraint::Length(1), // spacer
            Constraint::Length(1), // buttons
        ])
        .split(inner);

    let msg = Paragraph::new(dialog.message.as_str())
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.text));
    f.render_widget(msg, chunks[1]);

    let cancel_style = if !dialog.selected_confirm {
        Style::default()
            .fg(theme.text)
            .bg(theme.muted)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_secondary)
    };

    let confirm_style = if dialog.selected_confirm {
        Style::default()
            .fg(theme.text)
            .bg(theme.error)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_secondary)
    };

    let buttons = Paragraph::new(Line::from(vec![
        Span::raw("  "),
        Span::styled(" 取消 ", cancel_style),
        Span::raw("  "),
        Span::styled(" 确认 ", confirm_style),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(buttons, chunks[3]);
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p yushi-cli --features tui`

- [ ] **Step 3: Commit**

```bash
git add yushi-cli/src/tui/widgets/dialog.rs
git commit -m "feat(tui): implement confirmation dialog widget"
```

---

### Task 12: Implement `widgets/help_bar.rs`

**Files:**
- Modify: `yushi-cli/src/tui/widgets/help_bar.rs`

- [ ] **Step 1: Implement context-sensitive help bar**

```rust
// yushi-cli/src/tui/widgets/help_bar.rs
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Style,
    widgets::Paragraph,
};

use crate::tui::app::{App, CurrentView, InputMode};
use crate::tui::theme::ThemeColors;

pub fn draw(f: &mut Frame, app: &App, theme: &ThemeColors, area: Rect) {
    let text = match app.input_mode {
        InputMode::Normal => match app.current_view {
            CurrentView::Tasks => {
                "1/2/3:视图  ↑↓:导航  Tab/←→:筛选  a:添加  p:暂停  c:取消  d:删除  D:删文件  q:退出"
            }
            CurrentView::History => {
                "1/2/3:视图  ↑↓:导航  x:删除记录  C:清空全部  r:刷新  q:退出"
            }
            CurrentView::Settings => {
                "1/2/3:视图  ↑↓:选择  Enter/e:编辑  ←→:切换主题  r:重载  q:退出"
            }
        },
        InputMode::AddTask => "Tab:下一项  Shift+Tab:上一项  ←→:切换  Enter:确认  Esc:取消",
        InputMode::EditSetting => "Enter:保存  Esc:取消",
        InputMode::Confirm => "←→:选择  Enter:确认  Esc:取消",
    };

    let help = Paragraph::new(text)
        .style(Style::default().fg(theme.text_help))
        .alignment(Alignment::Center);

    f.render_widget(help, area);
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p yushi-cli --features tui`

- [ ] **Step 3: Commit**

```bash
git add yushi-cli/src/tui/widgets/help_bar.rs
git commit -m "feat(tui): implement context-sensitive help bar widget"
```

---

### Task 13: Rewrite `ui.rs` — Top-Level Layout Orchestration

**Files:**
- Modify: `yushi-cli/src/tui/ui.rs`

This is the main layout file that assembles all widgets together.

- [ ] **Step 1: Rewrite `ui.rs` with new layout**

```rust
// yushi-cli/src/tui/ui.rs
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::app::{App, CurrentView, InputMode};
use super::widgets;
use crate::ui::format_size;

const SIDEBAR_WIDTH: u16 = 5;

pub fn draw(f: &mut Frame, app: &App) {
    let theme = &app.theme;

    // Top-level: sidebar | content
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(SIDEBAR_WIDTH), Constraint::Min(0)])
        .split(f.area());

    // Sidebar
    widgets::sidebar::draw(f, app, theme, h_chunks[0]);

    // Right side: header + (filter tabs) + content + help
    let has_filter = app.current_view == CurrentView::Tasks;
    let v_constraints = if has_filter {
        vec![
            Constraint::Length(1), // header
            Constraint::Length(1), // filter tabs
            Constraint::Min(3),   // content
            Constraint::Length(1), // help
        ]
    } else {
        vec![
            Constraint::Length(1), // header
            Constraint::Min(3),   // content
            Constraint::Length(1), // help
        ]
    };

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(v_constraints)
        .split(h_chunks[1]);

    // Header
    draw_header(f, app, theme, v_chunks[0]);

    if has_filter {
        // Filter tabs
        widgets::filter_tabs::draw(f, app, theme, v_chunks[1]);
        // Content
        draw_content(f, app, theme, v_chunks[2]);
        // Help
        widgets::help_bar::draw(f, app, theme, v_chunks[3]);
    } else {
        // Content
        draw_content(f, app, theme, v_chunks[1]);
        // Help
        widgets::help_bar::draw(f, app, theme, v_chunks[2]);
    }

    // Overlay dialogs (on top of everything)
    if app.input_mode == InputMode::AddTask {
        if let Some(state) = &app.add_task_state {
            widgets::add_task::draw(f, state, theme, f.area());
        }
    }

    if app.input_mode == InputMode::Confirm {
        if let Some(dialog) = &app.confirm_dialog {
            widgets::dialog::draw(f, dialog, theme, f.area());
        }
    }
}

fn draw_header(
    f: &mut Frame,
    app: &App,
    theme: &crate::tui::theme::ThemeColors,
    area: Rect,
) {
    let (title, count_str) = match app.current_view {
        CurrentView::Tasks => {
            let count = app.filtered_indices.len();
            ("任务".to_string(), format!("{} 个任务", count))
        }
        CurrentView::History => {
            let count = app.history.completed_tasks.len();
            ("历史".to_string(), format!("{} 条记录", count))
        }
        CurrentView::Settings => ("设置".to_string(), String::new()),
    };

    let available = area.width as usize;
    let title_len = title.len() + 2; // spaces
    let count_len = count_str.len() + 2;
    let padding = available.saturating_sub(title_len + count_len);

    let line = Line::from(vec![
        Span::styled(
            format!(" {}", title),
            Style::default()
                .fg(theme.text)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" ".repeat(padding)),
        Span::styled(
            format!("{} ", count_str),
            Style::default().fg(theme.text_secondary),
        ),
    ]);

    f.render_widget(Paragraph::new(line), area);
}

fn draw_content(
    f: &mut Frame,
    app: &App,
    theme: &crate::tui::theme::ThemeColors,
    area: Rect,
) {
    match app.current_view {
        CurrentView::Tasks => draw_tasks_content(f, app, theme, area),
        CurrentView::History => draw_history_content(f, app, theme, area),
        CurrentView::Settings => widgets::settings_group::draw(f, app, theme, area),
    }
}

fn draw_tasks_content(
    f: &mut Frame,
    app: &App,
    theme: &crate::tui::theme::ThemeColors,
    area: Rect,
) {
    if app.filtered_indices.is_empty() {
        let (hint, msg) = match app.filter {
            crate::tui::app::TaskFilter::All => ("按 a 添加新任务", "暂无下载任务"),
            crate::tui::app::TaskFilter::Downloading => ("", "暂无进行中的任务"),
            crate::tui::app::TaskFilter::Completed => ("", "暂无已完成的任务"),
        };
        widgets::empty_state::draw(f, "⬇", msg, hint, theme, area);
        return;
    }

    // Render cards with scrolling
    let card_height = 5u16; // 3 content lines + 2 borders
    let visible_cards = (area.height / card_height).max(1) as usize;

    // Auto-scroll to keep selected visible
    let scroll = {
        let sel = app.selected_index;
        let scroll = app.task_scroll;
        if sel < scroll {
            sel
        } else if sel >= scroll + visible_cards {
            sel - visible_cards + 1
        } else {
            scroll
        }
    };

    let mut y = area.y;
    for vi in 0..visible_cards {
        let idx = scroll + vi;
        if idx >= app.filtered_indices.len() {
            break;
        }
        let task_idx = app.filtered_indices[idx];
        if let Some(task) = app.tasks.get(task_idx) {
            let card_area = Rect::new(area.x, y, area.width, card_height.min(area.y + area.height - y));
            let selected = idx == app.selected_index;
            widgets::task_card::draw(f, task, selected, theme, card_area);
            y += card_height;
            if y >= area.y + area.height {
                break;
            }
        }
    }
}

fn draw_history_content(
    f: &mut Frame,
    app: &App,
    theme: &crate::tui::theme::ThemeColors,
    area: Rect,
) {
    if app.history.completed_tasks.is_empty() {
        widgets::empty_state::draw(f, "📋", "暂无下载记录", "", theme, area);
        return;
    }

    let card_h = widgets::history_card::card_height();
    let visible = (area.height / card_h).max(1) as usize;

    let scroll = {
        let sel = app.history_index;
        if sel >= visible { sel - visible + 1 } else { 0 }
    };

    let mut y = area.y;
    for vi in 0..visible {
        let idx = scroll + vi;
        if idx >= app.history.completed_tasks.len() {
            break;
        }
        let task = &app.history.completed_tasks[idx];
        let card_area = Rect::new(area.x, y, area.width, card_h.min(area.y + area.height - y));
        let selected = idx == app.history_index;
        widgets::history_card::draw(f, task, selected, theme, card_area);
        y += card_h;
        if y >= area.y + area.height {
            break;
        }
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p yushi-cli --features tui`
Expected: may have minor issues to fix (unused imports from old code, etc.)

- [ ] **Step 3: Fix any compilation errors**

Common fixes: adjust import paths, remove unused `use` statements, fix type mismatches. The `format_size` import from `crate::ui` should already work.

- [ ] **Step 4: Run all tests**

Run: `cargo test -p yushi-cli`
Expected: all tests pass

- [ ] **Step 5: Commit**

```bash
git add yushi-cli/src/tui/ui.rs
git commit -m "feat(tui): rewrite layout with sidebar, card list, and widget orchestration"
```

---

### Task 14: Wire Everything Together and Fix Compilation

**Files:**
- Modify: `yushi-cli/src/tui/mod.rs` (final wiring)

This task ensures all modules are properly connected and the entire TUI compiles and runs.

- [ ] **Step 1: Ensure `mod.rs` declares all modules**

```rust
// yushi-cli/src/tui/mod.rs
mod app;
mod event;
pub mod theme;
mod ui;
pub mod widgets;

pub use app::App;
pub use event::{Event, EventHandler};

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

pub async fn run() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new().await?;
    let mut event_handler = EventHandler::new(250);

    let result = run_app(&mut terminal, &mut app, &mut event_handler).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    event_handler: &mut EventHandler,
) -> Result<()>
where
    <B as ratatui::backend::Backend>::Error: Send + Sync + 'static,
{
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if let Some(event) = event_handler.next().await {
            match event {
                Event::Key(key) => {
                    if !app.handle_key(key).await? {
                        app.persist_on_exit().await?;
                        return Ok(());
                    }
                }
                Event::Tick => {
                    app.on_tick().await?;
                }
            }
        }
    }
}
```

- [ ] **Step 2: Full compilation check**

Run: `cargo check -p yushi-cli --features tui`
Expected: compiles cleanly

- [ ] **Step 3: Run all tests**

Run: `cargo test -p yushi-cli`
Expected: all tests pass

- [ ] **Step 4: Run clippy**

Run: `cargo clippy -p yushi-cli --features tui -- -D warnings`
Expected: no warnings

- [ ] **Step 5: Fix any remaining issues**

Address any clippy warnings or compilation errors discovered in steps 2-4.

- [ ] **Step 6: Commit**

```bash
git add yushi-cli/src/tui/
git commit -m "feat(tui): wire all widgets and finalize compilation"
```

---

### Task 15: Manual Smoke Test and Polish

**Files:**
- Potentially any widget file for minor adjustments

- [ ] **Step 1: Run the TUI**

Run: `cargo run -p yushi-cli --features tui -- tui`

Verify:
- Sidebar renders with 3 nav icons and version
- Tasks view shows with filter tabs
- Empty state appears when no tasks
- `1`/`2`/`3` switches views
- `a` opens add task dialog
- Tab cycles filter tabs
- Settings shows grouped cards with theme toggle
- Help bar updates per context
- `q` quits cleanly

- [ ] **Step 2: Fix visual issues**

Adjust spacing, alignment, or color issues discovered during the smoke test. Common issues:
- Emoji width rendering (some terminals render emoji as 2 columns)
- Card border alignment
- Progress bar label centering

- [ ] **Step 3: Run full workspace validation**

Run: `cargo fmt --check && cargo clippy --workspace --all-targets --all-features && cargo test --workspace --all-features`
Expected: all pass

- [ ] **Step 4: Commit final polish**

```bash
git add -A
git commit -m "feat(tui): polish layout and fix visual issues after smoke test"
```
