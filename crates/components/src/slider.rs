use std::{cell::RefCell, rc::Rc};

use gpui::{
    Along, AnyElement, App, AppContext, Axis, Bounds, DragMoveEvent, ElementId, Entity,
    FocusHandle, Hsla, InteractiveElement, IntoElement, MouseButton, ParentElement, Pixels, Point,
    Render, RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled, Window, canvas,
    div, point, prelude::FluentBuilder, px, relative,
};
use gpui_base::{
    AxisExt, TestSupportExt,
    slider::{Slider as BaseSlider, SliderIndicator},
};
use gpui_material_foundation::{
    DragCapture, RangeThumb, color, material_theme, range_semantics, value_label,
};
use gpui_material_theme::{ColorRole, Direction};

use crate::ControlSize;

pub use gpui_base::slider::{SliderEvent, SliderScale, SliderState, SliderValue};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SliderKind {
    #[default]
    Standard,
    Centered,
}

/// An Expressive slider. Base owns the value and step grid. The caller retains
/// the state entity and observes its SliderEvent notifications.
#[derive(IntoElement)]
pub struct Slider {
    id: ElementId,
    name: SharedString,
    state: Entity<SliderState>,
    kind: SliderKind,
    axis: Axis,
    size: ControlSize,
    disabled: bool,
    icon: Option<AnyElement>,
    range_end_name: Option<SharedString>,
}

impl Slider {
    pub fn new(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        state: &Entity<SliderState>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            state: state.clone(),
            kind: SliderKind::Standard,
            axis: Axis::Horizontal,
            size: ControlSize::Small,
            disabled: false,
            icon: None,
            range_end_name: None,
        }
    }

    /// Supplies the two accessible names of a slider with a Base range value.
    pub fn range(
        id: impl Into<ElementId>,
        start_name: impl Into<SharedString>,
        end_name: impl Into<SharedString>,
        state: &Entity<SliderState>,
    ) -> Self {
        let mut slider = Self::new(id, start_name, state);
        slider.range_end_name = Some(end_name.into());
        slider
    }

    pub fn kind(mut self, kind: SliderKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }
}

#[derive(Default)]
struct Gesture {
    thumb: Option<RangeThumb>,
    origin: Option<Point<Pixels>>,
    capture: Option<DragCapture>,
}

struct SliderDrag(Entity<SliderState>);

impl Render for SliderDrag {
    fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
        div()
    }
}

#[derive(Clone, Copy)]
struct Geometry {
    axis: Axis,
    direction: Direction,
}

impl Geometry {
    fn reversed(self) -> bool {
        self.axis.is_horizontal() && self.direction == Direction::Rtl
    }

    fn position(self, bounds: Bounds<Pixels>, pointer: Point<Pixels>) -> Point<Pixels> {
        if self.reversed() {
            point(bounds.left() + (bounds.right() - pointer.x), pointer.y)
        } else {
            pointer
        }
    }

    fn fraction(self, bounds: Bounds<Pixels>, pointer: Point<Pixels>) -> f32 {
        let pointer = self.position(bounds, pointer);
        if self.axis.is_horizontal() {
            (pointer.x - bounds.left()) / bounds.size.width
        } else {
            (bounds.bottom() - pointer.y) / bounds.size.height
        }
    }

    fn place(self, element: gpui::Div, position: f32) -> gpui::Div {
        if self.axis.is_vertical() {
            element.bottom(relative(position))
        } else if self.reversed() {
            element.right(relative(position))
        } else {
            element.left(relative(position))
        }
    }
}

