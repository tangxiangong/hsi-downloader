use crate::{
    utils::{self, AddTaskDraft, HistoryAction, TaskAction},
    views::YuShiGUI,
};
use gpui::*;
use gpui_component::{button::*, input::Input, *};

impl YuShiGUI {
    pub fn open_add_task_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.add_url_input
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.add_dest_input
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.add_speed_input
            .update(cx, |input, cx| input.set_value("", window, cx));

        let app_state = self.app_state.clone();
        let add_url_input = self.add_url_input.clone();
        let add_dest_input = self.add_dest_input.clone();
        let add_speed_input = self.add_speed_input.clone();
        let primary_style =
            utils::button_style(utils::primary_color(cx), gpui_component::white(), cx);
        let secondary_style =
            utils::button_style(utils::panel_color(cx), utils::text_color(cx), cx);

        window.open_dialog(cx, move |dialog, _, _cx| {
            dialog
                .title("Add Download Task")
                .child(
                    v_flex()
                        .gap_3()
                        .child("Use the default download directory by leaving destination blank.")
                        .child(Input::new(&add_url_input))
                        .child(Input::new(&add_dest_input))
                        .child(Input::new(&add_speed_input)),
                )
                .footer(
                    div()
                        .child(
                            Button::new("submit-add-task")
                                .custom(primary_style)
                                .label("Add")
                                .on_click({
                                    let app_state = app_state.clone();
                                    let add_url_input = add_url_input.clone();
                                    let add_dest_input = add_dest_input.clone();
                                    let add_speed_input = add_speed_input.clone();
                                    move |_, window, cx| {
                                        let draft = match AddTaskDraft::parse(
                                            &add_url_input.read(cx).value(),
                                            &add_dest_input.read(cx).value(),
                                            &add_speed_input.read(cx).value(),
                                        ) {
                                            Ok(draft) => draft,
                                            Err(err) => {
                                                window.push_notification(err.to_string(), cx);
                                                return;
                                            }
                                        };

                                        let (queue, config) = app_state
                                            .read_with(cx, |state, _| {
                                                (state.queue.clone(), state.config.clone())
                                            });
                                        let app_state = app_state.clone();
                                        window
                                            .spawn(cx, async move |window| {
                                                let destination = draft
                                                    .resolve_destination(&queue, &config)
                                                    .await;
                                                let url = draft.url.clone();

                                                let result = async {
                                                    queue
                                                        .add_task_with_options(
                                                            draft.url.clone(),
                                                            destination.clone(),
                                                            yushi_core::TaskPriority::Normal,
                                                            None,
                                                            draft.speed_limit,
                                                            false,
                                                        )
                                                        .await?;
                                                    let tasks = queue.get_all_tasks().await;
                                                    Ok::<_, anyhow::Error>((tasks, destination))
                                                }
                                                .await;

                                                let _ = app_state.update_in(
                                                    window,
                                                    move |state, window, cx| match result {
                                                        Ok((tasks, destination)) => {
                                                            state.tasks = tasks;
                                                            state.status_message =
                                                                Some(format!("Added {}", url));
                                                            cx.notify();
                                                            window.close_dialog(cx);
                                                            window.push_notification(
                                                                format!(
                                                                    "Task added: {}",
                                                                    destination.display()
                                                                ),
                                                                cx,
                                                            );
                                                        }
                                                        Err(err) => {
                                                            window.push_notification(
                                                                err.to_string(),
                                                                cx,
                                                            );
                                                        }
                                                    },
                                                );
                                            })
                                            .detach();
                                    }
                                }),
                        )
                        .child(
                            Button::new("cancel-add-task")
                                .custom(secondary_style)
                                .label("Cancel")
                                .on_click(|_, window, cx| window.close_dialog(cx)),
                        ),
                )
        });
    }

    pub fn open_task_delete_file_dialog(
        &mut self,
        task_id: SharedString,
        destination: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let app_state = self.app_state.clone();
        let confirm_style = utils::button_style(cx.theme().red, gpui_component::white(), cx);
        let cancel_style = utils::button_style(utils::panel_color(cx), utils::text_color(cx), cx);
        let muted = utils::muted_text_color(cx);

        window.open_dialog(cx, move |dialog, _, _cx| {
            dialog
                .title("确认删除文件")
                .child(
                    v_flex()
                        .gap_3()
                        .child("这会删除本地文件，并将该任务从任务列表中移除。")
                        .child(div().text_sm().text_color(muted).child(destination.clone())),
                )
                .footer(
                    div()
                        .child(
                            Button::new(SharedString::from(format!(
                                "confirm-delete-task-file-{}",
                                task_id
                            )))
                            .custom(confirm_style)
                            .label("删除文件")
                            .on_click({
                                let app_state = app_state.clone();
                                let task_id = task_id.clone();
                                let destination = destination.clone();
                                move |_, window, cx| {
                                    let queue =
                                        app_state.read_with(cx, |state, _| state.queue.clone());
                                    let app_state = app_state.clone();
                                    let task_id_for_async = task_id.to_string();
                                    let notification_target = destination.clone();

                                    window
                                        .spawn(cx, async move |window| {
                                            let result = async {
                                                queue
                                                    .remove_task_with_file(&task_id_for_async)
                                                    .await?;
                                                let tasks = queue.get_all_tasks().await;
                                                Ok::<_, anyhow::Error>(tasks)
                                            }
                                            .await;

                                            let _ = app_state.update_in(
                                                window,
                                                move |state, window, cx| match result {
                                                    Ok(tasks) => {
                                                        state.tasks = tasks;
                                                        state.status_message = Some(format!(
                                                            "Deleted file for task {}",
                                                            task_id_for_async
                                                        ));
                                                        cx.notify();
                                                        window.close_dialog(cx);
                                                        window.push_notification(
                                                            format!(
                                                                "已删除文件: {}",
                                                                notification_target
                                                            ),
                                                            cx,
                                                        );
                                                    }
                                                    Err(err) => {
                                                        window
                                                            .push_notification(err.to_string(), cx);
                                                    }
                                                },
                                            );
                                        })
                                        .detach();
                                }
                            }),
                        )
                        .child(
                            Button::new(SharedString::from(format!(
                                "cancel-delete-task-file-{}",
                                task_id
                            )))
                            .custom(cancel_style)
                            .label("取消")
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                        ),
                )
        });
    }

    pub fn open_history_delete_file_dialog(
        &mut self,
        history_id: SharedString,
        destination: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let app_state = self.app_state.clone();
        let history_path = self
            .app_state
            .read_with(cx, |state, _| state.history_path.clone());
        let confirm_style = utils::button_style(cx.theme().red, gpui_component::white(), cx);
        let cancel_style = utils::button_style(utils::panel_color(cx), utils::text_color(cx), cx);
        let muted = utils::muted_text_color(cx);

        window.open_dialog(cx, move |dialog, _, _cx| {
            dialog
                .title("确认删除历史文件")
                .child(
                    v_flex()
                        .gap_3()
                        .child("这会删除本地文件，并同时移除这条历史记录。")
                        .child(
                            div()
                                .text_sm()
                                .text_color(muted)
                                .child(destination.clone()),
                        ),
                )
                .footer(
                    div()
                        .child(
                            Button::new(SharedString::from(format!(
                                "confirm-delete-history-file-{}",
                                history_id
                            )))
                            .custom(confirm_style)
                            .label("删除文件")
                            .on_click({
                                let app_state = app_state.clone();
                                let history_id = history_id.clone();
                                let history_path = history_path.clone();
                                let destination = destination.clone();
                                move |_, window, cx| {
                                    let app_state = app_state.clone();
                                    let history_path = history_path.clone();
                                    let history_id_for_async = history_id.to_string();
                                    let notification_target = destination.clone();

                                    window
                                        .spawn(cx, async move |window| {
                                            let result = async {
                                                yushi_core::DownloadHistory::remove_entry_and_file_from_file(
                                                    &history_path,
                                                    &history_id_for_async,
                                                )
                                                .await
                                            }
                                            .await;

                                            let _ = app_state.update_in(
                                                window,
                                                move |state, window, cx| match result {
                                                    Ok((history, true)) => {
                                                        state.history = history;
                                                        state.status_message = Some(format!(
                                                            "Deleted history file {}",
                                                            history_id_for_async
                                                        ));
                                                        cx.notify();
                                                        window.close_dialog(cx);
                                                        window.push_notification(
                                                            format!(
                                                                "已删除文件: {}",
                                                                notification_target
                                                            ),
                                                            cx,
                                                        );
                                                    }
                                                    Ok((_, false)) => window.push_notification(
                                                        "History item not found",
                                                        cx,
                                                    ),
                                                    Err(err) => {
                                                        window.push_notification(
                                                            err.to_string(),
                                                            cx,
                                                        );
                                                    }
                                                },
                                            );
                                        })
                                        .detach();
                                }
                            }),
                        )
                        .child(
                            Button::new(SharedString::from(format!(
                                "cancel-delete-history-file-{}",
                                history_id
                            )))
                            .custom(cancel_style)
                            .label("取消")
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                        ),
                )
        });
    }

    pub fn run_task_action(
        &mut self,
        task_id: SharedString,
        action: TaskAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let queue = self.app_state.read_with(cx, |state, _| state.queue.clone());
        let task_id_for_async = task_id.to_string();
        cx.spawn_in(window, async move |view, window| {
            let result = async {
                match action {
                    TaskAction::Pause => queue.pause_task(&task_id_for_async).await?,
                    TaskAction::Resume => queue.resume_task(&task_id_for_async).await?,
                    TaskAction::Cancel => queue.cancel_task(&task_id_for_async).await?,
                    TaskAction::Remove => queue.remove_task(&task_id_for_async).await?,
                    TaskAction::DeleteFile => {
                        queue.remove_task_with_file(&task_id_for_async).await?
                    }
                }

                Ok::<_, anyhow::Error>(queue.get_all_tasks().await)
            }
            .await;

            let _ = view.update_in(window, move |view, window, cx| match result {
                Ok(tasks) => {
                    view.app_state.update(cx, |state, cx| {
                        state.tasks = tasks;
                        state.status_message =
                            Some(format!("{} {}", action.label(), task_id_for_async));
                        cx.notify();
                    });
                }
                Err(err) => window.push_notification(err.to_string(), cx),
            });
        })
        .detach();
    }

    pub fn run_history_action(
        &mut self,
        history_id: SharedString,
        action: HistoryAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let history_path = self
            .app_state
            .read_with(cx, |state, _| state.history_path.clone());
        let history_id_for_async = history_id.to_string();

        cx.spawn_in(window, async move |view, window| {
            let result = async {
                let (history, changed) = match action {
                    HistoryAction::RemoveRecord => {
                        yushi_core::DownloadHistory::remove_from_file(
                            &history_path,
                            &history_id_for_async,
                        )
                        .await?
                    }
                    HistoryAction::DeleteFile => {
                        yushi_core::DownloadHistory::remove_entry_and_file_from_file(
                            &history_path,
                            &history_id_for_async,
                        )
                        .await?
                    }
                };

                Ok::<_, anyhow::Error>((history, changed))
            }
            .await;

            let _ = view.update_in(window, move |view, window, cx| match result {
                Ok((history, true)) => {
                    view.app_state.update(cx, |state, cx| {
                        state.history = history;
                        state.status_message =
                            Some(format!("{} {}", action.label(), history_id_for_async));
                        cx.notify();
                    });
                }
                Ok((_, false)) => window.push_notification(action.not_found_message(), cx),
                Err(err) => window.push_notification(err.to_string(), cx),
            });
        })
        .detach();
    }
}
