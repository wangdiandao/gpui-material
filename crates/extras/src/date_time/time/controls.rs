use std::rc::Rc;

use gpui::{
    AccessibleAction, AnyElement, App, ElementId, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, RenderOnce, Role, SharedString, StatefulInteractiveElement as _,
    Styled as _, Window, div, prelude::FluentBuilder as _, px,
};
use gpui_base::Button as BaseButton;
use gpui_material_components::{Text, TextField, TextFieldVariant};
use gpui_material_foundation::{color, current_material_theme};
use gpui_material_theme::{ColorRole, Direction, TypeRole};

use super::{
    dial::dial,
    dialog::{ModeSlot, TimePickerModeAction, mode_action},
    state::TimePickerState,
    types::{
        Presentation, TimeFormat, TimePickerLayout, TimePickerSegment, TimePickerStyle,
        TimePickerVariant, display_hour,
    },
};
use crate::date_time::{ScrollField, status};

/// An independently mountable Material time-editing panel.
#[derive(IntoElement)]
pub struct TimePickerPanel {
    id: ElementId,
    state: Entity<TimePickerState>,
    variant: Option<TimePickerVariant>,
    layout: TimePickerLayout,
    presentation: Presentation,
    toggle: Option<ModeSlot>,
}

impl TimePickerPanel {
    /// Creates a panel that follows `state.mode()` dynamically.
    pub fn new(id: impl Into<ElementId>, state: &Entity<TimePickerState>) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            variant: None,
            layout: TimePickerLayout::Auto,
            presentation: Presentation {
                style: TimePickerStyle::default(),
                vibrant: false,
            },
            toggle: None,
        }
    }

    /// Creates a panel pinned to [`TimePickerVariant::Dial`].
    pub fn dial(id: impl Into<ElementId>, state: &Entity<TimePickerState>) -> Self {
        Self::new(id, state).fixed(TimePickerVariant::Dial)
    }

    /// Creates a panel pinned to [`TimePickerVariant::Input`].
    pub fn input(id: impl Into<ElementId>, state: &Entity<TimePickerState>) -> Self {
        Self::new(id, state).fixed(TimePickerVariant::Input)
    }

    fn fixed(mut self, variant: TimePickerVariant) -> Self {
        self.variant = Some(variant);
        self
    }

    /// Sets the dial layout orientation. Defaults to [`TimePickerLayout::Auto`].
    pub fn layout(mut self, layout: TimePickerLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Overrides the panel's appearance and color tokens.
    pub fn style(mut self, style: TimePickerStyle) -> Self {
        self.presentation.style = style;
        self
    }

    /// Enables or disables the expressive vibrant styling preset.
    pub fn vibrant(mut self, vibrant: bool) -> Self {
        self.presentation.vibrant = vibrant;
        self
    }

    /// Renders a custom mode-toggle control inside the panel.
    pub fn mode_toggle_content<E: IntoElement>(
        mut self,
        content: impl FnOnce(TimePickerModeAction) -> E + 'static,
    ) -> Self {
        self.toggle = Some(Box::new(move |action| content(action).into_any_element()));
        self
    }
}

impl RenderOnce for TimePickerPanel {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let model = self.state.read(cx);
        let mode = self.variant.unwrap_or(model.mode);
        let appearance = self.presentation;
        let controls = match mode {
            TimePickerVariant::Dial => time_selector(&self.id, &self.state, appearance, window, cx),
            TimePickerVariant::Input => time_input(&self.id, &self.state, appearance, window, cx),
            TimePickerVariant::Scroll => time_scroll(&self.id, model, appearance),
        };
        let period = (model.format == TimeFormat::TwelveHour)
            .then(|| period_selector(&self.id, &self.state, appearance, window, cx));
        let horizontal = mode == TimePickerVariant::Dial
            && window.viewport_size().width >= px(600.)
            && match self.layout {
                TimePickerLayout::Vertical => false,
                TimePickerLayout::Horizontal => true,
                TimePickerLayout::Auto => window.viewport_size().height < px(650.),
            };
        let rtl = current_material_theme(window, cx).layout.direction == Direction::Rtl;
        let toggle = self
            .toggle
            .map(|content| content(mode_action(&self.state, model, mode, false)));
        div()
            .id((self.id.clone(), "panel"))
            .track_focus(&model.panel)
            .flex()
            .gap(px(12.))
            .items_center()
            .when(!horizontal, gpui::Styled::flex_col)
            .when(horizontal && rtl, gpui::Styled::flex_row_reverse)
            .when_some(appearance.style.container, |body, container| {
                body.bg(container)
            })
            .child(
                div()
                    .min_w_0()
                    .w(px(216.))
                    .max_w_full()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(controls)
                    .children(period)
                    .children(toggle),
            )
            .children(
                (mode == TimePickerVariant::Dial)
                    .then(|| dial(&self.id, &self.state, appearance, window, cx)),
            )
    }
}

