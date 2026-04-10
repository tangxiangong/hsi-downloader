use crate::{
    Result, storage,
    types::{Task, TaskStatus},
};
use serde::{Deserialize, Serialize};
use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

/// 已完成的下载任务记录
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompletedTask {
    /// 任务 ID
    pub id: String,
    /// 下载 URL
    pub url: String,
    /// 保存路径
    pub dest: PathBuf,
    /// 文件大小（字节）
    pub total_size: u64,
    /// 完成时间戳
    pub completed_at: u64,
    /// 下载耗时（秒）
    pub duration: u64,
    /// 平均速度（字节/秒）
    pub avg_speed: u64,
}

impl CompletedTask {
    pub fn from_task(task: &Task) -> Option<Self> {
        if task.status != TaskStatus::Completed {
            return None;
        }

        let completed_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let duration = completed_at.saturating_sub(task.created_at).max(1);

        Some(Self {
            id: task.id.clone(),
            url: task.url.clone(),
            dest: task.dest.clone(),
            total_size: task.total_size,
            completed_at,
            duration,
            avg_speed: if task.total_size == 0 {
                task.speed
            } else {
                task.total_size / duration
            },
        })
    }
}

/// 下载历史记录
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DownloadHistory {
    /// 已完成的任务列表
    pub completed_tasks: Vec<CompletedTask>,
    /// 最大历史记录数
    pub max_history: usize,
}

impl Default for DownloadHistory {
    fn default() -> Self {
        Self {
            completed_tasks: Vec::new(),
            max_history: 100,
        }
    }
}

impl DownloadHistory {
    pub fn add_completed(&mut self, task: CompletedTask) {
        self.completed_tasks
            .retain(|existing| existing.id != task.id);
        self.completed_tasks.insert(0, task);

        if self.completed_tasks.len() > self.max_history {
            self.completed_tasks.truncate(self.max_history);
        }
    }

    pub fn clear(&mut self) {
        self.completed_tasks.clear();
    }

