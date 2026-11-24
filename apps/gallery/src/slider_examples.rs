use gpui::{
    AppContext, Axis, Bounds, Context, Entity, IntoElement, ParentElement, Render, Styled, Window,
    div, point, px, quad, relative, size,
};
use gpui_material::{
    ControlSize, Icon, IconName, InteractionSource, RangeThumb, Slider, SliderColors, SliderKind,
    SliderState, SliderThumbStyle, SliderTrackStyle,
    foundation::{color, window_material_theme},
    theme::ColorRole,
};

use crate::story::{section, stack};

#[cfg(all(test, feature = "test-support"))]
mod customization_tests;

pub(crate) struct SliderExamples {
    level: Entity<SliderState>,
    range: Entity<SliderState>,
    range_sources: [Entity<InteractionSource>; 2],
    centered: Entity<SliderState>,
    reversed: Entity<SliderState>,
    sizes: [Entity<SliderState>; 3],
    overlap: Entity<SliderState>,
}

impl SliderExamples {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            level: cx.new(|_| SliderState::new().max(10.).step(1.).default_value(3.)),
            range: cx.new(|_| SliderState::new().max(10.).step(1.).default_value(3.0..7.0)),
            range_sources: std::array::from_fn(|_| cx.new(|_| InteractionSource::new())),
            centered: cx.new(|_| SliderState::new().min(-1.).max(1.).default_value(0.4)),
            reversed: cx.new(|_| SliderState::new().max(10.).step(1.).default_value(3.)),
            sizes: std::array::from_fn(|index| {
                cx.new(|_| {
                    SliderState::new()
                        .max(10.)
                        .default_value([2., 5., 8.][index])
                })
            }),
            overlap: cx.new(|_| SliderState::new().max(10.).step(1.).default_value(5.0..5.0)),
        }
    }
}

