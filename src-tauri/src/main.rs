#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;
mod tray;

use state::AppState;
use tauri::{Emitter, Manager, WindowEvent};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let handle = app.handle().clone();

            tauri::async_runtime::block_on(async {
                let (app_state, mut event_rx) = AppState::bootstrap()
                    .await
                    .expect("failed to bootstrap app state");

                handle.manage(app_state);

                // Forward DownloaderEvent to frontend
                tauri::async_runtime::spawn(async move {
                    while let Some(event) = event_rx.recv().await {
                        let _ = handle.emit("download-event", &event);
                    }
                });
            });

            // Setup tray icon
            tray::setup_tray(app.handle())?;

            // Hide window on close instead of quitting
            if let Some(window) = app.get_webview_window("main") {
                let win = window.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = win.hide();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_tasks,
            commands::add_task,
            commands::pause_task,
            commands::resume_task,
            commands::cancel_task,
            commands::retry_task,
            commands::remove_task,
            commands::remove_task_with_file,
            commands::clear_completed,
            commands::get_history,
            commands::remove_history,
            commands::remove_history_with_file,
            commands::clear_history,
            commands::get_config,
            commands::update_config,
            commands::list_torrent_files,
            commands::infer_destination,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
