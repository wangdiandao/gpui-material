use std::{cell::Cell, rc::Rc};

use gpui::{
    App, AppContext as _, Context, Entity, Hsla, InteractiveElement as _, IntoElement,
    KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, MouseButton, ParentElement as _, Render,
    RenderOnce, Role, StatefulInteractiveElement as _, Styled as _, TestAppContext,
    VisualTestContext, Window, div, point, px, rgb,
};
use gpui_base::test_support::find;
use gpui_material::foundation::set_window_material_theme;
use gpui_material::theme::{CornerShape, Direction, Elevation};
use gpui_material::{
    Card, CardColors, CardVariant, ControlBorder, ControlElevations, InteractionSource,
    InteractionState, LogicalEdges, MaterialRoot, MaterialTheme,
};

#[derive(IntoElement)]
struct InheritedColor(Rc<Cell<Hsla>>);

impl RenderOnce for InheritedColor {
    fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
        self.0.set(window.text_style().color);
        div().h(px(36.))
    }
}

struct CardDrag;

impl Render for CardDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

struct Preview {
    source: Entity<InteractionSource>,
    colors: [Rc<Cell<Hsla>>; 3],
    dragged: bool,
    disabled: bool,
    activated: usize,
}

impl Render for Preview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let caller = cx.weak_entity();
        let cancel = caller.clone();
        let activate = cx.listener(|this, _: &(), _, _| this.activated += 1);
        div()
            .w(px(320.))
            .flex()
            .flex_col()
            .gap_4()
            .child(
                Card::new("elevated")
                    .variant(CardVariant::Elevated)
                    .shape(CornerShape::Radii([9.; 4]))
                    .border(ControlBorder {
                        width: px(2.),
                        color: rgb(0x273352).into(),
                    })
                    .colors(CardColors {
                        container: Some(rgb(0xffeabb).into()),
                        content: Some(rgb(0x3b2500).into()),
                        ..Default::default()
                    })
                    .elevations(ControlElevations {
                        resting: Some(Elevation::Level2),
                        ..Default::default()
                    })
                    .interaction_source(&self.source)
                    .child(InheritedColor(self.colors[0].clone())),
            )
            .child(
                div()
                    .id("drag-owner")
                    .on_drag(CardDrag, move |_, _, _, cx| {
                        let _ = caller.update(cx, |this, cx| {
                            this.dragged = true;
                            cx.notify();
                        });
                        cx.new(|_| CardDrag)
                    })
                    .capture_key_down(move |event, window, cx| {
                        if event.keystroke.key == "escape" && cx.stop_active_drag(window) {
                            let _ = cancel.update(cx, |this, cx| {
                                this.dragged = false;
                                cx.notify();
                            });
                            cx.stop_propagation();
                        }
                    })
                    .child(
                        Card::new("filled")
                            .variant(CardVariant::Filled)
                            .shape(CornerShape::Radii([3., 6., 9., 12.]))
                            .colors(CardColors {
                                container: Some(rgb(0xe0c6f6).into()),
                                content: Some(rgb(0x2d153f).into()),
                                disabled_container: Some(rgb(0xe0d8e5).into()),
                                disabled_content: Some(rgb(0x615b69).into()),
                            })
                            .elevations(ControlElevations {
                                resting: Some(Elevation::Level1),
                                pressed: Some(Elevation::Level2),
                                focused: Some(Elevation::Level3),
                                hovered: Some(Elevation::Level4),
                                dragged: Some(Elevation::Level5),
                                disabled: Some(Elevation::Level0),
                            })
                            .interaction_source(&self.source)
                            .dragged(self.dragged)
                            .disabled(self.disabled)
                            .accessible_label("Open project")
                            .on_activate(move |window, cx| {
                                activate(&(), window, cx);
                            })
                            .child(InheritedColor(self.colors[1].clone())),
                    ),
            )
            .child(
                Card::new("outlined")
                    .variant(CardVariant::Outlined)
                    .shape(CornerShape::Radii([5.; 4]))
                    .border(ControlBorder {
                        width: px(0.),
                        color: rgb(0x7d1934).into(),
                    })
                    .colors(CardColors {
                        disabled_container: Some(rgb(0xdbe6ff).into()),
                        disabled_content: Some(rgb(0x4a5263).into()),
                        ..Default::default()
                    })
                    .elevations(ControlElevations {
                        disabled: Some(Elevation::Level1),
                        ..Default::default()
                    })
                    .disabled(true)
                    .accessible_label("Unavailable project")
                    .on_activate(|_, _| panic!("disabled card activated"))
                    .interaction_source(&self.source)
                    .child(InheritedColor(self.colors[2].clone())),
            )
    }
}

fn open(cx: &mut TestAppContext) -> (Entity<Preview>, &mut VisualTestContext) {
    cx.update(|cx| {
        MaterialRoot::init(MaterialTheme::default(), cx);
        cx.set_reduce_motion(true);
    });
    let (preview, cx) = cx.add_window_view(|window, cx| {
        window.activate_window();
        Preview {
            source: cx.new(|_| InteractionSource::new()),
            colors: std::array::from_fn(|_| Rc::new(Cell::new(gpui::transparent_black()))),
            dragged: false,
            disabled: false,
            activated: 0,
        }
    });
    draw(cx);
    (preview, cx)
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
    cx.run_until_parked();
}

