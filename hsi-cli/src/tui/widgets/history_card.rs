use crate::{tui::theme::ThemeColors, ui::format_size};
use hsi_core::CompletedTask;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// Height of a single history card including borders.
pub fn card_height() -> u16 {
    4
}

/// Format duration in seconds into a human-readable Chinese string.
fn format_duration(secs: u64) -> String {
    if secs >= 3600 {
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        let s = secs % 60;
        format!("{}时{}分{}秒", h, m, s)
    } else if secs >= 60 {
        let m = secs / 60;
        let s = secs % 60;
        format!("{}分{}秒", m, s)
    } else {
        format!("{}秒", secs)
    }
}

/// Convert a Unix timestamp (seconds) to (month, day) using Howard Hinnant's algorithm.
fn unix_to_md(ts: u64) -> (u32, u32) {
    // Days since epoch
    let days = (ts / 86400) as i64;
    // Shift epoch from 1970-01-01 to 0000-03-01
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64; // day of era [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // year of era [0, 399]
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year [0, 365]
    let mp = (5 * doy + 2) / 153; // month of year [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // day [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // month [1, 12]
    (m as u32, d as u32)
}

/// Draw a single history entry card inside the history list area.
pub fn draw(f: &mut Frame, task: &CompletedTask, selected: bool, theme: &ThemeColors, area: Rect) {
    let filename = task
        .dest
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

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

    // Inner layout: 2 rows (title, info)
    let rows = Layout::vertical([
        Constraint::Length(1), // title row
        Constraint::Length(1), // info row
    ])
    .split(inner);

    // --- Row 0: checkmark + filename + remove hint ---
    let hint = "[✕]";
    let hint_len = hint.chars().count() as u16;
    let prefix = "✅ ";
    let prefix_len = prefix.chars().count() as u16;
    let avail = (rows[0].width as i32 - prefix_len as i32 - hint_len as i32 - 1).max(8) as usize;

    // Truncate filename
    let chars: Vec<char> = filename.chars().collect();
    let short_name: String = if chars.len() <= avail {
        filename.to_string()
    } else {
        let t: String = chars[..avail.saturating_sub(1)].iter().collect();
        format!("{}…", t)
    };

    let padding = (rows[0].width as i32
        - prefix_len as i32
        - short_name.chars().count() as i32
        - hint_len as i32
        - 1)
    .max(0) as usize;

    let title_line = Line::from(vec![
        Span::styled(prefix, Style::default().fg(theme.success)),
        Span::styled(
            short_name,
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" ".repeat(padding)),
        Span::styled(hint, Style::default().fg(theme.muted)),
    ]);
    f.render_widget(Paragraph::new(title_line), rows[0]);

    // --- Row 1: size + avg speed + duration + date ---
    let (month, day) = unix_to_md(task.completed_at);
    let date_str = format!("{:02}-{:02}", month, day);
    let duration_str = format_duration(task.duration);

    let info = format!(
        "{} · 平均 {}/s · 用时 {} · {}",
        format_size(task.total_size),
        format_size(task.avg_speed),
        duration_str,
        date_str,
    );

    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            info,
            Style::default().fg(theme.text_secondary),
        ))),
        rows[1],
    );
}
