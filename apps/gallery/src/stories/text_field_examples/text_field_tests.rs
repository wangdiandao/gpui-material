use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui::{
    AppContext, ClipboardItem, Context, ElementId, Entity, EntityInputHandler, Focusable, Hsla,
    InputEvent, InteractiveElement, IntoElement, MouseButton, MouseDownEvent, MouseUpEvent,
    ParentElement, Render, Rgba, Role, Styled, TestAppContext, Window, WindowOptions, div, point,
    px, rgb, size,
};
use gpui_base::{TestSupportExt, test_support::find};
use gpui_material::{
    InteractionSource, MaterialRoot, MaterialTheme, TextField, TextFieldColorOverrides,
    TextFieldColors, TextFieldLayout, TextFieldMaskPattern, TextFieldState, TextFieldVariant,
    foundation::set_window_material_theme,
    material_theme, set_material_theme,
    theme::{Appearance, CornerShape, Density, Direction, TypeRole, TypographyEmphasis},
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
                .content_padding(gpui_material::LogicalEdges {
                    start: 20.,
                    end: 30.,
                    top: 6.,
                    bottom: 6.,
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

#[gpui::test]
fn secure_field_toggles_obfuscation_mode_and_clipboard_policy(cx: &mut TestAppContext) {
    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let entity = cx.new(|cx| TextFieldExamples::new(window, cx));
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, view.unwrap())
    });
    cx.simulate_window_resize(window.into(), size(px(560.), px(1200.)));
    let password = cx.update(|cx| view.read(cx).password.clone());
    let copy = if cfg!(target_os = "macos") {
        "cmd-c"
    } else {
        "ctrl-c"
    };
    let click_toggle = |cx: &mut TestAppContext| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            let field = find(window, &[], &"password".into()).unwrap();
            let toggle = find(window, field.path(), &"password-visibility".into())
                .unwrap()
                .bounds()
                .center();
            window.dispatch_event(
                MouseDownEvent {
                    button: MouseButton::Left,
                    position: toggle,
                    click_count: 1,
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
            window.dispatch_event(
                MouseUpEvent {
                    button: MouseButton::Left,
                    position: toggle,
                    click_count: 1,
                    ..Default::default()
                }
                .to_platform_input(),
                cx,
            );
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
    };

    cx.update_window(window.into(), |_, window, cx| {
        password.update(cx, |state, cx| {
            state.replace_all("s3cr3t!", window, cx);
            state.focus(window, cx);
            state.select_all(window, cx);
        });
        window.refresh();
        window.draw(cx).clear(cx);
        let field = find(window, &[], &"password".into()).unwrap();
        assert_eq!(field.role(), Some(Role::PasswordInput));
        assert_eq!(field.value(), None);
        assert_eq!(field.description(), None);
        assert!(!password.read(cx).is_copyable());
        let (ime_text, adjusted) = password.update(cx, |state, cx| {
            let mut adjusted = None;
            let text =
                gpui::EntityInputHandler::text_for_range(state, 0..7, &mut adjusted, window, cx);
            (text, adjusted)
        });
        assert_eq!(ime_text.as_deref(), Some("•••••••"));
        assert_eq!(adjusted, Some(0..7));
        cx.write_to_clipboard(ClipboardItem::new_string("sentinel".into()));
    })
    .unwrap();
    cx.simulate_keystrokes(window.into(), copy);
    cx.update(|cx| {
        assert_eq!(
            cx.read_from_clipboard().unwrap().text().as_deref(),
            Some("sentinel")
        );
    });

    click_toggle(cx);
    cx.update_window(window.into(), |_, window, cx| {
        let field = find(window, &[], &"password".into()).unwrap();
        assert_eq!(field.role(), Some(Role::TextInput));
        assert_eq!(field.value(), Some("s3cr3t!"));
        assert_eq!(field.description(), Some("7 / 24"));
        assert!(password.read(cx).is_copyable());
        let (ime_text, adjusted) = password.update(cx, |state, cx| {
            let mut adjusted = None;
            let text =
                gpui::EntityInputHandler::text_for_range(state, 0..7, &mut adjusted, window, cx);
            (text, adjusted)
        });
        assert_eq!(ime_text.as_deref(), Some("s3cr3t!"));
        assert_eq!(adjusted, Some(0..7));
        password.update(cx, |state, cx| {
            state.focus(window, cx);
            state.select_all(window, cx);
        });
    })
    .unwrap();
    cx.simulate_keystrokes(window.into(), copy);
    cx.update(|cx| {
        assert_eq!(
            cx.read_from_clipboard().unwrap().text().as_deref(),
            Some("s3cr3t!")
        );
    });

    click_toggle(cx);
    cx.update_window(window.into(), |_, window, cx| {
        let field = find(window, &[], &"password".into()).unwrap();
        assert_eq!(field.role(), Some(Role::PasswordInput));
        assert_eq!(field.value(), None);
        assert_eq!(field.description(), None);
        assert!(!password.read(cx).is_copyable());
        let (ime_text, adjusted) = password.update(cx, |state, cx| {
            let mut adjusted = None;
            let text =
                gpui::EntityInputHandler::text_for_range(state, 0..7, &mut adjusted, window, cx);
            (text, adjusted)
        });
        assert_eq!(ime_text.as_deref(), Some("•••••••"));
        assert_eq!(adjusted, Some(0..7));
        password.update(cx, |state, cx| {
            state.focus(window, cx);
            state.select_all(window, cx);
        });
        cx.write_to_clipboard(ClipboardItem::new_string("sentinel-2".into()));
    })
    .unwrap();
    cx.simulate_keystrokes(window.into(), copy);
    cx.update(|cx| {
        assert_eq!(
            cx.read_from_clipboard().unwrap().text().as_deref(),
            Some("sentinel-2")
        );
    });
}

#[gpui::test]
fn masked_pattern_and_validator_enforce_formatting_and_undo(cx: &mut TestAppContext) {
    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let entity = cx.new(|cx| TextFieldExamples::new(window, cx));
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, view.unwrap())
    });
    cx.simulate_window_resize(window.into(), size(px(560.), px(1400.)));
    let (phone, code) = cx.update(|cx| {
        let view = view.read(cx);
        (view.phone.clone(), view.code.clone())
    });
    let undo = if cfg!(target_os = "macos") {
        "cmd-z"
    } else {
        "ctrl-z"
    };

    cx.update_window(window.into(), |_, window, cx| {
        phone.update(cx, |state, cx| state.focus(window, cx));
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .unwrap();
    cx.simulate_input(window.into(), "4155550199");
    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(phone.read(cx).value().as_ref(), "(415) 555-0199");
        assert_eq!(phone.read(cx).unmask_value().as_ref(), "4155550199");
        let field = find(window, &[], &"phone".into()).unwrap();
        assert_eq!(field.value(), Some("(415) 555-0199"));
        assert_eq!(field.description(), Some("Unmasked value: 4155550199"));
    })
    .unwrap();
    cx.simulate_keystrokes(window.into(), undo);
    cx.update(|cx| {
        assert!(phone.read(cx).unmask_value().len() < 10);
    });

    cx.update_window(window.into(), |_, window, cx| {
        let number = cx.new(|cx| {
            TextFieldState::new(window, cx).mask_pattern(TextFieldMaskPattern::number(Some(',')))
        });
        number.update(cx, |state, cx| state.replace_all("12345", window, cx));
        assert_eq!(number.read(cx).value().as_ref(), "12,345");
        assert_eq!(number.read(cx).unmask_value().as_ref(), "12345");

        code.update(cx, |state, cx| {
            state.replace_text_in_range(None, "AB12", window, cx);
            assert_eq!(state.value().as_ref(), "AB12");
            state.replace_text_in_range(None, "-", window, cx);
            assert_eq!(state.value().as_ref(), "AB12");
            state.replace_all("AB12345", window, cx);
            assert_eq!(state.value().as_ref(), "AB12");
            state.replace_all("AB1234", window, cx);
            assert_eq!(state.value().as_ref(), "AB1234");
        });
        window.refresh();
        window.draw(cx).clear(cx);
        let field = find(window, &[], &"code".into()).unwrap();
        assert_eq!(field.value(), Some("AB1234"));
        assert_eq!(
            field.description(),
            Some("Rejects non-alphanumeric characters and excess length. 6 / 6")
        );
    })
    .unwrap();
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

