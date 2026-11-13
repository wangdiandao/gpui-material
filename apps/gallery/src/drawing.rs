use std::{cell::RefCell, rc::Rc};

use gpui::{
    ClickEvent, Context, FocusHandle, InteractiveElement, IntoElement, MouseButton, ParentElement,
    Render, Role, StatefulInteractiveElement, Styled, Window, canvas, div,
};
use gpui_base::Button;
use gpui_material::foundation::{
    InteractionFeedback, InteractionState, PolygonMorph, Ripple, Shape, animate, color,
    material_theme,
};
use gpui_material::theme::{ColorRole, MotionKind, MotionSpeed, PolygonRole};

pub(crate) struct Drawing {
    morph: Rc<PolygonMorph>,
    mesh: Rc<RefCell<Option<Shape>>>,
    focus: FocusHandle,
    expanded: bool,
    #[cfg(all(test, feature = "test-support"))]
    progress: f32,
    hovered: bool,
    ripple: Option<Rc<Ripple>>,
    pressed: bool,
}

impl Drawing {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            morph: Rc::new(PolygonMorph::new(
                PolygonRole::Triangle,
                PolygonRole::Square,
            )),
            mesh: Default::default(),
            focus: cx.focus_handle(),
            expanded: false,
            #[cfg(all(test, feature = "test-support"))]
            progress: 0.,
            hovered: false,
            ripple: None,
            pressed: false,
        }
    }
}

impl Render for Drawing {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = material_theme(cx);
        let fill = color(theme.colors.color(ColorRole::PrimaryContainer));
        let foreground = color(theme.colors.color(ColorRole::OnPrimaryContainer));
        let ring = color(theme.colors.color(ColorRole::Secondary));
        let reduced = cx.reduce_motion();
        let spec = theme
            .motion
            .spec(MotionKind::Spatial, MotionSpeed::Default, reduced);
        let progress = animate("drawing-morph", f32::from(self.expanded), spec, window, cx);

        #[cfg(all(test, feature = "test-support"))]
        {
            self.progress = progress;
        }

