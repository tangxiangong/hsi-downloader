use crate::{
    Result, storage,
    types::{Task, TaskStatus},
};
use serde::{Deserialize, Serialize};
use std::{
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
        storage::ensure_parent_dir(path).await?;
        let content = serde_json::to_string_pretty(self)?;
        fs_err::tokio::write(path, content).await?;
        Ok(())
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
        std::env::temp_dir().join(format!("yushi-core-{name}-{nonce}.json"))
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
