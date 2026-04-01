use crate::utils::ViewStats;
use gpui::*;
use gpui_component::*;

pub fn summary_row(stats: &ViewStats, cx: &App) -> Div {
    h_flex()
        .gap_3()
        .p_4()
        .child(stat_card(
            "总任务",
            stats.total_tasks,
            "队列中的所有任务",
            cx.theme().primary,
            cx,
        ))
        .child(stat_card(
            "进行中",
            stats.active_tasks,
            "包含等待和下载中",
            cx.theme().blue,
            cx,
        ))
        .child(stat_card(
            "已完成",
            stats.completed_tasks,
            "当前队列完成状态",
            cx.theme().green,
            cx,
        ))
        .child(stat_card(
            "历史",
            stats.history_items,
            "历史记录条目数",
            cx.theme().yellow,
            cx,
        ))
}

fn stat_card(
    title: &'static str,
    value: usize,
    subtitle: &'static str,
    color: Hsla,
    cx: &App,
) -> Div {
    v_flex()
        .gap_1()
        .min_w(px(150.))
        .p_3()
        .rounded(px(12.))
        .bg(cx.theme().secondary)
        .border_1()
        .border_color(cx.theme().border)
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(title),
        )
        .child(
            div()
                .text_xl()
                .font_semibold()
                .text_color(color)
                .child(value.to_string()),
        )
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(subtitle),
        )
}