fn snapshot(
    cx: &mut VisualTestContext,
    preview: &Entity<Preview>,
    id: &'static str,
) -> InteractionState {
    cx.update(|window, cx| {
        preview
            .read(cx)
            .source
            .read(cx)
            .state(window.window_handle().window_id(), &id.into())
            .unwrap()
    })
}

#[gpui::test]
fn cards_paint_independent_variants_and_keep_semantics(cx: &mut TestAppContext) {
    let (preview, cx) = open(cx);
    let variants = [
        ("elevated", 0xffeabb, 2., Some(0x273352), [9.; 4], 0x3b2500),
        ("filled", 0xe0c6f6, 1., None, [3., 6., 9., 12.], 0x2d153f),
        ("outlined", 0xdbe6ff, 0., None, [5.; 4], 0x4a5263),
    ];
    cx.update(|window, cx| {
        let quads = crate::painted_quads(window);
        let scale = window.scale_factor();
        for (index, (id, background, border, stroke, corners, foreground)) in
            variants.into_iter().enumerate()
        {
            let card = find(window, &[], &id.into()).unwrap();
            let expected_role = if index == 0 {
                Role::Group
            } else {
                Role::Button
            };
            assert_eq!(card.role(), Some(expected_role));
            let quad = quads
                .iter()
                .find(|quad| quad.background == Hsla::from(rgb(background)).into())
                .expect("variant's custom container pigment");
            assert_eq!(quad.border_widths.top, px(border).scale(scale));
            assert_eq!(
                quad.border_color,
                stroke.map_or_else(gpui::transparent_black, |stroke| rgb(stroke).into())
            );
            assert_eq!(
                [
                    quad.corner_radii.top_left,
                    quad.corner_radii.top_right,
                    quad.corner_radii.bottom_right,
                    quad.corner_radii.bottom_left,
                ],
                corners.map(|radius| px(radius).scale(scale)),
            );
            assert_eq!(preview.read(cx).colors[index].get(), rgb(foreground).into());
        }
        let filled = find(window, &[], &"filled".into()).unwrap();
        assert_eq!(filled.label(), Some("Open project"));
        assert_ne!(filled.disabled(), Some(true));
        let outlined = find(window, &[], &"outlined".into()).unwrap();
        assert_eq!(outlined.label(), Some("Unavailable project"));
        assert_eq!(outlined.disabled(), Some(true));
        let mut theme = MaterialTheme::default();
        theme.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(theme), window, cx);
    });
    draw(cx);
    cx.update(|window, _| {
        let quads = crate::painted_quads(window);
        let fill = quads
            .iter()
            .find(|quad| quad.background == Hsla::from(rgb(0xe0c6f6)).into())
            .unwrap();
        let scale = window.scale_factor();
        assert_eq!(fill.corner_radii.top_left, px(6.).scale(scale));
        assert_eq!(fill.corner_radii.top_right, px(3.).scale(scale));
    });
}

#[gpui::test]
fn actionable_card_keeps_base_activation_and_cancels_drag_click(cx: &mut TestAppContext) {
    let (preview, cx) = open(cx);
    let position = cx.update(|window, cx| {
        window.focus_next(cx);
        find(window, &[], &"filled".into())
            .unwrap()
            .bounds()
            .center()
    });
    draw(cx);
    let keystroke =
        cx.update(|_, cx| Keystroke::parse_with_os("enter", cx.platform_info().os).unwrap());
    cx.simulate_event(KeyDownEvent {
        keystroke: keystroke.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    draw(cx);
    let interaction = snapshot(cx, &preview, "filled");
    assert!(interaction.focus_visible && interaction.pressed);
    cx.simulate_event(KeyUpEvent { keystroke });
    draw(cx);
    cx.update(|_, cx| assert_eq!(preview.read(cx).activated, 1));

    cx.simulate_mouse_move(position, None, Modifiers::default());
    draw(cx);
    assert!(snapshot(cx, &preview, "filled").hovered);
    assert!(!snapshot(cx, &preview, "elevated").hovered);
    cx.simulate_mouse_down(position, MouseButton::Left, Modifiers::default());
    draw(cx);
    assert!(snapshot(cx, &preview, "filled").pressed);
    let moved = position + point(px(20.), px(0.));
    cx.simulate_mouse_move(moved, MouseButton::Left, Modifiers::default());
    draw(cx);
    let interaction = snapshot(cx, &preview, "filled");
    assert!(interaction.dragged && !interaction.pressed);
    cx.simulate_keystrokes("escape");
    draw(cx);
    assert!(!snapshot(cx, &preview, "filled").dragged);
    cx.simulate_mouse_up(moved, MouseButton::Left, Modifiers::default());
    draw(cx);
    cx.update(|_, cx| assert_eq!(preview.read(cx).activated, 1));

    cx.simulate_click(position, Modifiers::default());
    draw(cx);
    cx.update(|_, cx| assert_eq!(preview.read(cx).activated, 2));
    cx.update(|_, cx| {
        preview.update(cx, |this, cx| {
            this.dragged = true;
            this.disabled = true;
            cx.notify();
        });
    });
    draw(cx);
    let interaction = snapshot(cx, &preview, "filled");
    assert!(interaction.disabled && !interaction.dragged && !interaction.pressed);
    cx.simulate_click(position, Modifiers::default());
    draw(cx);
    cx.update(|window, cx| {
        assert_eq!(preview.read(cx).activated, 2);
        assert_eq!(preview.read(cx).colors[1].get(), rgb(0x615b69).into());
        assert!(
            crate::painted_quads(window)
                .iter()
                .any(|quad| { quad.background == Hsla::from(rgb(0xe0d8e5)).into() })
        );
    });
}

#[gpui::test]
fn nested_card_action_does_not_activate_its_parent(cx: &mut TestAppContext) {
    use super::DisplayExamples;
    use crate::story::Story;
    use gpui::{InputEvent, MouseDownEvent, MouseUpEvent, size};

    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let handle = MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
            let display = cx.new(|_| DisplayExamples::for_story(Story::Card));
            view = Some(display.clone());
            display
        })
        .unwrap();
        (handle, view.unwrap())
    });
    cx.simulate_window_resize(handle.into(), size(px(650.), px(870.)));
    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        window.refresh();
        window.draw(cx).clear(cx);
        let position = find(window, &[], &"card-child".into())
            .unwrap()
            .bounds()
            .center();
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
        assert_eq!(view.read(cx).child_count, 1);
        assert!(!view.read(cx).opened);
    })
    .unwrap();
}

