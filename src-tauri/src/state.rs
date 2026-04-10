use anyhow::Result;
use hsi_core::{
    AppConfig, CompletedTask, DownloadHistory, DownloaderEvent, Hsi, TaskStatus, config_path,
    history_path, queue_state_path,
};
use std::path::PathBuf;
use tokio::sync::{RwLock, mpsc};

pub struct AppState {
    pub queue: Hsi,
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

        let (mut queue, event_rx) = Hsi::with_config(
            config.downloader_config(),
            config.max_concurrent_tasks,
            q_path,
            config.bt.clone(),
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
                let _ = DownloadHistory::append_completed_to_file(&history_path, completed).await;
            }
        });

        queue.load_queue_from_state().await?;
        queue.start_pending_tasks().await?;

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

    pub async fn refresh_history_from_disk(&self) -> Result<Vec<CompletedTask>> {
        let history = DownloadHistory::load(&self.history_path).await?;
        let completed = history.get_all().to_vec();
        *self.history.write().await = history;
        Ok(completed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hsi_core::{BtConfig, Config};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!("src-tauri-{name}-{nonce}.json"))
    }

    #[tokio::test]
    async fn refresh_history_from_disk_updates_cached_state() {
        let history_path = temp_file("history");
        let queue_path = temp_file("queue");
        let config_path = temp_file("config");
        let task = CompletedTask {
            id: "done".into(),
            url: "https://example.com/file.bin".into(),
            dest: PathBuf::from("/tmp/file.bin"),
            total_size: 10,
            completed_at: 1,
            duration: 1,
            avg_speed: 10,
        };
        DownloadHistory::append_completed_to_file(&history_path, task.clone())
            .await
            .expect("append history entry");

        let (queue, _event_rx) =
            Hsi::with_config(Config::default(), 1, queue_path.clone(), BtConfig::default());
        let state = AppState {
            queue,
            config: RwLock::new(AppConfig::default()),
            history: RwLock::new(DownloadHistory::default()),
            config_path: config_path.clone(),
            history_path: history_path.clone(),
        };

        let list = state
            .refresh_history_from_disk()
            .await
            .expect("refresh history from disk");

        assert_eq!(list, vec![task.clone()]);
        assert_eq!(state.history.read().await.get_all(), &[task]);

        let _ = tokio::fs::remove_file(history_path).await;
        let _ = tokio::fs::remove_file(queue_path).await;
        let _ = tokio::fs::remove_file(config_path).await;
    }
}