impl Render for SliderExamples {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let level = self.level.read(cx).value().end();
        let range = self.range.read(cx).value();
        let balance = self.centered.read(cx).value().end();
        let reversed = self.reversed.read(cx).percentage().end;
        let colors = &window_material_theme(window, cx).colors;
        let primary = color(colors.color(ColorRole::Primary));
        let secondary = color(colors.color(ColorRole::Secondary));
        let tertiary = color(colors.color(ColorRole::Tertiary));
        let inactive = color(colors.color(ColorRole::TertiaryContainer));
        stack()
            .child(
                section("Discrete")
                    .description("Adjust the level in single-step increments.")
                    .child(div().child(format!("Level {level}")))
                    .child(
                        Slider::new("level", "Level", &self.level)
                            .tick_intervals(10)
                            .size(ControlSize::Medium)
                            .icon(Icon::new(IconName::Add)),
                    ),
            )
            .child(
                section("Styled range")
                    .child(div().child(format!("Range {}–{}", range.start(), range.end())))
                    .child(
                        Slider::range(
                            "material-range",
                            "Minimum level",
                            "Maximum level",
                            &self.range,
                        )
                        .tick_intervals(10)
                        .track_style(SliderTrackStyle {
                            thickness: Some(px(20.)),
                            outer_radius: Some(px(6.)),
                            inner_radius: Some(px(1.)),
                            thumb_gap: Some(px(4.)),
                        })
                        .colors(SliderColors {
                            active_track: Some(tertiary),
                            inactive_track: Some(inactive),
                            ..Default::default()
                        })
                        .thumb_style(
                            RangeThumb::Start,
                            SliderThumbStyle {
                                size: Some(size(px(10.), px(40.))),
                                pressed_width: Some(px(6.)),
                                ..Default::default()
                            },
                        )
                        .thumb_style(
                            RangeThumb::End,
                            SliderThumbStyle {
                                size: Some(size(px(12.), px(32.))),
                                pressed_width: Some(px(8.)),
                                ..Default::default()
                            },
                        )
                        .thumb(
                            RangeThumb::Start,
                            div().size_full().rounded_full().bg(primary),
                        )
                        .thumb(RangeThumb::End, div().size_full().rounded_md().bg(tertiary))
                        .interaction_source(RangeThumb::Start, &self.range_sources[0])
                        .interaction_source(RangeThumb::End, &self.range_sources[1])
                        .draw_stop_indicators(|marker, window, _| {
                            let half = marker.diameter / 2.;
                            window.paint_quad(quad(
                                Bounds::from_corners(
                                    marker.center - point(half, half),
                                    marker.center + point(half, half),
                                ),
                                px(0.),
                                marker.color,
                                px(0.),
                                marker.color,
                                gpui::BorderStyle::Solid,
                            ));
                        })
                        .format_value(|value| format!("{value:.0} steps").into()),
                    ),
            )
            .child(
                section("Reversed vertical").child(
                    Slider::new("reversed", "Reversed level", &self.reversed)
                        .axis(Axis::Vertical)
                        .reverse_vertical(true)
                        .track_style(SliderTrackStyle {
                            thickness: Some(px(8.)),
                            ..Default::default()
                        })
                        .track(
                            div()
                                .size_full()
                                .overflow_hidden()
                                .rounded_full()
                                .bg(inactive)
                                .child(div().w_full().h(relative(reversed)).bg(tertiary)),
                        )
                        .colors(SliderColors {
                            thumb: Some(secondary),
                            ..Default::default()
                        })
                        .value_indicator(false),
                ),
            )
            .child(
                section("Centered, vertical")
                    .child(div().child(format!("Balance {balance:.2}")))
                    .child(
                        Slider::new("balance", "Balance", &self.centered)
                            .kind(SliderKind::Centered)
                            .axis(Axis::Vertical)
                            .size(ControlSize::Medium)
                            .icon(Icon::new(IconName::Exposure)),
                    ),
            )
            .child(
                section("Sizes and disabled").children(
                    [
                        ControlSize::Small,
                        ControlSize::Large,
                        ControlSize::ExtraLarge,
                    ]
                    .into_iter()
                    .enumerate()
                    .map(|(index, size)| {
                        let name = format!("{size:?} level");
                        div().child(name.clone()).child(
                            Slider::new(("slider-size", index), name, &self.sizes[index])
                                .size(size)
                                .icon(Icon::new(IconName::Add))
                                .disabled(size == ControlSize::ExtraLarge),
                        )
                    }),
                ),
            )
            .child(
                section("Overlapping range")
                    .description(
                        "The handles start at the same value. Move either one to separate them.",
                    )
                    .child(
                        Slider::range(
                            "overlapping-range",
                            "Overlap start",
                            "Overlap end",
                            &self.overlap,
                        )
                        .size(ControlSize::Large),
                    ),
            )
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{
        ElementId, InputEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
        TestAppContext, point, px, size,
    };
    use gpui_base::test_support::find;
    use gpui_material::foundation::MaterialRoot;