/// An uncontained vibrant panel with separate, wrapping hour and minute wheels.
#[derive(IntoElement)]
pub struct TimeScroll(TimePickerPanel);

impl TimeScroll {
    /// Creates a vibrant scroll-wheel time panel bound to `state`.
    pub fn new(id: impl Into<ElementId>, state: &Entity<TimePickerState>) -> Self {
        Self(
            TimePickerPanel::new(id, state)
                .fixed(TimePickerVariant::Scroll)
                .vibrant(true),
        )
    }

    /// Overrides the panel's appearance and color tokens.
    pub fn style(mut self, style: TimePickerStyle) -> Self {
        self.0 = self.0.style(style);
        self
    }

    /// Renders a custom mode-toggle control inside the scroll panel.
    pub fn mode_toggle_content<E: IntoElement>(
        mut self,
        content: impl FnOnce(TimePickerModeAction) -> E + 'static,
    ) -> Self {
        self.0 = self.0.mode_toggle_content(content);
        self
    }
}

impl RenderOnce for TimeScroll {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.0
    }
}

fn time_input(
    id: &ElementId,
    state: &Entity<TimePickerState>,
    appearance: Presentation,
    window: &Window,
    cx: &App,
) -> AnyElement {
    let model = state.read(cx);
    let state = state.clone();
    let theme = current_material_theme(window, cx);
    let colors = appearance.input_colors(&theme);
    let field = |segment, label: &SharedString, part| {
        let editor = model.field(segment);
        let invalid = model.input_attempted
            && model
                .parse_segment(segment, &editor.read(cx).value())
                .is_none();
        let control = TextField::new((id.clone(), part), label.clone(), editor)
            .variant(TextFieldVariant::Outlined)
            .colors(colors)
            .invalid(invalid)
            .when_some(
                appearance
                    .style
                    .time_field_shape
                    .or_else(|| appearance.vibrant.then(|| appearance.selector_shape())),
                |field, shape| field.shape(shape),
            );
        div().flex_1().min_w_0().child(control)
    };
    div()
        .capture_action(move |_: &gpui_base::input::Escape, window, cx| {
            if state.update(cx, |state, cx| state.finish_composition(window, cx)) {
                cx.stop_propagation();
            }
        })
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(
            // Numeric HH:MM fields stay left-to-right under RTL per M3 & Unicode BiDi.
            div()
                .flex()
                .items_start()
                .gap(px(12.))
                .w_full()
                .child(field(TimePickerSegment::Hour, &model.locale.hour, "hour"))
                .child(field(
                    TimePickerSegment::Minute,
                    &model.locale.minute,
                    "minute",
                )),
        )
        .when(
            model.input_attempted && !model.is_input_valid(cx),
            |column| column.child(status(model.locale.validation_message.clone(), true)),
        )
        .into_any_element()
}

fn time_scroll(id: &ElementId, model: &TimePickerState, appearance: Presentation) -> AnyElement {
    let wheel = |segment, name: &SharedString, part| {
        let offset = usize::from(
            segment == TimePickerSegment::Hour && model.format == TimeFormat::TwelveHour,
        );
        let describe = model.locale.number.clone();
        let display = describe.clone();
        let scroll = ScrollField::new((id.clone(), part), name.clone(), model.wheel(segment))
            .colors(appearance.style.scroll)
            .time_field(appearance.style.time_field_shape, offset)
            .value_description(move |index| describe((index + offset) as u32))
            .item_content(move |index, selected, _| {
                Text::new(display((index + offset) as u32))
                    .role(if selected {
                        TypeRole::DisplayLarge
                    } else {
                        TypeRole::DisplayMedium
                    })
                    .emphasized(selected)
            });
        div().flex_1().min_w_0().child(scroll)
    };
    // Numeric HH:MM wheels stay left-to-right under RTL per M3 & Unicode BiDi.
    div()
        .flex()
        .w_full()
        .gap(px(4.))
        .child(wheel(
            TimePickerSegment::Hour,
            &model.locale.hour,
            "hour-scroll",
        ))
        .child(wheel(
            TimePickerSegment::Minute,
            &model.locale.minute,
            "minute-scroll",
        ))
        .into_any_element()
}

