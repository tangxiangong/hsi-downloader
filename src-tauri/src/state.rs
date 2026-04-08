use anyhow::Result;
use std::path::PathBuf;
use tokio::sync::{RwLock, mpsc};
use yushi_core::{
    AppConfig, CompletedTask, DownloadHistory, DownloaderEvent, TaskStatus, YuShi, config_path,
    history_path, queue_state_path,
};

pub struct AppState {
    pub queue: YuShi,
    pub config: RwLock<AppConfig>,
    pub history: RwLock<DownloadHistory>,
    pub config_path: PathBuf,
    pub history_path: PathBuf,
}

impl AppState {
    pub async fn bootstrap() -> Result<(Self, mpsc::Receiver<DownloaderEvent>)> {
        let cfg_path = config_path()?;
        let hist_path = history_path()?;
        let q_path = queue_state_path()?;

        let config = AppConfig::load(&cfg_path).await?;
        config.save(&cfg_path).await?;
        let history = DownloadHistory::load(&hist_path).await?;

        let (mut queue, event_rx) = YuShi::with_config(
            config.downloader_config(),
            config.max_concurrent_tasks,
            q_path,
        );

        // Install history tracking callback
        let queue_for_history = queue.clone();
        let hist_path_clone = hist_path.clone();
        queue.set_on_complete(move |task_id, result| {
            let queue = queue_for_history.clone();
            let history_path = hist_path_clone.clone();
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
                let _ =
                    DownloadHistory::append_completed_to_file(&history_path, completed).await;
            }
        });

        queue.load_queue_from_state().await?;

        Ok((
            Self {
                queue,
                config: RwLock::new(config),
                history: RwLock::new(history),
                config_path: cfg_path,
                history_path: hist_path,
            },
            event_rx,
        ))
    }
}
