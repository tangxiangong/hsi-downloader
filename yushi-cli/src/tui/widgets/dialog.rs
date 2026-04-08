use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::tui::app::ConfirmDialog;
use crate::tui::theme::ThemeColors;

/// Draw a modal confirmation dialog.
pub fn draw(f: &mut Frame, dialog: &ConfirmDialog, theme: &ThemeColors, area: Rect) {
    // Center: 30 wide, 7 tall
    let dialog_width = 30u16.min(area.width.saturating_sub(4));
    let dialog_height = 7u16.min(area.height.saturating_sub(2));

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
            format!(" {} ", dialog.title),
            Style::default().fg(theme.warning).add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(theme.warning));

    let inner = outer_block.inner(dialog_area);
    f.render_widget(outer_block, dialog_area);

    // Inner layout: spacer(1) + message(1) + spacer(1) + buttons(1) + spacer(1)
    let chunks = Layout::vertical([
        Constraint::Length(1), // top spacer
        Constraint::Length(1), // message
        Constraint::Length(1), // middle spacer
        Constraint::Length(1), // buttons
        Constraint::Min(0),    // bottom spacer
    ])
    .split(inner);

    // Message — centered
    let msg_para = Paragraph::new(dialog.message.as_str())
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.text));
    f.render_widget(msg_para, chunks[1]);

    // Buttons
    let cancel_style = if !dialog.selected_confirm {
        Style::default()
            .bg(theme.muted)
            .fg(theme.text)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_secondary)
    };

    let confirm_style = if dialog.selected_confirm {
        Style::default()
            .bg(theme.error)
            .fg(theme.text)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_secondary)
    };

    let buttons_line = Line::from(vec![
        Span::styled(" [取消] ", cancel_style),
        Span::raw("  "),
        Span::styled(" [确认] ", confirm_style),
    ]);

    let buttons_para = Paragraph::new(buttons_line).alignment(Alignment::Center);
    f.render_widget(buttons_para, chunks[3]);
}
