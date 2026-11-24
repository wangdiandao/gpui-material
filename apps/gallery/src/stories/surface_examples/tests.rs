use std::{cell::RefCell, collections::HashMap, rc::Rc};

use gpui::{
    App, FocusHandle, Hsla, InputEvent as _, KeyDownEvent, KeyUpEvent, Keystroke, Modifiers,
    Pixels, Point, RenderOnce, TestAppContext, VisualTestContext, WindowOptions, point, rgb,
};
use gpui_base::test_support::{ElementSnapshot, find};
use gpui_material::{
    MaterialRoot, MaterialTheme, foundation::set_window_material_theme, theme::Direction,
};

use super::*;

#[derive(IntoElement)]
struct Ink(&'static str, Rc<RefCell<HashMap<&'static str, Hsla>>>);

impl RenderOnce for Ink {
    fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
        self.1
            .borrow_mut()
            .insert(self.0, window.text_style().color);
        div().size(px(8.))
    }
}

struct StyledSurfaces(Rc<RefCell<HashMap<&'static str, Hsla>>>);

impl Render for StyledSurfaces {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let ink = &self.0;
        let outer = rgb(0x72223c).into();
        Surface::new(ColorRole::Surface)
            .tonal_elevation(Elevation::Level3)
            .shadow_elevation(Elevation::Level1)
            .content_color(outer)
            .shape(CornerShape::Radii([14., 5., 11., 7.]))
            .border(SurfaceBorder {
                width: px(3.),
                color: rgb(0x23367c).into(),
            })
            .w(px(220.))
            .h(px(140.))
            .p_3()
            .flex()
            .flex_col()
            .child(Ink("outer", ink.clone()))
            .child(
                Surface::new(ColorRole::Primary)
                    .inherit_content_color(true)
                    .child(Ink("inherit", ink.clone())),
            )
            .child(
                Surface::new(ColorRole::PrimaryContainer)
                    .container_color(rgb(0xe4d4e8).into())
                    .child(Ink("custom fill", ink.clone())),
            )
            .child(
                Surface::new(ColorRole::SecondaryContainer)
                    .inherit_content_color(true)
                    .content_color(rgb(0x215015).into())
                    .child(Ink("override", ink.clone())),
            )
            .child(Surface::new(ColorRole::Tertiary).child(Ink("paired", ink.clone())))
    }
}

fn draw(cx: &mut TestAppContext, handle: gpui::WindowHandle<gpui_base::Root>) {
    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        window.simulate_next_frame(cx);
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .unwrap();
    cx.run_until_parked();
}

#[gpui::test]
fn surface_passive_colors_and_explicit_shape_inherit_through_nested_layout(
    cx: &mut TestAppContext,
) {
    let observed = Rc::new(RefCell::new(HashMap::new()));
    let mut theme = MaterialTheme::default();
    theme.layout.direction = Direction::Rtl;
    let painted = color(theme.colors.surface_at_elevation(Elevation::Level3));
    let paired = color(theme.colors.color(ColorRole::OnTertiary));
    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            set_window_material_theme(Some(theme), window, cx);
            cx.new(|_| StyledSurfaces(observed.clone()))
        })
        .unwrap()
    });
    draw(cx, handle);
    let observed = observed.borrow();
    for id in ["outer", "inherit", "custom fill"] {
        assert_eq!(observed[id], Hsla::from(rgb(0x72223c)), "{id}");
    }
    assert_eq!(observed["override"], Hsla::from(rgb(0x215015)));
    assert_eq!(observed["paired"], paired);
    cx.update_window(handle.into(), |_, window, _| {
        let quads = crate::painted_quads(window);
        let fill = quads
            .iter()
            .find(|quad| quad.background.as_solid() == Some(painted))
            .unwrap();
        let border = quads
            .iter()
            .find(|quad| quad.border_color == Hsla::from(rgb(0x23367c)))
            .unwrap();
        let scale = window.scale_factor();
        assert_eq!(fill.bounds.size.width, px(220.).scale(scale));
        assert_eq!(fill.bounds.size.height, px(140.).scale(scale));
        assert_eq!(fill.bounds, border.bounds);
        assert_eq!(border.border_widths.top, px(3.).scale(scale));
        assert_eq!(fill.corner_radii.top_left, px(5.).scale(scale));
        assert_eq!(fill.corner_radii.top_right, px(14.).scale(scale));
    })
    .unwrap();
}

