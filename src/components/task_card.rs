use crate::{
    utils::{
        self, progress_percent, {TaskAction, task_actions},
    },
    views::YuShiGUI,
};
use gpui::*;
use gpui_component::{button::*, progress::Progress, *};
use yushi_core::DownloadTask;

pub fn task_card(task: DownloadTask, cx: &mut Context<YuShiGUI>) -> Div {
    let task_id: SharedString = task.id.clone().into();
    let actions = task_actions(task.status);

    v_flex()
        .gap_4()
        .p_5()
        .rounded_xl()
        .bg(utils::app_background(cx))
        .shadow_sm()
        .text_color(utils::text_color(cx))
        .child(
            h_flex()
                .justify_between()
                .items_start()
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div().text_lg().font_semibold().child(
                                task.url
                                    .split('/')
                                    .next_back()
                                    .unwrap_or(task.url.as_str())
                                    .to_string(),
                            ),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(utils::muted_text_color(cx))
                                .child(task.url.clone()),
                        ),
                )
                .child(utils::status_badge(task.status, cx)),
        )
        .child(
            div()
                .py_2()
                .child(Progress::new("progress").value(progress_percent(&task))),
        )
        .child(
            h_flex()
                .justify_between()
                .items_center()
                .child(
                    v_flex()
                        .gap_1()
                        .child(div().text_sm().font_medium().child(format!(
                            "{} / {}  ·  {} /s",
                            utils::format_bytes(task.downloaded),
                            utils::format_bytes(task.total_size),
                            utils::format_bytes(task.speed),
                        )))
                        .child(
                            div()
                                .text_xs()
                                .text_color(utils::muted_text_color(cx))
                                .child(match task.speed_limit {
                                    Some(limit) => format!(
                                        "限速 {}/s · 保存到 {}",
                                        utils::format_bytes(limit),
                                        task.dest.display()
                                    ),
                                    None => format!("不限速 · 保存到 {}", task.dest.display()),
                                }),
                        ),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .children(actions.into_iter().enumerate().map(|(index, action)| {
                            let button_id = SharedString::from(format!(
                                "task-{}-{}-{}",
                                task.id,
                                index,
                                action.id_suffix()
                            ));
                            let task_id = task_id.clone();
                            let destination = task.dest.display().to_string();
                            let button_style = if action.is_primary() {
                                utils::button_style(
                                    utils::primary_color(cx),
                                    gpui_component::white(),
                                    cx,
                                )
                            } else {
                                utils::button_style(
                                    utils::panel_color(cx),
                                    utils::text_color(cx),
                                    cx,
                                )
                            };

                            Button::new(button_id)
                                .custom(button_style)
                                .label(action.button_label())
                                .on_click(cx.listener(move |view, _, window, cx| {
                                    if action == TaskAction::DeleteFile {
                                        view.open_task_delete_file_dialog(
                                            task_id.clone(),
                                            destination.clone(),
                                            window,
                                            cx,
                                        );
                                    } else {
                                        view.run_task_action(task_id.clone(), action, window, cx);
                                    }
                                }))
                        })),
                ),
        )
}
