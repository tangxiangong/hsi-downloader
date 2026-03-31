use anyhow::Result;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::mpsc;
use yushi_core::{
    AppConfig, CompletedTask, DownloadHistory, DownloadTask, DownloaderEvent, TaskStatus, YuShi,
    config_path, history_path, queue_state_path,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewKind {
    AllTasks,
    Downloading,
    Completed,
    History,
    Settings,
}

pub struct AppState {
    pub queue: Arc<YuShi>,
    pub config: AppConfig,
    pub history: DownloadHistory,
    pub tasks: Vec<DownloadTask>,
    pub current_view: ViewKind,
    pub config_path: PathBuf,
    pub history_path: PathBuf,
    pub status_message: Option<String>,
}

impl AppState {
    pub async fn bootstrap() -> Result<(Self, mpsc::Receiver<DownloaderEvent>)> {
        let config_path = config_path()?;
        let history_path = history_path()?;
        let queue_path = queue_state_path()?;

        let config = AppConfig::load(&config_path).await?;
        config.save(&config_path).await?;
        let history = DownloadHistory::load(&history_path).await?;

        let (mut queue, event_rx) = YuShi::with_config(
            config.downloader_config(),
            config.max_concurrent_tasks,
            queue_path,
        );
        install_history_tracking(&mut queue, history_path.clone());

        queue.load_queue_from_state().await?;
        let tasks = queue.get_all_tasks().await;

        Ok((
            Self {
                queue: Arc::new(queue),
                config,
                history,
                tasks,
                current_view: ViewKind::AllTasks,
                config_path,
                history_path,
                status_message: None,
            },
            event_rx,
        ))
    }
}

fn install_history_tracking(queue: &mut YuShi, history_path: PathBuf) {
    let queue_for_history = queue.clone();
    queue.set_on_complete(move |task_id, result| {
        let queue = queue_for_history.clone();
        let history_path = history_path.clone();
        async move {
            if result.is_err() {
                return;
            }

            let Some(task) = queue.get_task(&task_id).await else {
                return;
            };
            if task.status != TaskStatus::Completed {
                return;
            }

            let Some(completed) = CompletedTask::from_task(&task) else {
                return;
            };

            let _ = DownloadHistory::append_completed_to_file(&history_path, completed).await;
        }
    });
}