struct Controls {
    active: bool,
    checked: bool,
    events: Rc<RefCell<Vec<&'static str>>>,
    requested: Rc<RefCell<Vec<bool>>>,
    child: FocusHandle,
    check: FocusHandle,
    source: Entity<InteractionSource>,
}

impl Render for Controls {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let parent = self.events.clone();
        let child = self.events.clone();
        let selected = self.events.clone();
        let disabled = self.events.clone();
        let requested = self.requested.clone();
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                Surface::clickable("outer", "Outer action", ColorRole::Surface)
                    .w(px(240.))
                    .p_3()
                    .on_activate(move |_, _| parent.borrow_mut().push("outer"))
                    .child(
                        Surface::clickable("inner", "Inner action", ColorRole::PrimaryContainer)
                            .w(px(100.))
                            .h(px(48.))
                            .track_focus(&self.child)
                            .interaction_source(&self.source)
                            .on_activate(move |_, _| child.borrow_mut().push("inner")),
                    ),
            )
            .child(
                Surface::selectable(
                    "selected",
                    "Preferred option",
                    ColorRole::SurfaceContainer,
                    self.active,
                )
                .w(px(180.))
                .h(px(48.))
                .on_activate(move |_, _| selected.borrow_mut().push("selected")),
            )
            .child(
                Surface::toggleable(
                    "checked",
                    "Enable updates",
                    ColorRole::Surface,
                    self.checked,
                )
                .shape(CornerShape::Radii([16.; 4]))
                .w(px(180.))
                .h(px(48.))
                .track_focus(&self.check)
                .interaction_source(&self.source)
                .feedback(SurfaceFeedback {
                    state_layer: false,
                    ripple: false,
                    focus_ring: false,
                    ..Default::default()
                })
                .on_checked_change(move |value, _, _| requested.borrow_mut().push(value)),
            )
            .child(
                Surface::clickable("disabled", "Managed action", ColorRole::SurfaceContainer)
                    .w(px(180.))
                    .h(px(48.))
                    .disabled(true)
                    .interaction_source(&self.source)
                    .on_activate(move |_, _| disabled.borrow_mut().push("disabled")),
            )
    }
}

fn element(window: &Window, id: &'static str) -> ElementSnapshot {
    find(window, &[], &id.into()).unwrap()
}

fn center(
    cx: &mut TestAppContext,
    handle: gpui::WindowHandle<gpui_base::Root>,
    id: &'static str,
) -> Point<Pixels> {
    cx.update_window(handle.into(), |_, window, _| {
        element(window, id).bounds().center()
    })
    .unwrap()
}

fn key(
    cx: &mut TestAppContext,
    handle: gpui::WindowHandle<gpui_base::Root>,
    name: &str,
    press: bool,
) {
    cx.update_window(handle.into(), |_, window, cx| {
        let keystroke = Keystroke::parse_with_os(name, cx.platform_info().os).unwrap();
        if press {
            window.dispatch_event(
                KeyDownEvent {
                    keystroke,
                    is_held: false,
                    prefer_character_input: false,
                }
                .to_platform_input(),
                cx,
            );
        } else {
            window.dispatch_event(KeyUpEvent { keystroke }.to_platform_input(), cx);
        }
    })
    .unwrap();
    draw(cx, handle);
}

