use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui::{
    AppContext, ClipboardItem, Context, Edges, Entity, EntityInputHandler, Hsla, InputEvent,
    IntoElement, MouseButton, MouseDownEvent, MouseUpEvent, Render, Rgba, Role, TestAppContext,
    Window, WindowOptions, point, px, rgb, size,
};
use gpui_base::test_support::find;
use gpui_material::{
    InteractionSource, MaterialRoot, TextField, TextFieldColorOverrides, TextFieldColors,
    TextFieldLayout, TextFieldState, TextFieldVariant, material_theme, set_material_theme,
    theme::{CornerShape, Direction, TypeRole, TypographyEmphasis},
};

use super::TextFieldExamples;

#[gpui::test]
fn decorated_field_keeps_its_editor_value_and_accessible_metadata(cx: &mut TestAppContext) {
    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let examples = cx.new(|cx| TextFieldExamples::new(window, cx));
            view = Some(examples.clone());
            examples
        })
        .unwrap();
        (window, view.unwrap())
    });
    cx.simulate_window_resize(window.into(), size(px(560.), px(1000.)));
    let verify = |cx: &mut TestAppContext, direction, value, description| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let field = find(window, &[], &"amount".into()).unwrap();
            assert_eq!(field.role(), Some(Role::TextInput));
            assert_eq!(field.label(), Some("Budget amount"));
            assert_eq!(field.placeholder(), Some("Amount"));
            assert_eq!(field.value(), Some(value));
            assert_eq!(field.description(), Some(description));
            assert_eq!(
                find(window, field.path(), &"placeholder".into()).is_some(),
                value.is_empty(),
            );
            let label = find(window, field.path(), &"label".into()).unwrap();
            let prefix = find(window, field.path(), &"currency-prefix".into()).unwrap();
            let suffix = find(window, field.path(), &"currency-suffix".into()).unwrap();
            assert!(label.bounds().bottom() <= prefix.bounds().top());
            if direction == Direction::Ltr {
                assert!(prefix.bounds().right() < suffix.bounds().left());
            } else {
                assert!(suffix.bounds().right() < prefix.bounds().left());
            }
            let password = find(window, &[], &"password".into()).unwrap();
            assert_eq!(password.role(), Some(Role::PasswordInput));
            assert_eq!(password.value(), None);
            assert_eq!(password.description(), None);
            assert_eq!(
                find(window, password.path(), &"placeholder".into()).is_some(),
                value.is_empty(),
            );
        })
        .unwrap();
    };
    verify(cx, Direction::Ltr, "", "Amount in US dollars. 0 / 8");
    cx.update_window(window.into(), |_, window, cx| {
        let field = find(window, &[], &"amount".into()).unwrap();
        let position = find(window, field.path(), &"placeholder".into())
            .unwrap()
            .bounds()
            .center();
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
        window.refresh();
        window.draw(cx).clear(cx);
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
        assert!(
            view.read(cx)
                .amount
                .read(cx)
                .presentation()
                .focus_handle()
                .is_focused(window)
        );
    })
    .unwrap();
    cx.simulate_input(window.into(), "１２🙂");
    cx.update_window(window.into(), |_, window, cx| {
        let mut theme = material_theme(cx).clone();
        theme.layout.direction = Direction::Rtl;
        set_material_theme(theme, cx);
        let password = view.read(cx).password.clone();
        password.update(cx, |state, cx| state.replace_all("masked", window, cx));
    })
    .unwrap();
    verify(cx, Direction::Rtl, "１２🙂", "Amount in US dollars. 3 / 8");
}

