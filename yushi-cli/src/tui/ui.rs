use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::app::{App, CurrentView, InputMode, TaskFilter};
use super::widgets;

const SIDEBAR_WIDTH: u16 = 5;

pub fn draw(f: &mut Frame, app: &App) {
    let theme = &app.theme;

    // Top-level horizontal split: sidebar | content
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(SIDEBAR_WIDTH), Constraint::Min(0)])
        .split(f.area());

    // Sidebar
    widgets::sidebar::draw(f, app, theme, h_chunks[0]);

    // Right side: header + [filter tabs] + content + help
    let has_filter = app.current_view == CurrentView::Tasks;
    let v_constraints: Vec<Constraint> = if has_filter {
        vec![
            Constraint::Length(1), // header
            Constraint::Length(1), // filter tabs
            Constraint::Min(3),   // content
            Constraint::Length(1), // help
        ]
    } else {
        vec![
            Constraint::Length(1), // header
            Constraint::Min(3),   // content
            Constraint::Length(1), // help
        ]
    };

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(v_constraints)
        .split(h_chunks[1]);

    // Header
    draw_header(f, app, theme, v_chunks[0]);

    if has_filter {
        widgets::filter_tabs::draw(f, app, theme, v_chunks[1]);
        draw_content(f, app, theme, v_chunks[2]);
        widgets::help_bar::draw(f, app, theme, v_chunks[3]);
    } else {
        draw_content(f, app, theme, v_chunks[1]);
        widgets::help_bar::draw(f, app, theme, v_chunks[2]);
    }

    // Overlay dialogs
    if app.input_mode == InputMode::AddTask
        && let Some(state) = &app.add_task_state
    {
        widgets::add_task::draw(f, state, theme, f.area());
    }

    if app.input_mode == InputMode::Confirm
        && let Some(dialog) = &app.confirm_dialog
    {
        widgets::dialog::draw(f, dialog, theme, f.area());
    }
}

fn draw_header(
    f: &mut Frame,
    app: &App,
    theme: &super::theme::ThemeColors,
    area: Rect,
) {
    let (title, count_str) = match app.current_view {
        CurrentView::Tasks => {
            let count = app.filtered_indices.len();
            ("任务".to_string(), format!("{} 个任务", count))
        }
        CurrentView::History => {
            let count = app.history.completed_tasks.len();
            ("历史".to_string(), format!("{} 条记录", count))
        }
        CurrentView::Settings => ("设置".to_string(), String::new()),
    };

    let available = area.width as usize;
    let title_display = format!(" {}", title);
    let count_display = format!("{} ", count_str);
    let padding = available.saturating_sub(title_display.len() + count_display.len());

    let line = Line::from(vec![
        Span::styled(
            title_display,
            Style::default()
                .fg(theme.text)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" ".repeat(padding)),
        Span::styled(count_display, Style::default().fg(theme.text_secondary)),
    ]);

    f.render_widget(Paragraph::new(line), area);
}

fn draw_content(
    f: &mut Frame,
    app: &App,
    theme: &super::theme::ThemeColors,
    area: Rect,
) {
    match app.current_view {
        CurrentView::Tasks => draw_tasks_content(f, app, theme, area),
        CurrentView::History => draw_history_content(f, app, theme, area),
        CurrentView::Settings => widgets::settings_group::draw(f, app, theme, area),
    }
}

fn draw_tasks_content(
    f: &mut Frame,
    app: &App,
    theme: &super::theme::ThemeColors,
    area: Rect,
) {
    if app.filtered_indices.is_empty() {
        let (hint, msg) = match app.filter {
            TaskFilter::All => ("按 a 添加新任务", "暂无下载任务"),
            TaskFilter::Downloading => ("", "暂无进行中的任务"),
            TaskFilter::Completed => ("", "暂无已完成的任务"),
        };
        widgets::empty_state::draw(f, "⬇", msg, hint, theme, area);
        return;
    }

    let card_h = widgets::task_card::card_height();
    let visible_count = (area.height / card_h).max(1) as usize;

    // Auto-scroll to keep selected item visible
    let scroll = {
        let sel = app.selected_index;
        if sel >= visible_count {
            sel - visible_count + 1
        } else {
            0
        }
    };

    let mut y = area.y;
    for vi in 0..visible_count {
        let idx = scroll + vi;
        if idx >= app.filtered_indices.len() {
            break;
        }
        let remaining_height = (area.y + area.height).saturating_sub(y);
        if remaining_height < card_h {
            break;
        }
        let task_idx = app.filtered_indices[idx];
        if let Some(task) = app.tasks.get(task_idx) {
            let card_area = Rect::new(area.x, y, area.width, card_h);
            let selected = idx == app.selected_index;
            widgets::task_card::draw(f, task, selected, theme, card_area);
            y += card_h;
        }
    }
}

fn draw_history_content(
    f: &mut Frame,
    app: &App,
    theme: &super::theme::ThemeColors,
    area: Rect,
) {
    if app.history.completed_tasks.is_empty() {
        widgets::empty_state::draw(f, "📋", "暂无下载记录", "", theme, area);
        return;
    }

    let card_h = widgets::history_card::card_height();
    let visible_count = (area.height / card_h).max(1) as usize;

    let scroll = {
        let sel = app.history_index;
        if sel >= visible_count {
            sel - visible_count + 1
        } else {
            0
        }
    };

    let mut y = area.y;
    for vi in 0..visible_count {
        let idx = scroll + vi;
        if idx >= app.history.completed_tasks.len() {
            break;
        }
        let remaining_height = (area.y + area.height).saturating_sub(y);
        if remaining_height < card_h {
            break;
        }
        let task = &app.history.completed_tasks[idx];
        let card_area = Rect::new(area.x, y, area.width, card_h);
        let selected = idx == app.history_index;
        widgets::history_card::draw(f, task, selected, theme, card_area);
        y += card_h;
    }
}
