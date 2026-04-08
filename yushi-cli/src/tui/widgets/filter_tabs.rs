use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::Paragraph,
};

use crate::tui::app::{App, TaskFilter};
use crate::tui::theme::ThemeColors;

/// Draw the filter tab bar (All / Downloading / Completed).
pub fn draw(f: &mut Frame, app: &App, theme: &ThemeColors, area: Rect) {
    let filters = [
        TaskFilter::All,
        TaskFilter::Downloading,
        TaskFilter::Completed,
    ];

    let chunks = Layout::horizontal([
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
    ])
    .split(area);

    for (i, filter) in filters.iter().enumerate() {
        let count = app.filter_count(*filter);
        let label = format!("{}({})", filter.label(), count);
        let is_active = app.filter == *filter;

        let style = if is_active {
            Style::default()
                .fg(theme.text)
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_secondary)
        };

        let para = Paragraph::new(Line::from(label))
            .style(style)
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(para, chunks[i]);
    }
}