#[gpui::test]
fn text_field_filled_and_outlined_shorthands_render_expected_variants(cx: &mut TestAppContext) {
    use gpui::{ParentElement, Styled, div};
    use gpui_material::{
        foundation::{color, window_material_theme},
        theme::ColorRole,
    };

    struct ShorthandFields {
        filled: Entity<TextFieldState>,
        outlined: Entity<TextFieldState>,
    }

    impl Render for ShorthandFields {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .flex()
                .flex_col()
                .gap_4()
                .p_4()
                .w(px(320.))
                .child(TextField::filled(
                    "shorthand-filled",
                    "Filled",
                    &self.filled,
                ))
                .child(TextField::outlined(
                    "shorthand-outlined",
                    "Outlined",
                    &self.outlined,
                ))
        }
    }

    let (window, filled_state, outlined_state) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut filled = None;
        let mut outlined = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let f = cx.new(|cx| TextFieldState::new(window, cx));
            let o = cx.new(|cx| TextFieldState::new(window, cx));
            filled = Some(f.clone());
            outlined = Some(o.clone());
            cx.new(|_| ShorthandFields {
                filled: f,
                outlined: o,
            })
        })
        .unwrap();
        (window, filled.unwrap(), outlined.unwrap())
    });

    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let theme = window_material_theme(window, cx);
        let scale = window.scale_factor();
        let filled_bg = color(theme.colors.color(ColorRole::SurfaceContainerHighest));
        let outline_color = color(theme.colors.color(ColorRole::Outline));

        let filled = find(window, &[], &"shorthand-filled".into()).unwrap();
        let outlined = find(window, &[], &"shorthand-outlined".into()).unwrap();
        let filled_container = find(window, filled.path(), &"container".into())
            .unwrap()
            .bounds()
            .scale(scale);
        let outlined_container = find(window, outlined.path(), &"container".into())
            .unwrap()
            .bounds()
            .scale(scale);

        let quads = window.painted_quads();
        assert!(
            quads.iter().any(|quad| {
                quad.bounds == filled_container && quad.background == filled_bg.into()
            }),
            "TextField::filled must paint SurfaceContainerHighest container"
        );
        assert!(
            quads.iter().any(|quad| {
                quad.bounds == outlined_container && quad.border_color == outline_color
            }),
            "TextField::outlined must paint Outline border"
        );

        filled_state.update(cx, |state, cx| state.set_value("hello", window, cx));
        outlined_state.update(cx, |state, cx| state.set_value("world", window, cx));
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(
            find(window, &[], &"shorthand-filled".into())
                .unwrap()
                .value(),
            Some("hello")
        );
        assert_eq!(
            find(window, &[], &"shorthand-outlined".into())
                .unwrap()
                .value(),
            Some("world")
        );
    })
    .unwrap();
}

