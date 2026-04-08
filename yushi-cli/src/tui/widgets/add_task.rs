use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use yushi_core::Priority;

use crate::tui::app::{AddTaskField, AddTaskState};
use crate::tui::theme::ThemeColors;

/// Draw the "Add Task" dialog overlay.
pub fn draw(f: &mut Frame, state: &AddTaskState, theme: &ThemeColors, area: Rect) {
    // Center the dialog: 50 wide, 18 tall
    let dialog_width = 50u16.min(area.width.saturating_sub(4));
    let dialog_height = 18u16.min(area.height.saturating_sub(2));

    let x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
    let y = area.y + (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect {
        x,
        y,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear background
    f.render_widget(Clear, dialog_area);

    let outer_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " 添加下载任务 ",
            Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(theme.primary));

    let inner = outer_block.inner(dialog_area);
    f.render_widget(outer_block, dialog_area);

    // Inner layout (fixed lines):
    // 0: URL label
    // 1: URL input box (3 tall)  → but we use Length(3) for borders
    // We'll flatten: label(1) + input(3) + path label(1) + path input(3) +
    //                priority label(1) + priority buttons(1) +
    //                speed label(1) + speed input(3) + error(1) + buttons(1)
    // Total = 16, but inner height may be smaller; clamp gracefully.

    let constraints = [
        Constraint::Length(1), // URL label
        Constraint::Length(3), // URL input
        Constraint::Length(1), // Path label
        Constraint::Length(3), // Path input
        Constraint::Length(1), // Priority label
        Constraint::Length(1), // Priority buttons
        Constraint::Length(1), // Speed label
        Constraint::Length(3), // Speed input
        Constraint::Length(1), // Error
        Constraint::Length(1), // Buttons row
    ];

    let chunks = Layout::vertical(constraints).split(inner);

    // --- URL ---
    render_label(f, "下载链接", chunks[0], theme);
    render_input(
        f,
        &state.url,
        state.focused_field == AddTaskField::Url,
        false,
        chunks[1],
        theme,
    );

    // --- Path ---
    render_label(f, "保存位置", chunks[2], theme);
    render_input(
        f,
        &state.path,
        state.focused_field == AddTaskField::Path,
        false,
        chunks[3],
        theme,
    );

    // --- Priority ---
    render_label(f, "优先级", chunks[4], theme);
    render_priority(f, state, chunks[5], theme);

    // --- Speed Limit ---
    render_label(f, "限速 (留空不限)", chunks[6], theme);
    render_input(
        f,
        &state.speed_limit,
        state.focused_field == AddTaskField::SpeedLimit,
        false,
        chunks[7],
        theme,
    );

    // --- Error ---
    if let Some(err) = &state.error {
        let err_para = Paragraph::new(Span::styled(
            err.as_str(),
            Style::default().fg(theme.error),
        ));
        f.render_widget(err_para, chunks[8]);
    }

    // --- Buttons ---
    render_buttons(f, state, chunks[9], theme);
}

fn render_label(f: &mut Frame, text: &str, area: Rect, theme: &ThemeColors) {
    let para = Paragraph::new(Span::styled(
        text,
        Style::default().fg(theme.text_secondary),
    ));
    f.render_widget(para, area);
}

fn render_input(
    f: &mut Frame,
    value: &str,
    focused: bool,
    _editing: bool,
    area: Rect,
    theme: &ThemeColors,
) {
    let border_style = if focused {
        Style::default().fg(theme.border_active)
    } else {
        Style::default().fg(theme.border)
    };

    let display = if focused {
        format!("{}▎", value)
    } else {
        value.to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);

    let para = Paragraph::new(display)
        .block(block)
        .style(Style::default().fg(theme.text));

    f.render_widget(para, area);
}

fn render_priority(f: &mut Frame, state: &AddTaskState, area: Rect, theme: &ThemeColors) {
    let focused = state.focused_field == AddTaskField::Priority;
    let options: &[(&str, Priority)] = &[
        ("低", Priority::Low),
        ("正常", Priority::Normal),
        ("高", Priority::High),
    ];

    let mut spans: Vec<Span> = Vec::new();
    for (label, variant) in options {
        let active = state.priority == *variant;
        let btn = format!(" {} ", label);
        let style = if active {
            Style::default()
                .bg(theme.primary)
                .fg(theme.bg)
                .add_modifier(Modifier::BOLD)
        } else if focused {
            Style::default().fg(theme.text)
        } else {
            Style::default().fg(theme.text_secondary)
        };

        spans.push(Span::raw("["));
        spans.push(Span::styled(btn, style));
        spans.push(Span::raw("] "));
    }

    let para = Paragraph::new(Line::from(spans));
    f.render_widget(para, area);
}

fn render_buttons(f: &mut Frame, state: &AddTaskState, area: Rect, theme: &ThemeColors) {
    let focused = state.focused_field == AddTaskField::Buttons;

    let cancel_style = if focused && !state.button_confirm {
        Style::default()
            .bg(theme.muted)
            .fg(theme.text)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_secondary)
    };

    let confirm_style = if focused && state.button_confirm {
        Style::default()
            .bg(theme.primary)
            .fg(theme.bg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_secondary)
    };

    let line = Line::from(vec![
        Span::styled(" [取消] ", cancel_style),
        Span::raw("  "),
        Span::styled(" [添加] ", confirm_style),
    ]);

    let para = Paragraph::new(line).alignment(Alignment::Center);
    f.render_widget(para, area);
}
