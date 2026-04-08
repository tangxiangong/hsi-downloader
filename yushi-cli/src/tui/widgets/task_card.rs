use ratatui::{Frame, layout::Rect};
use yushi_core::DownloadTask;

use crate::tui::theme::ThemeColors;

/// Draw a single task card inside the task list area.
pub fn draw(f: &mut Frame, task: &DownloadTask, area: Rect, selected: bool, colors: &ThemeColors) {
    let _ = (f, task, area, selected, colors);
}
