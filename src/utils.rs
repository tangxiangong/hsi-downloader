use std::path::PathBuf;

use gpui::*;
use gpui_component::{ActiveTheme, button::ButtonCustomVariant};
use yushi_core::{
    AppConfig, CompletedTask, DownloadHistory, DownloadTask, TaskStatus, YuShi, parse_speed_limit,
};

pub fn app_background(cx: &App) -> Hsla {
    if cx.theme().is_dark() {
        hsla(0.62, 0.18, 0.11, 1.0)
    } else {
        white()
    }
}

pub fn panel_color(cx: &App) -> Hsla {
    if cx.theme().is_dark() {
        hsla(0.62, 0.16, 0.15, 1.0)
    } else {
        hsla(0.60, 0.18, 0.96, 1.0)
    }
}

pub fn card_color(cx: &App) -> Hsla {
    if cx.theme().is_dark() {
        hsla(0.62, 0.14, 0.19, 1.0)
    } else {
        hsla(0.60, 0.12, 0.93, 1.0)
    }
}

pub fn border_color(cx: &App) -> Hsla {
    if cx.theme().is_dark() {
        hsla(0.62, 0.08, 0.28, 1.0)
    } else {
        hsla(0.60, 0.08, 0.84, 1.0)
    }
}

pub fn text_color(cx: &App) -> Hsla {
    if cx.theme().is_dark() {
        white()
    } else {
        black()
    }
}

pub fn muted_text_color(cx: &App) -> Hsla {
    if cx.theme().is_dark() {
        hsla(0.60, 0.03, 0.72, 1.0)
    } else {
        hsla(0.60, 0.04, 0.32, 1.0)
    }
}

pub fn primary_color(cx: &App) -> Hsla {
    if cx.theme().is_dark() {
        hsla(0.58, 0.80, 0.58, 1.0)
    } else {
        hsla(0.58, 0.78, 0.48, 1.0)
    }
}

pub fn button_style(bg: Hsla, fg: Hsla, cx: &App) -> ButtonCustomVariant {
    ButtonCustomVariant::new(cx)
        .color(bg)
        .foreground(fg)
        .hover(bg.opacity(0.92))
        .active(bg.opacity(0.82))
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let bytes_f = bytes as f64;
    if bytes_f >= GB {
        format!("{bytes_f:.1} GB", bytes_f = bytes_f / GB)
    } else if bytes_f >= MB {
        format!("{bytes_f:.1} MB", bytes_f = bytes_f / MB)
    } else if bytes_f >= KB {
        format!("{bytes_f:.1} KB", bytes_f = bytes_f / KB)
    } else {
        format!("{bytes} B")
    }
}

pub fn status_badge(status: TaskStatus, cx: &App) -> Div {
    let (label, color) = match status {
        TaskStatus::Pending => ("等待中", cx.theme().yellow),
        TaskStatus::Downloading => ("下载中", cx.theme().blue),
        TaskStatus::Paused => ("已暂停", cx.theme().muted_foreground),
        TaskStatus::Completed => ("已完成", cx.theme().green),
        TaskStatus::Failed => ("失败", cx.theme().red),
        TaskStatus::Cancelled => ("已取消", cx.theme().muted_foreground),
    };

    div()
        .px_2()
        .py_1()
        .rounded(px(999.))
        .bg(color.opacity(0.14))
        .text_color(color)
        .text_xs()
        .child(label)
}

pub fn search_history(history: &DownloadHistory, query: &str) -> Vec<CompletedTask> {
    if query.trim().is_empty() {
        history.get_all().to_vec()
    } else {
        history.search(query)
    }
}

pub struct ViewStats {
    pub total_tasks: usize,
    pub active_tasks: usize,
    pub completed_tasks: usize,
    pub history_items: usize,
}

impl ViewStats {
    pub fn from_state(tasks: &[DownloadTask], history_items: usize) -> Self {
        Self {
            total_tasks: tasks.len(),
            active_tasks: tasks
                .iter()
                .filter(|task| matches!(task.status, TaskStatus::Pending | TaskStatus::Downloading))
                .count(),
            completed_tasks: tasks
                .iter()
                .filter(|task| task.status == TaskStatus::Completed)
                .count(),
            history_items,
        }
    }
}

