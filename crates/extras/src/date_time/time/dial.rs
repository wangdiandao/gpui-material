use std::{cell::Cell, f32::consts::TAU, rc::Rc};

use gpui::{
    AccessibleAction, AnyElement, App, Bounds, ElementId, Entity, InteractiveElement as _,
    IntoElement, MouseButton, ParentElement as _, PathBuilder, Pixels, Point, Role,
    StatefulInteractiveElement as _, Styled as _, Window, canvas, div, point,
    prelude::FluentBuilder as _, px,
};
use gpui_base::{Button as BaseButton, TestSupportExt as _};
use gpui_material_components::Text;
use gpui_material_foundation::{color, current_material_theme};
use gpui_material_theme::{ColorRole, TypeRole};

use super::{
    state::TimePickerState,
    types::{Presentation, TimeFormat, TimePickerSegment, display_hour, format_time},
};

fn polar(step: u32, divisions: u32, radius: Pixels) -> Point<Pixels> {
    let angle = step as f32 * TAU / divisions as f32;
    point(radius * angle.sin(), -radius * angle.cos())
}

pub(super) fn dial_position(
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

pub(super) fn dial_value(
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

const MINUTE_MARKS: &[u32] = &[0, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 55];
const TWELVE_HOUR_MARKS: &[u32] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
const TWENTY_FOUR_HOUR_MARKS: &[u32] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
];
const MARK_IDS: [&str; 60] = [
    "mark-0", "mark-1", "mark-2", "mark-3", "mark-4", "mark-5", "mark-6", "mark-7", "mark-8",
    "mark-9", "mark-10", "mark-11", "mark-12", "mark-13", "mark-14", "mark-15", "mark-16",
    "mark-17", "mark-18", "mark-19", "mark-20", "mark-21", "mark-22", "mark-23", "mark-24",
    "mark-25", "mark-26", "mark-27", "mark-28", "mark-29", "mark-30", "mark-31", "mark-32",
    "mark-33", "mark-34", "mark-35", "mark-36", "mark-37", "mark-38", "mark-39", "mark-40",
    "mark-41", "mark-42", "mark-43", "mark-44", "mark-45", "mark-46", "mark-47", "mark-48",
    "mark-49", "mark-50", "mark-51", "mark-52", "mark-53", "mark-54", "mark-55", "mark-56",
    "mark-57", "mark-58", "mark-59",
];

pub(super) fn dial(
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
        model.minute()
    };
    let label = if segment == TimePickerSegment::Hour {
        &locale.hour
    } else {
        &locale.minute
    };
    let theme = current_material_theme(window, cx);
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
    let down_state = state.clone();
    let move_state = state.clone();
    let up_state = state.clone();
    let up_out_state = state.clone();
    let down_bounds = bounds.clone();
    let move_bounds = bounds.clone();
    let up_bounds = bounds.clone();
    let up_out_bounds = bounds;
    let click_focus = focus.clone();
    let keyboard_focus = focus.is_focused(window) && window.last_input_was_keyboard();
    let marks: &[u32] = match (segment, format) {
        (TimePickerSegment::Minute, _) => MINUTE_MARKS,
        (_, TimeFormat::TwelveHour) => TWELVE_HOUR_MARKS,
        (_, TimeFormat::TwentyFourHour) => TWENTY_FOUR_HOUR_MARKS,
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
            let offset = event.position - down_bounds.get().center();
            let distance = f32::from(offset.x).hypot(f32::from(offset.y));
            if distance <= f32::from(diameter / 2.) {
                click_focus.focus(window, cx);
                down_state.update(cx, |state, cx| {
                    state.dial_dragging = true;
                    let value = dial_value(offset, state.segment, state.format, diameter);
                    state.select(value, false, window, cx);
                });
            }
        })
        .on_mouse_move(move |event, window, cx| {
            if !move_state.read(cx).dial_dragging {
                return;
            }
            if event.pressed_button != Some(MouseButton::Left) {
                move_state.update(cx, |state, _| state.dial_dragging = false);
                return;
            }
            let offset = event.position - move_bounds.get().center();
            move_state.update(cx, |state, cx| {
                let value = dial_value(offset, state.segment, state.format, diameter);
                state.select(value, false, window, cx);
            });
        })
        .on_mouse_up(MouseButton::Left, move |event, window, cx| {
            if !up_state.read(cx).dial_dragging {
                return;
            }
            let offset = event.position - up_bounds.get().center();
            up_state.update(cx, |state, cx| {
                state.dial_dragging = false;
                let value = dial_value(offset, state.segment, state.format, diameter);
                state.select(value, true, window, cx);
            });
        })
        .on_mouse_up_out(MouseButton::Left, move |event, window, cx| {
            if !up_out_state.read(cx).dial_dragging {
                return;
            }
            let offset = event.position - up_out_bounds.get().center();
            up_out_state.update(cx, |state, cx| {
                state.dial_dragging = false;
                let value = dial_value(offset, state.segment, state.format, diameter);
                state.select(value, true, window, cx);
            });
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
        .children(marks.iter().copied().map(|value| {
            let position = dial_position(value, segment, format, diameter);
            let active = value == selected;
            let label = (locale.number)(value);
            let state = state.clone();
            BaseButton::new((id.clone(), MARK_IDS[value as usize]))
                .role(Role::ListBoxOption)
                .aria_selected(active)
                .accessibility_label(label.clone())
                .tab_stop(false)
                .absolute()
                .size(handle)
                .left(center + position.x - handle / 2.)
                .top(center + position.y - handle / 2.)
                .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                    state.update(cx, |state, cx| {
                        state.dial.focus(window, cx);
                        state.select(value, true, window, cx);
                    });
                })
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

    #[cfg(feature = "test-support")]
    #[gpui::test]
    fn dial_pointer_hold_drag_updates_hour_and_transitions_to_minute_on_release(
        cx: &mut gpui::TestAppContext,
    ) {
        use gpui::{
            AppContext as _, Context, InputEvent as _, IntoElement, MouseDownEvent, MouseMoveEvent,
            MouseUpEvent, Render, WindowOptions, size,
        };
        use gpui_material_foundation::MaterialRoot;
        use gpui_material_theme::MaterialTheme;

        use super::super::types::TimePickerStyle;

        struct Fixture {
            state: Entity<TimePickerState>,
            bounds: Rc<Cell<Bounds<Pixels>>>,
        }

        impl Render for Fixture {
            fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                let capture = self.bounds.clone();
                div()
                    .relative()
                    .w(px(256.))
                    .h(px(256.))
                    .child(dial(
                        &"drag-clock".into(),
                        &self.state,
                        Presentation {
                            style: TimePickerStyle::default(),
                            vibrant: false,
                        },
                        window,
                        cx,
                    ))
                    .child(
                        canvas(move |bounds, _, _| capture.set(bounds), |_, _, _, _| {})
                            .absolute()
                            .inset_0(),
                    )
            }
        }

        let bounds = Rc::new(Cell::new(Bounds::default()));
        let mut state = None;
        let window: gpui::AnyWindowHandle = cx
            .update(|cx| {
                MaterialRoot::init(MaterialTheme::default(), cx);
                gpui_material_components::init_components(cx);
                cx.set_reduce_motion(true);
                MaterialRoot::open_window(
                    WindowOptions {
                        window_bounds: Some(gpui::WindowBounds::Windowed(Bounds::new(
                            point(px(0.), px(0.)),
                            size(px(600.), px(600.)),
                        ))),
                        ..Default::default()
                    },
                    cx,
                    |window, cx| {
                        let picker = cx.new(|cx| TimePickerState::new(window, cx));
                        state = Some(picker.clone());
                        cx.new(|cx| {
                            cx.observe(&picker, |_, _, cx| cx.notify()).detach();
                            Fixture {
                                state: picker,
                                bounds: bounds.clone(),
                            }
                        })
                    },
                )
                .unwrap()
            })
            .into();
        let state = state.unwrap();

        cx.update_window(window, |_, window, cx| {
            window.activate_window();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        let dial_bounds = bounds.get();
        let center = dial_bounds.center();
        let diameter = dial_bounds.size.width;
        let pos_3 = center
            + dial_position(
                3,
                TimePickerSegment::Hour,
                TimeFormat::TwentyFourHour,
                diameter,
            );
        let pos_6 = center
            + dial_position(
                6,
                TimePickerSegment::Hour,
                TimeFormat::TwentyFourHour,
                diameter,
            );
        let pos_9 = center
            + dial_position(
                9,
                TimePickerSegment::Hour,
                TimeFormat::TwentyFourHour,
                diameter,
            );

        cx.update_window(window, |_, window, cx| {
            window.dispatch_event(
                MouseDownEvent {
                    button: MouseButton::Left,
                    position: pos_3,
                    click_count: 1,
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
            assert_eq!(state.read(cx).hour(), 3);
            assert_eq!(state.read(cx).segment(), TimePickerSegment::Hour);

            window.dispatch_event(
                MouseMoveEvent {
                    position: pos_6,
                    pressed_button: Some(MouseButton::Left),
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
            assert_eq!(state.read(cx).hour(), 6);
            assert_eq!(state.read(cx).segment(), TimePickerSegment::Hour);

            window.dispatch_event(
                MouseMoveEvent {
                    position: pos_9,
                    pressed_button: Some(MouseButton::Left),
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
            assert_eq!(state.read(cx).hour(), 9);
            assert_eq!(state.read(cx).segment(), TimePickerSegment::Hour);

            window.dispatch_event(
                MouseUpEvent {
                    button: MouseButton::Left,
                    position: pos_9,
                    click_count: 1,
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
            assert_eq!(state.read(cx).hour(), 9);
            assert_eq!(state.read(cx).segment(), TimePickerSegment::Minute);
        })
        .unwrap();
    }
}
