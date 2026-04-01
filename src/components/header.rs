use crate::{utils, views::YuShiGUI};
use gpui::*;
use gpui_component::{button::*, *};

pub fn header(cx: &mut Context<YuShiGUI>) -> impl IntoElement {
    h_flex()
        .w_full()
        .justify_between()
        .items_center()
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .text_color(utils::text_color(cx))
                .child(IconName::Inbox)
                .child(v_flex().gap_1().child(div().font_semibold().child("YuShi"))),
        )
        .child(
            h_flex().gap_2().items_center().child(
                Button::new("new-task")
                    .custom(utils::button_style(
                        utils::primary_color(cx),
                        gpui_component::white(),
                        cx,
                    ))
                    .label("新建任务")
                    .on_click(cx.listener(|view, _, window, cx| {
                        view.open_add_task_dialog(window, cx);
                    })),
            ),
        )
}