fn time_selector(
    id: &ElementId,
    state: &Entity<TimePickerState>,
    appearance: Presentation,
    window: &Window,
    cx: &App,
) -> AnyElement {
    let model = state.read(cx);
    let (format, locale) = (model.format, &model.locale);
    let theme = current_material_theme(window, cx);
    let tone = |role| color(theme.colors.color(role));
    let palette = appearance.style.time_selector;
    let [tl, tr, br, bl] = appearance
        .selector_shape()
        .radii(96., 80., theme.layout.direction)
        .map(px);
    let segment = |part, label: &SharedString, value| {
        let selected = part == model.segment;
        let state = state.clone();
        let text = (locale.number)(value);
        let action = Rc::new(move |window: &mut Window, cx: &mut App| {
            state.update(cx, |state, cx| {
                state.set_segment(part, cx);
                state.dial.focus(window, cx);
            });
        });
        let click = action.clone();
        BaseButton::new((
            id.clone(),
            if part == TimePickerSegment::Hour {
                "select-hour"
            } else {
                "select-minute"
            },
        ))
        .role(Role::RadioButton)
        .aria_toggled(if selected {
            gpui::Toggled::True
        } else {
            gpui::Toggled::False
        })
        .accessibility_label(format!("{label} {text}"))
        .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
            action(window, cx)
        })
        .on_click(move |_, window, cx| click(window, cx))
        .w(px(96.))
        .h(px(80.))
        .max_w_full()
        .min_w_0()
        .flex_shrink(1.)
        .rounded_tl(tl)
        .rounded_tr(tr)
        .rounded_br(br)
        .rounded_bl(bl)
        .bg(if appearance.vibrant {
            (if selected {
                palette.selected_container
            } else {
                palette.container
            })
            .unwrap_or_else(|| tone(ColorRole::SurfaceContainerLowest))
        } else if selected {
            palette
                .selected_container
                .unwrap_or_else(|| tone(ColorRole::PrimaryContainer))
        } else {
            palette
                .container
                .unwrap_or_else(|| tone(ColorRole::SurfaceContainerHighest))
        })
        .text_color(if selected {
            palette.selected_content.unwrap_or_else(|| {
                tone(if appearance.vibrant {
                    ColorRole::Primary
                } else {
                    ColorRole::OnPrimaryContainer
                })
            })
        } else {
            palette
                .content
                .unwrap_or_else(|| tone(ColorRole::OnSurface))
        })
        .when(appearance.vibrant && selected, |button| {
            button.border_2().border_color(tone(ColorRole::Primary))
        })
        .child(Text::new(text).role(TypeRole::DisplayMedium))
    };
    // Numeric HH:MM selector stays left-to-right under RTL per M3 & Unicode BiDi.
    div()
        .flex()
        .items_center()
        .justify_center()
        .gap(px(4.))
        .child(segment(
            TimePickerSegment::Hour,
            &locale.hour,
            display_hour(model.draft, format),
        ))
        .child(Text::new(locale.separator.clone()).role(TypeRole::DisplayMedium))
        .child(segment(
            TimePickerSegment::Minute,
            &locale.minute,
            model.minute(),
        ))
        .into_any_element()
}

