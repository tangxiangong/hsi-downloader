mod app_state;
mod components;
mod views;

use anyhow::Result;
use app_state::AppState;
use gpui::*;
use gpui_component::{Root, Theme, ThemeMode, TitleBar};
use views::app_view::AppView;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let app = Application::new();

    app.run(move |cx| {
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            let (state, mut event_rx) = AppState::bootstrap().await?;
            let app_state = cx.new(|_| state)?;
            let event_state = app_state.clone();

            cx.spawn(async move |cx| {
                while let Some(event) = event_rx.recv().await {
                    let queue: std::sync::Arc<yushi_core::YuShi> =
                        event_state.read_with(cx, |state, _| state.queue.clone())?;
                    let tasks = queue.get_all_tasks().await;
                    let refresh_history = matches!(
                        event,
                        yushi_core::DownloaderEvent::Task(yushi_core::TaskEvent::Completed { .. })
                    );
                    let history = if refresh_history {
                        let history_path: std::path::PathBuf =
                            event_state.read_with(cx, |state, _| state.history_path.clone())?;
                        Some(yushi_core::DownloadHistory::load(&history_path).await?)
                    } else {
                        None
                    };

                    event_state.update(cx, |state, cx| {
                        state.tasks = tasks;
                        if let Some(history) = history {
                            state.history = history;
                        }
                        cx.notify();
                    })?;
                }

                Ok::<_, anyhow::Error>(())
            })
            .detach();

            cx.open_window(
                WindowOptions {
                    titlebar: Some(TitleBar::title_bar_options()),
                    ..Default::default()
                },
                |window, cx| {
                    window.activate_window();
                    window.set_window_title("YuShi");

                    match app_state.read(cx).config.theme.as_str() {
                        "dark" => Theme::change(ThemeMode::Dark, Some(window), cx),
                        "light" => Theme::change(ThemeMode::Light, Some(window), cx),
                        _ => Theme::sync_system_appearance(Some(window), cx),
                    }

                    let theme = Theme::global_mut(cx);
                    if cfg!(target_os = "macos") {
                        theme.font_family = "PingFang SC".into();
                        theme.mono_font_family = "Menlo".into();
                    }
                    theme.font_size = px(15.);
                    theme.mono_font_size = px(13.);

                    let view = cx.new(|cx| AppView::new(app_state.clone(), window, cx));
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });

    Ok(())
}
