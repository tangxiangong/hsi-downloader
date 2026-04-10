use console::style;
use hsi_core::utils::XByte;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

pub struct ProgressManager {
    multi: MultiProgress,
    bars: Arc<RwLock<HashMap<String, ProgressBar>>>,
}

impl ProgressManager {
    pub fn new() -> Self {
        Self {
            multi: MultiProgress::new(),
            bars: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_task(&self, task_id: String, total_size: Option<u64>) {
        let pb = if let Some(size) = total_size {
            // 已知大小，使用进度条
            let pb = self.multi.add(ProgressBar::new(size));
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                    .unwrap()
                    .progress_chars("#>-"),
            );
            pb
        } else {
            // 未知大小，使用旋转器
            let pb = self.multi.add(ProgressBar::new_spinner());
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{msg}\n{spinner:.green} [{elapsed_precise}] {bytes} ({bytes_per_sec}) - 流式下载")
                    .unwrap(),
            );
            pb
        };
        pb.set_message(format!("📥 {}", &short_task_id(&task_id)));

        let mut bars = self.bars.write().await;
        bars.insert(task_id, pb);
    }

    pub async fn update_progress(&self, task_id: &str, downloaded: u64, speed: u64) {
        let bars = self.bars.read().await;
        if let Some(pb) = bars.get(task_id) {
            pb.set_position(downloaded);
            let speed_mb = speed as f64 / 1024.0 / 1024.0;
            pb.set_message(format!(
                "📥 {} @ {:.2} MB/s",
                &short_task_id(task_id),
                speed_mb
            ));
        }
    }

    pub async fn finish_task(&self, task_id: &str, success: bool) {
        let mut bars = self.bars.write().await;
        if let Some(pb) = bars.remove(task_id) {
            if success {
                pb.finish_with_message(format!("✅ {} 完成", &short_task_id(task_id)));
            } else {
                pb.finish_with_message(format!("❌ {} 失败", &short_task_id(task_id)));
            }
        }
    }
}

pub(crate) fn short_task_id(task_id: &str) -> String {
    task_id.chars().take(8).collect()
}

pub fn format_size(bytes: u64) -> String {
    XByte::from_bytes(bytes).to_string()
}

pub fn print_success(msg: &str) {
    println!("{} {}", style("✓").green().bold(), msg);
}

pub fn print_error(msg: &str) {
    eprintln!("{} {}", style("✗").red().bold(), msg);
}

pub fn print_info(msg: &str) {
    println!("{} {}", style("ℹ").blue().bold(), msg);
}

#[allow(dead_code)]
pub fn print_warning(msg: &str) {
    println!("{} {}", style("⚠").yellow().bold(), msg);
}

#[cfg(test)]
mod tests {
    use super::short_task_id;

    #[test]
    fn short_task_id_handles_short_and_unicode_ids() {
        assert_eq!(short_task_id("abc"), "abc");
        assert_eq!(short_task_id("任务编号123456"), "任务编号1234");
    }
}