impl RenderOnce for Slider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let ([start_focus, end_focus], gesture) = window
            .use_keyed_state(
                ElementId::from((self.id.clone(), "slider-state")),
                cx,
                |_, cx| {
                    (
                        [cx.focus_handle(), cx.focus_handle()],
                        Rc::new(RefCell::new(Gesture::default())),
                    )
                },
            )
            .read(cx)
            .clone();
        let geometry = Geometry {
            axis: self.axis,
            direction: material_theme(cx).layout.direction,
        };
        let horizontal = self.axis.is_horizontal();
        let value = self.state.read(cx).value();
        let percentage = self.state.read(cx).percentage();
        let start = percentage.start;
        let end = percentage.end;
        let range = value.is_range();
        let center = if self.kind == SliderKind::Centered && !range {
            0.5
        } else {
            start
        };
        // Material slider specs, expressive track and handle sizes.
        let (thickness, thumb_extent, track_radius, icon_size): (f32, f32, f32, Option<f32>) =
            match self.size {
                ControlSize::ExtraSmall => (16., 44., 8., None),
                ControlSize::Small => (24., 44., 8., None),
                ControlSize::Medium => (40., 52., 12., Some(24.)),
                ControlSize::Large => (56., 68., 16., Some(24.)),
                ControlSize::ExtraLarge => (96., 108., 28., Some(32.)),
            };
        let target = thumb_extent.max(48.);
        let resolve = |role| color(material_theme(cx).colors.color(role));
        let disabled = resolve(ColorRole::OnSurface);
        let active = if self.disabled {
            disabled.opacity(0.38)
        } else {
            resolve(ColorRole::Primary)
        };
        let inactive = if self.disabled {
            disabled.opacity(0.12)
        } else {
            resolve(ColorRole::SecondaryContainer)
        };
        let ring = resolve(ColorRole::Secondary);

        let track = SliderIndicator::new(&self.state)
            .relative()
            .size_full()
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, _| {
                        let p = |f: f32| {
                            if horizontal {
                                let f = if geometry.reversed() { 1. - f } else { f };
                                point(bounds.left() + bounds.size.width * f, bounds.center().y)
                            } else {
                                point(bounds.center().x, bounds.bottom() - bounds.size.height * f)
                            }
                        };
                        let segment = |a: f32, b: f32, tint: Hsla, window: &mut Window| {
                            let pa = p(a);
                            let pb = p(b);
                            let half = px(thickness / 2.);
                            let rect = if horizontal {
                                Bounds::from_corners(
                                    point(pa.x.min(pb.x), pa.y - half),
                                    point(pa.x.max(pb.x), pb.y + half),
                                )
                            } else {
                                Bounds::from_corners(
                                    point(pa.x - half, pa.y.min(pb.y)),
                                    point(pb.x + half, pa.y.max(pb.y)),
                                )
                            };
                            if rect.size.width > px(0.) && rect.size.height > px(0.) {
                                window.paint_quad(gpui::quad(
                                    rect,
                                    px(track_radius),
                                    tint,
                                    px(0.),
                                    tint,
                                    gpui::BorderStyle::Solid,
                                ));
                            }
                        };
                        let first = center.min(end);
                        let last = center.max(end);
                        segment(0., first, inactive, window);
                        segment(first, last, active, window);
                        segment(last, 1., inactive, window);
                    },
                )
                .absolute()
                .inset_0()
                .size_full(),
            );
        let dragging = gesture.borrow().thumb;
        let thumb = |which, position, name, focus: &FocusHandle| {
            let keyboard_state = self.state.clone();
            let engaged = dragging == Some(which);
            let handle_width = if engaged { 2. } else { 4. };
            let half_handle = px(handle_width / 2.);
            let (width, height) = if horizontal {
                (handle_width, thumb_extent)
            } else {
                (thumb_extent, handle_width)
            };
            let label = value_label(&self.state, which, cx, |value| format!("{value:.2}").into());
            let label_style = div()
                .absolute()
                .px_2()
                .py_1()
                .rounded_md()
                .bg(resolve(ColorRole::InverseSurface))
                .text_color(resolve(ColorRole::InverseOnSurface))
                .text_xs()
                .when(horizontal, |label| label.bottom(px(height + 8.)))
                .when(!horizontal, |label| label.left(px(width + 8.)))
                .child(label);
            let element = geometry
                .place(
                    div()
                        .absolute()
                        .when(horizontal, |thumb| {
                            thumb
                                .top(px((target - height) / 2.))
                                .ml(-half_handle)
                                .when(geometry.reversed(), |thumb| {
                                    thumb.ml(px(0.)).mr(-half_handle)
                                })
                        })
                        .when(!horizontal, |thumb| {
                            thumb.left(px((target - width) / 2.)).mb(-half_handle)
                        }),
                    position,
                )
                .id(ElementId::from((
                    self.id.clone(),
                    if which == RangeThumb::Start {
                        "start"
                    } else {
                        "end"
                    },
                )))
                .when(!self.disabled, |thumb| thumb.role(Role::Slider))
                .when(!self.disabled, |thumb| thumb.aria_label(name))
                .w(px(width))
                .h(px(height))
                .rounded_full()
                .bg(active)
                .focus_visible(move |style| style.bg(ring))
                .when(
                    engaged
                        || !self.disabled
                            && focus.is_focused(window)
                            && window.last_input_was_keyboard(),
                    |thumb| thumb.child(label_style),
                );

            let element = if self.disabled {
                element
            } else {
                range_semantics(element, &self.state, which, self.axis, cx).capture_key_down(
                    move |event, window, cx| {
                        let sign = match event.keystroke.key.as_str() {
                            "left" if geometry.reversed() => 1.,
                            "right" if geometry.reversed() => -1.,
                            _ => return,
                        };
                        keyboard_state.update(cx, |state, cx| {
                            state.step_thumb(
                                which == RangeThumb::Start,
                                sign * state.accessibility_step(),
                                window,
                                cx,
                            );
                        });
                        cx.stop_propagation();
                    },
                )
            };
            element.test_support().when(!self.disabled, |thumb| {
                thumb.track_focus(&focus.clone().tab_stop(true))
            })
        };
        let disabled_name = self.range_end_name.as_ref().map_or_else(
            || self.name.clone(),
            |end| format!("{} – {end}", self.name).into(),
        );
        let mut root = div()
            .id(self.id.clone())
            .relative()
            .when(horizontal, |root| root.w_full().h(px(target)).px_2())
            .when(!horizontal, |root| root.w(px(target)).h(px(200.)).py_2())
            .child(track)
            .when(range, |root| {
                root.child(thumb(
                    RangeThumb::Start,
                    start,
                    self.name.clone(),
                    &start_focus,
                ))
            })
            .child(thumb(
                RangeThumb::End,
                end,
                self.range_end_name.unwrap_or(self.name),
                &end_focus,
            ));
        if let (Some(icon), Some(icon_size)) = (self.icon, icon_size) {
            root = root.child(
                div()
                    .absolute()
                    .when(geometry.reversed(), |icon| icon.right_3())
                    .when(!geometry.reversed(), |icon| icon.left_3())
                    .top_1()
                    .text_size(px(icon_size))
                    .text_color(active)
                    .child(icon),
            );
        }
        if self.disabled || !window.is_window_active() && gesture.borrow().thumb.is_some() {
            if gesture.borrow().thumb.is_some() {
                let state = self.state.clone();
                let gesture = gesture.clone();
                window.defer(cx, move |window, cx| {
                    finish(&state, &gesture, window, cx, true)
                });
            }
            return if self.disabled {
                BaseSlider::new(&self.state)
                    .axis(self.axis)
                    .disabled(true)
                    .accessibility_label(disabled_name)
                    .child(root)
                    .into_any_element()
            } else {
                root.into_any_element()
            };
        }

        let down = gesture.clone();
        let begin = gesture.clone();
        let drag = gesture.clone();
        let release = gesture.clone();
        let release_out = gesture.clone();
        let scroll = gesture.clone();
        let escape = gesture;
        let down_state = self.state.clone();
        let begin_state = self.state.clone();
        let drag_state = self.state.clone();
        let drag_source = self.state.clone();
        let drag_focus = [start_focus.clone(), end_focus.clone()];
        let release_state = self.state.clone();
        let outside_state = self.state.clone();
        let escape_state = self.state;
        root.on_mouse_down(MouseButton::Left, move |event, window, cx| {
            let bounds = down_state.read(cx).bounds();
            if bounds.size.along(geometry.axis) <= px(0.) {
                return;
            }
            let fraction = geometry.fraction(bounds, event.position);
            let which = choose_thumb(range, start, end, fraction);
            {
                let mut down = down.borrow_mut();
                down.thumb = Some(which);
                down.origin = Some(event.position);
            }
            if which == RangeThumb::Start {
                start_focus.focus(window, cx);
            } else {
                end_focus.focus(window, cx);
            }
            update(&down_state, geometry, which, event.position, window, cx);
        })
        .on_drag(SliderDrag(drag_source), move |payload, _, window, cx| {
            let gesture = Rc::downgrade(&begin);
            let state = begin_state.clone();
            begin.borrow_mut().capture = Some(DragCapture::watch(window, cx, move |window, cx| {
                if let Some(gesture) = gesture.upgrade() {
                    finish(&state, &gesture, window, cx, false);
                }
            }));
            cx.stop_propagation();
            cx.new(|_| SliderDrag(payload.0.clone()))
        })
        .on_drag_move(move |event: &DragMoveEvent<SliderDrag>, window, cx| {
            if event.drag(cx).0.entity_id() != drag_state.entity_id() {
                return;
            }
            let mut gesture = drag.borrow_mut();
            let Some(mut which) = gesture.thumb else {
                return;
            };
            if range
                && start == end
                && let Some(origin) = gesture.origin.take()
            {
                let bounds = drag_state.read(cx).bounds();
                which = if geometry.fraction(bounds, event.event.position)
                    < geometry.fraction(bounds, origin)
                {
                    RangeThumb::Start
                } else {
                    RangeThumb::End
                };
                gesture.thumb = Some(which);
                drag_focus[usize::from(which == RangeThumb::End)].focus(window, cx);
            }
            update(
                &drag_state,
                geometry,
                which,
                event.event.position,
                window,
                cx,
            );
            cx.stop_propagation();
        })
        .on_mouse_up(MouseButton::Left, move |_, window, cx| {
            finish(&release_state, &release, window, cx, false)
        })
        .on_mouse_up_out(MouseButton::Left, move |_, window, cx| {
            finish(&outside_state, &release_out, window, cx, false)
        })
        .capture_key_down(move |event, window, cx| {
            if event.keystroke.key == "escape" && escape.borrow().thumb.is_some() {
                finish(&escape_state, &escape, window, cx, true);
                cx.stop_propagation();
            }
        })
        .on_scroll(move |_, _, cx| {
            // Wheel events over the active range gesture stay with it.
            if scroll.borrow().thumb.is_some() {
                cx.stop_propagation();
            }
        })
        .into_any_element()
    }
}

fn choose_thumb(range: bool, start: f32, end: f32, fraction: f32) -> RangeThumb {
    if range && fraction < (start + end) / 2. {
        RangeThumb::Start
    } else {
        RangeThumb::End
    }
}

fn update(
    state: &Entity<SliderState>,
    geometry: Geometry,
    thumb: RangeThumb,
    pointer: Point<Pixels>,
    window: &mut Window,
    cx: &mut App,
) {
    state.update(cx, |state, cx| {
        let position = geometry.position(state.bounds(), pointer);
        state.update_value_by_position(
            geometry.axis,
            position,
            thumb == RangeThumb::Start,
            window,
            cx,
        );
    });
}

fn finish(
    state: &Entity<SliderState>,
    gesture: &RefCell<Gesture>,
    window: &mut Window,
    cx: &mut App,
    cancel: bool,
) {
    let mut gesture = gesture.borrow_mut();
    if gesture.thumb.take().is_none() {
        return;
    }
    if let Some(capture) = gesture.capture.take()
        && cancel
    {
        capture.cancel(window, cx);
    }
    gesture.origin = None;
    state.update(cx, |state, cx| state.handle_release(cx));
    window.refresh();
}
