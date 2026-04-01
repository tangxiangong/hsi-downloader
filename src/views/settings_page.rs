use crate::{components::settings_form::settings_form, views::YuShiGUI};
use gpui::*;

impl YuShiGUI {
    pub fn render_settings(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        settings_form(self, cx)
    }
}
