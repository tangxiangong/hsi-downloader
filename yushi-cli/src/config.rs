use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use yushi_core::{
    AppConfig, CompletedTask, DownloadHistory, DownloaderEvent, TaskStatus, YuShi, config_path,
    history_path, queue_state_path,
};

pub struct ConfigStore;

impl ConfigStore {
    pub async fn load() -> Result<AppConfig> {
        let path = config_path().context("unable to resolve shared config path")?;
        let mut config = AppConfig::load(&path).await?;

        if !config.default_download_path.is_absolute() {
            config.default_download_path = make_absolute(&config.default_download_path)?;
        }
        // Normalize persisted config into the shared schema after compatibility loading.
        config.save(&path).await?;

        Ok(config)
    }

    pub async fn save(config: &AppConfig) -> Result<()> {
        let path = config_path().context("unable to resolve shared config path")?;
        config.save(&path).await?;
        Ok(())
    }

    pub fn config_path() -> Result<PathBuf> {
        config_path().context("unable to resolve shared config path")
    }

    pub fn history_path() -> Result<PathBuf> {
        history_path().context("unable to resolve shared history path")
    }

    pub fn queue_state_path() -> Result<PathBuf> {
        queue_state_path().context("unable to resolve shared queue path")
    }

    pub async fn build_queue(
        config: &AppConfig,
        connections: Option<usize>,
        max_tasks: Option<usize>,
    ) -> Result<(YuShi, tokio::sync::mpsc::Receiver<DownloaderEvent>)> {
        let mut downloader_config = config.downloader_config();
        if let Some(connections) = connections {
            downloader_config.max_concurrent = connections;
        }

        let queue_path = queue_state_path().context("unable to resolve shared queue path")?;
        let mut queue = YuShi::with_config(
            downloader_config,
            max_tasks.unwrap_or(config.max_concurrent_tasks),
            queue_path,
        );

        install_history_tracking(&mut queue.0);
        Ok(queue)
    }
}

fn install_history_tracking(queue: &mut YuShi) {
    let queue_for_history = queue.clone();
    queue.set_on_complete(move |task_id, result| {
        let queue = queue_for_history.clone();
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
            let Some(completed_task) = CompletedTask::from_task(&task) else {
                return;
            };

            let Ok(path) = history_path() else {
                return;
            };
            let _ = DownloadHistory::append_completed_to_file(&path, completed_task).await;
        }
    });
}

fn make_absolute(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }

    Ok(std::env::current_dir()?.join(path))
}
