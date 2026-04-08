use ratatui::{Frame, layout::Rect};

use crate::tui::app::ConfirmDialog;
use crate::tui::theme::ThemeColors;

/// Draw a modal confirmation dialog.
pub fn draw(f: &mut Frame, dialog: &ConfirmDialog, area: Rect, colors: &ThemeColors) {
    let _ = (f, dialog, area, colors);
}
