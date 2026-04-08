use super::app::{App, CurrentView, InputMode, SETTINGS_FIELDS};
use crate::ui::format_size;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
};
use yushi_core::TaskStatus;

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
            Constraint::Length(5),
        ])
        .split(f.area());

    draw_header(f, app, chunks[0]);
    draw_main_content(f, app, chunks[1]);
    draw_status_bar(f, app, chunks[2]);
    draw_help(f, app, chunks[3]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let title = format!(
        "YuShi 下载管理器  |  [1]任务 [{}] [2]历史 [{}] [3]设置 [{}]",
        if app.current_view == CurrentView::Tasks { "*" } else { " " },
        if app.current_view == CurrentView::History { "*" } else { " " },
        if app.current_view == CurrentView::Settings { "*" } else { " " }
    );

    let title = Paragraph::new(title)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, area);
}

fn draw_main_content(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    match app.current_view {
        CurrentView::Tasks => {
            draw_task_list(f, app, chunks[0]);
            draw_task_details(f, app, chunks[1]);
        }
        CurrentView::History => {
            draw_history_list(f, app, chunks[0]);
            draw_history_details(f, app, chunks[1]);
        }
        CurrentView::Settings => {
            draw_settings_list(f, app, chunks[0]);
            draw_settings_details(f, app, chunks[1]);
        }
    }
}

fn draw_task_list(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .filtered_indices
        .iter()
        .enumerate()
        .filter_map(|(fi, &real_i)| app.tasks.get(real_i).map(|task| (fi, task)))
        .map(|(fi, task)| {
            let status_icon = match task.status {
                TaskStatus::Pending => "⏸",
                TaskStatus::Downloading => "⬇",
                TaskStatus::Paused => "⏸",
                TaskStatus::Completed => "✓",
                TaskStatus::Failed => "✗",
                TaskStatus::Cancelled => "⊗",
            };

            let status_color = match task.status {
                TaskStatus::Pending => Color::Yellow,
                TaskStatus::Downloading => Color::Blue,
                TaskStatus::Paused => Color::Magenta,
                TaskStatus::Completed => Color::Green,
                TaskStatus::Failed => Color::Red,
                TaskStatus::Cancelled => Color::DarkGray,
            };

            let progress = if task.total_size > 0 {
                (task.downloaded as f64 / task.total_size as f64 * 100.0) as u16
            } else {
                0
            };

            let filename = task
                .dest
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            let size_str = if task.total_size > 0 {
                format!(
                    "{} / {}",
                    format_size(task.downloaded),
                    format_size(task.total_size)
                )
            } else {
                "未知大小".to_string()
            };

            let speed_str = if task.speed > 0 {
                format!(" @ {}/s", format_size(task.speed))
            } else {
                String::new()
            };

            let content = vec![
                Line::from(vec![
                    Span::styled(
                        format!("{} ", status_icon),
                        Style::default().fg(status_color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(filename, Style::default().add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::raw(format!("  {}%  ", progress)),
                    Span::styled(size_str, Style::default().fg(Color::Gray)),
                    Span::styled(speed_str, Style::default().fg(Color::Cyan)),
                ]),
            ];

            let style = if fi == app.selected_index {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("任务列表")
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(list, area);
}

fn draw_task_details(f: &mut Frame, app: &App, area: Rect) {
    if let Some(task) = app.get_selected_task() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(10), Constraint::Length(3)])
            .split(area);

        let mut lines = vec![
            detail_line("ID", &task.id),
            Line::from(""),
            detail_line("URL", &task.url),
            Line::from(""),
            detail_line("输出", &task.dest.display().to_string()),
            Line::from(""),
            Line::from(vec![
                Span::styled("状态: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("{:?}", task.status),
                    Style::default().fg(match task.status {
                        TaskStatus::Completed => Color::Green,
                        TaskStatus::Failed => Color::Red,
                        TaskStatus::Downloading => Color::Blue,
                        _ => Color::Yellow,
                    }),
                ),
            ]),
            Line::from(""),
            detail_line("优先级", &format!("{:?}", task.priority)),
        ];

        if let Some(limit) = task.speed_limit {
            lines.push(Line::from(""));
            lines.push(detail_line("限速", &format!("{}/s", format_size(limit))));
        }

        if let Some(error) = &task.error {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "错误: ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )]));
            lines.push(Line::from(format!("  {}", error)));
        }

        if let Some(eta) = task.eta {
            lines.push(Line::from(""));
            lines.push(detail_line("预计剩余", &format!("{}s", eta)));
        }

        let details = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("任务详情"))
            .wrap(Wrap { trim: true });

        f.render_widget(details, chunks[0]);

        let progress = if task.total_size > 0 {
            (task.downloaded as f64 / task.total_size as f64 * 100.0) as u16
        } else {
            0
        };

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("进度"))
            .gauge_style(
                Style::default()
                    .fg(Color::Cyan)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .percent(progress)
            .label(format!("{}%", progress));

        f.render_widget(gauge, chunks[1]);
    } else {
        let empty = Paragraph::new("没有选中的任务")
            .block(Block::default().borders(Borders::ALL).title("任务详情"))
            .alignment(Alignment::Center);
        f.render_widget(empty, area);
    }
}

fn draw_history_list(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .history
        .completed_tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let filename = task
                .dest
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            let style = if i == app.history_index {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled("✓ ", Style::default().fg(Color::Green)),
                    Span::styled(filename, Style::default().add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled(format_size(task.total_size), Style::default().fg(Color::Gray)),
                    Span::raw("  "),
                    Span::styled(
                        format!("平均 {}/s", format_size(task.avg_speed)),
                        Style::default().fg(Color::Cyan),
                    ),
                ]),
            ])
            .style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("历史记录")
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(list, area);
}