#[gpui::test]
fn disabled_card_supplies_content_color_without_fading_custom_children(cx: &mut TestAppContext) {
    use gpui::WindowOptions;
    use gpui_material::{
        Text,
        foundation::{color, window_material_theme},
        theme::ColorRole,
    };

    #[derive(gpui::IntoElement)]
    struct Probe(Rc<Cell<Hsla>>);
    impl RenderOnce for Probe {
        fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
            self.0.set(window.text_style().color);
            Text::new("Inherited card content")
        }
    }
    struct Example {
        inherited: Rc<Cell<Hsla>>,
        explicit: Rc<Cell<Hsla>>,
    }
    impl Render for Example {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let primary = color(
                window_material_theme(window, cx)
                    .colors
                    .color(ColorRole::Primary),
            );
            Card::new("disabled-card")
                .disabled(true)
                .child(Probe(self.inherited.clone()))
                .child(
                    div()
                        .text_color(primary)
                        .child(Probe(self.explicit.clone())),
                )
        }
    }
    let inherited = Rc::new(Cell::new(gpui::transparent_black()));
    let explicit = Rc::new(Cell::new(gpui::transparent_black()));
    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|_| Example {
                inherited: inherited.clone(),
                explicit: explicit.clone(),
            })
        })
        .unwrap()
    });
    cx.update_window(handle.into(), |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
        let theme = window_material_theme(window, cx);
        assert_eq!(
            inherited.get(),
            color(theme.colors.color(ColorRole::OnSurface)).opacity(0.38)
        );
        assert_eq!(
            explicit.get(),
            color(theme.colors.color(ColorRole::Primary))
        );
    })
    .unwrap();
}

#[gpui::test]
fn static_card_supports_accessible_label_padding_gap_and_clipping(cx: &mut TestAppContext) {
    use gpui::{WindowOptions, size};
    use gpui_base::TestSupportExt as _;

    struct Example;
    impl Render for Example {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().p_4().child(
                Card::outlined("static-summary")
                    .accessible_label("Summary card")
                    .content_padding(LogicalEdges::all(12.))
                    .gap(px(20.))
                    .child(div().id("card-slot-a").test_support().w(px(60.)).h(px(18.)))
                    .child(div().id("card-slot-b").test_support().w(px(80.)).h(px(22.))),
            )
        }
    }

    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| cx.new(|_| Example))
            .unwrap()
    });
    cx.simulate_window_resize(handle.into(), size(px(320.), px(240.)));
    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        window.refresh();
        window.draw(cx).clear(cx);
        window.focus_next(cx);
        window.refresh();
        window.draw(cx).clear(cx);

        let card = find(window, &[], &"static-summary".into()).unwrap();
        assert_eq!(card.role(), Some(Role::Group));
        assert_eq!(card.label(), Some("Summary card"));
        assert_ne!(card.focused(), Some(true));

        let a = find(window, &[], &"card-slot-a".into()).unwrap().bounds();
        let b = find(window, &[], &"card-slot-b".into()).unwrap().bounds();
        // Default 1 px card border adds 1 px inset around the 12 px content_padding.
        assert_eq!((a.left() - card.bounds().left()).round(), px(13.));
        assert_eq!((a.top() - card.bounds().top()).round(), px(13.));
        assert_eq!((b.top() - a.bottom()).round(), px(20.));
        assert_eq!((card.bounds().bottom() - b.bottom()).round(), px(13.));
    })
    .unwrap();
}