pub fn progress_percent(task: &DownloadTask) -> f32 {
    if task.total_size == 0 {
        0.0
    } else {
        ((task.downloaded as f64 / task.total_size as f64) * 100.0).clamp(0.0, 100.0) as f32
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddTaskDraft {
    pub url: String,
    pub destination_input: String,
    pub speed_limit: Option<u64>,
}

impl AddTaskDraft {
    pub fn parse(url: &str, destination_input: &str, speed_limit_input: &str) -> Result<Self> {
        let url = url.trim();
        if url.is_empty() {
            return Err(anyhow::anyhow!("URL is required"));
        }

        let speed_limit = parse_optional_speed_limit(speed_limit_input)?;

        Ok(Self {
            url: url.to_string(),
            destination_input: destination_input.trim().to_string(),
            speed_limit,
        })
    }

    pub async fn resolve_destination(&self, queue: &YuShi, config: &AppConfig) -> PathBuf {
        if self.destination_input.is_empty() {
            queue
                .infer_destination_in_dir(&self.url, config.default_download_path.clone())
                .await
        } else {
            PathBuf::from(&self.destination_input)
        }
    }
}

pub fn parse_optional_speed_limit(speed_limit_input: &str) -> Result<Option<u64>> {
    if speed_limit_input.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(
            parse_speed_limit(speed_limit_input.trim()).ok_or_else(|| {
                anyhow::anyhow!("Invalid speed limit: {}", speed_limit_input.trim())
            })?,
        ))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TaskAction {
    Pause,
    Resume,
    Cancel,
    Remove,
    DeleteFile,
}

impl TaskAction {
    pub fn button_label(self) -> &'static str {
        match self {
            Self::Pause => "暂停下载",
            Self::Resume => "继续下载",
            Self::Cancel => "取消下载",
            Self::Remove => "删除任务",
            Self::DeleteFile => "删除文件",
        }
    }

    pub fn id_suffix(self) -> &'static str {
        match self {
            Self::Pause => "pause",
            Self::Resume => "resume",
            Self::Cancel => "cancel",
            Self::Remove => "remove",
            Self::DeleteFile => "delete-file",
        }
    }

    pub fn is_primary(self) -> bool {
        matches!(self, Self::Pause | Self::Resume)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Pause => "Paused task",
            Self::Resume => "Resumed task",
            Self::Cancel => "Cancelled download",
            Self::Remove => "Removed task",
            Self::DeleteFile => "Deleted file for task",
        }
    }
}

pub fn task_actions(status: TaskStatus) -> Vec<TaskAction> {
    match status {
        TaskStatus::Pending | TaskStatus::Downloading => {
            vec![TaskAction::Pause, TaskAction::Cancel]
        }
        TaskStatus::Paused => vec![TaskAction::Resume, TaskAction::Cancel],
        TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled => {
            vec![TaskAction::Remove, TaskAction::DeleteFile]
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HistoryAction {
    RemoveRecord,
    DeleteFile,
}

impl HistoryAction {
    pub fn button_label(self) -> &'static str {
        match self {
            Self::RemoveRecord => "删除记录",
            Self::DeleteFile => "删除文件",
        }
    }

    pub fn id_suffix(self) -> &'static str {
        match self {
            Self::RemoveRecord => "remove-record",
            Self::DeleteFile => "delete-file",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::RemoveRecord => "Removed history record",
            Self::DeleteFile => "Deleted history file",
        }
    }

    pub fn not_found_message(self) -> &'static str {
        match self {
            Self::RemoveRecord | Self::DeleteFile => "History item not found",
        }
    }
}

pub fn history_actions() -> [HistoryAction; 2] {
    [HistoryAction::RemoveRecord, HistoryAction::DeleteFile]
}

#[cfg(test)]
mod tests {
    use super::{AddTaskDraft, parse_optional_speed_limit};

    #[test]
    fn rejects_empty_url() {
        assert!(AddTaskDraft::parse("", "", "").is_err());
    }

    #[test]
    fn parses_speed_limit() {
        let draft =
            AddTaskDraft::parse("https://example.com/file.bin", "", "2M").expect("parse draft");

        assert_eq!(draft.url, "https://example.com/file.bin");
        assert_eq!(draft.speed_limit, Some(2 * 1024 * 1024));
    }

    #[test]
    fn preserves_explicit_destination() {
        let draft = AddTaskDraft::parse("https://example.com/file.bin", "/tmp/output.bin", "")
            .expect("parse draft");

        assert_eq!(draft.destination_input, "/tmp/output.bin");
        assert_eq!(draft.speed_limit, None);
    }

    #[test]
    fn parses_optional_speed_limit() {
        assert_eq!(parse_optional_speed_limit("").expect("empty"), None);
        assert_eq!(
            parse_optional_speed_limit("512K").expect("speed"),
            Some(512 * 1024)
        );
    }
}
