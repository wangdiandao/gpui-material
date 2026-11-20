use gpui::{
    Context, Entity, InteractiveElement, IntoElement, ParentElement, Render, Role,
    StatefulInteractiveElement, Styled, Window, div,
};
use gpui_material::{
    Switch, Text,
    foundation::{color, window_material_theme},
    theme::{ColorRole, TypeRole},
    windows_chrome::TitleBar,
};
use gpui_material_story::Gallery;

pub(super) struct ChromePreview {
    gallery: Entity<Gallery>,
    block_close: bool,
    blocked_requests: usize,
}

impl ChromePreview {
    pub(super) fn new(gallery: Entity<Gallery>, window: &Window, cx: &mut Context<Self>) -> Self {
        let preview = cx.weak_entity();
        window.on_window_should_close(cx, move |_, cx| {
            preview
                .update(cx, |this, cx| {
                    if !this.block_close {
                        return true;
                    }
                    this.blocked_requests += 1;
                    cx.notify();
                    false
                })
                .unwrap_or(true)
        });
        Self {
            gallery,
            block_close: false,
            blocked_requests: 0,
        }
    }
}

impl Render for ChromePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let surface = color(theme.colors.color(ColorRole::Surface));
        let preview = cx.weak_entity();
        let block_close = Switch::new("block-window-close", "Block close", self.block_close)
            .on_change(move |value, _, cx| {
                preview
                    .update(cx, |this, cx| {
                        this.block_close = value;
                        cx.notify();
                    })
                    .ok();
            });
        let status = format!("Blocked close requests: {}", self.blocked_requests);

        div()
            .flex()
            .flex_col()
            .size_full()
            .child(TitleBar::new("windows-title-bar").content(block_close))
            .child(
                div()
                    .id("blocked-close-status")
                    .role(Role::Status)
                    .aria_label(status.clone())
                    .bg(surface)
                    .px_4()
                    .pb_1()
                    .child(Text::new(status).role(TypeRole::LabelMedium)),
            )
            .child(div().flex_1().min_h_0().child(self.gallery.clone()))
    }
}
