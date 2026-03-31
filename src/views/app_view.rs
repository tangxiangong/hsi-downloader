use crate::{
    app_state::{AppState, ViewKind, default_destination},
    components::task_item::progress_percent,
    views::{history::search_history, settings::sanitize_theme, task_list::filter_tasks},
};
use anyhow::Result;
use gpui::*;
use gpui_component::{
    ActiveTheme as _, IconName, Root, Selectable, StyledExt, TitleBar, WindowExt,
    button::*,
    h_flex,
    input::{Input, InputState},
    progress::Progress,
    v_flex,
};
use std::path::PathBuf;
use tokio::{runtime::Handle, task::block_in_place};
use yushi_core::{AppConfig, DownloadTask, TaskStatus};

pub struct AppView {
    app_state: Entity<AppState>,
    add_url_input: Entity<InputState>,
    add_dest_input: Entity<InputState>,
    history_search_input: Entity<InputState>,
    settings_path_input: Entity<InputState>,
    settings_downloads_input: Entity<InputState>,
    settings_tasks_input: Entity<InputState>,
    settings_chunk_input: Entity<InputState>,
    settings_timeout_input: Entity<InputState>,
    settings_user_agent_input: Entity<InputState>,
    theme_choice: String,
}

impl AppView {
    pub fn new(app_state: Entity<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let config = app_state.read(cx).config.clone();

        Self {
            app_state,
            add_url_input: cx
                .new(|cx| InputState::new(window, cx).placeholder("https://example.com/file.iso")),
            add_dest_input: cx.new(|cx| {
                InputState::new(window, cx).placeholder("Leave empty to use default path")
            }),
            history_search_input: cx
                .new(|cx| InputState::new(window, cx).placeholder("Search URL or file path")),
            settings_path_input: cx.new(|cx| {
                InputState::new(window, cx)
                    .default_value(config.default_download_path.display().to_string())
            }),
            settings_downloads_input: cx.new(|cx| {
                InputState::new(window, cx)
                    .default_value(config.max_concurrent_downloads.to_string())
            }),
            settings_tasks_input: cx.new(|cx| {
                InputState::new(window, cx).default_value(config.max_concurrent_tasks.to_string())
            }),
            settings_chunk_input: cx
                .new(|cx| InputState::new(window, cx).default_value(config.chunk_size.to_string())),
            settings_timeout_input: cx
                .new(|cx| InputState::new(window, cx).default_value(config.timeout.to_string())),
            settings_user_agent_input: cx
                .new(|cx| InputState::new(window, cx).default_value(config.user_agent.clone())),
            theme_choice: config.theme,
        }
    }

    fn run_async<T>(&self, future: impl std::future::Future<Output = Result<T>>) -> Result<T> {
        block_in_place(|| Handle::current().block_on(future))
    }

    fn set_view(&mut self, view: ViewKind, window: &mut Window, cx: &mut Context<Self>) {
        self.app_state.update(cx, |state, cx| {
            state.current_view = view;
            cx.notify();
        });
        if view == ViewKind::Settings {
            self.sync_settings_inputs(window, cx);
        }
    }

    fn sync_settings_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let config = self.app_state.read(cx).config.clone();

