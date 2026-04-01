use crate::{
    components::{
        content_panel::content_panel, header::header, nav_sidebar::nav_sidebar,
        summary_row::summary_row,
    },
    state::ViewKind,
    utils::{self, ViewStats},
    views::{YuShiGUI, task_list::filter_tasks},
};
use gpui::*;
use gpui_component::*;

impl Render for YuShiGUI {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (current_view, tasks, status_message) = self.app_state.read_with(cx, |state, _| {
            (
                state.current_view,
                state.tasks.clone(),
                state.status_message.clone(),
            )
        });
        let stats = ViewStats::from_state(
            &tasks,
            self.app_state.read(cx).history.completed_tasks.len(),
        );

        let content = match current_view {
            ViewKind::AllTasks | ViewKind::Downloading | ViewKind::Completed => self
                .render_task_list(
                    filter_tasks(&tasks, current_view)
                        .into_iter()
                        .cloned()
                        .collect(),
                    window,
                    cx,
                ),
            ViewKind::History => self.render_history(window, cx),
            ViewKind::Settings => self.render_settings(window, cx),
        };
        let content_panel = content_panel(current_view, content, status_message, cx);
        let sidebar = nav_sidebar(current_view, &stats, cx);
        let summary_row = summary_row(&stats, cx);

        div()
            .size_full()
            .bg(utils::app_background(cx))
            .text_color(utils::text_color(cx))
            .child(
                v_flex()
                    .size_full()
                    .child(TitleBar::new().child(header(cx)))
                    .child(
                        h_flex().size_full().child(sidebar).child(
                            v_flex()
                                .size_full()
                                .gap_3()
                                .child(summary_row)
                                .child(content_panel),
                        ),
                    ),
            )
            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_notification_layer(window, cx))
    }
}
