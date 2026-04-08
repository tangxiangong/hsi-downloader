use anyhow::{Result, anyhow};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;
use yushi_core::{
    AppConfig, DownloadHistory, DownloadTask, DownloaderEvent, Priority, ProgressEvent,
    QueueEvent, TaskEvent, TaskStatus, YuShi, parse_speed_limit,
};
use yushi_core::config::AppTheme;

use crate::config::ConfigStore;
use crate::tui::theme::ThemeColors;

// ==================== Enums ====================

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    AddTask,
    EditSetting,
    Confirm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddTaskField {
    Url,
    Path,
    Priority,
    SpeedLimit,
    Buttons,
}

impl AddTaskField {
    fn next(self) -> Self {
        match self {
            Self::Url => Self::Path,
            Self::Path => Self::Priority,
            Self::Priority => Self::SpeedLimit,
            Self::SpeedLimit => Self::Buttons,
            Self::Buttons => Self::Url,
        }
    }

    fn prev(self) -> Self {
        match self {
            Self::Url => Self::Buttons,
            Self::Path => Self::Url,
            Self::Priority => Self::Path,
            Self::SpeedLimit => Self::Priority,
            Self::Buttons => Self::SpeedLimit,
        }
    }
}

/// State for the "Add Task" dialog.
#[derive(Debug, Clone)]
pub struct AddTaskState {
    pub url: String,
    pub path: String,
    pub priority: Priority,
    pub speed_limit: String,
    pub focused_field: AddTaskField,
    pub error: Option<String>,
    /// When `focused_field == Buttons`, true = Confirm button, false = Cancel button.
    pub button_confirm: bool,
}

impl Default for AddTaskState {
    fn default() -> Self {
        Self {
            url: String::new(),
            path: String::new(),
            priority: Priority::Normal,
            speed_limit: String::new(),
            focused_field: AddTaskField::Url,
            error: None,
            button_confirm: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    ClearHistory,
    DeleteTaskWithFile { task_id: String },
}

#[derive(Debug, Clone)]
pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
    /// true = "确认" selected, false = "取消" selected
    pub selected_confirm: bool,
    pub action: ConfirmAction,
}

// ==================== Views ====================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentView {
    Tasks,
    History,
    Settings,
}

// ==================== Settings ====================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingField {
    OutputDir,
    Connections,
    MaxTasks,
    Proxy,
    UserAgent,
    Timeout,
    ChunkSize,
    SpeedLimit,
    Theme,
}

/// Order matches group layout: 下载 → 网络 → 外观
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

/// A logical grouping of settings fields shown together in the UI.
#[derive(Debug, Clone)]
pub struct SettingsGroup {
    pub title: &'static str,
    pub fields: &'static [SettingField],
}