        self.settings_path_input.update(cx, |input, cx| {
            input.set_value(
                config.default_download_path.display().to_string(),
                window,
                cx,
            )
        });
        self.settings_downloads_input.update(cx, |input, cx| {
            input.set_value(config.max_concurrent_downloads.to_string(), window, cx)
        });
        self.settings_tasks_input.update(cx, |input, cx| {
            input.set_value(config.max_concurrent_tasks.to_string(), window, cx)
        });
        self.settings_chunk_input.update(cx, |input, cx| {
            input.set_value(config.chunk_size.to_string(), window, cx)
        });
        self.settings_timeout_input.update(cx, |input, cx| {
            input.set_value(config.timeout.to_string(), window, cx)
        });
        self.settings_user_agent_input.update(cx, |input, cx| {
            input.set_value(config.user_agent.clone(), window, cx)
        });
        self.theme_choice = config.theme;
    }

    fn open_add_task_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.add_url_input
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.add_dest_input
            .update(cx, |input, cx| input.set_value("", window, cx));

        let app_state = self.app_state.clone();
        let add_url_input = self.add_url_input.clone();
        let add_dest_input = self.add_dest_input.clone();
        let primary_style = button_style(primary_color(cx), white(), cx);
        let secondary_style = button_style(panel_color(cx), text_color(cx), cx);

        window.open_dialog(cx, move |dialog, _, _cx| {
            dialog
                .title("Add Download Task")
                .child(
                    v_flex()
                        .gap_3()
                        .child("Use the default download directory by leaving destination blank.")
                        .child(Input::new(&add_url_input))
                        .child(Input::new(&add_dest_input)),
                )
                .footer(
                    h_flex()
                        .gap_2()
                        .justify_end()
                        .child(
                            Button::new("submit-add-task")
                                .custom(primary_style)
                                .label("Add")
                                .on_click({
                                    let app_state = app_state.clone();
                                    let add_url_input = add_url_input.clone();
                                    let add_dest_input = add_dest_input.clone();
                                    move |_, window, cx| {
                                        let url = add_url_input.read(cx).value().to_string();
                                        let dest_value =
                                            add_dest_input.read(cx).value().to_string();

                                        if url.trim().is_empty() {
                                            window.push_notification("URL is required", cx);
                                            return;
                                        }

                                        let (queue, config) = app_state
                                            .read_with(cx, |state, _| {
                                                (state.queue.clone(), state.config.clone())
                                            });
                                        let destination = if dest_value.trim().is_empty() {
                                            default_destination(&config, &url)
                                        } else {
                                            PathBuf::from(dest_value)
                                        };

                                        let result = block_in_place(|| {
                                            Handle::current().block_on(async {
                                                queue
                                                    .add_task(url.clone(), destination.clone())
                                                    .await?;
                                                let tasks = queue.get_all_tasks().await;
                                                Ok::<_, anyhow::Error>(tasks)
                                            })
                                        });

                                        match result {
                                            Ok(tasks) => {
                                                app_state.update(cx, |state, cx| {
                                                    state.tasks = tasks;
                                                    state.status_message =
                                                        Some(format!("Added {}", url));
                                                    cx.notify();
                                                });
                                                window.close_dialog(cx);
                                                window.push_notification("Task added", cx);
                                            }
                                            Err(err) => {
                                                window.push_notification(err.to_string(), cx)
                                            }
                                        }
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

    fn run_task_action(
        &mut self,
        task_id: SharedString,
        action: TaskAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let queue = self.app_state.read_with(cx, |state, _| state.queue.clone());
        let task_id_for_async = task_id.clone();
        let result = self.run_async(async move {
            match action {
                TaskAction::Pause => queue.pause_task(task_id_for_async.as_ref()).await?,
                TaskAction::Resume => queue.resume_task(task_id_for_async.as_ref()).await?,
                TaskAction::Cancel => queue.cancel_task(task_id_for_async.as_ref()).await?,
                TaskAction::Remove => queue.remove_task(task_id_for_async.as_ref()).await?,
            }

            Ok(queue.get_all_tasks().await)
        });

        match result {
            Ok(tasks) => {
                self.app_state.update(cx, |state, cx| {
                    state.tasks = tasks;
                    state.status_message = Some(format!("{} {}", action.label(), task_id));
                    cx.notify();
                });
            }
            Err(err) => window.push_notification(err.to_string(), cx),
        }
    }

    fn save_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let new_config = match self.read_settings(cx) {
            Ok(config) => config,
            Err(err) => {
                window.push_notification(err.to_string(), cx);
                return;
            }
        };
        let config_path = self
            .app_state
            .read_with(cx, |state, _| state.config_path.clone());
        let save_result = self.run_async({
            let config = new_config.clone();
            async move {
                config.validate()?;
                config.save(&config_path).await?;
                Ok(())
            }
        });

        match save_result {
            Ok(()) => {
                self.app_state.update(cx, |state, cx| {
                    state.config = new_config.clone();
                    state.status_message =
                        Some("Settings saved. Queue/runtime changes apply on next launch.".into());
                    cx.notify();
                });
                window.push_notification("Settings saved", cx);
            }
            Err(err) => window.push_notification(err.to_string(), cx),
        }
    }

    fn read_settings(&self, cx: &App) -> Result<AppConfig> {
        Ok(AppConfig {
            default_download_path: PathBuf::from(
                self.settings_path_input.read(cx).value().to_string(),
            ),
            max_concurrent_downloads: self.settings_downloads_input.read(cx).value().parse()?,
            max_concurrent_tasks: self.settings_tasks_input.read(cx).value().parse()?,
            chunk_size: self.settings_chunk_input.read(cx).value().parse()?,
            timeout: self.settings_timeout_input.read(cx).value().parse()?,
            user_agent: self.settings_user_agent_input.read(cx).value().to_string(),
            theme: sanitize_theme(&self.theme_choice),
        })
    }

    fn render_task_list(
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
                .child(
                    div()
                        .text_2xl()
                        .font_semibold()
                        .child("还没有下载任务"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("点击右上角 New Task 创建一个新的下载任务。"),
                )
                .child(
                    Button::new("empty-new-task")
                        .custom(button_style(primary_color(cx), white(), cx))
                        .label("新建任务")
                        .on_click(cx.listener(|view, _, window, cx| {
                            view.open_add_task_dialog(window, cx);
                        })),
                )
                .into_any_element();
        }

        v_flex()
            .gap_3()
            .children(tasks.into_iter().map(|task| {
                let task_id: SharedString = task.id.clone().into();
                let primary_label = if task.status == TaskStatus::Paused {
                    "Resume"
                } else {
                    "Pause"
                };
                let primary_action = if task.status == TaskStatus::Paused {
                    TaskAction::Resume
                } else {
                    TaskAction::Pause
                };

                v_flex()
                    .gap_2()
                    .p_4()
                    .border_1()
                    .border_color(border_color(cx))
                    .rounded(px(12.))
                    .bg(card_color(cx))
                    .text_color(text_color(cx))
                    .child(
                        h_flex()
                            .justify_between()
                            .items_start()
                            .child(
                                v_flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .font_semibold()
                                            .child(
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
                                            .text_color(muted_text_color(cx))
                                            .child(task.url.clone()),
                                    ),
                            )
                            .child(status_badge(task.status, cx)),
                    )
                    .child(
                        Progress::new(format!("progress-{}", task.id))
                            .value(progress_percent(&task)),
                    )
                    .child(format!(
                        "{} / {}  ·  {} /s",
                        format_bytes(task.downloaded),
                        format_bytes(task.total_size),
                        format_bytes(task.speed),
                    ))
                        .child(
                            div()
                                .text_sm()
                                .text_color(muted_text_color(cx))
                                .child(format!("保存到 {}", task.dest.display())),
                        )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new(format!("primary-{}", task.id))
                                    .custom(button_style(primary_color(cx), white(), cx))
                                    .label(match task.status {
                                        TaskStatus::Downloading => "暂停",
                                        TaskStatus::Paused => "继续",
                                        _ => primary_label,
                                    })
                                    .on_click(cx.listener({
                                        let task_id = task_id.clone();
                                        move |view, _, window, cx| {
                                            view.run_task_action(
                                                task_id.clone(),
                                                primary_action,
                                                window,
                                                cx,
                                            );
                                        }
                                    })),
                            )
                            .child(
                                Button::new(format!("cancel-{}", task.id))
                                    .custom(button_style(panel_color(cx), text_color(cx), cx))
                                    .label("取消")
                                    .on_click(cx.listener({
                                        let task_id = task_id.clone();
                                        move |view, _, window, cx| {
                                            view.run_task_action(
                                                task_id.clone(),
                                                TaskAction::Cancel,
                                                window,
                                                cx,
                                            );
                                        }
                                    })),
                            )
                            .child(
                                Button::new(format!("remove-{}", task.id))
                                    .custom(button_style(panel_color(cx), text_color(cx), cx))
                                    .label("移除")
                                    .on_click(cx.listener({
                                        let task_id = task_id.clone();
                                        move |view, _, window, cx| {
                                            view.run_task_action(
                                                task_id.clone(),
                                                TaskAction::Remove,
                                                window,
                                                cx,
                                            );
                                        }
                                    })),
                            ),
                    )
            }))
            .into_any_element()
    }

    fn render_history(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let query = self.history_search_input.read(cx).value().to_string();
        let history = self.app_state.read(cx).history.clone();
        let items = search_history(&history, &query);

        let list = if items.is_empty() {
            v_flex()
                .gap_2()
                .text_color(text_color(cx))
                .child("没有匹配的历史记录")
                .into_any_element()
        } else {
            v_flex()
                .gap_3()
                .children(items.into_iter().map(|item| {
                    let id = item.id.clone();
                    v_flex()
                        .gap_2()
                        .p_4()
                        .border_1()
                        .border_color(border_color(cx))
                        .rounded(px(12.))
                        .bg(card_color(cx))
                        .text_color(text_color(cx))
                        .child(
                            div()
                                .font_semibold()
                                .child(
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
                                .text_color(muted_text_color(cx))
                                .child(item.url),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(muted_text_color(cx))
                                .child(item.dest.display().to_string()),
                        )
                        .child(format!(
                            "{}  ·  平均 {} /s  ·  用时 {}s",
                            format_bytes(item.total_size),
                            format_bytes(item.avg_speed),
                            item.duration
                        ))
                        .child(
                            Button::new(format!("history-remove-{}", id))
                                .custom(button_style(panel_color(cx), text_color(cx), cx))
                                .label("删除")
                                .on_click(cx.listener(move |view, _, window, cx| {
                                    let history_path = view
                                        .app_state
                                        .read_with(cx, |state, _| state.history_path.clone());
                                    let result = view.run_async({
                                        let id = id.clone();
                                        async move {
                                            let mut history =
                                                yushi_core::DownloadHistory::load(&history_path)
                                                    .await?;
                                            let removed = history.remove(&id);
                                            if removed {
                                                history.save(&history_path).await?;
                                            }
                                            Ok((history, removed))
                                        }
                                    });

                                    match result {
                                        Ok((history, true)) => {
                                            view.app_state.update(cx, |state, cx| {
                                                state.history = history;
                                                state.status_message =
                                                    Some("Removed history item".into());
                                                cx.notify();
                                            });
                                        }
                                        Ok((_, false)) => {
                                            window.push_notification("History item not found", cx)
                                        }
                                        Err(err) => window.push_notification(err.to_string(), cx),
                                    }
                                })),
                        )
                }))
                .into_any_element()
        };

        v_flex()
            .gap_3()
            .text_color(text_color(cx))
            .child(
                h_flex()
                    .gap_2()
                    .child(Input::new(&self.history_search_input))
                    .child(
                        Button::new("refresh-history-search")
                            .custom(button_style(panel_color(cx), text_color(cx), cx))
                            .label("搜索")
                            .on_click(cx.listener(|_, _, _, cx| cx.notify())),
                    )
                    .child(
                        Button::new("clear-history")
                            .custom(button_style(panel_color(cx), text_color(cx), cx))
                            .label("清空历史")
                            .on_click(cx.listener(|view, _, window, cx| {
                                let history_path = view
                                    .app_state
                                    .read_with(cx, |state, _| state.history_path.clone());
                                let result = view.run_async(async move {
                                    let mut history =
                                        yushi_core::DownloadHistory::load(&history_path).await?;
                                    history.clear();
                                    history.save(&history_path).await?;
                                    Ok(history)
                                });

                                match result {
                                    Ok(history) => {
                                        view.app_state.update(cx, |state, cx| {
                                            state.history = history;
                                            state.status_message = Some("Cleared history".into());
                                            cx.notify();
                                        });
                                    }
                                    Err(err) => window.push_notification(err.to_string(), cx),
                                }
                            })),
                    ),
            )
            .child(list)
            .into_any_element()
    }

    fn render_settings(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .gap_3()
            .text_color(text_color(cx))
            .child(
                div()
                    .font_semibold()
                    .child("下载设置"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(muted_text_color(cx))
                    .child("修改后的配置会保存到共享配置文件，并在下次启动时生效。"),
            )
            .child(div().text_sm().child("默认下载目录"))
            .child(Input::new(&self.settings_path_input))
            .child(div().text_sm().child("每任务并发连接数"))
            .child(Input::new(&self.settings_downloads_input))
            .child(div().text_sm().child("最大并发任务数"))
            .child(Input::new(&self.settings_tasks_input))
            .child(div().text_sm().child("分块大小（字节）"))
            .child(Input::new(&self.settings_chunk_input))
            .child(div().text_sm().child("超时（秒）"))
            .child(Input::new(&self.settings_timeout_input))
            .child(div().text_sm().child("User-Agent"))
            .child(Input::new(&self.settings_user_agent_input))
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .text_sm()
                            .text_color(muted_text_color(cx))
                            .child("主题"),
                    )
                    .child(
                        Button::new("theme-light")
                            .custom(button_style(panel_color(cx), text_color(cx), cx))
                            .label("浅色")
                            .selected(self.theme_choice == "light")
                            .on_click(cx.listener(|view, _, _, cx| {
                                view.theme_choice = "light".into();
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("theme-dark")
                            .custom(button_style(panel_color(cx), text_color(cx), cx))
                            .label("深色")
                            .selected(self.theme_choice == "dark")
                            .on_click(cx.listener(|view, _, _, cx| {
                                view.theme_choice = "dark".into();
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("theme-system")
                            .custom(button_style(panel_color(cx), text_color(cx), cx))
                            .label("跟随系统")
                            .selected(self.theme_choice == "system")
                            .on_click(cx.listener(|view, _, _, cx| {
                                view.theme_choice = "system".into();
                                cx.notify();
                            })),
                    ),
            )
            .child(
                Button::new("save-settings")
                    .custom(button_style(primary_color(cx), white(), cx))
                    .label("保存设置")
                    .on_click(cx.listener(|view, _, window, cx| {
                        view.save_settings(window, cx);
                    })),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(muted_text_color(cx))
                    .child("正在运行中的任务队列不会立即重建；新的配置主要影响后续会话。"),
            )
            .into_any_element()
    }
}

impl Render for AppView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (current_view, tasks, status_message) = self.app_state.read_with(cx, |state, _| {
            (
                state.current_view,
                state.tasks.clone(),
                state.status_message.clone(),
            )
        });
        let stats = ViewStats::from_state(&tasks, self.app_state.read(cx).history.completed_tasks.len());

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
        let content_panel = if let Some(message) = status_message {
            v_flex()
                .size_full()
                .p_4()
                .gap_4()
                .text_color(text_color(cx))
                .bg(app_background(cx))
                .child(
                    div()
                        .pb_2()
                        .border_b_1()
                        .border_color(border_color(cx))
                        .child(view_title(current_view)),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(muted_text_color(cx))
                        .child(view_description(current_view)),
                )
                .child(content)
                .child(
                    div()
                        .text_sm()
                        .text_color(muted_text_color(cx))
                        .child(message),
                )
        } else {
            v_flex()
                .size_full()
                .p_4()
                .gap_4()
                .text_color(text_color(cx))
                .bg(app_background(cx))
                .child(
                    div()
                        .pb_2()
                        .border_b_1()
                        .border_color(border_color(cx))
                        .child(view_title(current_view)),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(muted_text_color(cx))
                        .child(view_description(current_view)),
                )
                .child(content)
        };

        let sidebar = v_flex()
            .w(px(240.))
            .h_full()
            .flex_shrink_0()
            .justify_between()
            .bg(panel_color(cx))
            .border_r_1()
            .border_color(border_color(cx))
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
                                    .text_color(text_color(cx))
                                    .child("导航"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(muted_text_color(cx))
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
                        IconName::HardDrive,
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
                    .bg(card_color(cx))
                    .border_1()
                    .border_color(border_color(cx))
                    .child(
                        div()
                            .text_sm()
                            .font_semibold()
                            .text_color(text_color(cx))
                            .child("当前概览"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(muted_text_color(cx))
                            .child(format!(
                                "共 {} 个任务，{} 条历史",
                                stats.total_tasks, stats.history_items
                            )),
                    ),
            );

        let summary_row = h_flex()
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
            ));

        div()
            .size_full()
            .bg(app_background(cx))
            .text_color(text_color(cx))
            .child(
                v_flex()
                    .size_full()
                    .child(
                        TitleBar::new().child(
                            h_flex()
                                .w_full()
                                .justify_between()
                                .items_center()
                                .child(
                                    h_flex()
                                        .gap_2()
                                        .items_center()
                                        .text_color(text_color(cx))
                                        .child(IconName::Inbox)
                                        .child(
                                            v_flex()
                                                .gap_1()
                                                .child(
                                                    div()
                                                        .font_semibold()
                                                        .child("YuShi"),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(muted_text_color(cx))
                                                        .child("Rust Desktop Downloader"),
                                                ),
                                        ),
                                )
                                .child(h_flex().gap_2().items_center().child(
                                    Button::new("new-task")
                                        .custom(button_style(primary_color(cx), white(), cx))
                                        .label("新建任务")
                                        .on_click(cx.listener(|view, _, window, cx| {
                                            view.open_add_task_dialog(window, cx);
                                        })),
                                )),
                        ),
                    )
                    .child(
                        h_flex()
                            .size_full()
                            .child(sidebar)
                            .child(
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

fn app_background(cx: &App) -> Hsla {
    if cx.theme().is_dark() {
        hsla(0.62, 0.18, 0.11, 1.0)
    } else {
        white()
    }
}

fn panel_color(cx: &App) -> Hsla {
    if cx.theme().is_dark() {
        hsla(0.62, 0.16, 0.15, 1.0)
    } else {
        hsla(0.60, 0.18, 0.96, 1.0)
    }
}

fn card_color(cx: &App) -> Hsla {
    if cx.theme().is_dark() {
        hsla(0.62, 0.14, 0.19, 1.0)
    } else {
        hsla(0.60, 0.12, 0.93, 1.0)
    }
}

fn border_color(cx: &App) -> Hsla {
    if cx.theme().is_dark() {
        hsla(0.62, 0.08, 0.28, 1.0)
    } else {
        hsla(0.60, 0.08, 0.84, 1.0)
    }
}

fn text_color(cx: &App) -> Hsla {
    if cx.theme().is_dark() {
        white()
    } else {
        black()
    }
}

fn muted_text_color(cx: &App) -> Hsla {
    if cx.theme().is_dark() {
        hsla(0.60, 0.03, 0.72, 1.0)
    } else {
        hsla(0.60, 0.04, 0.32, 1.0)
    }
}

fn primary_color(cx: &App) -> Hsla {
    if cx.theme().is_dark() {
        hsla(0.58, 0.80, 0.58, 1.0)
    } else {
        hsla(0.58, 0.78, 0.48, 1.0)
    }
}

fn button_style(bg: Hsla, fg: Hsla, cx: &App) -> ButtonCustomVariant {
    ButtonCustomVariant::new(cx)
        .color(bg)
        .foreground(fg)
        .hover(bg.opacity(0.92))
        .active(bg.opacity(0.82))
}

fn nav_item(
    label: &'static str,
    icon: IconName,
    active: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    cx: &App,
) -> impl IntoElement {
    Button::new(format!("nav-{label}"))
        .custom(button_style(
            if active {
                primary_color(cx).opacity(0.16)
            } else {
                panel_color(cx)
            },
            if active {
                primary_color(cx)
            } else {
                text_color(cx)
            },
            cx,
        ))
        .icon(icon)
        .label(label)
        .w_full()
        .justify_start()
        .on_click(on_click)
}

struct ViewStats {
    total_tasks: usize,
    active_tasks: usize,
    completed_tasks: usize,
    history_items: usize,
}

impl ViewStats {
    fn from_state(tasks: &[DownloadTask], history_items: usize) -> Self {
        Self {
            total_tasks: tasks.len(),
            active_tasks: tasks
                .iter()
                .filter(|task| {
                    matches!(task.status, TaskStatus::Pending | TaskStatus::Downloading)
                })
                .count(),
            completed_tasks: tasks
                .iter()
                .filter(|task| task.status == TaskStatus::Completed)
                .count(),
            history_items,
        }
    }
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

fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let bytes_f = bytes as f64;
    if bytes_f >= GB {
        format!("{bytes_f:.1} GB", bytes_f = bytes_f / GB)
    } else if bytes_f >= MB {
        format!("{bytes_f:.1} MB", bytes_f = bytes_f / MB)
    } else if bytes_f >= KB {
        format!("{bytes_f:.1} KB", bytes_f = bytes_f / KB)
    } else {
        format!("{bytes} B")
    }
}

fn status_badge(status: TaskStatus, cx: &App) -> Div {
    let (label, color) = match status {
        TaskStatus::Pending => ("等待中", cx.theme().yellow),
        TaskStatus::Downloading => ("下载中", cx.theme().blue),
        TaskStatus::Paused => ("已暂停", cx.theme().muted_foreground),
        TaskStatus::Completed => ("已完成", cx.theme().green),
        TaskStatus::Failed => ("失败", cx.theme().red),
        TaskStatus::Cancelled => ("已取消", cx.theme().muted_foreground),
    };

    div()
        .px_2()
        .py_1()
        .rounded(px(999.))
        .bg(color.opacity(0.14))
        .text_color(color)
        .text_xs()
        .child(label)
}

#[derive(Clone, Copy)]
enum TaskAction {
    Pause,
    Resume,
    Cancel,
    Remove,
}

impl TaskAction {
    fn label(self) -> &'static str {
        match self {
            Self::Pause => "Paused",
            Self::Resume => "Resumed",
            Self::Cancel => "Cancelled",
            Self::Remove => "Removed",
        }
    }
}
