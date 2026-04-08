use ratatui::{Frame, layout::Rect};

use crate::tui::app::{App, SettingsGroup};
use crate::tui::theme::ThemeColors;

/// Draw a settings group block with its fields.
pub fn draw(f: &mut Frame, app: &App, group: &SettingsGroup, area: Rect, colors: &ThemeColors) {
    let _ = (f, app, group, area, colors);
}