    pub fn remove(&mut self, id: &str) -> bool {
        if let Some(pos) = self.completed_tasks.iter().position(|t| t.id == id) {
            self.completed_tasks.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn get_all(&self) -> &[CompletedTask] {
        &self.completed_tasks
    }

    pub fn search(&self, query: &str) -> Vec<CompletedTask> {
        let query_lower = query.to_lowercase();
        self.completed_tasks
            .iter()
            .filter(|task| {
                task.url.to_lowercase().contains(&query_lower)
                    || task
                        .dest
                        .to_string_lossy()
                        .to_lowercase()
                        .contains(&query_lower)
            })
            .cloned()
            .collect()
    }

    pub async fn load(path: &Path) -> Result<Self> {
        storage::migrate_legacy_file(path).await?;

        if path.exists() {
            let content = fs_err::tokio::read_to_string(path).await?;
            let history = serde_json::from_str(&content)?;
            Ok(history)
        } else {
            Ok(Self::default())
        }
    }

    pub async fn save(&self, path: &Path) -> Result<()> {
        let _lock = storage::acquire_file_lock(path).await?;
        self.save_unlocked(path).await
    }

    async fn save_unlocked(&self, path: &Path) -> Result<()> {
        storage::ensure_parent_dir(path).await?;
        let content = serde_json::to_string_pretty(self)?;
        storage::atomic_write_string(path, &content).await?;
        Ok(())
    }

    pub async fn append_completed_to_file(path: &Path, task: CompletedTask) -> Result<Self> {
        let (history, _) = Self::mutate_file(path, move |history| {
            history.add_completed(task);
        })
        .await?;
        Ok(history)
    }

    pub async fn remove_from_file(path: &Path, id: &str) -> Result<(Self, bool)> {
        Self::mutate_file(path, |history| history.remove(id)).await
    }

    pub async fn remove_entry_and_file_from_file(path: &Path, id: &str) -> Result<(Self, bool)> {
        storage::migrate_legacy_file(path).await?;
        let _lock = storage::acquire_file_lock(path).await?;

        let mut history = if path.exists() {
            let content = fs_err::tokio::read_to_string(path).await?;
            serde_json::from_str(&content)?
        } else {
            Self::default()
        };

        let Some(pos) = history
            .completed_tasks
            .iter()
            .position(|task| task.id == id)
        else {
            return Ok((history, false));
        };

        let task = history.completed_tasks[pos].clone();
        remove_file_if_exists(&task.dest).await?;
        history.completed_tasks.remove(pos);
        history.save_unlocked(path).await?;

        Ok((history, true))
    }

    pub async fn clear_file(path: &Path) -> Result<Self> {
        let (history, _) = Self::mutate_file(path, |history| history.clear()).await?;
        Ok(history)
    }

    async fn mutate_file<R, F>(path: &Path, mutate: F) -> Result<(Self, R)>
    where
        F: FnOnce(&mut Self) -> R,
    {
        storage::migrate_legacy_file(path).await?;
        let _lock = storage::acquire_file_lock(path).await?;

        let mut history = if path.exists() {
            let content = fs_err::tokio::read_to_string(path).await?;
            serde_json::from_str(&content)?
        } else {
            Self::default()
        };

        let result = mutate(&mut history);
        history.save_unlocked(path).await?;
        Ok((history, result))
    }
}

async fn remove_file_if_exists(path: &Path) -> Result<()> {
    match fs_err::tokio::remove_file(path).await {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::{CompletedTask, DownloadHistory};
    use std::{
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_file(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!("hsi-core-{name}-{nonce}.json"))
    }

    fn sample_task(id: &str, path: &str) -> CompletedTask {
        CompletedTask {
            id: id.into(),
            url: format!("https://example.com/{id}"),
            dest: PathBuf::from(path),
            total_size: 10,
            completed_at: 1,
            duration: 1,
            avg_speed: 10,
        }
    }

    #[tokio::test]
    async fn roundtrip_history_save_and_load() {
        let path = temp_file("history-roundtrip");
        let mut history = DownloadHistory::default();
        history.add_completed(sample_task("one", "/tmp/one.bin"));

        history.save(&path).await.expect("save history");
        let loaded = DownloadHistory::load(&path).await.expect("load history");

        assert_eq!(loaded, history);
        let _ = fs_err::tokio::remove_file(path).await;
    }

    #[tokio::test]
    async fn file_mutation_helpers_persist_atomically() {
        let path = temp_file("history-mutate");

        let history =
            DownloadHistory::append_completed_to_file(&path, sample_task("one", "/tmp/one.bin"))
                .await
                .expect("append history");
        assert_eq!(history.completed_tasks.len(), 1);

        let (history, removed) = DownloadHistory::remove_from_file(&path, "one")
            .await
            .expect("remove history");
        assert!(removed);
        assert!(history.completed_tasks.is_empty());

        let history =
            DownloadHistory::append_completed_to_file(&path, sample_task("two", "/tmp/two.bin"))
                .await
                .expect("append second history");
        assert_eq!(history.completed_tasks.len(), 1);

        let history = DownloadHistory::clear_file(&path)
            .await
            .expect("clear history");
        assert!(history.completed_tasks.is_empty());

        let _ = fs_err::tokio::remove_file(path).await;
    }

    #[tokio::test]
    async fn remove_entry_and_file_from_file_deletes_artifact() {
        let path = temp_file("history-delete-file");
        let artifact = std::env::temp_dir().join("hsi-history-delete-artifact.bin");
        fs_err::tokio::write(&artifact, b"payload")
            .await
            .expect("write artifact");

        let history = DownloadHistory::append_completed_to_file(
            &path,
            sample_task("artifact", &artifact.display().to_string()),
        )
        .await
        .expect("append history");
        assert_eq!(history.completed_tasks.len(), 1);

        let (history, removed) =
            DownloadHistory::remove_entry_and_file_from_file(&path, "artifact")
                .await
                .expect("remove history and file");
        assert!(removed);
        assert!(history.completed_tasks.is_empty());
        assert!(!artifact.exists());

        let _ = fs_err::tokio::remove_file(path).await;
    }

    #[test]
    fn search_remove_and_truncate_work() {
        let mut history = DownloadHistory {
            max_history: 2,
            ..DownloadHistory::default()
        };

        history.add_completed(sample_task("one", "/tmp/one.bin"));
        history.add_completed(sample_task("two", "/tmp/two.bin"));
        history.add_completed(sample_task("three", "/tmp/three.bin"));

        assert_eq!(history.completed_tasks.len(), 2);
        assert_eq!(history.completed_tasks[0].id, "three");
        assert_eq!(history.search("two").len(), 1);
        assert!(history.remove("two"));
        assert!(!history.remove("missing"));
    }
}
