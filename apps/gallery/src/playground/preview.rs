use gpui::{
    ClipboardItem, Context, Font, FontFallbacks, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px, transparent_black,
};
use gpui_material::{
    Button, ButtonVariant, ControlSize, Icon, IconButton, IconName, NavigationItem,
    NavigationItems, TabVariant, Tabs, TabsColors, Text, ThemeScope,
    foundation::{color, window_material_theme},
    theme::{Appearance, ColorRole, Density, Direction, TypeRole},
};

use super::{ComponentPageView, PREVIEW_PRESETS, PageTab};
use crate::shell::styles_examples::preset_label;

impl ComponentPageView {
    pub(super) fn render_card_header(
        &self,
        window: &Window,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let outline = color(theme.colors.color(ColorRole::OutlineVariant));
        let owner = cx.weak_entity();
        let effective_appearance = self
            .preview_appearance
            .unwrap_or_else(|| theme.colors.appearance());

        let mode_tabs = {
            let owner = owner.clone();
            let active_id = match self.tab {
                PageTab::Preview => "preview",
                PageTab::Specs => "specs",
            };
            let items = NavigationItems::new(
                "page-tab",
                "Component view",
                [
                    NavigationItem::new("preview", "Live preview"),
                    NavigationItem::new("specs", "Specs"),
                ],
            )
            .selected(active_id)
            .on_select(move |id, window, cx| {
                let next = match id.as_ref() {
                    "specs" => PageTab::Specs,
                    _ => PageTab::Preview,
                };
                let _ = owner.update(cx, |this, cx| this.set_tab(next, window, cx));
            });
            let mut compact = theme.clone();
            compact.layout.density = Density::Compact;
            div().w(px(210.)).flex_none().child(
                ThemeScope::new(Tabs::new(items).variant(TabVariant::Secondary).colors(
                    TabsColors {
                        container: Some(transparent_black()),
                        divider: Some(transparent_black()),
                        ..Default::default()
                    },
                ))
                .theme(compact),
            )
        };

        let preset_cycle = {
            let owner = owner.clone();
            let current = self.preview_preset;
            Button::new(
                "preview-preset-cycle",
                format!("Theme: {}", preset_label(current)),
            )
            .variant(ButtonVariant::Outlined)
            .size(ControlSize::ExtraSmall)
            .on_activate(move |_, cx| {
                let _ = owner.update(cx, |this, cx| {
                    let ix = PREVIEW_PRESETS
                        .iter()
                        .position(|p| *p == this.preview_preset)
                        .unwrap_or(0);
                    this.preview_preset = PREVIEW_PRESETS[(ix + 1) % PREVIEW_PRESETS.len()];
                    cx.notify();
                });
            })
        };

        let mode_toggle = {
            let owner = owner;
            let next_mode = match effective_appearance {
                Appearance::Light => Appearance::Dark,
                Appearance::Dark => Appearance::Light,
            };
            let (icon, label) = match effective_appearance {
                Appearance::Light => (IconName::DarkMode, "Preview dark mode"),
                Appearance::Dark => (IconName::LightMode, "Preview light mode"),
            };
            IconButton::new("preview-mode-toggle", label, Icon::new(icon))
                .size(ControlSize::ExtraSmall)
                .on_activate(move |_, cx| {
                    let _ = owner.update(cx, |this, cx| {
                        this.preview_appearance = Some(next_mode);
                        cx.notify();
                    });
                })
        };

        div()
            .id("playground-card-header")
            .w_full()
            .px_4()
            .py_2()
            .border_b_1()
            .border_color(outline)
            .flex()
            .flex_wrap()
            .items_center()
            .justify_between()
            .gap_2()
            .when(rtl, Styled::flex_row_reverse)
            .child(mode_tabs)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .when(rtl, Styled::flex_row_reverse)
                    .child(preset_cycle)
                    .child(mode_toggle),
            )
    }

    pub(super) fn render_code_card(
        &self,
        code: String,
        window: &Window,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let outline = color(theme.colors.color(ColorRole::OutlineVariant));
        let card_bg = color(theme.colors.color(ColorRole::SurfaceContainerLow));
        let code_bg = color(theme.colors.color(ColorRole::SurfaceContainerHighest));
        let on_surface = color(theme.colors.color(ColorRole::OnSurface));
        let copied = self.copied_code;
        let owner = cx.weak_entity();
        let copy_text = code.clone();

        div()
            .id("playground-code-card")
            .w_full()
            .rounded(px(16.))
            .bg(card_bg)
            .border_1()
            .border_color(outline)
            .overflow_hidden()
            .flex()
            .flex_col()
            .child(
                div()
                    .px_4()
                    .py_2()
                    .border_b_1()
                    .border_color(outline)
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        Text::new("Rust code")
                            .id("code-title")
                            .role(TypeRole::TitleSmall),
                    )
                    .child(
                        Button::new(
                            "playground-copy-code",
                            if copied { "Copied" } else { "Copy" },
                        )
                        .variant(ButtonVariant::Text)
                        .size(ControlSize::ExtraSmall)
                        .leading_icon(Icon::new(if copied {
                            IconName::Check
                        } else {
                            IconName::Description
                        }))
                        .on_activate(move |_, cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(copy_text.clone()));
                            let _ = owner.update(cx, |this, cx| {
                                this.copied_code = true;
                                cx.notify();
                            });
                        }),
                    ),
            )
            .child(
                div()
                    .id("playground-code-scroll")
                    .w_full()
                    .overflow_x_scroll()
                    .p_4()
                    .bg(code_bg)
                    .font(Font {
                        family: "Consolas".into(),
                        fallbacks: Some(FontFallbacks::from_fonts(vec![
                            "Cascadia Mono".to_owned(),
                            "JetBrains Mono".to_owned(),
                            "SF Mono".to_owned(),
                            "monospace".to_owned(),
                        ])),
                        ..Default::default()
                    })
                    .text_xs()
                    .text_color(on_surface)
                    .whitespace_nowrap()
                    .child(code),
            )
    }
}
