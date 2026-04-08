use ratatui::{Frame, layout::Rect};

use crate::tui::app::App;
use crate::tui::theme::ThemeColors;

/// Draw the bottom help/key-binding bar.
pub fn draw(f: &mut Frame, app: &App, area: Rect, colors: &ThemeColors) {
    let _ = (f, app, area, colors);
}
