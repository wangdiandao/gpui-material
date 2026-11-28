use std::{cell::RefCell, rc::Rc};

use gpui::{
    AppContext, Context, Entity, Hsla, InteractiveElement, IntoElement, ParentElement, Render,
    Rgba, Styled, TestAppContext, Window, WindowOptions, div, point, px, rgb, size,
};
use gpui_base::{TestSupportExt, test_support::find};
use gpui_material::{
    InteractionSource, MaterialRoot, MaterialTheme, TextField, TextFieldColorOverrides,
    TextFieldColors, TextFieldLayout, TextFieldState, TextFieldVariant,
    foundation::set_window_material_theme,
    theme::{Appearance, CornerShape, Density, TypeRole, TypographyEmphasis},
};

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