#[gpui::test]
fn text_field_clearable_shows_when_editable_nonempty_clears_and_keeps_focus(
    cx: &mut TestAppContext,
) {
    struct ClearableView {
        editable: Entity<TextFieldState>,
        readonly: Entity<TextFieldState>,
        disabled: Entity<TextFieldState>,
    }

    impl Render for ClearableView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .w(px(360.))
                .p_4()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    TextField::outlined("query", "Search", &self.editable)
                        .clearable(true)
                        .trailing(div().id("custom-end").test_support().child("GO")),
                )
                .child(TextField::outlined("ro", "Readonly", &self.readonly).clearable(true))
                .child(TextField::outlined("dis", "Disabled", &self.disabled).clearable(true))
        }
    }

    let mut states = None;
    let window = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let editable = cx.new(|cx| TextFieldState::new(window, cx));
            let readonly = cx.new(|cx| {
                let mut state = TextFieldState::new(window, cx).default_value("Fixed");
                state.set_readonly(true, cx);
                state
            });
            let disabled = cx.new(|cx| {
                let mut state = TextFieldState::new(window, cx).default_value("Locked");
                state.set_disabled(true, cx);
                state
            });
            states = Some(editable.clone());
            cx.new(|_| ClearableView {
                editable,
                readonly,
                disabled,
            })
        })
        .unwrap()
    });
    let editable = states.unwrap();

    let clear_id = |id: &'static str| ElementId::from((ElementId::from(id), "clear-button"));
    cx.update_window(window.into(), |_, window, cx| {
        window.activate_window();
        window.refresh();
        window.draw(cx).clear(cx);

        // Hidden when empty, readonly, or disabled.
        assert!(find(window, &[], &clear_id("query")).is_none());
        assert!(find(window, &[], &clear_id("ro")).is_none());
        assert!(find(window, &[], &clear_id("dis")).is_none());

        // Populating text reveals the clear button with default "Clear" accessible label.
        editable.update(cx, |state, cx| state.set_value("hello", window, cx));
        window.refresh();
        window.draw(cx).clear(cx);
        let clear = find(window, &[], &clear_id("query")).expect("clear button visible");
        assert_eq!(clear.label(), Some("Clear"));
        let custom_end = find(window, &[], &"custom-end".into()).unwrap();
        assert!(
            clear.bounds().center().x < custom_end.bounds().center().x,
            "clear button should precede custom trailing in LTR"
        );

        // Clicking clear empties the editor, hides the button, and keeps focus on the editor.
        let pos = clear.bounds().center();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: pos,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position: pos,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(editable.read(cx).value(), "");
        assert!(find(window, &[], &clear_id("query")).is_none());
        assert!(
            editable.read(cx).focus_handle(cx).is_focused(window),
            "editor must retain focus after clear"
        );

        // In RTL, clear button sits to the right of custom trailing (logical order mirrored).
        let mut rtl = MaterialTheme::baseline(Appearance::Light);
        rtl.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(rtl), window, cx);
        editable.update(cx, |state, cx| state.set_value("rtl", window, cx));
        window.refresh();
        window.draw(cx).clear(cx);
        let clear_rtl = find(window, &[], &clear_id("query")).unwrap();
        let custom_rtl = find(window, &[], &"custom-end".into()).unwrap();
        assert!(
            clear_rtl.bounds().center().x > custom_rtl.bounds().center().x,
            "clear button should precede custom trailing in RTL"
        );
    })
    .unwrap();
}