#[gpui::test]
fn surface_interactive_semantics_controlled_values_and_feedback(cx: &mut TestAppContext) {
    let events = Rc::new(RefCell::new(Vec::new()));
    let requested = Rc::new(RefCell::new(Vec::new()));
    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut entity = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let view = cx.new(|cx| Controls {
                active: false,
                checked: false,
                events: events.clone(),
                requested: requested.clone(),
                child: cx.focus_handle(),
                check: cx.focus_handle(),
                source: cx.new(|_| InteractionSource::new()),
            });
            entity = Some(view.clone());
            view
        })
        .unwrap();
        (handle, entity.unwrap())
    });
    draw(cx, handle);
    cx.update_window(handle.into(), |_, window, cx| {
        let clicked = element(window, "inner");
        assert_eq!(clicked.role(), Some(Role::Button));
        assert_eq!(clicked.label(), Some("Inner action"));
        let selected = element(window, "selected");
        assert_eq!(selected.role(), Some(Role::ListBoxOption));
        assert_eq!(selected.selected(), Some(false));
        let checked = element(window, "checked");
        assert_eq!(checked.role(), Some(Role::CheckBox));
        assert_eq!(checked.checked(), Some(false));
        assert_eq!(element(window, "disabled").disabled(), Some(true));
        assert!(
            view.read(cx)
                .source
                .read(cx)
                .state(handle.window_id(), &"disabled".into())
                .unwrap()
                .disabled
        );
    })
    .unwrap();
    for id in ["inner", "selected", "disabled"] {
        let position = center(cx, handle, id);
        VisualTestContext::from_window(handle.into(), cx)
            .simulate_click(position, Modifiers::default());
        draw(cx, handle);
    }
    assert_eq!(&*events.borrow(), &["inner", "selected"]);

    cx.update_window(handle.into(), |_, window, cx| {
        let check = view.read(cx).check.clone();
        check.focus(window, cx);
    })
    .unwrap();
    draw(cx, handle);
    key(cx, handle, "space", true);
    cx.update_window(handle.into(), |_, window, cx| {
        let state = view
            .read(cx)
            .source
            .read(cx)
            .state(handle.window_id(), &"checked".into())
            .unwrap();
        assert!(state.pressed && state.focus_visible);
        assert!(requested.borrow().is_empty());
        assert_eq!(element(window, "checked").checked(), Some(false));
    })
    .unwrap();
    key(cx, handle, "space", false);
    assert_eq!(&*requested.borrow(), &[true]);
    cx.update_window(handle.into(), |_, window, cx| {
        let state = view
            .read(cx)
            .source
            .read(cx)
            .state(handle.window_id(), &"checked".into())
            .unwrap();
        assert!(!state.pressed);
        assert_eq!(element(window, "checked").checked(), Some(false));
        assert_eq!(element(window, "selected").selected(), Some(false));
    })
    .unwrap();
    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.active = true;
            view.checked = true;
            cx.notify();
        })
    });
    draw(cx, handle);
    cx.update_window(handle.into(), |_, window, _| {
        assert_eq!(element(window, "checked").checked(), Some(true));
        assert_eq!(element(window, "selected").selected(), Some(true));
    })
    .unwrap();
    key(cx, handle, "enter", true);
    key(cx, handle, "enter", false);
    assert_eq!(&*requested.borrow(), &[true, false]);
}

struct PolygonSurfaces {
    progress: f32,
    activations: Rc<RefCell<Vec<&'static str>>>,
    heart_focus: FocusHandle,
    source: Entity<InteractionSource>,
}

impl Render for PolygonSurfaces {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let heart_events = self.activations.clone();
        let morph_events = self.activations.clone();
        let morphed = ShapeMorph::new(MaterialShape::Square, MaterialShape::Circle)
            .sample_polygon(self.progress);
        div()
            .flex()
            .gap_4()
            .p_4()
            .child(
                Surface::clickable("shape-heart", "Heart action", ColorRole::PrimaryContainer)
                    .polygon_shape(MaterialShape::Heart)
                    .shape_clip_backdrop(rgb(0xf5f2fa).into())
                    .border(SurfaceBorder {
                        width: px(2.),
                        color: rgb(0x4f378b).into(),
                    })
                    .shadow_elevation(Elevation::Level1)
                    .track_focus(&self.heart_focus)
                    .interaction_source(&self.source)
                    .size(px(100.))
                    .on_activate(move |_, _| heart_events.borrow_mut().push("heart"))
                    .child(div().w(px(120.)).h(px(24.)).bg(rgb(0x6750a4))),
            )
            .child(
                Surface::clickable(
                    "shape-morph",
                    "Morphed action",
                    ColorRole::SecondaryContainer,
                )
                .polygon_shape(morphed)
                .size(px(100.))
                .on_activate(move |_, _| morph_events.borrow_mut().push("morph")),
            )
    }
}

