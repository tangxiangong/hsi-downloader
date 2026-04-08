use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use yushi_core::config::AppTheme;

use crate::tui::app::{App, InputMode, SETTINGS_GROUPS, SettingField};
use crate::tui::theme::ThemeColors;

/// Draw all settings groups with card-like blocks, plus an About card at the bottom.
pub fn draw(f: &mut Frame, app: &App, theme: &ThemeColors, area: Rect) {
    // Build layout constraints: one per group + About card + spacer
    let mut constraints: Vec<Constraint> = SETTINGS_GROUPS
        .iter()
        .map(|g| Constraint::Length(g.fields.len() as u16 + 2))
        .collect();
    constraints.push(Constraint::Length(3)); // About
    constraints.push(Constraint::Min(0));   // spacer

    let chunks = Layout::vertical(constraints).split(area);

    // Compute global field offset for each group
    let mut field_offset = 0usize;
    for (gi, group) in SETTINGS_GROUPS.iter().enumerate() {
        let group_area = chunks[gi];
        draw_group(f, app, theme, group_area, group.title, group.fields, field_offset);
        field_offset += group.fields.len();
    }

    // About card
    let about_area = chunks[SETTINGS_GROUPS.len()];
    let about_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " 关于 ",
            Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(theme.border));

    let about_text = Paragraph::new(Line::from(vec![
        Span::styled("YuShi v0.1.0  ", Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
        Span::styled("驭时 - 异步下载管理器", Style::default().fg(theme.text_secondary)),
    ]))
    .block(about_block);

    f.render_widget(about_text, about_area);
}

fn draw_group(
    f: &mut Frame,
    app: &App,
    theme: &ThemeColors,
    area: Rect,
    title: &str,
    fields: &[SettingField],
    field_offset: usize,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(theme.border));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // One line per field
    if inner.height == 0 || fields.is_empty() {
        return;
    }

    let field_constraints: Vec<Constraint> = fields
        .iter()
        .map(|_| Constraint::Length(1))
        .collect();
    let field_chunks = Layout::vertical(field_constraints).split(inner);

    for (i, &field) in fields.iter().enumerate() {
        let global_idx = field_offset + i;
        let is_selected = app.setting_index == global_idx;
        let is_editing = is_selected && app.input_mode == InputMode::EditSetting;

        let label = field.label();
        let label_padded = format!("{:<14}", label);

        let value_text: String = if field == SettingField::Theme {
            // Theme field: rendered specially below via spans
            String::new()
        } else if is_editing {
            format!("{}▎", app.edit_buffer)
        } else {
            field.current_value(&app.config).unwrap_or_default()
        };

        if field == SettingField::Theme {
            // Special rendering: [浅色] [*深色*] [系统]
            let current_theme = app.config.theme;
            let options: &[(&str, AppTheme)] =
                &[("浅色", AppTheme::Light), ("深色", AppTheme::Dark), ("系统", AppTheme::System)];

            let mut spans: Vec<Span> = Vec::new();

            if is_selected {
                spans.push(Span::styled(
                    label_padded.clone(),
                    Style::default()
                        .bg(theme.selection_bg)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                spans.push(Span::styled(
                    label_padded.clone(),
                    Style::default().fg(theme.text),
                ));
            }

            for (label_str, variant) in options {
                let active = current_theme == *variant;
                let btn = if active {
                    format!("[*{}*]", label_str)
                } else {
                    format!("[{}]", label_str)
                };
                let style = if active {
                    Style::default()
                        .bg(theme.primary)
                        .fg(theme.bg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text_secondary)
                };
                spans.push(Span::styled(btn, style));
                spans.push(Span::raw(" "));
            }

            let line_style = if is_selected {
                Style::default().bg(theme.selection_bg)
            } else {
                Style::default()
            };

            let para = Paragraph::new(Line::from(spans)).style(line_style);
            f.render_widget(para, field_chunks[i]);
        } else {
            let label_style = if is_selected {
                Style::default()
                    .bg(theme.selection_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };
            let value_style = if is_selected {
                Style::default().bg(theme.selection_bg).fg(theme.text)
            } else {
                Style::default().fg(theme.text_secondary)
            };

            let line = Line::from(vec![
                Span::styled(label_padded, label_style),
                Span::styled(value_text, value_style),
            ]);
            let line_style = if is_selected {
                Style::default().bg(theme.selection_bg)
            } else {
                Style::default()
            };
            let para = Paragraph::new(line).style(line_style);
            f.render_widget(para, field_chunks[i]);
        }
    }
}
