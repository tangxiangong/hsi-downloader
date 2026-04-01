use crate::{components::task_card::task_card, utils, views::YuShiGUI};
use gpui::*;
use gpui_component::{button::*, *};
use yushi_core::DownloadTask;

impl YuShiGUI {
    pub fn render_task_list(
        &mut self,
        tasks: Vec<DownloadTask>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if tasks.is_empty() {
            return v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .gap_4()
                .text_color(cx.theme().foreground)
                .child(div().text_2xl().font_semibold().child("还没有下载任务"))
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("点击右上角 New Task 创建一个新的下载任务。"),
                )
                .child(
                    Button::new("empty-new-task")
                        .custom(utils::button_style(
                            utils::primary_color(cx),
                            gpui_component::white(),
                            cx,
                        ))
                        .label("新建任务")
                        .on_click(cx.listener(|view, _, window, cx| {
                            view.open_add_task_dialog(window, cx);
                        })),
                )
                .into_any_element();
        }

        v_flex()
            .gap_3()
            .children(tasks.into_iter().map(|task| task_card(task, cx)))
            .into_any_element()
    }
}