pub const SETTINGS_GROUPS: [SettingsGroup; 3] = [
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

// ==================== App ====================

pub struct App {
    pub queue: YuShi,
    pub config: AppConfig,
    pub history: DownloadHistory,
    pub tasks: Vec<DownloadTask>,
    pub current_view: CurrentView,

    // Task list
    pub selected_index: usize,
    pub filter: TaskFilter,
    pub filtered_indices: Vec<usize>,

    // History list
    pub history_index: usize,

    // Settings
    pub setting_index: usize,
    pub edit_buffer: String,

    // Dialogs / overlays
    pub add_task_state: Option<AddTaskState>,
    pub confirm_dialog: Option<ConfirmDialog>,

    // Mode
    pub input_mode: InputMode,

    // Theme
    pub theme: ThemeColors,

    // Status bar
    pub status_message: String,

    event_rx: mpsc::Receiver<QueueEvent>,
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
            current_view: CurrentView::Tasks,
            selected_index: 0,
            filter: TaskFilter::All,
            filtered_indices: Vec::new(),

            history_index: 0,
            setting_index: 0,
            edit_buffer: String::new(),
            add_task_state: None,
            confirm_dialog: None,
            input_mode: InputMode::Normal,
            theme,
            status_message: "就绪".to_string(),
            event_rx,
        };
        app.recompute_filtered();
        Ok(app)
    }

    // -------------------- Filtering --------------------

    pub fn recompute_filtered(&mut self) {
        self.filtered_indices = self
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, t)| self.filter.matches(t.status))
            .map(|(i, _)| i)
            .collect();

        // Clamp selection to the new filtered list length.
        let len = self.filtered_indices.len();
        if len == 0 {
            self.selected_index = 0;
        } else if self.selected_index >= len {
            self.selected_index = len - 1;
        }
    }

    pub fn selected_task(&self) -> Option<&DownloadTask> {
        self.filtered_indices
            .get(self.selected_index)
            .and_then(|&i| self.tasks.get(i))
    }

    pub fn filter_count(&self, filter: TaskFilter) -> usize {
        self.tasks
            .iter()
            .filter(|t| filter.matches(t.status))
            .count()
    }

    // -------------------- Persistence --------------------

    pub async fn persist_on_exit(&self) -> Result<()> {
        self.queue.persist_queue_state().await?;
        Ok(())
    }

    // -------------------- Key dispatch --------------------

    pub async fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key).await,
            InputMode::AddTask => self.handle_add_task_key(key).await,
            InputMode::EditSetting => self.handle_edit_setting_key(key).await,
            InputMode::Confirm => self.handle_confirm_key(key).await,
        }
    }

    async fn handle_normal_key(&mut self, key: KeyEvent) -> Result<bool> {
        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), KeyModifiers::NONE)
            | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                return Ok(false);
            }
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
            _ => match self.current_view {
                CurrentView::Tasks => self.handle_tasks_key(key).await?,
                CurrentView::History => self.handle_history_key(key).await?,
                CurrentView::Settings => self.handle_settings_key(key).await?,
            },
        }
        Ok(true)
    }

    async fn handle_tasks_key(&mut self, key: KeyEvent) -> Result<()> {
        match (key.code, key.modifiers) {
            (KeyCode::Up | KeyCode::Char('k'), KeyModifiers::NONE) => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            (KeyCode::Down | KeyCode::Char('j'), KeyModifiers::NONE) => {
                let max = self.filtered_indices.len().saturating_sub(1);
                if self.selected_index < max {
                    self.selected_index += 1;
                }
            }
            (KeyCode::Home | KeyCode::Char('g'), KeyModifiers::NONE) => {
                self.selected_index = 0;
            }
            (KeyCode::End | KeyCode::Char('G'), KeyModifiers::SHIFT) => {
                self.selected_index = self.filtered_indices.len().saturating_sub(1);
            }
            // Cycle filter: Tab / Left / Right
            (KeyCode::Tab, KeyModifiers::NONE) | (KeyCode::Right, KeyModifiers::NONE) => {
                self.filter = self.filter.next();
                self.selected_index = 0;
                self.recompute_filtered();
            }
            (KeyCode::Left, KeyModifiers::NONE) => {
                self.filter = self.filter.prev();
                self.selected_index = 0;
                self.recompute_filtered();
            }
            (KeyCode::Char('a'), KeyModifiers::NONE) => {
                self.add_task_state = Some(AddTaskState::default());
                self.input_mode = InputMode::AddTask;
                self.status_message = "添加新任务".to_string();
            }
            (KeyCode::Char('p'), KeyModifiers::NONE) => {
                if let Some(task) = self.selected_task().cloned() {
                    match task.status {
                        TaskStatus::Downloading => {
                            self.queue.pause_task(&task.id).await?;
                            self.status_message = format!("已暂停任务: {}", &task.id[..8]);
                        }
                        TaskStatus::Paused => {
                            self.queue.resume_task(&task.id).await?;
                            self.status_message = format!("已恢复任务: {}", &task.id[..8]);
                        }
                        _ => {}
                    }
                }
            }
            (KeyCode::Char('c'), KeyModifiers::NONE) => {
                if let Some(task) = self.selected_task().cloned()
                    && matches!(
                        task.status,
                        TaskStatus::Pending | TaskStatus::Downloading | TaskStatus::Paused
                    )
                {
                    self.queue.cancel_task(&task.id).await?;
                    self.status_message = format!("已取消任务: {}", &task.id[..8]);
                }
            }
            (KeyCode::Char('d'), KeyModifiers::NONE) => {
                if let Some(task) = self.selected_task().cloned()
                    && matches!(
                        task.status,
                        TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
                    )
                {
                    self.queue.remove_task(&task.id).await?;
                    self.status_message = format!("已删除任务: {}", &task.id[..8]);
                    self.refresh_tasks().await?;
                }
            }
            // 'D' — delete with file (confirm first)
            (KeyCode::Char('D'), KeyModifiers::SHIFT) => {
                if let Some(task) = self.selected_task().cloned() {
                    let short = task.id[..8.min(task.id.len())].to_string();
                    let filename = task
                        .dest
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("(unknown)")
                        .to_string();
                    self.confirm_dialog = Some(ConfirmDialog {
                        title: "删除任务及文件".to_string(),
                        message: format!(
                            "确定要删除任务 {} 及其本地文件 \"{}\" 吗？此操作不可撤销。",
                            short, filename
                        ),
                        selected_confirm: false,
                        action: ConfirmAction::DeleteTaskWithFile { task_id: task.id },
                    });
                    self.input_mode = InputMode::Confirm;
                }
            }
            (KeyCode::Char('r'), KeyModifiers::NONE) | (KeyCode::F(5), KeyModifiers::NONE) => {
                self.refresh_tasks().await?;
                self.status_message = "已刷新".to_string();
            }
            _ => {}
        }
        Ok(())
    }

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
                self.status_message = "历史记录已刷新".to_string();
            }
            // 'C' — clear history with confirmation
            (KeyCode::Char('C'), KeyModifiers::SHIFT) => {
                self.confirm_dialog = Some(ConfirmDialog {
                    title: "清空历史记录".to_string(),
                    message: "确定要清空所有下载历史记录吗？此操作不可撤销。".to_string(),
                    selected_confirm: false,
                    action: ConfirmAction::ClearHistory,
                });
                self.input_mode = InputMode::Confirm;
            }
            _ => {}
        }
        Ok(())
    }

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
                    self.cycle_theme().await?;
                } else {
                    self.input_mode = InputMode::EditSetting;
                    self.edit_buffer = field.current_value(&self.config).unwrap_or_default();
                    self.status_message = format!(
                        "编辑设置 {}，Enter 保存，Esc 取消",
                        field.label()
                    );
                }
            }
            // Left / Right on Theme field also cycles the theme.
            (KeyCode::Left | KeyCode::Right, KeyModifiers::NONE) => {
                if self.selected_setting() == SettingField::Theme {
                    self.cycle_theme().await?;
                }
            }
            (KeyCode::Char('r'), KeyModifiers::NONE) | (KeyCode::F(5), KeyModifiers::NONE) => {
                self.config = ConfigStore::load().await?;
                self.theme = ThemeColors::from_app_theme(self.config.theme);
                self.status_message = "设置已从磁盘重新加载".to_string();
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_edit_setting_key(&mut self, key: KeyEvent) -> Result<bool> {
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
        Ok(true)
    }

    async fn handle_add_task_key(&mut self, key: KeyEvent) -> Result<bool> {
        let state = match self.add_task_state.as_mut() {
            Some(s) => s,
            None => {
                self.input_mode = InputMode::Normal;
                return Ok(true);
            }
        };

        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => {
                self.add_task_state = None;
                self.input_mode = InputMode::Normal;
                self.status_message = "已取消添加任务".to_string();
            }
            (KeyCode::Tab, KeyModifiers::NONE) => {
                state.focused_field = state.focused_field.next();
            }
            (KeyCode::BackTab, KeyModifiers::SHIFT) => {
                state.focused_field = state.focused_field.prev();
            }
            (KeyCode::Left, KeyModifiers::NONE) => match state.focused_field {
                AddTaskField::Priority => {
                    state.priority = match state.priority {
                        Priority::Normal => Priority::Low,
                        Priority::High => Priority::Normal,
                        Priority::Low => Priority::Low,
                    };
                }
                AddTaskField::Buttons => {
                    state.button_confirm = true;
                }
                _ => {}
            },
            (KeyCode::Right, KeyModifiers::NONE) => match state.focused_field {
                AddTaskField::Priority => {
                    state.priority = match state.priority {
                        Priority::Low => Priority::Normal,
                        Priority::Normal => Priority::High,
                        Priority::High => Priority::High,
                    };
                }
                AddTaskField::Buttons => {
                    state.button_confirm = false;
                }
                _ => {}
            },
            (KeyCode::Enter, KeyModifiers::NONE) => match state.focused_field {
                AddTaskField::Buttons => {
                    if state.button_confirm {
                        // Move state out and submit.
                        let submit_state = self.add_task_state.take().unwrap();
                        self.input_mode = InputMode::Normal;
                        self.submit_add_task(submit_state).await?;
                    } else {
                        self.add_task_state = None;
                        self.input_mode = InputMode::Normal;
                        self.status_message = "已取消添加任务".to_string();
                    }
                }
                _ => {
                    // Advance to next field on Enter for text fields.
                    state.focused_field = state.focused_field.next();
                }
            },
            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                match state.focused_field {
                    AddTaskField::Url => state.url.push(c),
                    AddTaskField::Path => state.path.push(c),
                    AddTaskField::SpeedLimit => state.speed_limit.push(c),
                    _ => {}
                }
            }
            (KeyCode::Backspace, _) => match state.focused_field {
                AddTaskField::Url => {
                    state.url.pop();
                }
                AddTaskField::Path => {
                    state.path.pop();
                }
                AddTaskField::SpeedLimit => {
                    state.speed_limit.pop();
                }
                _ => {}
            },
            _ => {}
        }
        Ok(true)
    }

    async fn submit_add_task(&mut self, state: AddTaskState) -> Result<()> {
        let url = state.url.trim().to_string();
        if url.is_empty() {
            // Restore dialog with error.
            let mut s = state;
            s.error = Some("URL 不能为空".to_string());
            s.focused_field = AddTaskField::Url;
            self.add_task_state = Some(s);
            self.input_mode = InputMode::AddTask;
            return Ok(());
        }

        let output = if state.path.trim().is_empty() {
            self.queue
                .infer_destination_in_dir(&url, self.config.default_download_path.clone())
                .await
        } else {
            std::path::PathBuf::from(state.path.trim())
        };

        let speed_limit = if state.speed_limit.trim().is_empty() {
            None
        } else {
            match parse_speed_limit(state.speed_limit.trim()) {
                Some(limit) => Some(limit),
                None => {
                    let mut s = state;
                    s.error = Some(format!("无效的速度限制: {}", s.speed_limit.trim()));
                    s.focused_field = AddTaskField::SpeedLimit;
                    self.add_task_state = Some(s);
                    self.input_mode = InputMode::AddTask;
                    return Ok(());
                }
            }
        };

        match self
            .queue
            .add_task_with_options(url.clone(), output, state.priority, None, speed_limit, false)
            .await
        {
            Ok(task_id) => {
                self.status_message = format!("已添加任务: {}", &task_id[..8.min(task_id.len())]);
                self.refresh_tasks().await?;
            }
            Err(e) => {
                self.status_message = format!("添加任务失败: {}", e);
            }
        }
        Ok(())
    }

    async fn handle_confirm_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Left => {
                if let Some(d) = self.confirm_dialog.as_mut() {
                    d.selected_confirm = true;
                }
            }
            KeyCode::Right => {
                if let Some(d) = self.confirm_dialog.as_mut() {
                    d.selected_confirm = false;
                }
            }
            KeyCode::Enter => {
                let dialog = match self.confirm_dialog.take() {
                    Some(d) => d,
                    None => {
                        self.input_mode = InputMode::Normal;
                        return Ok(true);
                    }
                };
                self.input_mode = InputMode::Normal;
                if dialog.selected_confirm {
                    self.execute_confirm_action(dialog.action).await?;
                } else {
                    self.status_message = "已取消".to_string();
                }
            }
            KeyCode::Esc => {
                self.confirm_dialog = None;
                self.input_mode = InputMode::Normal;
                self.status_message = "已取消".to_string();
            }
            _ => {}
        }
        Ok(true)
    }

    async fn execute_confirm_action(&mut self, action: ConfirmAction) -> Result<()> {
        match action {
            ConfirmAction::ClearHistory => {
                let path = ConfigStore::history_path()?;
                let empty = DownloadHistory::default();
                empty.save(&path).await?;
                self.history = empty;
                self.history_index = 0;
                self.status_message = "历史记录已清空".to_string();
            }
            ConfirmAction::DeleteTaskWithFile { task_id } => {
                self.queue.remove_task_with_file(&task_id).await?;
                self.status_message =
                    format!("已删除任务及文件: {}", &task_id[..8.min(task_id.len())]);
                self.refresh_tasks().await?;
            }
        }
        Ok(())
    }

    // -------------------- Theme cycling --------------------

    async fn cycle_theme(&mut self) -> Result<()> {
        self.config.theme = match self.config.theme {
            AppTheme::Light => AppTheme::Dark,
            AppTheme::Dark => AppTheme::System,
            AppTheme::System => AppTheme::Light,
        };
        self.theme = ThemeColors::from_app_theme(self.config.theme);
        ConfigStore::save(&self.config).await?;
        self.status_message = format!("主题已切换为: {}", self.config.theme);
        Ok(())
    }

    // -------------------- Settings helpers --------------------

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
        self.status_message = format!("已保存设置: {}", field.label());
        Ok(())
    }

    pub fn selected_setting(&self) -> SettingField {
        SETTINGS_FIELDS[self.setting_index]
    }

    // -------------------- Tick / refresh --------------------

    pub async fn on_tick(&mut self) -> Result<()> {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                DownloaderEvent::Progress(ProgressEvent::Updated { task_id, .. }) => {
                    if let Some(task) = self.queue.get_task(&task_id).await
                        && let Some(idx) = self.tasks.iter().position(|t| t.id == task_id)
                    {
                        self.tasks[idx] = task;
                        self.recompute_filtered();
                    }
                }
                DownloaderEvent::Task(TaskEvent::Completed { task_id }) => {
                    self.status_message = format!("任务完成: {}", &task_id[..8.min(task_id.len())]);
                    self.refresh_tasks().await?;
                    self.refresh_history().await?;
                }
                DownloaderEvent::Task(TaskEvent::Failed { task_id, error }) => {
                    self.status_message =
                        format!("任务失败: {} - {}", &task_id[..8.min(task_id.len())], error);
                    self.refresh_tasks().await?;
                }
                DownloaderEvent::Task(TaskEvent::Added { .. })
                | DownloaderEvent::Task(TaskEvent::Paused { .. })
                | DownloaderEvent::Task(TaskEvent::Resumed { .. })
                | DownloaderEvent::Task(TaskEvent::Cancelled { .. })
                | DownloaderEvent::Task(TaskEvent::Started { .. }) => {
                    self.refresh_tasks().await?;
                }
                _ => {}
            }
        }
        Ok(())
    }

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

}

