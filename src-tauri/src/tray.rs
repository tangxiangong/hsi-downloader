use tauri::{
    AppHandle, Manager, WindowEvent,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

#[cfg(target_os = "macos")]
use tauri_plugin_nspopover::{
    AppExt as PopoverAppExt, ToPopoverOptions, WindowExt as PopoverWindowExt,
};

#[cfg(any(target_os = "linux", windows))]
use tauri_plugin_positioner::{Position, WindowExt as PositionerWindowExt};

#[cfg(any(target_os = "linux", windows))]
use tauri::WebviewWindow;

pub const MAIN_WINDOW_LABEL: &str = "main";
pub const TRAY_WINDOW_LABEL: &str = "tray";
pub const TRAY_ICON_ID: &str = "main";

pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let open_main = MenuItemBuilder::with_id("show-main", "打开主窗口").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;
    let menu = MenuBuilder::new(app).items(&[&open_main, &quit]).build()?;

    TrayIconBuilder::with_id(TRAY_ICON_ID)
        .tooltip("驭时 (YuShi)")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app: &AppHandle, event| match event.id().as_ref() {
            "show-main" => {
                let _ = show_main_window(app);
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            #[cfg(any(target_os = "linux", windows))]
            {
                tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);
            }

            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_tray_window(tray.app_handle());
            }
        })
        .build(app)?;

    configure_tray_window(app)?;

    Ok(())
}

pub fn register_window_handlers(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let main = window.clone();
        window.on_window_event(move |event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = main.hide();
            }
        });
    }

    if let Some(window) = app.get_webview_window(TRAY_WINDOW_LABEL) {
        let tray = window.clone();
        window.on_window_event(move |event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = tray.hide();
            }
            #[cfg(any(target_os = "linux", windows))]
            WindowEvent::Focused(false) => {
                let _ = tray.hide();
            }
            _ => {}
        });
    }
}

pub fn show_main_window(app: &AppHandle) -> tauri::Result<()> {
    let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        return Ok(());
    };

    window.unminimize()?;
    window.show()?;
    window.set_focus()?;
    Ok(())
}

fn toggle_tray_window(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    {
        if app.is_popover_shown() {
            let _ = app.hide_popover();
        } else {
            let _ = app.show_popover();
        }
        return;
    }

    #[cfg(any(target_os = "linux", windows))]
    {
        let Some(window) = app.get_webview_window(TRAY_WINDOW_LABEL) else {
            return;
        };

        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
            return;
        }

        let _ = reveal_tray_window(&window);
    }
}

#[cfg(target_os = "macos")]
fn configure_tray_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(TRAY_WINDOW_LABEL) {
        window.to_popover(ToPopoverOptions {
            is_fullsize_content: true,
        });
    }

    Ok(())
}

#[cfg(any(target_os = "linux", windows))]
fn configure_tray_window(_app: &AppHandle) -> tauri::Result<()> {
    Ok(())
}

#[cfg(any(target_os = "linux", windows))]
fn reveal_tray_window(window: &WebviewWindow) -> tauri::Result<()> {
    let _ = window.as_ref().window().move_window(Position::TrayCenter);
    window.unminimize()?;
    window.show()?;
    window.set_focus()?;
    Ok(())
}