#[gpui::test]
fn styled_text_field_observes_layout_and_keeps_the_custom_outline_cutout(cx: &mut TestAppContext) {
    fn tint(hex: u32) -> Hsla {
        rgb(hex).into()
    }

    struct StyledField {
        state: Entity<TextFieldState>,
        interaction: Entity<InteractionSource>,
        variant: TextFieldVariant,
        invalid: bool,
        layouts: Rc<RefCell<Vec<TextFieldLayout>>>,
    }

    impl Render for StyledField {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let layouts = self.layouts.clone();
            let colors = |hex| TextFieldColorOverrides {
                indicator: Some(tint(hex)),
                ..Default::default()
            };
            TextField::new("styled-field", "Project", &self.state)
                .variant(self.variant)
                .shape(CornerShape::Radii([12., 18., 10., 8.]))
                .content_padding(Edges {
                    left: px(20.),
                    right: px(30.),
                    top: px(6.),
                    bottom: px(6.),
                })
                .text_style(TypeRole::TitleLarge.style(TypographyEmphasis::Baseline))
                .colors(TextFieldColors {
                    unfocused: colors(0x103355),
                    focused: colors(0x105500),
                    invalid: colors(0x901800),
                    disabled: colors(0x666666),
                    hovered_indicator: Some(tint(0x103355)),
                })
                .invalid(self.invalid)
                .interaction_source(&self.interaction)
                .on_text_layout(move |layout, _, _| layouts.borrow_mut().push(layout))
        }
    }

    let layouts = Rc::new(RefCell::new(Vec::new()));
    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let state = cx.new(|cx| TextFieldState::new(window, cx).default_value("Roadmap"));
            let interaction = cx.new(|_| InteractionSource::new());
            let entity = cx.new(|_| StyledField {
                state,
                interaction,
                variant: TextFieldVariant::Outlined,
                invalid: false,
                layouts: layouts.clone(),
            });
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, view.unwrap())
    });
    cx.simulate_window_resize(window.into(), size(px(460.), px(300.)));
    let render = |cx: &mut TestAppContext, expected_tint| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let field = find(window, &[], &"styled-field".into()).unwrap();
            let container = find(window, field.path(), &"container".into()).unwrap();
            let label = find(window, field.path(), &"label-text".into()).unwrap();
            let factor = window.scale_factor();
            let bounds = container.bounds().scale(factor);
            let inside_label = point(
                label.bounds().center().x,
                container.bounds().top() + px(0.25),
            )
            .scale(factor);
            let quads: Vec<_> = window
                .painted_quads()
                .into_iter()
                .filter(|quad| {
                    quad.border_widths.top > px(0.).scale(factor)
                        && quad.bounds.origin == point(bounds.left().round(), bounds.top().round())
                })
                .collect();
            let rendered: Rgba = quads
                .first()
                .expect("custom outline paints")
                .border_color
                .into();
            let expected = rgb(expected_tint);
            assert!(
                (rendered.r - expected.r).abs() < 0.001
                    && (rendered.g - expected.g).abs() < 0.001
                    && (rendered.b - expected.b).abs() < 0.001,
                "border tint {rendered:?} differs from {expected:?}"
            );
            assert_eq!(quads[0].corner_radii.top_left, px(12.).scale(factor));
            assert_eq!(quads[0].corner_radii.top_right, px(18.).scale(factor));
            assert!(
                quads
                    .iter()
                    .all(|quad| !quad.content_mask.bounds.contains(&inside_label))
            );
            container.bounds()
        })
        .unwrap()
    };
    let container = render(cx, 0x103355);
    let observed = |cx: &mut TestAppContext| {
        cx.update(|cx| {
            view.read(cx)
                .interaction
                .read(cx)
                .state(window.window_id(), &"styled-field".into())
                .unwrap()
        })
    };
    assert!(!observed(cx).disabled);
    let layout = *layouts.borrow().last().unwrap();
    assert_eq!(layout.editor_bounds.left(), container.left() + px(20.));
    assert_eq!(layout.editor_bounds.right(), container.right() - px(30.));
    assert_eq!(layout.line_count, 1);
    assert!(layout.text_bounds.is_some());
    let stable_count = layouts.borrow().len();
    render(cx, 0x103355);
    assert_eq!(layouts.borrow().len(), stable_count);
    cx.update_window(window.into(), |_, window, cx| {
        view.read(cx)
            .state
            .clone()
            .update(cx, |state, cx| state.focus(window, cx));
    })
    .unwrap();
    cx.simulate_keystrokes(window.into(), "home");
    render(cx, 0x105500);
    assert!(observed(cx).focus_visible);
    view.update(cx, |view, cx| {
        view.invalid = true;
        cx.notify();
    });
    render(cx, 0x901800);
    cx.update_window(window.into(), |_, _, cx| {
        view.read(cx)
            .state
            .clone()
            .update(cx, |state, cx| state.set_disabled(true, cx));
    })
    .unwrap();
    render(cx, 0x666666);
    assert!(observed(cx).disabled);
    cx.simulate_window_resize(window.into(), size(px(350.), px(300.)));
    render(cx, 0x666666);
    assert!(
        layouts.borrow().last().unwrap().editor_bounds.size.width < layout.editor_bounds.size.width
    );

    view.update(cx, |view, cx| {
        view.variant = TextFieldVariant::Filled;
        cx.notify();
    });
    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let field = find(window, &[], &"styled-field".into()).unwrap();
        let container = find(window, field.path(), &"container".into()).unwrap();
        let factor = window.scale_factor();
        let bounds = container.bounds().scale(factor);
        assert!(
            window.painted_quads().iter().any(|quad| {
                quad.bounds.bottom() == bounds.bottom().round()
                    && quad.border_color == tint(0x666666)
                    && quad.border_widths.bottom > px(0.).scale(factor)
                    && quad.border_widths.top == px(0.).scale(factor)
            }),
            "the filled indicator reaches the container's bottom edge"
        );
    })
    .unwrap();
}

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
fn fields_preserve_native_editing_and_secure_copy(cx: &mut TestAppContext) {
    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let entity = cx.new(|cx| TextFieldExamples::new(window, cx));
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, view.unwrap())
    });
    let (name, password, account) = cx.update(|cx| {
        let view = view.read(cx);
        (
            view.name.clone(),
            view.password.clone(),
            view.account.clone(),
        )
    });
    cx.update_window(window.into(), |_, window, cx| {
        name.update(cx, |state, cx| state.focus(window, cx));
        window.refresh();
        window.draw(cx).clear(cx);
        name.update(cx, |state, cx| {
            state.replace_text_in_range(None, "🙂", window, cx);
            state.replace_and_mark_text_in_range(None, "zhong", Some(5..5), window, cx);
            assert_eq!(state.marked_text_range(window, cx), Some(2..7));
            state.replace_text_in_range(None, "中", window, cx);
            assert_eq!(state.marked_text_range(window, cx), None);
            assert_eq!(state.value().as_ref(), "🙂中");
        });
        account.update(cx, |state, cx| {
            state.replace_text_in_range(None, "changed", window, cx);
            assert_eq!(state.value().as_ref(), "Demo");
        });
        password.update(cx, |state, cx| state.focus(window, cx));
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .unwrap();
    cx.simulate_input(window.into(), "sample-password");
    cx.update_window(window.into(), |_, window, cx| {
        assert_eq!(password.read(cx).value().as_ref(), "sample-password");
        assert!(!password.read(cx).is_copyable());
        password.update(cx, |state, cx| state.select_all(window, cx));
        cx.write_to_clipboard(ClipboardItem::new_string("sentinel".into()));
    })
    .unwrap();
    let copy = if cfg!(target_os = "macos") {
        "cmd-c"
    } else {
        "ctrl-c"
    };
    cx.simulate_keystrokes(window.into(), copy);
    cx.update(|cx| {
        assert_eq!(
            cx.read_from_clipboard().unwrap().text().as_deref(),
            Some("sentinel")
        );
    });
}