// ==================== SettingField impl ====================

impl SettingField {
    pub fn label(self) -> &'static str {
        match self {
            Self::OutputDir => "默认下载路径",
            Self::Connections => "单任务连接数",
            Self::MaxTasks => "最大并发任务数",
            Self::Proxy => "代理 URL",
            Self::UserAgent => "User-Agent",
            Self::Timeout => "超时（秒）",
            Self::ChunkSize => "分块大小（字节）",
            Self::SpeedLimit => "默认限速",
            Self::Theme => "主题",
        }
    }

    pub fn current_value(self, config: &AppConfig) -> Option<String> {
        Some(match self {
            Self::OutputDir => config.default_download_path.display().to_string(),
            Self::Connections => config.max_concurrent_downloads.to_string(),
            Self::MaxTasks => config.max_concurrent_tasks.to_string(),
            Self::Proxy => config.proxy.clone().unwrap_or_default(),
            Self::UserAgent => config.user_agent.clone(),
            Self::Timeout => config.timeout.to_string(),
            Self::ChunkSize => config.chunk_size.to_string(),
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
            Self::Proxy => {
                config.proxy = if value.trim().is_empty() {
                    None
                } else {
                    Some(value.trim().to_string())
                };
            }
            Self::UserAgent => {
                config.user_agent = value.to_string();
            }
            Self::Timeout => {
                config.timeout = value.trim().parse()?;
            }
            Self::ChunkSize => {
                config.chunk_size = value.trim().parse()?;
            }
            Self::SpeedLimit => {
                config.speed_limit = if value.trim().is_empty() {
                    None
                } else {
                    Some(
                        parse_speed_limit(value.trim())
                            .ok_or_else(|| anyhow!("无效的速度限制: {}", value.trim()))?,
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

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::{AddTaskField, AddTaskState, SettingField, TaskFilter};
    use yushi_core::{AppConfig, TaskStatus};

    // --- Pre-existing tests ---

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

    // --- New tests ---

    #[test]
    fn task_filter_matches() {
        assert!(TaskFilter::All.matches(TaskStatus::Pending));
        assert!(TaskFilter::All.matches(TaskStatus::Completed));

        assert!(TaskFilter::Downloading.matches(TaskStatus::Pending));
        assert!(TaskFilter::Downloading.matches(TaskStatus::Downloading));
        assert!(TaskFilter::Downloading.matches(TaskStatus::Paused));
        assert!(!TaskFilter::Downloading.matches(TaskStatus::Completed));
        assert!(!TaskFilter::Downloading.matches(TaskStatus::Failed));

        assert!(TaskFilter::Completed.matches(TaskStatus::Completed));
        assert!(TaskFilter::Completed.matches(TaskStatus::Failed));
        assert!(TaskFilter::Completed.matches(TaskStatus::Cancelled));
        assert!(!TaskFilter::Completed.matches(TaskStatus::Pending));
        assert!(!TaskFilter::Completed.matches(TaskStatus::Downloading));
    }

    #[test]
    fn task_filter_cycle() {
        let f = TaskFilter::All;
        assert_eq!(f.next(), TaskFilter::Downloading);
        assert_eq!(f.next().next(), TaskFilter::Completed);
        assert_eq!(f.next().next().next(), TaskFilter::All);

        assert_eq!(f.prev(), TaskFilter::Completed);
        assert_eq!(f.prev().prev(), TaskFilter::Downloading);
        assert_eq!(f.prev().prev().prev(), TaskFilter::All);
    }

    #[test]
    fn add_task_state_focus_cycle() {
        let mut state = AddTaskState::default();
        assert_eq!(state.focused_field, AddTaskField::Url);

        state.focused_field = state.focused_field.next();
        assert_eq!(state.focused_field, AddTaskField::Path);

        state.focused_field = state.focused_field.next();
        assert_eq!(state.focused_field, AddTaskField::Priority);

        state.focused_field = state.focused_field.next();
        assert_eq!(state.focused_field, AddTaskField::SpeedLimit);

        state.focused_field = state.focused_field.next();
        assert_eq!(state.focused_field, AddTaskField::Buttons);

        state.focused_field = state.focused_field.next();
        assert_eq!(state.focused_field, AddTaskField::Url);

        // Reverse
        state.focused_field = state.focused_field.prev();
        assert_eq!(state.focused_field, AddTaskField::Buttons);
    }
}
