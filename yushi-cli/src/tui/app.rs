use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;
use yushi_core::{
    DownloadTask, DownloaderEvent, Priority, ProgressEvent, QueueEvent, TaskEvent, TaskStatus,
    YuShi,
};

use crate::config::{ConfigStore, default_output_for_url};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    AddUrl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedPanel {
    TaskList,
    Details,
}

pub struct App {
    pub queue: YuShi,
    pub tasks: Vec<DownloadTask>,
    pub selected_index: usize,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub selected_panel: SelectedPanel,
    pub status_message: String,
    event_rx: mpsc::Receiver<QueueEvent>,
}

impl App {
    pub async fn new() -> Result<Self> {
        let config = ConfigStore::load().await?;
        let (queue, event_rx) = ConfigStore::build_queue(&config, None, None).await?;
        queue.load_queue_from_state().await?;
        let tasks = queue.get_all_tasks().await;

        Ok(Self {
            queue,
            tasks,
            selected_index: 0,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            selected_panel: SelectedPanel::TaskList,
            status_message: "就绪".to_string(),
            event_rx,
        })
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key).await,
            InputMode::AddUrl => self.handle_input_key(key).await,
        }
    }

    async fn handle_normal_key(&mut self, key: KeyEvent) -> Result<bool> {
        match (key.code, key.modifiers) {
            // 退出
            (KeyCode::Char('q'), KeyModifiers::NONE)
            | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                return Ok(false);
            }
            // 导航
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
            // 切换面板
            (KeyCode::Tab, KeyModifiers::NONE) => {
                self.selected_panel = match self.selected_panel {
                    SelectedPanel::TaskList => SelectedPanel::Details,
                    SelectedPanel::Details => SelectedPanel::TaskList,
                };
            }
            // 添加任务
            (KeyCode::Char('a'), KeyModifiers::NONE) => {
                self.input_mode = InputMode::AddUrl;
                self.input_buffer.clear();
                self.status_message = "输入 URL (格式: URL|输出路径|优先级)".to_string();
            }
            // 暂停/恢复
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
            // 取消任务
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
            // 删除任务
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
            // 清空已完成
            (KeyCode::Char('C'), KeyModifiers::SHIFT) => {
                self.queue.clear_completed().await?;
                self.status_message = "已清空已完成任务".to_string();
                self.selected_index = 0;
            }
            // 刷新
            (KeyCode::Char('r'), KeyModifiers::NONE) | (KeyCode::F(5), KeyModifiers::NONE) => {
                self.refresh_tasks().await?;
                self.status_message = "已刷新".to_string();
            }
            _ => {}
        }
        Ok(true)
    }

    async fn handle_input_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                if !self.input_buffer.is_empty() {
                    self.add_task_from_input().await?;
                }
                self.input_buffer.clear();
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
        let output = if parts.len() > 1 {
            std::path::PathBuf::from(parts[1].trim())
        } else {
            let config = ConfigStore::load().await?;
            default_output_for_url(&config, &url)
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

        match self
            .queue
            .add_task_with_options(url.clone(), output.clone(), priority, None, false)
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

    pub async fn on_tick(&mut self) -> Result<()> {
        // 处理队列事件
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                DownloaderEvent::Progress(ProgressEvent::Updated { task_id, .. }) => {
                    // 更新任务进度
                    if let Some(task) = self.queue.get_task(&task_id).await
                        && let Some(idx) = self.tasks.iter().position(|t| t.id == task_id)
                    {
                        self.tasks[idx] = task;
                    }
                }
                DownloaderEvent::Task(TaskEvent::Completed { task_id }) => {
                    self.status_message = format!("任务完成: {}", &task_id[..8]);
                    self.refresh_tasks().await?;
                }
                DownloaderEvent::Task(TaskEvent::Failed { task_id, error }) => {
                    self.status_message = format!("任务失败: {} - {}", &task_id[..8], error);
                    self.refresh_tasks().await?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    async fn refresh_tasks(&mut self) -> Result<()> {
        self.tasks = self.queue.get_all_tasks().await;
        if self.selected_index >= self.tasks.len() && !self.tasks.is_empty() {
            self.selected_index = self.tasks.len() - 1;
        }
        Ok(())
    }

    pub fn get_selected_task(&self) -> Option<&DownloadTask> {
        self.tasks.get(self.selected_index)
    }
}
