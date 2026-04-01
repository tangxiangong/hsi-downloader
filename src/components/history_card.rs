use crate::{
    utils::{self, HistoryAction, history_actions},
    views::YuShiGUI,
};
use gpui::*;
use gpui_component::{button::*, *};
use yushi_core::CompletedTask;

pub fn history_card(item: CompletedTask, cx: &mut Context<YuShiGUI>) -> Div {
    let id = item.id.clone();

    v_flex()
        .gap_2()
        .p_4()
        .border_1()
        .border_color(utils::border_color(cx))
        .rounded(px(12.))
        .bg(utils::card_color(cx))
        .text_color(utils::text_color(cx))
        .child(
            div().font_semibold().child(
                item.url
                    .split('/')
                    .next_back()
                    .unwrap_or(item.url.as_str())
                    .to_string(),
            ),
        )
        .child(
            div()
                .text_sm()
                .text_color(utils::muted_text_color(cx))
                .child(item.url),
        )
        .child(
            div()
                .text_sm()
                .text_color(utils::muted_text_color(cx))
                .child(item.dest.display().to_string()),
        )
        .child(format!(
            "{}  ·  平均 {} /s  ·  用时 {}s",
            utils::format_bytes(item.total_size),
            utils::format_bytes(item.avg_speed),
            item.duration
        ))
        .child(
            h_flex()
                .gap_2()
                .children(
                    history_actions()
                        .into_iter()
                        .enumerate()
                        .map(|(index, action)| {
                            let button_id = SharedString::from(format!(
                                "history-{}-{}-{}",
                                id,
                                index,
                                action.id_suffix()
                            ));
                            let history_id: SharedString = id.clone().into();
                            let destination = item.dest.display().to_string();

                            Button::new(button_id)
                                .custom(utils::button_style(
                                    utils::panel_color(cx),
                                    utils::text_color(cx),
                                    cx,
                                ))
                                .label(action.button_label())
                                .on_click(cx.listener(move |view, _, window, cx| {
                                    if action == HistoryAction::DeleteFile {
                                        view.open_history_delete_file_dialog(
                                            history_id.clone(),
                                            destination.clone(),
                                            window,
                                            cx,
                                        );
                                    } else {
                                        view.run_history_action(
                                            history_id.clone(),
                                            action,
                                            window,
                                            cx,
                                        );
                                    }
                                }))
                        }),
                ),
        )
}
