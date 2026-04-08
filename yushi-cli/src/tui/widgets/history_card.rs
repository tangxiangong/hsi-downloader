use ratatui::{Frame, layout::Rect};
use yushi_core::CompletedTask;

use crate::tui::theme::ThemeColors;

/// Draw a single history entry card inside the history list area.
pub fn draw(
    f: &mut Frame,
    task: &CompletedTask,
    area: Rect,
    selected: bool,
    colors: &ThemeColors,
) {
    let _ = (f, task, area, selected, colors);
}
