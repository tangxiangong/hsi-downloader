use crate::{
    components::history_card::history_card,
    utils::{self, search_history},
    views::YuShiGUI,
};
use gpui::*;
use gpui_component::{button::*, input::Input, *};

impl YuShiGUI {
    pub fn render_history(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let query = self.history_search_input.read(cx).value().to_string();
        let history = self.app_state.read(cx).history.clone();
        let items = search_history(&history, &query);

        let list = if items.is_empty() {
            v_flex()
                .gap_2()
                .text_color(utils::text_color(cx))
                .child("没有匹配的历史记录")
                .into_any_element()
        } else {
            v_flex()
                .gap_3()
                .children(items.into_iter().map(|item| history_card(item, cx)))
                .into_any_element()
        };

        v_flex()
            .gap_3()
            .text_color(utils::text_color(cx))
            .child(
                h_flex()
                    .gap_2()
                    .child(Input::new(&self.history_search_input))
                    .child(
                        Button::new("refresh-history-search")
                            .custom(utils::button_style(
                                utils::panel_color(cx),
                                utils::text_color(cx),
                                cx,
                            ))
                            .label("搜索")
                            .on_click(cx.listener(|_, _, _, cx| cx.notify())),
                    )
                    .child(
                        Button::new("clear-history")
                            .custom(utils::button_style(
                                utils::panel_color(cx),
                                utils::text_color(cx),
                                cx,
                            ))
                            .label("清空历史")
                            .on_click(cx.listener(|view, _, window, cx| {
                                let history_path = view
                                    .app_state
                                    .read_with(cx, |state, _| state.history_path.clone());
                                cx.spawn_in(window, async move |view, window| {
                                    let result = async move {
                                        let history =
                                            yushi_core::DownloadHistory::clear_file(&history_path)
                                                .await?;
                                        Ok::<_, anyhow::Error>(history)
                                    }
                                    .await;

                                    let _ =
                                        view.update_in(
                                            window,
                                            move |view, window, cx| match result {
                                                Ok(history) => {
                                                    view.app_state.update(cx, |state, cx| {
                                                        state.history = history;
                                                        state.status_message =
                                                            Some("Cleared history".into());
                                                        cx.notify();
                                                    });
                                                }
                                                Err(err) => {
                                                    window.push_notification(err.to_string(), cx)
                                                }
                                            },
                                        );
                                })
                                .detach();
                            })),
                    ),
            )
            .child(list)
            .into_any_element()
    }
}
