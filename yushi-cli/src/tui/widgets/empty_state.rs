use ratatui::{Frame, layout::Rect};

use crate::tui::theme::ThemeColors;

/// Draw an empty-state placeholder when a list has no items.
pub fn draw(f: &mut Frame, message: &str, area: Rect, colors: &ThemeColors) {
    let _ = (f, message, area, colors);
}
