pub mod dialogs;
mod history_page;
mod layout;
mod settings_page;
pub mod task_list;
mod tasks_page;

use crate::{
    state::{AppState, ViewKind},
    utils::parse_optional_speed_limit,
};
use anyhow::Result;
use gpui::*;
use gpui_component::{Theme, ThemeMode, WindowExt, input::InputState};
use std::path::PathBuf;
use yushi_core::{AppConfig, config::AppTheme};

pub struct YuShiGUI {
    pub(crate) app_state: Entity<AppState>,
    pub(crate) add_url_input: Entity<InputState>,
    pub(crate) add_dest_input: Entity<InputState>,
    pub(crate) add_speed_input: Entity<InputState>,
    pub(crate) history_search_input: Entity<InputState>,
    pub(crate) settings_path_input: Entity<InputState>,
    pub(crate) settings_downloads_input: Entity<InputState>,
    pub(crate) settings_tasks_input: Entity<InputState>,
    pub(crate) settings_chunk_input: Entity<InputState>,
    pub(crate) settings_timeout_input: Entity<InputState>,
    pub(crate) settings_user_agent_input: Entity<InputState>,
    pub(crate) settings_proxy_input: Entity<InputState>,
    pub(crate) settings_speed_limit_input: Entity<InputState>,
    pub(crate) theme_choice: AppTheme,
}

impl YuShiGUI {
    pub fn new(app_state: Entity<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let config = app_state.read(cx).config.clone();

        Self {
            app_state,
            add_url_input: cx
                .new(|cx| InputState::new(window, cx).placeholder("https://example.com/file.iso")),
            add_dest_input: cx.new(|cx| {
                InputState::new(window, cx).placeholder("Leave empty to use default path")
            }),
            add_speed_input: cx
                .new(|cx| InputState::new(window, cx).placeholder("Optional speed limit, e.g. 2M")),
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
            settings_proxy_input: cx.new(|cx| {
                InputState::new(window, cx).default_value(config.proxy.unwrap_or_default())
            }),
            settings_speed_limit_input: cx.new(|cx| {
                InputState::new(window, cx).default_value(
                    config
                        .speed_limit
                        .map(|limit| limit.to_string())
                        .unwrap_or_default(),
                )
            }),
            theme_choice: config.theme,
        }
    }

    pub fn set_view(&mut self, view: ViewKind, window: &mut Window, cx: &mut Context<Self>) {
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
        self.settings_proxy_input.update(cx, |input, cx| {
            input.set_value(config.proxy.unwrap_or_default(), window, cx)
        });
        self.settings_speed_limit_input.update(cx, |input, cx| {
            input.set_value(
                config
                    .speed_limit
                    .map(|limit| limit.to_string())
                    .unwrap_or_default(),
                window,
                cx,
            )
        });
        self.theme_choice = config.theme;
    }

    fn apply_theme(theme: AppTheme, window: &mut Window, cx: &mut App) {
        match theme {
            AppTheme::Dark => Theme::change(ThemeMode::Dark, Some(window), cx),
            AppTheme::Light => Theme::change(ThemeMode::Light, Some(window), cx),
            AppTheme::System => Theme::sync_system_appearance(Some(window), cx),
        }
    }

    pub(crate) fn save_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let new_config = match self.read_settings(cx) {
            Ok(config) => config,
            Err(err) => {
                window.push_notification(err.to_string(), cx);
                return;
            }
        };
        let (queue, config_path) = self.app_state.read_with(cx, |state, _| {
            (state.queue.clone(), state.config_path.clone())
        });
        let theme = new_config.theme;

        cx.spawn_in(window, async move |view, window| {
            let config_for_save = new_config.clone();
            let result = async {
                config_for_save.validate()?;
                config_for_save.save(&config_path).await?;
                queue
                    .apply_runtime_config(
                        config_for_save.downloader_config(),
                        config_for_save.max_concurrent_tasks,
                    )
                    .await?;
                Ok::<_, anyhow::Error>(())
            }
            .await;

            let _ = view.update_in(window, move |view, window, cx| match result {
                Ok(()) => {
                    view.app_state.update(cx, |state, cx| {
                        state.config = new_config.clone();
                        state.status_message = Some(
                            "Settings saved. New tasks now use the updated runtime config.".into(),
                        );
                        cx.notify();
                    });
                    Self::apply_theme(theme, window, cx);
                    window.push_notification("Settings saved", cx);
                }
                Err(err) => window.push_notification(err.to_string(), cx),
            });
        })
        .detach();
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
            proxy: match self.settings_proxy_input.read(cx).value().trim() {
                "" => None,
                value => Some(value.to_string()),
            },
            speed_limit: match self.settings_speed_limit_input.read(cx).value().trim() {
                "" => None,
                value => parse_optional_speed_limit(value)?,
            },
            theme: self.theme_choice,
        })
    }
}