#[gpui::test]
fn text_field_visibility_toggle_syncs_obfuscation_and_toggles_labels(cx: &mut TestAppContext) {
    struct SecretView {
        secret: Entity<TextFieldState>,
        disabled_secret: Entity<TextFieldState>,
        plain: Entity<TextFieldState>,
        visible: bool,
        disabled_clicks: usize,
    }

    impl Render for SecretView {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let view = cx.entity();
            let dis_view = cx.entity();
            div()
                .w(px(360.))
                .p_4()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    TextField::secure("secret-field", "Secret", &self.secret)
                        .variant(TextFieldVariant::Outlined)
                        .visibility_toggle(self.visible, move |next, _, cx| {
                            view.update(cx, |this, cx| {
                                this.visible = next;
                                cx.notify();
                            });
                        }),
                )
                .child(
                    TextField::secure("disabled-secret", "Locked secret", &self.disabled_secret)
                        .visibility_toggle(false, move |_, _, cx| {
                            dis_view.update(cx, |this, _| this.disabled_clicks += 1);
                        }),
                )
                .child(
                    TextField::new("plain-field", "Plain", &self.plain)
                        .visibility_toggle(false, |_, _, _| {}),
                )
        }
    }

    let mut view_handle = None;
    let mut secret_entity = None;
    let window = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let secret = cx.new(|cx| {
                TextFieldState::new(window, cx)
                    .masked(true)
                    .default_value("hunter2")
            });
            let disabled_secret = cx.new(|cx| {
                let mut state = TextFieldState::new(window, cx)
                    .masked(true)
                    .default_value("locked");
                state.set_disabled(true, cx);
                state
            });
            let plain = cx.new(|cx| TextFieldState::new(window, cx).default_value("abc"));
            secret_entity = Some(secret.clone());
            let entity = cx.new(|_| SecretView {
                secret,
                disabled_secret,
                plain,
                visible: false,
                disabled_clicks: 0,
            });
            view_handle = Some(entity.clone());
            entity
        })
        .unwrap()
    });
    let view = view_handle.unwrap();
    let secret = secret_entity.unwrap();

    let toggle_id = |id: &'static str| ElementId::from((ElementId::from(id), "visibility-toggle"));
    cx.update_window(window.into(), |_, window, cx| {
        window.activate_window();
        window.refresh();
        window.draw(cx).clear(cx);

        // Non-secure field ignores visibility_toggle.
        assert!(find(window, &[], &toggle_id("plain-field")).is_none());

        // Secure field starts masked with "Show password" label.
        let field = find(window, &[], &"secret-field".into()).unwrap();
        assert_eq!(field.role(), Some(Role::PasswordInput));
        assert_eq!(field.value(), None);
        let toggle = find(window, &[], &toggle_id("secret-field")).unwrap();
        assert_eq!(toggle.label(), Some("Show password"));

        // Click toggle to reveal password.
        let pos = toggle.bounds().center();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: pos,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position: pos,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
        window.refresh();
        window.draw(cx).clear(cx);

        assert!(view.read(cx).visible);
        let field = find(window, &[], &"secret-field".into()).unwrap();
        assert_eq!(field.role(), Some(Role::TextInput));
        assert_eq!(field.value(), Some("hunter2"));
        assert!(secret.read(cx).focus_handle(cx).is_focused(window));
        assert_eq!(
            find(window, &[], &toggle_id("secret-field"))
                .unwrap()
                .label(),
            Some("Hide password")
        );

        // Disabled secure field disables its toggle and ignores clicks.
        let dis_toggle = find(window, &[], &toggle_id("disabled-secret")).unwrap();
        assert_eq!(dis_toggle.disabled(), Some(true));
        let dis_pos = dis_toggle.bounds().center();
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: dis_pos,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position: dis_pos,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(view.read(cx).disabled_clicks, 0);
    })
    .unwrap();
}

