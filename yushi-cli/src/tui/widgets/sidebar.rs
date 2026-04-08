use ratatui::{Frame, layout::Rect};

use crate::tui::app::App;
use crate::tui::theme::ThemeColors;

/// Draw the icon sidebar showing the current active view.
pub fn draw(f: &mut Frame, app: &App, area: Rect, _colors: &ThemeColors) {
    let _ = (f, app, area);
}
