use anyhow::{Result, anyhow};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;
use yushi_core::{
    AppConfig, DownloadHistory, DownloadTask, DownloaderEvent, Priority, ProgressEvent, QueueEvent,
    TaskEvent, TaskStatus, YuShi, parse_speed_limit,
};

use crate::config::ConfigStore;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    AddUrl,
    EditSetting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedPanel {
    TaskList,
    Details,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentView {
    Tasks,
    History,
    Settings,
}

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

pub const SETTINGS_FIELDS: [SettingField; 9] = [
    SettingField::OutputDir,
    SettingField::Connections,
    SettingField::MaxTasks,
    SettingField::ChunkSize,
    SettingField::Timeout,
    SettingField::UserAgent,
    SettingField::Proxy,
    SettingField::SpeedLimit,
    SettingField::Theme,
];

pub struct App {
    pub queue: YuShi,
    pub config: AppConfig,
    pub history: DownloadHistory,
    pub tasks: Vec<DownloadTask>,
    pub current_view: CurrentView,
    pub selected_index: usize,
    pub history_index: usize,
    pub setting_index: usize,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub selected_panel: SelectedPanel,
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

        Ok(Self {
            queue,
            config,
            history,
            tasks,
            current_view: CurrentView::Tasks,
            selected_index: 0,
            history_index: 0,
            setting_index: 0,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            selected_panel: SelectedPanel::TaskList,
            status_message: "就绪".to_string(),
            event_rx,
        })
    }

    pub async fn persist_on_exit(&self) -> Result<()> {
        self.queue.persist_queue_state().await?;
        Ok(())
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key).await,
            InputMode::AddUrl | InputMode::EditSetting => self.handle_input_key(key).await,
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
            (KeyCode::Tab, KeyModifiers::NONE) => {
                self.selected_panel = match self.selected_panel {
                    SelectedPanel::TaskList => SelectedPanel::Details,
                    SelectedPanel::Details => SelectedPanel::TaskList,
                };
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
                if self.selected_index < self.tasks.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
            }
            (KeyCode::Home | KeyCode::Char('g'), KeyModifiers::NONE) => {
                self.selected_index = 0;
            }
            (KeyCode::End | KeyCode::Char('G'), KeyModifiers::SHIFT) => {
                self.selected_index = self.tasks.len().saturating_sub(1);
            }
            (KeyCode::Char('a'), KeyModifiers::NONE) => {
                self.input_mode = InputMode::AddUrl;
                self.input_buffer.clear();
                self.status_message = "输入 URL (格式: URL|输出路径|优先级|限速)".to_string();
            }
            (KeyCode::Char('p'), KeyModifiers::NONE) => {
                if let Some(task) = self.tasks.get(self.selected_index) {
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
                if let Some(task) = self.tasks.get(self.selected_index)
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
                if let Some(task) = self.tasks.get(self.selected_index)
                    && matches!(
                        task.status,
                        TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
                    )
                {
                    self.queue.remove_task(&task.id).await?;
                    self.status_message = format!("已删除任务: {}", &task.id[..8]);
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                }
            }
            (KeyCode::Char('C'), KeyModifiers::SHIFT) => {
                self.queue.clear_completed().await?;
                self.status_message = "已清空已完成任务".to_string();
                self.selected_index = 0;
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
            (KeyCode::Char('x'), KeyModifiers::NONE) => {
                if let Some(task) = self
                    .history
                    .completed_tasks
                    .get(self.history_index)
                    .cloned()
                {
                    let path = ConfigStore::history_path()?;
                    let (history, removed) =
                        DownloadHistory::remove_from_file(&path, &task.id).await?;
                    if !removed {
                        return Ok(());
                    }
                    self.history = history;
                    if self.history_index > 0
                        && self.history_index >= self.history.completed_tasks.len()
                    {
                        self.history_index -= 1;
                    }
                    self.status_message = format!("已删除历史记录: {}", &task.id[..8]);
                }
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
                self.input_mode = InputMode::EditSetting;
                self.input_buffer = self
                    .selected_setting()
                    .current_value(&self.config)
                    .unwrap_or_default();
                self.status_message = format!(
                    "编辑设置 {}，Enter 保存，Esc 取消",
                    self.selected_setting().label()
                );
            }
            (KeyCode::Char('r'), KeyModifiers::NONE) | (KeyCode::F(5), KeyModifiers::NONE) => {
                self.config = ConfigStore::load().await?;
                self.status_message = "设置已从磁盘重新加载".to_string();
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_input_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Enter => {
                let result = match self.input_mode {
                    InputMode::AddUrl => {
                        self.input_mode = InputMode::Normal;
                        let result = if self.input_buffer.is_empty() {
                            Ok(())
                        } else {
                            self.add_task_from_input().await
                        };
                        self.input_buffer.clear();
                        result
                    }
                    InputMode::EditSetting => {
                        self.input_mode = InputMode::Normal;
                        let value = self.input_buffer.clone();
                        self.input_buffer.clear();
                        self.apply_setting_value(value).await
                    }
                    InputMode::Normal => Ok(()),
                };
                result?;
            }
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.input_buffer.clear();
                self.status_message = "已取消".to_string();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            _ => {}
        }
        Ok(true)
    }

    async fn add_task_from_input(&mut self) -> Result<()> {
        let parts: Vec<&str> = self.input_buffer.split('|').collect();
        if parts.is_empty() {
            self.status_message = "错误: URL 不能为空".to_string();
            return Ok(());
        }

        let url = parts[0].trim().to_string();
        let output = if parts.len() > 1 && !parts[1].trim().is_empty() {
            std::path::PathBuf::from(parts[1].trim())
        } else {
            self.queue
                .infer_destination_in_dir(&url, self.config.default_download_path.clone())
                .await
        };

        let priority = if parts.len() > 2 {
            match parts[2].trim().to_lowercase().as_str() {
                "high" | "h" | "高" => Priority::High,
                "low" | "l" | "低" => Priority::Low,
                _ => Priority::Normal,
            }
        } else {
            Priority::Normal
        };

        let speed_limit = if parts.len() > 3 && !parts[3].trim().is_empty() {
            match parse_speed_limit(parts[3].trim()) {
                Some(limit) => Some(limit),
                None => {
                    self.status_message = format!("无效的速度限制: {}", parts[3].trim());
                    return Ok(());
                }
            }
        } else {
            None
        };

        match self
            .queue
            .add_task_with_options(
                url.clone(),
                output.clone(),
                priority,
                None,
                speed_limit,
                false,
            )
            .await
        {
            Ok(task_id) => {
                self.status_message = format!("已添加任务: {}", &task_id[..8]);
                self.refresh_tasks().await?;
            }
            Err(e) => {
                self.status_message = format!("添加任务失败: {}", e);
            }
        }

        Ok(())
    }

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

    pub async fn on_tick(&mut self) -> Result<()> {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                DownloaderEvent::Progress(ProgressEvent::Updated { task_id, .. }) => {
                    if let Some(task) = self.queue.get_task(&task_id).await
                        && let Some(idx) = self.tasks.iter().position(|t| t.id == task_id)
                    {
                        self.tasks[idx] = task;
                    }
                }
                DownloaderEvent::Task(TaskEvent::Completed { task_id }) => {
                    self.status_message = format!("任务完成: {}", &task_id[..8]);
                    self.refresh_tasks().await?;
                    self.refresh_history().await?;
                }
                DownloaderEvent::Task(TaskEvent::Failed { task_id, error }) => {
                    self.status_message = format!("任务失败: {} - {}", &task_id[..8], error);
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
        if self.selected_index >= self.tasks.len() && !self.tasks.is_empty() {
            self.selected_index = self.tasks.len() - 1;
        }
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

    pub fn get_selected_task(&self) -> Option<&DownloadTask> {
        self.tasks.get(self.selected_index)
    }

    pub fn get_selected_history(&self) -> Option<&yushi_core::CompletedTask> {
        self.history.completed_tasks.get(self.history_index)
    }

    pub fn selected_setting(&self) -> SettingField {
        SETTINGS_FIELDS[self.setting_index]
    }
}

impl SettingField {
    pub fn label(self) -> &'static str {
        match self {
            Self::OutputDir => "默认输出目录",
            Self::Connections => "每任务并发连接数",
            Self::MaxTasks => "最大并发任务数",
            Self::ChunkSize => "分块大小",
            Self::Timeout => "超时",
            Self::UserAgent => "User-Agent",
            Self::Proxy => "代理 URL",
            Self::SpeedLimit => "默认任务限速",
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
            Self::Theme => config.theme.clone(),
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
                            .ok_or_else(|| anyhow!("无效的速度限制: {}", value.trim()))?,
                    )
                };
            }
            Self::Theme => {
                config.theme = value.trim().to_string();
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SettingField;
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
}
