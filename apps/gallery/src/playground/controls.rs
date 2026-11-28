use gpui::{
    Context, ElementId, InteractiveElement, IntoElement, ParentElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material::{
    Button, ButtonVariant, Chip, ChipVariant, ControlSize, Slider, Switch, Text, TextField,
    TextFieldVariant,
    foundation::{color, window_material_theme},
    theme::{ColorRole, TypeRole},
};

use super::{ComponentPageView, Config, Control, Value};

impl ComponentPageView {
    pub(super) fn render_config_card(
        &self,
        stacked: bool,
        window: &Window,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let outline = color(theme.colors.color(ColorRole::OutlineVariant));
        let card_bg = color(theme.colors.color(ColorRole::SurfaceContainerLow));
        let on_surface_variant = color(theme.colors.color(ColorRole::OnSurfaceVariant));
        let owner = cx.weak_entity();
        let is_default = self.config == Config::defaults(self.playground);

        let scenario_chips = std::iter::once("Default")
            .chain(self.playground.scenarios.iter().map(|s| s.name))
            .enumerate()
            .map(|(ix, name)| {
                let owner = owner.clone();
                Chip::new(("playground-scenario", ix), format!("Scenario: {name}"))
                    .content(name)
                    .variant(ChipVariant::Filter)
                    .selected(self.active_scenario == Some(ix))
                    .on_change(move |_, window, cx| {
                        let _ = owner.update(cx, |this, cx| {
                            this.select_scenario(ix, window, cx);
                        });
                    })
            });

        let sections = self
            .playground
            .sections
            .iter()
            .enumerate()
            .map(|(s_ix, section)| {
                let section_title = section.title;
                let controls = section.controls.iter().map(|control| {
                    let owner = owner.clone();
                    match *control {
                        Control::Choice {
                            id, label, options, ..
                        } => {
                            let selected = self.config.option(id);
                            div()
                                .id((ElementId::from("cfg-choice"), id))
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(div().text_xs().text_color(on_surface_variant).child(label))
                                .child(div().flex().flex_wrap().gap_1().children(
                                    options.iter().enumerate().map(|(opt_ix, &opt)| {
                                        let owner = owner.clone();
                                        Chip::new(
                                            (ElementId::from("cfg-opt"), format!("{id}-{opt_ix}")),
                                            format!("{label}: {opt}"),
                                        )
                                        .content(opt)
                                        .variant(ChipVariant::Filter)
                                        .selected(opt == selected)
                                        .on_change(
                                            move |_, window, cx| {
                                                let _ = owner.update(cx, |this, cx| {
                                                    this.set_value(
                                                        id,
                                                        Value::Choice(opt_ix),
                                                        window,
                                                        cx,
                                                    );
                                                });
                                            },
                                        )
                                    }),
                                ))
                                .into_any_element()
                        }
                        Control::Switch { id, label, .. } => {
                            let checked = self.config.bool(id);
                            div()
                                .id((ElementId::from("cfg-switch-row"), id))
                                .w_full()
                                .flex()
                                .items_center()
                                .justify_between()
                                .gap_2()
                                .child(div().flex_1().min_w_0().text_sm().child(label))
                                .child(
                                    Switch::new(
                                        (ElementId::from("cfg-switch"), id),
                                        format!("{section_title}: {label}"),
                                        checked,
                                    )
                                    .show_label(false)
                                    .on_change(
                                        move |next, window, cx| {
                                            let _ = owner.update(cx, |this, cx| {
                                                this.set_value(id, Value::Bool(next), window, cx);
                                            });
                                        },
                                    ),
                                )
                                .into_any_element()
                        }
                        Control::Slider { id, label, .. } => {
                            let value = self.config.number(id);
                            let state = &self.sliders[id];
                            div()
                                .id((ElementId::from("cfg-slider-row"), id))
                                .w_full()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(on_surface_variant)
                                                .child(label),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(on_surface_variant)
                                                .child(format!("{value:.1}")),
                                        ),
                                )
                                .child(
                                    Slider::new((ElementId::from("cfg-slider"), id), label, state)
                                        .size(ControlSize::ExtraSmall),
                                )
                                .into_any_element()
                        }
                        Control::Text { id, label, .. } => {
                            let state = &self.texts[id];
                            div()
                                .id((ElementId::from("cfg-text-row"), id))
                                .w_full()
                                .child(
                                    TextField::new((ElementId::from("cfg-text"), id), label, state)
                                        .variant(TextFieldVariant::Outlined),
                                )
                                .into_any_element()
                        }
                    }
                });

                div()
                    .id(("playground-section", s_ix))
                    .w_full()
                    .pt_3()
                    .border_t_1()
                    .border_color(outline)
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        Text::new(section.title)
                            .id("title")
                            .role(TypeRole::LabelLarge)
                            .color(ColorRole::OnSurfaceVariant),
                    )
                    .children(controls)
            });

        let reset_owner = owner.clone();
        div()
            .id("playground-config-card")
            .test_support()
            .when(stacked, Styled::w_full)
            .when(!stacked, |card| card.w(px(332.)).flex_none())
            .rounded(px(16.))
            .bg(card_bg)
            .border_1()
            .border_color(outline)
            .p_4()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        Text::new("Configuration")
                            .id("config-title")
                            .role(TypeRole::TitleMedium)
                            .emphasized(true),
                    )
                    .child(
                        Button::new("playground-reset", "Reset")
                            .variant(ButtonVariant::Text)
                            .size(ControlSize::ExtraSmall)
                            .disabled(is_default)
                            .on_activate(move |window, cx| {
                                let _ = reset_owner.update(cx, |this, cx| this.reset(window, cx));
                            }),
                    ),
            )
            .when(!self.playground.scenarios.is_empty(), |card| {
                card.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(on_surface_variant)
                                .child("Scenarios"),
                        )
                        .child(div().flex().flex_wrap().gap_1().children(scenario_chips)),
                )
            })
            .children(sections)
    }
}