#[gpui::test]
fn text_field_show_label_density_min_height_and_indicator_width(cx: &mut TestAppContext) {
    struct CompactFields {
        filled_default: Entity<TextFieldState>,
        outlined_default: Entity<TextFieldState>,
        no_label_compact: Entity<TextFieldState>,
        invalid_override: Entity<TextFieldState>,
        borderless_filled: Entity<TextFieldState>,
        focus_indicator_outlined: Entity<TextFieldState>,
    }

    const CUSTOM_INDICATOR: u32 = 0x2a8f63;

    impl Render for CompactFields {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let indicator_color: Hsla = rgb(CUSTOM_INDICATOR).into();
            let indicator_colors = TextFieldColors {
                unfocused: TextFieldColorOverrides {
                    indicator: Some(indicator_color),
                    ..Default::default()
                },
                focused: TextFieldColorOverrides {
                    indicator: Some(indicator_color),
                    ..Default::default()
                },
                ..Default::default()
            };
            div()
                .w(px(360.))
                .flex()
                .flex_col()
                .gap_3()
                .p_3()
                .child(TextField::filled(
                    "filled-default",
                    "Filled default",
                    &self.filled_default,
                ))
                .child(TextField::outlined(
                    "outlined-default",
                    "Outlined default",
                    &self.outlined_default,
                ))
                .child(
                    TextField::filled("no-label-compact", "Search query", &self.no_label_compact)
                        .show_label(false)
                        .min_height(px(36.))
                        .placeholder_content("Find", "Find...")
                        .prefix(div().id("compact-prefix").test_support().child("@"))
                        .suffix(div().id("compact-suffix").test_support().child(".rs")),
                )
                .child(
                    TextField::outlined(
                        "invalid-override",
                        "Invalid override",
                        &self.invalid_override,
                    )
                    .min_height(px(-10.))
                    .min_height(px(f32::NAN))
                    .indicator_width(px(-2.), px(f32::INFINITY)),
                )
                .child(
                    TextField::filled(
                        "borderless-filled",
                        "Borderless filled",
                        &self.borderless_filled,
                    )
                    .show_label(false)
                    .colors(indicator_colors)
                    .indicator_width(px(0.), px(0.)),
                )
                .child(
                    TextField::outlined(
                        "focus-indicator-outlined",
                        "Focus indicator",
                        &self.focus_indicator_outlined,
                    )
                    .show_label(false)
                    .colors(indicator_colors)
                    .indicator_width(px(0.), px(3.)),
                )
        }
    }

    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut entity = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let v = cx.new(|cx| CompactFields {
                filled_default: cx.new(|cx| TextFieldState::new(window, cx)),
                outlined_default: cx.new(|cx| TextFieldState::new(window, cx)),
                no_label_compact: cx.new(|cx| TextFieldState::new(window, cx)),
                invalid_override: cx.new(|cx| TextFieldState::new(window, cx)),
                borderless_filled: cx.new(|cx| TextFieldState::new(window, cx)),
                focus_indicator_outlined: cx.new(|cx| TextFieldState::new(window, cx)),
            });
            entity = Some(v.clone());
            v
        })
        .unwrap();
        (window, entity.unwrap())
    });
    cx.simulate_window_resize(window.into(), size(px(440.), px(900.)));

    let custom_indicator: Hsla = rgb(CUSTOM_INDICATOR).into();
    let container_height = |window: &Window, id: &'static str| {
        let field = find(window, &[], &id.into()).unwrap();
        find(window, field.path(), &"container".into())
            .unwrap()
            .bounds()
            .size
            .height
    };

    cx.update_window(window.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);

        // Standard density: 56dp default height for both Filled and Outlined,
        // invalid min_height overrides ignored.
        assert_eq!(container_height(window, "filled-default"), px(56.));
        assert_eq!(container_height(window, "outlined-default"), px(56.));
        assert_eq!(container_height(window, "invalid-override"), px(56.));

        // show_label(false): preserves aria_label, omits visual label, keeps
        // placeholder and affixes immediately visible, and allows shrinking via min_height.
        let compact = find(window, &[], &"no-label-compact".into()).unwrap();
        assert_eq!(compact.label(), Some("Search query"));
        assert!(find(window, compact.path(), &"label".into()).is_none());
        assert!(find(window, compact.path(), &"placeholder".into()).is_some());
        assert!(find(window, compact.path(), &"compact-prefix".into()).is_some());
        assert!(find(window, compact.path(), &"compact-suffix".into()).is_some());
        assert_eq!(container_height(window, "no-label-compact"), px(36.));

        // Unfocused: both indicator_width(0, 0) and indicator_width(0, 3) omit indicator quads.
        assert!(
            !window
                .painted_quads()
                .iter()
                .any(|quad| quad.border_color == custom_indicator),
            "unfocused 0px indicator_width must omit border/indicator quad"
        );

        // Density::Compact -> 52dp default height for both Filled and Outlined.
        let mut compact_theme = MaterialTheme::baseline(Appearance::Light);
        compact_theme.layout.density = Density::Compact;
        set_window_material_theme(Some(compact_theme), window, cx);
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(container_height(window, "filled-default"), px(52.));
        assert_eq!(container_height(window, "outlined-default"), px(52.));

        // Density::Comfortable -> 60dp default height for both Filled and Outlined.
        let mut comfortable_theme = MaterialTheme::baseline(Appearance::Light);
        comfortable_theme.layout.density = Density::Comfortable;
        set_window_material_theme(Some(comfortable_theme), window, cx);
        window.refresh();
        window.draw(cx).clear(cx);
        assert_eq!(container_height(window, "filled-default"), px(60.));
        assert_eq!(container_height(window, "outlined-default"), px(60.));

        // Focusing borderless-filled with indicator_width(0, 0) still paints no indicator.
        let borderless_focus = view
            .read(cx)
            .borderless_filled
            .read(cx)
            .presentation()
            .focus_handle()
            .clone();
        borderless_focus.focus(window, cx);
        window.refresh();
        window.draw(cx).clear(cx);
        assert!(
            !window
                .painted_quads()
                .iter()
                .any(|quad| quad.border_color == custom_indicator),
            "focused 0px indicator_width must omit indicator quad"
        );

        // Focusing focus-indicator-outlined with indicator_width(0, 3) paints a 3px border.
        let focus_indicator = view
            .read(cx)
            .focus_indicator_outlined
            .read(cx)
            .presentation()
            .focus_handle()
            .clone();
        focus_indicator.focus(window, cx);
        window.refresh();
        window.draw(cx).clear(cx);
        let scale = window.scale_factor();
        assert!(
            window.painted_quads().iter().any(|quad| {
                quad.border_color == custom_indicator
                    && quad.border_widths.top == px(3.).scale(scale)
            }),
            "focused indicator_width(0, 3) must paint 3px border"
        );
    })
    .unwrap();
}

