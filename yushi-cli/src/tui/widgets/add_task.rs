use ratatui::{Frame, layout::Rect};

use crate::tui::app::{AddTaskState, App};
use crate::tui::theme::ThemeColors;

/// Draw the "Add Task" dialog overlay.
pub fn draw(f: &mut Frame, app: &App, state: &AddTaskState, area: Rect, colors: &ThemeColors) {
    let _ = (f, app, state, area, colors);
}
