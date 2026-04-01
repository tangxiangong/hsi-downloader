use crate::{utils, views::YuShiGUI};
use gpui::*;
use gpui_component::{
    button::*,
    input::{Input, InputState},
    *,
};
use yushi_core::config::AppTheme;

pub fn settings_form(view: &mut YuShiGUI, cx: &mut Context<YuShiGUI>) -> AnyElement {
    v_flex()
        .gap_3()
        .text_color(utils::text_color(cx))
        .child(div().font_semibold().child("下载设置"))
        .child(
            div()
                .text_sm()
                .text_color(utils::muted_text_color(cx))
                .child("修改后的配置会立即写回共享配置文件，并应用到新的下载任务。"),
        )
        .child(settings_field("默认下载目录", &view.settings_path_input))
        .child(settings_field(
            "每任务并发连接数",
            &view.settings_downloads_input,
        ))
        .child(settings_field("最大并发任务数", &view.settings_tasks_input))
        .child(settings_field(
            "分块大小（字节）",
            &view.settings_chunk_input,
        ))
        .child(settings_field("超时（秒）", &view.settings_timeout_input))
        .child(settings_field(
            "User-Agent",
            &view.settings_user_agent_input,
        ))
        .child(settings_field("代理 URL", &view.settings_proxy_input))
        .child(settings_field(
            "默认任务限速（支持 1M / 500K）",
            &view.settings_speed_limit_input,
        ))
        .child(theme_selector(view.theme_choice, cx))
        .child(
            Button::new("save-settings")
                .custom(utils::button_style(
                    utils::primary_color(cx),
                    gpui_component::white(),
                    cx,
                ))
                .label("保存设置")
                .on_click(cx.listener(|view, _, window, cx| {
                    view.save_settings(window, cx);
                })),
        )
        .child(
            div()
                .text_sm()
                .text_color(utils::muted_text_color(cx))
                .child("已经在运行的任务不会被中断重建，但新的任务会立刻使用最新配置。"),
        )
        .into_any_element()
}

fn settings_field(label: &'static str, input: &Entity<InputState>) -> Div {
    v_flex()
        .gap_1()
        .child(div().text_sm().child(label))
        .child(Input::new(input))
}

fn theme_selector(theme_choice: AppTheme, cx: &mut Context<YuShiGUI>) -> Div {
    h_flex()
        .gap_2()
        .items_center()
        .child(
            div()
                .text_sm()
                .text_color(utils::muted_text_color(cx))
                .child("主题"),
        )
        .child(theme_button(
            "theme-light",
            "浅色",
            theme_choice == AppTheme::Light,
            AppTheme::Light,
            cx,
        ))
        .child(theme_button(
            "theme-dark",
            "深色",
            theme_choice == AppTheme::Dark,
            AppTheme::Dark,
            cx,
        ))
        .child(theme_button(
            "theme-system",
            "跟随系统",
            theme_choice == AppTheme::System,
            AppTheme::System,
            cx,
        ))
}

fn theme_button(
    id: &'static str,
    label: &'static str,
    selected: bool,
    theme: AppTheme,
    cx: &mut Context<YuShiGUI>,
) -> Button {
    Button::new(id)
        .custom(utils::button_style(
            utils::panel_color(cx),
            utils::text_color(cx),
            cx,
        ))
        .label(label)
        .selected(selected)
        .on_click(cx.listener(move |view, _, _, cx| {
            view.theme_choice = theme;
            cx.notify();
        }))
}
