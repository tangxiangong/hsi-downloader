use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

use crate::tui::app::{App, CurrentView};
use crate::tui::theme::ThemeColors;

/// Draw the icon sidebar showing the current active view.
pub fn draw(f: &mut Frame, app: &App, theme: &ThemeColors, area: Rect) {
    // Outer block with right border
    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(theme.border));
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Layout: logo(2) + spacer(1) + icon*3(1 each) + flex + version(3)
    let chunks = Layout::vertical([
        Constraint::Length(2), // logo
        Constraint::Length(1), // spacer
        Constraint::Length(1), // Tasks icon
        Constraint::Length(1), // History icon
        Constraint::Length(1), // Settings icon
        Constraint::Min(0),    // flex spacer
        Constraint::Length(3), // version
    ])
    .split(inner);

    // Logo
    let logo = Paragraph::new(Line::from("Hsi"))
        .style(
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(logo, chunks[0]);

    // Navigation icons
    let icons = [
        (CurrentView::Tasks, "📥"),
        (CurrentView::History, "📋"),
        (CurrentView::Settings, "⚙ "),
    ];

    for (i, (view, icon)) in icons.iter().enumerate() {
        let is_active = app.current_view == *view;
        let style = if is_active {
            Style::default()
                .fg(theme.text)
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.muted)
        };
        let para = Paragraph::new(Line::from(*icon))
            .style(style)
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(para, chunks[2 + i]);
    }

    // Version at bottom — split into 3 lines: "v0", ".1", ".0"
    // Show version vertically by splitting into chunks of 2 chars
    let version_chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(chunks[6]);

    let parts = ["v0", ".1", ".0"];
    for (i, part) in parts.iter().enumerate() {
        let para = Paragraph::new(Line::from(*part))
            .style(Style::default().fg(theme.muted))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(para, version_chunks[i]);
    }
}
