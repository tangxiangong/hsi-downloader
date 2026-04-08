use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Style,
    widgets::Paragraph,
};

use crate::tui::app::{App, CurrentView, InputMode};
use crate::tui::theme::ThemeColors;

/// Draw the bottom context-sensitive help bar.
pub fn draw(f: &mut Frame, app: &App, theme: &ThemeColors, area: Rect) {
    let text = match app.input_mode {
        InputMode::Normal => match app.current_view {
            CurrentView::Tasks => {
                "1/2/3:视图  ↑↓:导航  Tab/←→:筛选  a:添加  p:暂停  c:取消  d:删除  D:删文件  q:退出"
            }
            CurrentView::History => {
                "1/2/3:视图  ↑↓:导航  x:删除记录  C:清空全部  r:刷新  q:退出"
            }
            CurrentView::Settings => {
                "1/2/3:视图  ↑↓:选择  Enter/e:编辑  ←→:切换主题  r:重载  q:退出"
            }
        },
        InputMode::AddTask => {
            "Tab:下一项  Shift+Tab:上一项  ←→:切换  Enter:确认  Esc:取消"
        }
        InputMode::EditSetting => "Enter:保存  Esc:取消",
        InputMode::Confirm => "←→:选择  Enter:确认  Esc:取消",
    };

    let para = Paragraph::new(text)
        .style(Style::default().fg(theme.text_help))
        .alignment(Alignment::Center);

    f.render_widget(para, area);
}