fn draw_history_details(f: &mut Frame, app: &App, area: Rect) {
    if let Some(task) = app.get_selected_history() {
        let lines = vec![
            detail_line("ID", &task.id),
            Line::from(""),
            detail_line("URL", &task.url),
            Line::from(""),
            detail_line("输出", &task.dest.display().to_string()),
            Line::from(""),
            detail_line("文件大小", &format_size(task.total_size)),
            detail_line("平均速度", &format!("{}/s", format_size(task.avg_speed))),
            detail_line("耗时", &format!("{}s", task.duration)),
            detail_line("完成时间", &task.completed_at.to_string()),
        ];

        let details = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("历史详情"))
            .wrap(Wrap { trim: true });
        f.render_widget(details, area);
    } else {
        let empty = Paragraph::new("没有历史记录")
            .block(Block::default().borders(Borders::ALL).title("历史详情"))
            .alignment(Alignment::Center);
        f.render_widget(empty, area);
    }
}

fn draw_settings_list(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = SETTINGS_FIELDS
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let style = if i == app.setting_index {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(vec![
                Span::styled(field.label(), Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled(
                    field.current_value(&app.config).unwrap_or_default(),
                    Style::default().fg(Color::Gray),
                ),
            ]))
            .style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("设置")
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(list, area);
}

fn draw_settings_details(f: &mut Frame, app: &App, area: Rect) {
    let field = app.selected_setting();
    let mut lines = vec![
        detail_line("字段", field.label()),
        Line::from(""),
        detail_line(
            "当前值",
            &field
                .current_value(&app.config)
                .unwrap_or_else(|| "未设置".into()),
        ),
        Line::from(""),
        Line::from("按 Enter 或 e 编辑当前字段。"),
        Line::from("保存后会立即写入共享配置文件。"),
    ];

    if app.input_mode == InputMode::EditSetting {
        lines.push(Line::from(""));
        lines.push(detail_line("输入中", &app.edit_buffer));
    }

    let details = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("设置详情"))
        .wrap(Wrap { trim: true });
    f.render_widget(details, area);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let status_text = match app.input_mode {
        InputMode::EditSetting => format!("输入: {}", app.edit_buffer),
        InputMode::AddTask => "添加任务中...".to_string(),
        InputMode::Confirm => "确认操作...".to_string(),
        InputMode::Normal => app.status_message.clone(),
    };

    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("状态"));

    f.render_widget(status, area);
}

fn draw_help(f: &mut Frame, app: &App, area: Rect) {
    let help_text = match app.input_mode {
        InputMode::Normal => match app.current_view {
            CurrentView::Tasks => {
                "1/2/3:切视图 | q:退出 | ↑↓/jk:导航 | Tab/←→:切过滤 | a:添加 | p:暂停/恢复 | c:取消 | d:删除 | D:删除含文件 | r:刷新"
            }
            CurrentView::History => {
                "1/2/3:切视图 | q:退出 | ↑↓/jk:导航 | C:清空历史 | r:刷新历史"
            }
            CurrentView::Settings => {
                "1/2/3:切视图 | q:退出 | ↑↓/jk:选择字段 | Enter/e:编辑/切换主题 | ←→:切换主题 | r:重载配置"
            }
        },
        InputMode::AddTask => "Tab:下一字段 | Shift+Tab:上一字段 | ←→:切换选项 | Enter:确认/下一步 | Esc:取消",
        InputMode::EditSetting => "Enter:保存设置 | Esc:取消编辑",
        InputMode::Confirm => "←→:选择 | Enter:确认 | Esc:取消",
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("帮助"));

    f.render_widget(help, area);
}

fn detail_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{label}: "),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(value.to_string()),
    ])
}