#[gpui::test]
fn text_field_focus_boundaries_outside_blur_trailing_controls_and_disable(cx: &mut TestAppContext) {
    use gpui::{FocusHandle, MouseMoveEvent, Subscription};
    use gpui_material::{
        Button, DropdownOption, ExposedDropdown, ExposedDropdownState, TextFieldEvent,
        TextFieldLabelPosition,
    };
    use std::cell::Cell;

    struct FocusHarness {
        primary: Entity<TextFieldState>,
        secondary: Entity<TextFieldState>,
        secure: Entity<TextFieldState>,
        revealed: bool,
        dropdown: Entity<ExposedDropdownState>,
        outside_button: FocusHandle,
        primary_blurs: Rc<Cell<usize>>,
        secure_blurs: Rc<Cell<usize>>,
        _subs: [Subscription; 2],
    }

    impl FocusHarness {
        fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
            let primary = cx.new(|cx| TextFieldState::new(window, cx).default_value("hello"));
            let secondary = cx.new(|cx| TextFieldState::new(window, cx));
            let secure = cx.new(|cx| {
                TextFieldState::new(window, cx)
                    .masked(true)
                    .default_value("secret")
            });
            let dropdown = cx.new(|cx| ExposedDropdownState::new("Alpha", true, window, cx));
            let primary_blurs = Rc::new(Cell::new(0));
            let secure_blurs = Rc::new(Cell::new(0));
            let primary_blurs_sub = primary_blurs.clone();
            let secure_blurs_sub = secure_blurs.clone();
            let primary_sub = cx.subscribe(&primary, move |_, _, event: &TextFieldEvent, _| {
                if matches!(event, TextFieldEvent::Blur) {
                    primary_blurs_sub.set(primary_blurs_sub.get() + 1);
                }
            });
            let secure_sub = cx.subscribe(&secure, move |_, _, event: &TextFieldEvent, _| {
                if matches!(event, TextFieldEvent::Blur) {
                    secure_blurs_sub.set(secure_blurs_sub.get() + 1);
                }
            });
            Self {
                primary,
                secondary,
                secure,
                revealed: false,
                dropdown,
                outside_button: cx.focus_handle(),
                primary_blurs,
                secure_blurs,
                _subs: [primary_sub, secure_sub],
            }
        }
    }

    impl Render for FocusHarness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let owner = cx.entity().downgrade();
            div()
                .size_full()
                .p_4()
                .flex()
                .flex_col()
                .gap_4()
                .child(
                    TextField::outlined("primary", "Primary", &self.primary)
                        .label_position(TextFieldLabelPosition::Above)
                        .supporting_text("Helper line")
                        .clearable(true),
                )
                .child(TextField::outlined(
                    "secondary",
                    "Secondary",
                    &self.secondary,
                ))
                .child(
                    TextField::secure("secure", "Password", &self.secure).visibility_toggle(
                        self.revealed,
                        move |next, _, cx| {
                            let _ = owner.update(cx, |this, cx| {
                                this.revealed = next;
                                cx.notify();
                            });
                        },
                    ),
                )
                .child(ExposedDropdown::new(
                    "combo",
                    "Choice",
                    &self.dropdown,
                    [
                        DropdownOption::new("alpha", "Alpha"),
                        DropdownOption::new("beta", "Beta"),
                    ],
                ))
                .child(Button::filled("outside-btn", "Outside").track_focus(&self.outside_button))
                .child(div().id("blank-space").test_support().h(px(80.)).w_full())
        }
    }

    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let harness = cx.new(|cx| FocusHarness::new(window, cx));
            view = Some(harness.clone());
            harness
        })
        .unwrap();
        (window, view.unwrap())
    });
    cx.simulate_window_resize(window.into(), size(px(560.), px(760.)));

    let draw = |cx: &mut TestAppContext| {
        cx.update_window(window.into(), |_, window, cx| {
            window.activate_window();
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    };
    let click = |cx: &mut TestAppContext, position| {
        cx.update_window(window.into(), |_, window, cx| {
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
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    };
    let is_focused = |cx: &mut TestAppContext, state: &Entity<TextFieldState>| {
        cx.update_window(window.into(), |_, window, cx| {
            state
                .read(cx)
                .presentation()
                .focus_handle()
                .is_focused(window)
        })
        .unwrap()
    };

    draw(cx);
    let (primary, secondary, secure, dropdown, outside_btn, primary_blurs, secure_blurs) = cx
        .update(|cx| {
            let harness = view.read(cx);
            (
                harness.primary.clone(),
                harness.secondary.clone(),
                harness.secure.clone(),
                harness.dropdown.clone(),
                harness.outside_button.clone(),
                harness.primary_blurs.clone(),
                harness.secure_blurs.clone(),
            )
        });

    let (
        primary_container,
        primary_label,
        primary_supporting,
        primary_clear,
        secondary_container,
        secure_container,
        secure_toggle,
        blank_center,
        outside_btn_center,
    ) = cx
        .update_window(window.into(), |_, window, _| {
            let p = find(window, &[], &"primary".into()).unwrap();
            let s = find(window, &[], &"secondary".into()).unwrap();
            let sec = find(window, &[], &"secure".into()).unwrap();
            (
                find(window, p.path(), &"container".into())
                    .unwrap()
                    .bounds()
                    .center(),
                find(window, p.path(), &"label".into())
                    .unwrap()
                    .bounds()
                    .center(),
                find(window, p.path(), &"supporting".into())
                    .unwrap()
                    .bounds()
                    .center(),
                find(
                    window,
                    p.path(),
                    &(ElementId::from("primary"), "clear-button").into(),
                )
                .unwrap()
                .bounds()
                .center(),
                find(window, s.path(), &"container".into())
                    .unwrap()
                    .bounds()
                    .center(),
                find(window, sec.path(), &"container".into())
                    .unwrap()
                    .bounds()
                    .center(),
                find(
                    window,
                    sec.path(),
                    &(ElementId::from("secure"), "visibility-toggle").into(),
                )
                .unwrap()
                .bounds()
                .center(),
                find(window, &[], &"blank-space".into())
                    .unwrap()
                    .bounds()
                    .center(),
                find(window, &[], &"outside-btn".into())
                    .unwrap()
                    .bounds()
                    .center(),
            )
        })
        .unwrap();

    // 1. Clicking supporting row while unfocused does NOT focus the editor.
    click(cx, primary_supporting);
    assert!(!is_focused(cx, &primary));

    // 2. Clicking the Above label focuses the editor and clicking it again does not blur.
    click(cx, primary_label);
    assert!(is_focused(cx, &primary));
    assert_eq!(primary_blurs.get(), 0);
    click(cx, primary_label);
    assert!(is_focused(cx, &primary));
    assert_eq!(primary_blurs.get(), 0);

    // 3. Clicking the built-in clear button clears without an intermediate Blur.
    click(cx, primary_clear);
    assert!(is_focused(cx, &primary));
    assert_eq!(cx.update(|cx| primary.read(cx).value()), "");
    assert_eq!(primary_blurs.get(), 0);

    // 4. Dragging from inside the editor out to blank space keeps focus.
    cx.update_window(window.into(), |_, window, cx| {
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position: primary_container,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseMoveEvent {
                position: blank_center,
                pressed_button: Some(MouseButton::Left),
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position: blank_center,
                click_count: 1,
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    assert!(is_focused(cx, &primary));
    assert_eq!(primary_blurs.get(), 0);

    // 5. Clicking blank space outside blurs the focused editor.
    click(cx, blank_center);
    assert!(!is_focused(cx, &primary));
    assert_eq!(primary_blurs.get(), 1);

    // 6. Clicking supporting row while focused blurs the editor.
    click(cx, primary_container);
    assert!(is_focused(cx, &primary));
    click(cx, primary_supporting);
    assert!(!is_focused(cx, &primary));
    assert_eq!(primary_blurs.get(), 2);

    // 7. Clicking another TextField or Button transfers focus cleanly without losing it.
    click(cx, primary_container);
    assert!(is_focused(cx, &primary));
    click(cx, secondary_container);
    assert!(!is_focused(cx, &primary));
    assert!(is_focused(cx, &secondary));
    click(cx, outside_btn_center);
    assert!(!is_focused(cx, &secondary));
    assert!(
        cx.update_window(window.into(), |_, window, _| outside_btn.is_focused(window))
            .unwrap()
    );

    // 8. Clicking visibility_toggle toggles password visibility without intermediate Blur.
    click(cx, secure_container);
    assert!(is_focused(cx, &secure));
    assert_eq!(secure_blurs.get(), 0);
    click(cx, secure_toggle);
    assert!(is_focused(cx, &secure));
    assert!(cx.update(|cx| view.read(cx).revealed));
    assert_eq!(secure_blurs.get(), 0);

    // 9. Disabling a focused TextField blurs it.
    cx.update(|cx| secure.update(cx, |state, cx| state.set_disabled(true, cx)));
    draw(cx);
    cx.run_until_parked();
    assert!(!is_focused(cx, &secure));
    assert_eq!(secure_blurs.get(), 1);

    // 10. Open ExposedDropdown keeps editor focus when clicking an option, and blurs on outside blank click.
    let combo_toggle = cx
        .update_window(window.into(), |_, window, _| {
            let combo = find(window, &[], &(ElementId::from("combo"), "combobox").into()).unwrap();
            find(
                window,
                combo.path(),
                &(ElementId::from("combo"), "toggle-options").into(),
            )
            .unwrap()
            .bounds()
            .center()
        })
        .unwrap();
    click(cx, combo_toggle);
    cx.run_until_parked();
    draw(cx);
    let combo_editor = cx.update(|cx| {
        assert!(dropdown.read(cx).is_open());
        let combo_input = cx
            .update_window(window.into(), |_, window, _| {
                let combo =
                    find(window, &[], &(ElementId::from("combo"), "combobox").into()).unwrap();
                find(
                    window,
                    combo.path(),
                    &(ElementId::from("combo"), "value").into(),
                )
                .unwrap()
                .focused()
            })
            .unwrap();
        assert_eq!(combo_input, Some(true));
        dropdown.clone()
    });
    let beta_center = cx
        .update_window(window.into(), |_, window, _| {
            let options = find(window, &[], &(ElementId::from("combo"), "options").into()).unwrap();
            find(window, options.path(), &"beta".into())
                .unwrap()
                .bounds()
                .center()
        })
        .unwrap();
    click(cx, beta_center);
    cx.run_until_parked();
    draw(cx);
    assert_eq!(
        cx.update(|cx| combo_editor.read(cx).selected_option_id().cloned()),
        Some(ElementId::from("beta"))
    );
    assert_eq!(
        cx.update_window(window.into(), |_, window, _| {
            let combo = find(window, &[], &(ElementId::from("combo"), "combobox").into()).unwrap();
            find(
                window,
                combo.path(),
                &(ElementId::from("combo"), "value").into(),
            )
            .unwrap()
            .focused()
        })
        .unwrap(),
        Some(true)
    );

    // Reopen dropdown, then click blank space outside -> closes AND blurs editor.
    click(cx, combo_toggle);
    cx.run_until_parked();
    draw(cx);
    assert!(cx.update(|cx| dropdown.read(cx).is_open()));
    click(cx, blank_center);
    cx.run_until_parked();
    draw(cx);
    assert!(!cx.update(|cx| dropdown.read(cx).is_open()));
    assert_eq!(
        cx.update_window(window.into(), |_, window, _| {
            let combo = find(window, &[], &(ElementId::from("combo"), "combobox").into()).unwrap();
            find(
                window,
                combo.path(),
                &(ElementId::from("combo"), "value").into(),
            )
            .unwrap()
            .focused()
        })
        .unwrap(),
        Some(false)
    );
}