fn period_selector(
    id: &ElementId,
    state: &Entity<TimePickerState>,
    appearance: Presentation,
    window: &Window,
    cx: &App,
) -> AnyElement {
    let model = state.read(cx);
    let locale = &model.locale;
    let theme = current_material_theme(window, cx);
    let rtl = theme.layout.direction == Direction::Rtl;
    let tone = |role| color(theme.colors.color(role));
    let palette = appearance.style.period_selector;
    let [tl, tr, br, bl] = appearance
        .period_shape()
        .radii(108., 38., theme.layout.direction)
        .map(px);
    let button = |pm, label: SharedString| {
        let selected = model.is_pm() == pm;
        let state = state.clone();
        let activate = Rc::new(move |_: &mut Window, cx: &mut App| {
            state.update(cx, |state, cx| {
                let next_hour = state.draft.hour() % 12 + if pm { 12 } else { 0 };
                let next = state
                    .draft
                    .with()
                    .hour(next_hour)
                    .build()
                    .expect("hour in 0..24 is always valid");
                if state.draft != next {
                    state.draft = next;
                    cx.notify();
                }
            });
        });
        let click = activate.clone();
        BaseButton::new((id.clone(), if pm { "pm" } else { "am" }))
            .role(Role::RadioButton)
            .aria_toggled(if selected {
                gpui::Toggled::True
            } else {
                gpui::Toggled::False
            })
            .accessibility_label(label.clone())
            .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                activate(window, cx)
            })
            .on_click(move |_, window, cx| click(window, cx))
            .flex_1()
            .h(px(38.))
            .rounded_tl(tl)
            .rounded_tr(tr)
            .rounded_br(br)
            .rounded_bl(bl)
            .when_some(
                if selected {
                    Some(palette.selected_container.unwrap_or_else(|| {
                        tone(if appearance.vibrant {
                            ColorRole::PrimaryContainer
                        } else {
                            ColorRole::TertiaryContainer
                        })
                    }))
                } else {
                    palette.container.or_else(|| {
                        appearance
                            .vibrant
                            .then(|| tone(ColorRole::SurfaceContainerLowest))
                    })
                },
                |button, container| button.bg(container),
            )
            .text_color(if selected {
                palette.selected_content.unwrap_or_else(|| {
                    tone(if appearance.vibrant {
                        ColorRole::OnPrimaryContainer
                    } else {
                        ColorRole::OnTertiaryContainer
                    })
                })
            } else {
                palette
                    .content
                    .unwrap_or_else(|| tone(ColorRole::OnSurfaceVariant))
            })
            .child(Text::new(label).role(TypeRole::TitleMedium))
    };
    div()
        .flex()
        .w_full()
        .rounded_tl(tl)
        .rounded_tr(tr)
        .rounded_br(br)
        .rounded_bl(bl)
        .border_1()
        .border_color(
            appearance
                .style
                .period_selector_border
                .unwrap_or_else(|| tone(ColorRole::Outline)),
        )
        .when(rtl, gpui::Styled::flex_row_reverse)
        .child(button(false, locale.am.clone()))
        .child(button(true, locale.pm.clone()))
        .into_any_element()
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::{AppContext as _, Context, Hsla, Render, WindowOptions, rgb};
    use gpui_material_foundation::MaterialRoot;
    use gpui_material_theme::{Direction, MaterialTheme};

    use super::{super::types::TimePickerSelectionColors, *};

    #[gpui::test]
    fn rtl_keeps_hh_mm_selector_left_to_right(cx: &mut gpui::TestAppContext) {
        const HOUR_COLOR: u32 = 0x112233;
        const MINUTE_COLOR: u32 = 0x445566;

        struct Fixture {
            state: Entity<TimePickerState>,
        }

        impl Render for Fixture {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                TimePickerPanel::dial("rtl-dial", &self.state).style(TimePickerStyle {
                    time_selector: TimePickerSelectionColors {
                        selected_container: Some(rgb(HOUR_COLOR).into()),
                        container: Some(rgb(MINUTE_COLOR).into()),
                        ..Default::default()
                    },
                    ..Default::default()
                })
            }
        }

        let window: gpui::AnyWindowHandle = cx
            .update(|cx| {
                let mut theme = MaterialTheme::default();
                theme.layout.direction = Direction::Rtl;
                MaterialRoot::init(theme, cx);
                gpui_material_components::init_components(cx);
                cx.set_reduce_motion(true);
                MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
                    let picker = cx.new(|cx| TimePickerState::new(window, cx));
                    cx.new(|_| Fixture { state: picker })
                })
                .unwrap()
            })
            .into();

        cx.update_window(window, |_, window, cx| {
            window.draw(cx).clear(cx);
            let hour_fill: Hsla = rgb(HOUR_COLOR).into();
            let minute_fill: Hsla = rgb(MINUTE_COLOR).into();
            let quads = window.painted_quads();
            let hour_left = quads
                .iter()
                .find(|quad| quad.background == hour_fill.into())
                .expect("hour selector quad")
                .bounds
                .left();
            let minute_left = quads
                .iter()
                .find(|quad| quad.background == minute_fill.into())
                .expect("minute selector quad")
                .bounds
                .left();
            assert!(
                hour_left < minute_left,
                "expected hour ({hour_left:?}) left of minute ({minute_left:?}) under RTL"
            );
        })
        .unwrap();
    }
}
