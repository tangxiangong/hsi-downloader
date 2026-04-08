use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::Paragraph,
};

use crate::tui::theme::ThemeColors;

/// Draw an empty-state placeholder when a list has no items.
pub fn draw(f: &mut Frame, icon: &str, message: &str, hint: &str, theme: &ThemeColors, area: Rect) {
    // Vertically center a 4-line block: icon, blank, message, hint
    let chunks = Layout::vertical([
        Constraint::Min(0),    // top flex
        Constraint::Length(4), // content
        Constraint::Min(0),    // bottom flex
    ])
    .split(area);

    let content_area = chunks[1];

    let rows = Layout::vertical([
        Constraint::Length(1), // icon
        Constraint::Length(1), // blank
        Constraint::Length(1), // message
        Constraint::Length(1), // hint
    ])
    .split(content_area);

    // Icon
    f.render_widget(
        Paragraph::new(Line::from(icon))
            .style(Style::default().fg(theme.primary))
            .alignment(Alignment::Center),
        rows[0],
    );

    // Message
    f.render_widget(
        Paragraph::new(Line::from(message))
            .style(
                Style::default()
                    .fg(theme.text_secondary)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center),
        rows[2],
    );

    // Hint
    f.render_widget(
        Paragraph::new(Line::from(hint))
            .style(Style::default().fg(theme.text_help))
            .alignment(Alignment::Center),
        rows[3],
    );
}