    #[gpui::test]
    fn material_range_uses_base_step_values(cx: &mut TestAppContext) {
        let (handle, view) = cx.update(|cx| {
            crate::init_gallery(cx);
            let mut view = None;
            let handle = MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
                let sliders = cx.new(SliderExamples::new);
                view = Some(sliders.clone());
                sliders
            })
            .unwrap();
            (handle, view.unwrap())
        });
        cx.simulate_window_resize(handle.into(), size(px(640.), px(440.)));
        cx.update_window(handle.into(), |_, window, cx| {
            window.activate_window();
            window.refresh();
            window.draw(cx).clear(cx);
            let state = view.read(cx).range.clone();
            let bounds = state.read(cx).bounds();
            let position = point(bounds.left() + bounds.size.width * 0.42, bounds.center().y);
            window.dispatch_event(
                MouseDownEvent {
                    button: MouseButton::Left,
                    position,
                    click_count: 1,
                    first_mouse: false,
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
            window.refresh();
            window.draw(cx).clear(cx);
            window.dispatch_event(
                MouseUpEvent {
                    button: MouseButton::Left,
                    position,
                    click_count: 1,
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
            window.refresh();
            window.draw(cx).clear(cx);
            assert_eq!(state.read(cx).value(), (4.0, 7.0).into());
            let id = ElementId::from((ElementId::from("material-range"), "start"));
            let start = find(window, &[], &id).unwrap();
            assert_eq!(start.label(), Some("Minimum level"));
            assert_eq!(start.focused(), Some(true));
            let bounds = state.read(cx).bounds();
            let end = find(
                window,
                &[],
                &(ElementId::from("material-range"), "end").into(),
            )
            .unwrap();
            let start_visual = find(
                window,
                &[],
                &(ElementId::from("material-range"), "start-visual").into(),
            )
            .unwrap();
            let end_visual = find(
                window,
                &[],
                &(ElementId::from("material-range"), "end-visual").into(),
            )
            .unwrap();
            assert_eq!(start.bounds().size.width, bounds.size.width);
            assert_eq!(end.bounds().size.width, bounds.size.width);
            for (thumb, fraction) in [(start_visual, 0.4), (end_visual, 0.7)] {
                let expected = bounds.left() + bounds.size.width * fraction;
                assert!((thumb.bounds().center().x - expected).abs() <= px(1.));
                assert_eq!(thumb.bounds().center().y, bounds.center().y);
            }
        })
        .unwrap();
        cx.simulate_keystrokes(handle.into(), "right");
        cx.run_until_parked();
        cx.update(|cx| assert_eq!(view.read(cx).range.read(cx).value(), (5.0, 7.0).into()));

        cx.update_window(handle.into(), |_, window, cx| {
            let state = view.read(cx).range.clone();
            let bounds = state.read(cx).bounds();
            let at = |fraction| {
                point(
                    bounds.left() + bounds.size.width * fraction,
                    bounds.center().y,
                )
            };
            let draw = |window: &mut Window, cx: &mut gpui::App| {
                window.refresh();
                window.draw(cx).clear(cx);
            };
            window.dispatch_event(
                MouseDownEvent {
                    button: MouseButton::Left,
                    position: at(0.5),
                    click_count: 1,
                    first_mouse: false,
                    modifiers: Default::default(),
                }
                .to_platform_input(),
                cx,
            );
            draw(window, cx);
            for fraction in [0.45, 0.3] {
                window.dispatch_event(
                    MouseMoveEvent {
                        position: at(fraction),
                        pressed_button: Some(MouseButton::Left),
                        modifiers: Default::default(),
                    }
                    .to_platform_input(),
                    cx,
                );
                draw(window, cx);
            }
            assert!(cx.has_active_drag());
            assert_eq!(state.read(cx).value(), (3.0, 7.0).into());
            let keystroke =
                gpui::Keystroke::parse_with_os("escape", cx.platform_info().os).unwrap();
            window.dispatch_event(
                gpui::KeyDownEvent {
                    keystroke,
                    is_held: false,
                    prefer_character_input: false,
                }
                .to_platform_input(),
                cx,
            );
            draw(window, cx);
            assert!(!cx.has_active_drag());
            for (phase, fraction) in [
                (gpui::TouchPhase::Started, 0.65),
                (gpui::TouchPhase::Moved, 0.82),
                (gpui::TouchPhase::Ended, 0.82),
            ] {
                window.dispatch_event(
                    gpui::TouchDragEvent {
                        phase,
                        start_position: at(0.65),
                        position: at(fraction),
                    }
                    .to_platform_input(),
                    cx,
                );
                draw(window, cx);
            }
            assert_eq!(state.read(cx).value(), (5.0, 8.0).into());
        })
        .unwrap();
    }
}
