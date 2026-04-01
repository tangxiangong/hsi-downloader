use crate::{state::ViewKind, utils};
use gpui::*;
use gpui_component::*;

pub fn content_panel(
    current_view: ViewKind,
    content: AnyElement,
    status_message: Option<String>,
    cx: &App,
) -> Div {
    let panel = v_flex()
        .size_full()
        .p_4()
        .gap_4()
        .text_color(utils::text_color(cx))
        .bg(utils::app_background(cx))
        .child(
            div()
                .pb_2()
                .border_b_1()
                .border_color(utils::border_color(cx))
                .child(view_title(current_view)),
        )
        .child(
            div()
                .text_sm()
                .text_color(utils::muted_text_color(cx))
                .child(view_description(current_view)),
        )
        .child(content);

    match status_message {
        Some(message) => panel.child(
            div()
                .text_sm()
                .text_color(utils::muted_text_color(cx))
                .child(message),
        ),
        None => panel,
    }
}

fn view_title(view: ViewKind) -> &'static str {
    match view {
        ViewKind::AllTasks => "所有任务",
        ViewKind::Downloading => "下载中",
        ViewKind::Completed => "已完成",
        ViewKind::History => "历史记录",
        ViewKind::Settings => "设置",
    }
}

fn view_description(view: ViewKind) -> &'static str {
    match view {
        ViewKind::AllTasks => "查看和管理队列中的全部下载任务。",
        ViewKind::Downloading => "关注正在执行或等待执行的任务。",
        ViewKind::Completed => "只显示当前队列里已经完成的任务。",
        ViewKind::History => "浏览历史下载记录并进行搜索或清理。",
        ViewKind::Settings => "修改共享配置，供桌面端、CLI 和 TUI 共用。",
    }
}
