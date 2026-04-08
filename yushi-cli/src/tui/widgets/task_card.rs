use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
};
use yushi_core::{DownloadTask, TaskStatus};

use crate::tui::theme::ThemeColors;
use crate::ui::format_size;

/// Height of a single task card including borders.
pub fn card_height() -> u16 {
    5
}

/// Format ETA in seconds into a human-readable string.
fn format_eta(secs: u64) -> String {
    if secs >= 3600 {
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        format!("{}h {}m", h, m)
    } else if secs >= 60 {
        let m = secs / 60;
        let s = secs % 60;
        format!("{}m {}s", m, s)
    } else {
        format!("{}s", secs)
    }
}

/// Determine file-type icon by file extension.
fn file_icon(filename: &str) -> &'static str {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" => "📦",
        "iso" | "img" | "dmg" => "💿",
        "pdf" | "doc" | "docx" | "txt" | "md" => "📄",
        "mp4" | "mkv" | "avi" | "mov" | "webm" => "🎬",
        "mp3" | "flac" | "wav" | "ogg" | "aac" => "🎵",
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" => "🖼",
        "exe" | "msi" | "deb" | "rpm" | "appimage" => "⚙",
        _ => "📎",
    }
}

/// Truncate a string to at most `max_chars` chars, appending "…" if truncated.
fn truncate(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = chars[..max_chars.saturating_sub(1)].iter().collect();
        format!("{}…", truncated)
    }
}

/// Draw a single task card inside the task list area.
pub fn draw(f: &mut Frame, task: &DownloadTask, selected: bool, theme: &ThemeColors, area: Rect) {
    let filename = task
        .dest
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let icon = file_icon(filename);

    let border_style = if selected {
        Style::default().fg(theme.border_active)
    } else {
        Style::default().fg(theme.border)
    };
    let bg_style = if selected {
        Style::default().bg(theme.selection_bg)
    } else {
        Style::default()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(bg_style);

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Inner layout: 3 rows (title, info, progress)
    let rows = Layout::vertical([
        Constraint::Length(1), // title row
        Constraint::Length(1), // info row
        Constraint::Length(1), // progress row
    ])
    .split(inner);

    // --- Row 0: icon + filename + action hints ---
    let (status_icon, action_hints) = match task.status {
        TaskStatus::Downloading => ("⬇ ", "[⏸][✕]"),
        TaskStatus::Paused => ("⏸ ", "[▶][✕]"),
        TaskStatus::Pending => ("⏳", "[✕]"),
        TaskStatus::Completed => ("✅", "[✕]"),
        TaskStatus::Failed => ("❌", "[✕]"),
        TaskStatus::Cancelled => ("⊗ ", "[✕]"),
    };

    // Calculate available width for filename
    let hint_len = action_hints.chars().count() as u16;
    let prefix_len = (icon.chars().count() + status_icon.chars().count() + 2) as u16;
    let avail_for_name =
        (rows[0].width as i32 - prefix_len as i32 - hint_len as i32 - 2).max(8) as usize;
    let short_name = truncate(filename, avail_for_name);

    let title_line = Line::from(vec![
        Span::raw(format!("{} {} ", icon, status_icon)),
        Span::styled(
            short_name,
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        // right-pad with spaces
        Span::raw(
            " ".repeat(
                (rows[0].width as i32 - prefix_len as i32 - avail_for_name as i32 - hint_len as i32)
                    .max(0) as usize,
            ),
        ),
        Span::styled(action_hints, Style::default().fg(theme.muted)),
    ]);
    f.render_widget(Paragraph::new(title_line), rows[0]);

    // --- Row 1: size / speed / eta info ---
    let info_text = match task.status {
        TaskStatus::Downloading => {
            let size_str = format!(
                "{} / {}",
                format_size(task.downloaded),
                format_size(task.total_size)
            );
            let speed_str = format!(" · {}/s", format_size(task.speed));
            let eta_str = task
                .eta
                .map(|e| format!(" · 剩余 {}", format_eta(e)))
                .unwrap_or_default();
            format!("{}{}{}", size_str, speed_str, eta_str)
        }
        TaskStatus::Paused => {
            format!(
                "{} / {} · 已暂停",
                format_size(task.downloaded),
                format_size(task.total_size)
            )
        }
        TaskStatus::Pending => "等待中".to_string(),
        TaskStatus::Completed => format!("{} · 已完成", format_size(task.total_size)),
        TaskStatus::Failed => task.error.clone().unwrap_or_else(|| "下载失败".to_string()),
        TaskStatus::Cancelled => "已取消".to_string(),
    };

    let info_color = match task.status {
        TaskStatus::Downloading => theme.primary,
        TaskStatus::Paused => theme.warning,
        TaskStatus::Pending => theme.muted,
        TaskStatus::Completed => theme.success,
        TaskStatus::Failed => theme.error,
        TaskStatus::Cancelled => theme.muted,
    };

    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            info_text,
            Style::default().fg(info_color),
        ))),
        rows[1],
    );

    // --- Row 2: progress gauge ---
    let progress = if task.total_size > 0 {
        ((task.downloaded as f64 / task.total_size as f64) * 100.0).min(100.0) as u16
    } else {
        0
    };

    let gauge_color = match task.status {
        TaskStatus::Downloading => theme.primary,
        TaskStatus::Paused => theme.warning,
        TaskStatus::Pending => theme.muted,
        TaskStatus::Completed => theme.success,
        TaskStatus::Failed => theme.error,
        TaskStatus::Cancelled => theme.muted,
    };

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(gauge_color).bg(theme.border))
        .percent(progress)
        .label(format!("{:.1}%", progress as f64));
    f.render_widget(gauge, rows[2]);
}
