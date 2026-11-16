use std::{cell::Cell, f32::consts::TAU, rc::Rc};

use chrono::Timelike;
use gpui::{
    AccessibleAction, AnyElement, App, Bounds, ElementId, Entity, Hsla, InteractiveElement as _,
    IntoElement, MouseButton, ParentElement as _, PathBuilder, Pixels, Point, RenderOnce, Role,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, canvas, div, point,
    prelude::FluentBuilder as _, px,
};
use gpui_base::{Button as BaseButton, TestSupportExt as _};
use gpui_material_components::{Text, TextField, TextFieldColors, TextFieldVariant};
use gpui_material_foundation::{color, window_material_theme};
use gpui_material_theme::{ColorRole, CornerRole, CornerShape, Direction, MaterialTheme, TypeRole};

use super::{
    TimeFormat, TimePickerLayout, TimePickerSegment, TimePickerState, TimePickerVariant,
    dialog::{ModeSlot, TimePickerModeAction, mode_action},
    display_hour, format_time,
};
use crate::date_time::{ScrollField, ScrollFieldColors, status};

/// Container and content colors for selected and unselected controls.
#[derive(Clone, Copy, Debug, Default)]
pub struct TimePickerSelectionColors {
    pub selected_container: Option<Hsla>,
    pub container: Option<Hsla>,
    pub selected_content: Option<Hsla>,
    pub content: Option<Hsla>,
}

/// Instance appearance for the dial, input, and scroll panels.
#[derive(Clone, Copy, Debug, Default)]
pub struct TimePickerStyle {
    pub time_field_shape: Option<CornerShape>,
    pub period_selector_shape: Option<CornerShape>,
    pub container: Option<Hsla>,
    pub clock_dial: Option<Hsla>,
    pub clock_dial_content: Option<Hsla>,
    pub clock_dial_selected_content: Option<Hsla>,
    pub selector: Option<Hsla>,
    pub time_selector: TimePickerSelectionColors,
    pub period_selector: TimePickerSelectionColors,
    pub period_selector_border: Option<Hsla>,
    pub input: TextFieldColors,
    pub scroll: ScrollFieldColors,
}

#[derive(Clone, Copy)]
struct Presentation {
    style: TimePickerStyle,
    vibrant: bool,
}

impl Presentation {
    fn selector_shape(self) -> CornerShape {
        self.style.time_field_shape.unwrap_or_else(|| {
            if self.vibrant {
                CornerRole::Large
            } else {
                CornerRole::Small
            }
            .shape()
        })
    }

    fn period_shape(self) -> CornerShape {
        self.style.period_selector_shape.unwrap_or_else(|| {
            if self.vibrant {
                CornerRole::Full
            } else {
                CornerRole::Small
            }
            .shape()
        })
    }

