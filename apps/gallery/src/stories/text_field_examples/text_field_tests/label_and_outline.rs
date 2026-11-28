use std::time::Duration;

use gpui::{
    AppContext, Context, Entity, InputEvent, IntoElement, MouseButton, MouseDownEvent,
    MouseUpEvent, Render, TestAppContext, Window, WindowOptions, point, px, size,
};
use gpui_base::test_support::find;
use gpui_material::{
    MaterialRoot, TextField, TextFieldState, material_theme, set_material_theme, theme::Direction,
};

use super::super::TextFieldExamples;

#[gpui::test]
fn scaled_text_field_wraps_long_label_without_clipping(cx: &mut TestAppContext) {
    let window = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| TextFieldExamples::new(window, cx))
        })
        .unwrap()
    });
    cx.simulate_window_resize(window.into(), size(px(290.), px(800.)));
    cx.simulate_window_scale_factor_change(window.into(), 1.5);
    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let scaled = find(window, &[], &"scaled-label".into()).unwrap();
        let baseline = find(window, &[], &"name".into()).unwrap();
        assert!(scaled.visible());
        assert!(scaled.bounds().size.height > baseline.bounds().size.height);
        assert!(scaled.bounds().right() <= window.viewport_size().width);
        assert_eq!(
            scaled.label(),
            Some("Preferred name for project correspondence")
        );
    })
    .unwrap();
}

#[gpui::test]
fn outlined_floating_label_cuts_the_top_border(cx: &mut TestAppContext) {
    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            cx.new(|cx| TextFieldExamples::new(window, cx))
        })
        .unwrap()
    });
    cx.update_window(handle.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let field = find(window, &[], &"email".into()).unwrap();
        let label = find(window, field.path(), &"label".into()).unwrap();
        assert_eq!(
            label.bounds().left() - field.bounds().left(),
            px(12.),
            "4dp cutout padding must surround text aligned at the editor's 16dp inset"
        );
        let factor = window.scale_factor();
        // The outlined example starts with a value, so its label overlaps the top stroke.
        let y = field.bounds().top() + px(0.25);
        let inside_label = point(label.bounds().left() + px(9.), y).scale(factor);
        let before_label = point(field.bounds().left() + px(10.), y).scale(factor);
        let outline: Vec<_> = window
            .painted_quads()
            .into_iter()
            .filter(|quad| {
                quad.bounds.origin.y == field.bounds().origin.y.scale(factor)
                    && quad.border_widths.top > px(0.).scale(factor)
            })
            .collect();
        assert!(
            outline
                .iter()
                .any(|quad| quad.content_mask.bounds.contains(&before_label))
        );
        assert!(
            outline
                .iter()
                .all(|quad| !quad.content_mask.bounds.contains(&inside_label))
        );
    })
    .unwrap();
}

#[gpui::test]
fn outlined_label_avoids_the_stroke_through_focus_transitions(cx: &mut TestAppContext) {
    let (handle, examples) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut examples = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let entity = cx.new(|cx| TextFieldExamples::new(window, cx));
            examples = Some(entity.clone());
            entity
        })
        .unwrap();
        (handle, examples.unwrap())
    });
    let sample = |cx: &mut TestAppContext, id: &'static str| {
        cx.update_window(handle.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let field = find(window, &[], &id.into()).unwrap();
            let label = find(window, field.path(), &"label-text".into()).unwrap();
            let factor = window.scale_factor();
            let field = field.bounds().scale(factor);
            let label = label.bounds().scale(factor);
            let outline: Vec<_> = window
                .painted_quads()
                .into_iter()
                .filter(|quad| {
                    quad.bounds.origin == point(field.left().round(), field.top().round())
                        && quad.border_widths.top > px(0.).scale(factor)
                })
                .collect();
            let border = outline.first().expect("outlined field stroke");
            let width = u32::from(border.border_widths.top);
            let top = border.bounds.top();
            if label.top() >= top + border.border_widths.top || label.bottom() <= top {
                return width;
            }
            for row in 0..width {
                let y = top + gpui::ScaledPixels(row as f32 + 0.5);
                for x in [
                    label.left() + px(5.).scale(factor),
                    label.right() - px(5.).scale(factor),
                ] {
                    assert!(
                        outline
                            .iter()
                            .all(|quad| !quad.content_mask.bounds.contains(&point(x, y))),
                        "{id}: stroke pixel row {row} crosses the label at scale {factor}"
                    );
                }
            }
            width
        })
        .unwrap()
    };
    for (direction, factor) in [(Direction::Ltr, 1.), (Direction::Rtl, 1.5)] {
        cx.simulate_window_scale_factor_change(handle.into(), factor);
        cx.update(|cx| {
            let mut theme = material_theme(cx).clone();
            theme.layout.direction = direction;
            set_material_theme(theme, cx);
        });
        for (id, value) in [
            ("email", "name@"),
            ("email", ""),
            ("notes", ""),
            ("notes", "有内容"),
        ] {
            cx.update_window(handle.into(), |_, window, cx| {
                if id == "email" {
                    let email = examples.read(cx).email.clone();
                    email.update(cx, |state, cx| state.replace_all(value, window, cx));
                } else {
                    let notes = examples.read(cx).notes.clone();
                    notes.update(cx, |state, cx| state.replace_all(value, window, cx));
                }
                window.blur(cx);
            })
            .unwrap();
            sample(cx, id);
            cx.executor().advance_clock(Duration::from_secs(1));
            let resting_width = sample(cx, id);
            cx.update_window(handle.into(), |_, window, cx| {
                let field = find(window, &[], &id.into()).unwrap();
                let position = point(field.bounds().center().x, field.bounds().top() + px(24.));
                window.dispatch_event(
                    MouseDownEvent {
                        button: MouseButton::Left,
                        position,
                        click_count: 1,
                        ..Default::default()
                    }
                    .to_platform_input(),
                    cx,
                );
                window.dispatch_event(
                    MouseUpEvent {
                        button: MouseButton::Left,
                        position,
                        click_count: 1,
                        ..Default::default()
                    }
                    .to_platform_input(),
                    cx,
                );
            })
            .unwrap();
            sample(cx, id);
            let mut focused_width = resting_width;
            for _ in 0..45 {
                cx.executor().advance_clock(Duration::from_millis(7));
                focused_width = sample(cx, id);
            }
            assert!(
                focused_width > resting_width,
                "{id}: the focus stroke did not grow"
            );
            cx.update_window(handle.into(), |_, window, cx| window.blur(cx))
                .unwrap();
            sample(cx, id);
            for _ in 0..45 {
                cx.executor().advance_clock(Duration::from_millis(7));
                sample(cx, id);
            }
            assert_eq!(
                sample(cx, id),
                resting_width,
                "{id}: the focus stroke did not return"
            );
        }
    }
}