#[gpui::test]
fn polygon_surface_supports_expressive_morph_clipping_and_shape_hit_testing(
    cx: &mut TestAppContext,
) {
    let activations = Rc::new(RefCell::new(Vec::new()));
    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut entity = None;
        let handle = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            let view = cx.new(|cx| PolygonSurfaces {
                progress: 0.,
                activations: activations.clone(),
                heart_focus: cx.focus_handle(),
                source: cx.new(|_| InteractionSource::new()),
            });
            entity = Some(view.clone());
            view
        })
        .unwrap();
        (handle, entity.unwrap())
    });
    draw(cx, handle);

    let (heart_bounds, morph_bounds) = cx
        .update_window(handle.into(), |_, window, _| {
            (
                element(window, "shape-heart").bounds(),
                element(window, "shape-morph").bounds(),
            )
        })
        .unwrap();

    // Outside corner of Heart's bounding box must NOT trigger hover or click activation.
    let outside_heart = heart_bounds.origin + point(px(4.), px(96.));
    let inside_heart = heart_bounds.center();
    {
        let mut visual = VisualTestContext::from_window(handle.into(), cx);
        visual.simulate_mouse_move(outside_heart, None, Modifiers::default());
    }
    draw(cx, handle);
    cx.update_window(handle.into(), |_, _, cx| {
        let state = view
            .read(cx)
            .source
            .read(cx)
            .state(handle.window_id(), &"shape-heart".into())
            .unwrap();
        assert!(!state.hovered, "hover must ignore points outside Heart");
    })
    .unwrap();
    VisualTestContext::from_window(handle.into(), cx)
        .simulate_click(outside_heart, Modifiers::default());
    draw(cx, handle);
    assert!(
        activations.borrow().is_empty(),
        "click outside Heart contour must not activate"
    );

    // Moving and clicking inside Heart triggers hover and activation.
    {
        let mut visual = VisualTestContext::from_window(handle.into(), cx);
        visual.simulate_mouse_move(inside_heart, None, Modifiers::default());
    }
    draw(cx, handle);
    cx.update_window(handle.into(), |_, _, cx| {
        let state = view
            .read(cx)
            .source
            .read(cx)
            .state(handle.window_id(), &"shape-heart".into())
            .unwrap();
        assert!(state.hovered, "hover must activate inside Heart");
    })
    .unwrap();
    VisualTestContext::from_window(handle.into(), cx)
        .simulate_click(inside_heart, Modifiers::default());
    draw(cx, handle);
    assert_eq!(&*activations.borrow(), &["heart"]);

    // Keyboard activation still works regardless of pointer location.
    cx.update_window(handle.into(), |_, window, cx| {
        view.read(cx).heart_focus.clone().focus(window, cx);
    })
    .unwrap();
    draw(cx, handle);
    key(cx, handle, "enter", true);
    key(cx, handle, "enter", false);
    assert_eq!(&*activations.borrow(), &["heart", "heart"]);

    // Morphed surface hit region follows live progress (Square at 0.0 hits near corner; Circle at 1.0 misses).
    let near_corner = morph_bounds.center() + point(px(40.), px(34.));
    VisualTestContext::from_window(handle.into(), cx)
        .simulate_click(near_corner, Modifiers::default());
    draw(cx, handle);
    assert_eq!(&*activations.borrow(), &["heart", "heart", "morph"]);

    cx.update(|cx| {
        view.update(cx, |view, cx| {
            view.progress = 1.;
            cx.notify();
        });
    });
    draw(cx, handle);
    VisualTestContext::from_window(handle.into(), cx)
        .simulate_click(near_corner, Modifiers::default());
    draw(cx, handle);
    assert_eq!(
        &*activations.borrow(),
        &["heart", "heart", "morph"],
        "Circle endpoint must reject corner clicks that Square accepted"
    );
}