    fn input_colors(self, theme: &MaterialTheme) -> TextFieldColors {
        let mut input = self.style.input;
        if self.vibrant {
            let tone = |role| color(theme.colors.color(role));
            for state in [&mut input.unfocused, &mut input.focused] {
                state
                    .container
                    .get_or_insert_with(|| tone(ColorRole::SurfaceContainerLowest));
            }
            input
                .focused
                .text
                .get_or_insert_with(|| tone(ColorRole::Primary));
            input
                .invalid
                .container
                .get_or_insert_with(|| tone(ColorRole::ErrorContainer));
            input
                .invalid
                .text
                .get_or_insert_with(|| tone(ColorRole::OnErrorContainer));
        }
        input
    }
}

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

    pub fn dial(id: impl Into<ElementId>, state: &Entity<TimePickerState>) -> Self {
        Self::new(id, state).fixed(TimePickerVariant::Dial)
    }

    pub fn input(id: impl Into<ElementId>, state: &Entity<TimePickerState>) -> Self {
        Self::new(id, state).fixed(TimePickerVariant::Input)
    }

    fn fixed(mut self, variant: TimePickerVariant) -> Self {
        self.variant = Some(variant);
        self
    }

    pub fn layout(mut self, layout: TimePickerLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn style(mut self, style: TimePickerStyle) -> Self {
        self.presentation.style = style;
        self
    }

    pub fn vibrant(mut self, vibrant: bool) -> Self {
        self.presentation.vibrant = vibrant;
        self
    }

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
            TimePickerVariant::Scroll => time_scroll(&self.id, model, appearance, window, cx),
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
        let rtl = window_material_theme(window, cx).layout.direction == Direction::Rtl;
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
    pub fn new(id: impl Into<ElementId>, state: &Entity<TimePickerState>) -> Self {
        Self(
            TimePickerPanel::new(id, state)
                .fixed(TimePickerVariant::Scroll)
                .vibrant(true),
        )
    }

    pub fn style(mut self, style: TimePickerStyle) -> Self {
        self.0 = self.0.style(style);
        self
    }

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
    let theme = window_material_theme(window, cx);
    let colors = appearance.input_colors(theme);
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
        .capture_key_down(move |event, window, cx| {
            let key = &event.keystroke;
            if key.key == "escape"
                && !key.modifiers.modified()
                && state.update(cx, |state, cx| state.finish_composition(window, cx))
            {
                cx.stop_propagation();
            }
        })
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(
            div()
                .flex()
                .items_start()
                .gap(px(12.))
                .w_full()
                .when(
                    theme.layout.direction == Direction::Rtl,
                    gpui::Styled::flex_row_reverse,
                )
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

fn time_scroll(
    id: &ElementId,
    model: &TimePickerState,
    appearance: Presentation,
    window: &Window,
    cx: &App,
) -> AnyElement {
    let theme = window_material_theme(window, cx);
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
    div()
        .flex()
        .w_full()
        .gap(px(4.))
        .when(
            theme.layout.direction == Direction::Rtl,
            gpui::Styled::flex_row_reverse,
        )
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
    let theme = window_material_theme(window, cx);
    let rtl = theme.layout.direction == Direction::Rtl;
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
    div()
        .flex()
        .items_center()
        .justify_center()
        .gap(px(4.))
        .when(rtl, gpui::Styled::flex_row_reverse)
        .child(segment(
            TimePickerSegment::Hour,
            &locale.hour,
            display_hour(model.draft, format),
        ))
        .child(Text::new(locale.separator.clone()).role(TypeRole::DisplayMedium))
        .child(segment(
            TimePickerSegment::Minute,
            &locale.minute,
            model.draft.minute(),
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
    let theme = window_material_theme(window, cx);
    let rtl = theme.layout.direction == Direction::Rtl;
    let tone = |role| color(theme.colors.color(role));
    let palette = appearance.style.period_selector;
    let [tl, tr, br, bl] = appearance
        .period_shape()
        .radii(108., 38., theme.layout.direction)
        .map(px);
    let button = |pm, label: SharedString| {
        let selected = (model.draft.hour() >= 12) == pm;
        let state = state.clone();
        let activate = Rc::new(move |_: &mut Window, cx: &mut App| {
            state.update(cx, |state, cx| {
                state.draft = state
                    .draft
                    .with_hour(state.draft.hour() % 12 + if pm { 12 } else { 0 })
                    .unwrap();
                cx.notify();
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

fn polar(step: u32, divisions: u32, radius: Pixels) -> Point<Pixels> {
    let angle = step as f32 * TAU / divisions as f32;
    point(radius * angle.sin(), -radius * angle.cos())
}

fn dial_position(
    value: u32,
    segment: TimePickerSegment,
    format: TimeFormat,
    diameter: Pixels,
) -> Point<Pixels> {
    if segment == TimePickerSegment::Minute {
        return polar(value, 60, diameter * 0.365);
    }
    let inner = format == TimeFormat::TwentyFourHour && (value == 0 || value > 12);
    polar(value % 12, 12, diameter * if inner { 0.205 } else { 0.365 })
}

fn dial_value(
    offset: Point<Pixels>,
    segment: TimePickerSegment,
    format: TimeFormat,
    diameter: Pixels,
) -> u32 {
    let (x, y) = (f32::from(offset.x), f32::from(offset.y));
    let turn = x.atan2(-y).rem_euclid(TAU) / TAU;
    if segment == TimePickerSegment::Minute {
        return ((turn * 60.).round() as u32) % 60;
    }
    let hour = ((turn * 12.).round() as u32) % 12;
    let inner = format == TimeFormat::TwentyFourHour && x.hypot(y) < f32::from(diameter * 0.285);
    match (inner, hour) {
        (true, 0) => 0,
        (true, _) => hour + 12,
        (_, 0) => 12,
        _ => hour,
    }
}

fn dial(
    id: &ElementId,
    state: &Entity<TimePickerState>,
    appearance: Presentation,
    window: &Window,
    cx: &App,
) -> AnyElement {
    let model = state.read(cx);
    let (format, locale) = (model.format, &model.locale);
    let segment = model.segment;
    let focus = model.dial.clone();
    let selected = if segment == TimePickerSegment::Hour {
        display_hour(model.draft, format)
    } else {
        model.draft.minute()
    };
    let label = if segment == TimePickerSegment::Hour {
        &locale.hour
    } else {
        &locale.minute
    };
    let theme = window_material_theme(window, cx);
    let tone = |role| color(theme.colors.color(role));
    let diameter = (window.viewport_size().width - px(110.)).clamp(px(120.), px(256.));
    let center = diameter / 2.;
    let handle = diameter * (48. / 256.);
    let selected_position = dial_position(selected, segment, format, diameter);
    let bounds = Rc::new(Cell::new(Bounds::default()));
    let measured = bounds.clone();
    let accent = appearance
        .style
        .selector
        .unwrap_or_else(|| tone(ColorRole::Primary));
    let content = appearance
        .style
        .clock_dial_content
        .unwrap_or_else(|| tone(ColorRole::OnSurface));
    let selected_content = appearance
        .style
        .clock_dial_selected_content
        .unwrap_or_else(|| tone(ColorRole::OnPrimary));
    let track = canvas(
        move |bounds, _, _| {
            measured.set(bounds);
        },
        move |bounds, _, window, _| {
            let start = bounds.center();
            let mut line = PathBuilder::stroke(px(2.));
            line.move_to(start);
            line.line_to(start + selected_position);
            if let Ok(path) = line.build() {
                window.paint_path(path, accent);
            }
        },
    )
    .absolute()
    .size_full();
    let key_state = state.clone();
    let click_state = state.clone();
    let click_focus = focus.clone();
    let keyboard_focus = focus.is_focused(window) && window.last_input_was_keyboard();
    let marks: Vec<u32> = match (segment, format) {
        (TimePickerSegment::Minute, _) => (0..60).step_by(5).collect(),
        (_, TimeFormat::TwelveHour) => (1..=12).collect(),
        (_, TimeFormat::TwentyFourHour) => (0..24).collect(),
    };
    div()
        .id((id.clone(), "dial"))
        .test_support()
        .role(Role::Group)
        .aria_label(format!(
            "{label}, {}",
            format_time(model.draft, format, locale)
        ))
        .track_focus(&focus)
        .tab_stop(true)
        .size(diameter)
        .flex_none()
        .relative()
        .rounded_full()
        .bg(appearance.style.clock_dial.unwrap_or_else(|| {
            tone(if appearance.vibrant {
                ColorRole::SurfaceContainerLowest
            } else {
                ColorRole::SurfaceContainerHighest
            })
        }))
        .when(keyboard_focus, |dial| {
            dial.border_2().border_color(tone(ColorRole::Secondary))
        })
        .child(track)
        .child(
            div()
                .absolute()
                .size(px(8.))
                .left(center - px(4.))
                .top(center - px(4.))
                .rounded_full()
                .bg(accent),
        )
        .child(
            div()
                .absolute()
                .size(handle)
                .left(center + selected_position.x - handle / 2.)
                .top(center + selected_position.y - handle / 2.)
                .rounded_full()
                .bg(accent),
        )
        .on_mouse_down(MouseButton::Left, move |event, window, cx| {
            let offset = event.position - bounds.get().center();
            let distance = f32::from(offset.x).hypot(f32::from(offset.y));
            if distance <= f32::from(diameter / 2.) {
                click_focus.focus(window, cx);
                let value = dial_value(offset, segment, format, diameter);
                click_state.update(cx, |state, cx| state.select(value, true, window, cx));
            }
        })
        .capture_key_down(move |event, window, cx| {
            if !focus.is_focused(window) || event.keystroke.modifiers.modified() {
                return;
            }
            let divisions = match (segment, format) {
                (TimePickerSegment::Minute, _) => 60,
                (_, TimeFormat::TwelveHour) => 12,
                (_, TimeFormat::TwentyFourHour) => 24,
            };
            let value = match event.keystroke.key.as_str() {
                "left" | "down" => Some((selected + divisions - 1) % divisions),
                "up" | "right" => Some((selected + 1) % divisions),
                "home" => Some(0),
                "end" => Some(divisions - 1),
                "space" if segment == TimePickerSegment::Hour => Some(selected),
                _ => None,
            };
            if let Some(value) = value {
                key_state.update(cx, |state, cx| {
                    state.select(value, event.keystroke.key == "space", window, cx)
                });
                cx.stop_propagation();
            }
        })
        .children(marks.into_iter().map(|value| {
            let position = dial_position(value, segment, format, diameter);
            let active = value == selected;
            let label = (locale.number)(value);
            let state = state.clone();
            let activate = Rc::new(move |window: &mut Window, cx: &mut App| {
                state.update(cx, |state, cx| {
                    state.dial.focus(window, cx);
                    state.select(value, true, window, cx);
                });
            });
            let click = activate.clone();
            BaseButton::new((id.clone(), format!("mark-{value}")))
                .role(Role::ListBoxOption)
                .aria_selected(active)
                .accessibility_label(label.clone())
                .tab_stop(false)
                .absolute()
                .size(handle)
                .left(center + position.x - handle / 2.)
                .top(center + position.y - handle / 2.)
                .on_mouse_down(MouseButton::Left, move |_, _, cx| cx.stop_propagation())
                .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                    activate(window, cx)
                })
                .on_click(move |_, window, cx| click(window, cx))
                .text_color(if active { selected_content } else { content })
                .child(Text::new(label).role(TypeRole::BodyLarge))
        }))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dial_position_round_trips_both_hour_rings_and_every_minute() {
        let size = px(256.);
        for (segment, format, values) in [
            (TimePickerSegment::Hour, TimeFormat::TwentyFourHour, 0..24),
            (TimePickerSegment::Hour, TimeFormat::TwelveHour, 1..13),
            (TimePickerSegment::Minute, TimeFormat::TwentyFourHour, 0..60),
        ] {
            for value in values {
                let point = dial_position(value, segment, format, size);
                assert_eq!(dial_value(point, segment, format, size), value);
            }
        }
    }
}