#[gpui::test]
fn empty_text_field_floats_its_label_on_focus(cx: &mut TestAppContext) {
    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let examples = cx.new(|cx| TextFieldExamples::new(window, cx));
            view = Some(examples.clone());
            examples
        })
        .unwrap();
        (window, view.unwrap())
    });
    let sample = |cx: &mut TestAppContext| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let field = find(window, &[], &"name".into()).unwrap();
            let label = find(window, field.path(), &"label".into()).unwrap();
            label.bounds().top() - field.bounds().top()
        })
        .unwrap()
    };
    let rest = sample(cx);
    cx.update_window(window.into(), |_, window, cx| {
        view.read(cx)
            .name
            .read(cx)
            .presentation()
            .focus_handle()
            .clone()
            .focus(window, cx);
    })
    .unwrap();
    assert_eq!(sample(cx), rest);
    cx.executor().advance_clock(Duration::from_millis(40));
    let middle = sample(cx);
    assert!(middle < rest);
    cx.executor().advance_clock(Duration::from_secs(1));
    let floated = sample(cx);
    assert!(floated < middle);
    assert_eq!(floated, px(8.), "Compose filled label top padding");
    cx.update_window(window.into(), |_, window, cx| {
        cx.set_reduce_motion(true);
        window.blur(cx);
    })
    .unwrap();
    assert_eq!(sample(cx), rest);
}

#[gpui::test]
fn resting_label_reserves_trailing_inset_in_ltr_and_rtl(cx: &mut TestAppContext) {
    use gpui_material::{Button, TextFieldLabelAlignment};

    struct TrailingLabelField(Entity<TextFieldState>);

    impl Render for TrailingLabelField {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            TextField::new("trailing-field", "Destination", &self.0)
                .label_alignment(TextFieldLabelAlignment::End, TextFieldLabelAlignment::End)
                .trailing(Button::new("trailing-btn", "Open"))
        }
    }

    let (window, state) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut state = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let input = cx.new(|cx| TextFieldState::new(window, cx));
            state = Some(input.clone());
            cx.new(|_| TrailingLabelField(input))
        })
        .unwrap();
        (window, state.unwrap())
    });

    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let field = find(window, &[], &"trailing-field".into()).unwrap();
        let container = find(window, field.path(), &"container".into()).unwrap();
        let label = find(window, field.path(), &"label".into()).unwrap();
        assert!(
            container.bounds().right() - label.bounds().right() >= px(64.),
            "LTR resting label reserves 16px padding + 48px trailing inset: {:?}",
            (container.bounds(), label.bounds())
        );

        let mut theme = material_theme(cx).clone();
        theme.layout.direction = Direction::Rtl;
        set_material_theme(theme, cx);
        window.refresh();
        window.draw(cx).clear(cx);
        let field = find(window, &[], &"trailing-field".into()).unwrap();
        let container = find(window, field.path(), &"container".into()).unwrap();
        let label = find(window, field.path(), &"label".into()).unwrap();
        assert!(
            label.bounds().left() - container.bounds().left() >= px(64.),
            "RTL resting label reserves 16px padding + 48px trailing inset on left: {:?}",
            (container.bounds(), label.bounds())
        );

        state.update(cx, |state, cx| state.focus(window, cx));
        window.refresh();
        window.draw(cx).clear(cx);
        let field = find(window, &[], &"trailing-field".into()).unwrap();
        let container = find(window, field.path(), &"container".into()).unwrap();
        let label = find(window, field.path(), &"label".into()).unwrap();
        assert_eq!(
            label.bounds().left() - container.bounds().left(),
            px(16.),
            "Floated label returns to standard horizontal padding"
        );
    })
    .unwrap();
}
