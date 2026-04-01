use crate::{
    state::ViewKind,
    utils::{self, ViewStats},
    views::YuShiGUI,
};
use gpui::*;
use gpui_component::{button::*, *};

pub fn nav_sidebar(current_view: ViewKind, stats: &ViewStats, cx: &mut Context<YuShiGUI>) -> Div {
    v_flex()
        .w(px(240.))
        .h_full()
        .flex_shrink_0()
        .justify_between()
        .bg(utils::panel_color(cx))
        .border_r_1()
        .border_color(utils::border_color(cx))
        .p_4()
        .child(
            v_flex()
                .gap_4()
                .child(
                    v_flex()
                        .gap_2()
                        .child(
                            div()
                                .font_semibold()
                                .text_color(utils::text_color(cx))
                                .child("导航"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(utils::muted_text_color(cx))
                                .child("任务、历史和设置都在这里。"),
                        ),
                )
                .child(nav_item(
                    "All Tasks",
                    IconName::LayoutDashboard,
                    current_view == ViewKind::AllTasks,
                    cx.listener(|view, _, window, cx| {
                        view.set_view(ViewKind::AllTasks, window, cx);
                    }),
                    cx,
                ))
                .child(nav_item(
                    "Downloading",
                    IconName::ArrowDown,
                    current_view == ViewKind::Downloading,
                    cx.listener(|view, _, window, cx| {
                        view.set_view(ViewKind::Downloading, window, cx);
                    }),
                    cx,
                ))
                .child(nav_item(
                    "Completed",
                    IconName::CircleCheck,
                    current_view == ViewKind::Completed,
                    cx.listener(|view, _, window, cx| {
                        view.set_view(ViewKind::Completed, window, cx);
                    }),
                    cx,
                ))
                .child(nav_item(
                    "History",
                    IconName::BookOpen,
                    current_view == ViewKind::History,
                    cx.listener(|view, _, window, cx| {
                        view.set_view(ViewKind::History, window, cx);
                    }),
                    cx,
                ))
                .child(nav_item(
                    "Settings",
                    IconName::Settings,
                    current_view == ViewKind::Settings,
                    cx.listener(|view, _, window, cx| {
                        view.set_view(ViewKind::Settings, window, cx);
                    }),
                    cx,
                )),
        )
        .child(
            v_flex()
                .gap_1()
                .p_3()
                .rounded(px(12.))
                .bg(utils::card_color(cx))
                .border_1()
                .border_color(utils::border_color(cx))
                .child(
                    div()
                        .text_sm()
                        .font_semibold()
                        .text_color(utils::text_color(cx))
                        .child("当前概览"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(utils::muted_text_color(cx))
                        .child(format!(
                            "共 {} 个任务，{} 条历史",
                            stats.total_tasks, stats.history_items
                        )),
                ),
        )
}

fn nav_item(
    label: &'static str,
    icon: IconName,
    active: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    cx: &App,
) -> impl IntoElement {
    Button::new(SharedString::from(format!("nav-{label}")))
        .custom(utils::button_style(
            if active {
                utils::primary_color(cx).opacity(0.16)
            } else {
                utils::panel_color(cx)
            },
            if active {
                utils::primary_color(cx)
            } else {
                utils::text_color(cx)
            },
            cx,
        ))
        .icon(icon)
        .label(label)
        .w_full()
        .justify_start()
        .on_click(on_click)
}