        let mesh = self.mesh.clone();
        let morph = self.morph.clone();
        let paint_mesh = mesh.clone();
        let ripple = self.ripple.clone();
        let focus = self.focus.clone();
        let hovered = self.hovered;
        let pressed = self.pressed;
        div()
            .flex()
            .items_center()
            .gap_4()
            .child(
                Button::new("morph-shape")
                    .size_20()
                    .flex_none()
                    .track_focus(&self.focus)
                    .accessibility_label("Morph shape")
                    .on_hover(cx.listener(|this, hovered, _, cx| {
                        this.hovered = *hovered;
                        cx.notify();
                    }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, event: &gpui::MouseDownEvent, _, cx| {
                            if let Some(shape) = this.mesh.borrow().as_ref() {
                                this.ripple = Ripple::start(
                                    shape,
                                    Some(event.position),
                                    cx.background_executor().now(),
                                )
                                .map(Rc::new);
                                this.pressed = this.ripple.is_some();
                            }
                            cx.notify();
                        }),
                    )
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.pressed = false;
                            cx.notify();
                        }),
                    )
                    .on_mouse_up_out(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.pressed = false;
                            this.ripple = None;
                            cx.notify();
                        }),
                    )
                    .on_click(cx.listener(|this, event, _, cx| {
                        let mesh = this.mesh.borrow();
                        let Some(shape) = mesh.as_ref() else {
                            return;
                        };
                        match event {
                            ClickEvent::Mouse(event) => {
                                if !shape.contains(event.down.position)
                                    || !shape.contains(event.up.position)
                                {
                                    this.ripple = None;
                                    cx.notify();
                                    return;
                                }
                            }
                            ClickEvent::Touch(event) => {
                                this.ripple = Ripple::start(
                                    shape,
                                    Some(event.position),
                                    cx.background_executor().now(),
                                )
                                .map(Rc::new);
                                if this.ripple.is_none() {
                                    return;
                                }
                            }
                            ClickEvent::Keyboard(_) => {
                                this.ripple =
                                    Ripple::start(shape, None, cx.background_executor().now())
                                        .map(Rc::new);
                            }
                        }
                        this.expanded = !this.expanded;
                        cx.notify();
                    }))
                    .child(
                        canvas(
                            move |bounds, _, _| *mesh.borrow_mut() = morph.sample(bounds, progress),
                            move |_, _, window, cx| {
                                let mesh = paint_mesh.borrow();
                                if let Some(shape) = mesh.as_ref() {
                                    shape.paint(fill, window);
                                    let mut state = InteractionState::default();
                                    state.hovered =
                                        hovered && shape.contains(window.mouse_position());
                                    state.pressed = pressed && state.hovered;
                                    state.focus_visible = focus.is_focused(window)
                                        && window.last_input_was_keyboard();
                                    InteractionFeedback::new(state, foreground, ring).paint(
                                        shape,
                                        ripple.as_deref(),
                                        cx.background_executor().now(),
                                        reduced,
                                        window,
                                    );
                                }
                            },
                        )
                        .size_full(),
                    ),
            )
            .child({
                let label = if self.expanded {
                    "Rounded square"
                } else {
                    "Rounded triangle"
                };
                div()
                    .id("shape-status")
                    .role(Role::Status)
                    .aria_label(label)
                    .child(label)
            })
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{
        AppContext, InputEvent, MouseDownEvent, MouseUpEvent, Pixels, Point, TestAppContext, px,
        size,
    };
    use gpui_base::test_support::find;

    fn click(position: Point<Pixels>, window: &mut Window, cx: &mut gpui::App) {
        window.dispatch_event(
            MouseDownEvent {
                button: MouseButton::Left,
                position,
                modifiers: Default::default(),
                click_count: 1,
                first_mouse: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(
            MouseUpEvent {
                button: MouseButton::Left,
                position,
                modifiers: Default::default(),
                click_count: 1,
            }
            .to_platform_input(),
            cx,
        );
    }

    #[gpui::test]
    fn shape_activates_inside_the_mesh_and_motion_settles(cx: &mut TestAppContext) {
        let (window, drawing) = cx.update(|cx| {
            gpui_material::foundation::MaterialRoot::init(Default::default(), cx);
            let drawing = gpui::AppContext::new(cx, Drawing::new);
            let window = gpui_material::foundation::MaterialRoot::open_window(
                gpui::WindowOptions {
                    window_bounds: Some(gpui::WindowBounds::centered(size(px(400.), px(150.)), cx)),
                    ..Default::default()
                },
                cx,
                |_, _| drawing.clone(),
            )
            .unwrap();
            (window, drawing)
        });
        cx.run_until_parked();

        let bounds = cx
            .update_window(window.into(), |_, window, cx| {
                window.activate_window();
                window.set_scale_factor(1.25);
                window.refresh();
                window.draw(cx).clear(cx);
                find(window, &[], &"morph-shape".into()).unwrap().bounds()
            })
            .unwrap();
        cx.update_window(window.into(), |_, window, cx| {
            click(bounds.origin + gpui::point(px(1.), px(1.)), window, cx);
        })
        .unwrap();
        cx.run_until_parked();
        assert!(!cx.update(|cx| drawing.read(cx).expanded));
        cx.update_window(window.into(), |_, window, cx| {
            click(bounds.center(), window, cx)
        })
        .unwrap();
        cx.run_until_parked();
        assert!(cx.update(|cx| drawing.read(cx).expanded));
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(50));
        let sample = |cx: &mut TestAppContext| {
            cx.update_window(window.into(), |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
                drawing.read(cx).progress
            })
            .unwrap()
        };
        let outbound = sample(cx);
        assert!((0. ..1.).contains(&outbound));
        cx.update_window(window.into(), |_, window, cx| {
            click(bounds.center(), window, cx)
        })
        .unwrap();
        assert!(!cx.update(|cx| drawing.read(cx).expanded));
        assert_eq!(sample(cx), outbound);
        cx.update(|cx| cx.set_reduce_motion(true));
        assert_eq!(sample(cx), 0.);
        cx.executor()
            .advance_clock(std::time::Duration::from_secs(2));
        cx.update_window(window.into(), |_, window, cx| {
            window.simulate_next_frame(cx)
        })
        .unwrap();
        cx.run_until_parked();
        assert_eq!(
            cx.update_window(window.into(), |_, window, cx| window
                .simulate_next_frame(cx))
                .unwrap(),
            0
        );
    }
}
