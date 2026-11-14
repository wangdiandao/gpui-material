use std::{cell::RefCell, rc::Rc};

use gpui::{
    Along, AnyElement, App, AppContext, AvailableSpace, Axis, Bounds, DragMoveEvent, ElementId,
    Entity, FocusHandle, HitboxBehavior, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Pixels, Point, Render, RenderOnce, Role, SharedString,
    StatefulInteractiveElement, Styled, TouchDragEvent, TouchPhase, Window, canvas, div, point,
    prelude::FluentBuilder, px, relative,
};
use gpui_base::{
    AxisExt, TestSupportExt,
    slider::{Slider as BaseSlider, SliderIndicator},
};
use gpui_material_foundation::{
    DragCapture, RangeThumb, animate, color, range_semantics, shape_text, value_label,
    window_material_theme,
};
use gpui_material_theme::{ColorRole, Direction, TypeRole, TypographyEmphasis};

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
    tick_intervals: u16,
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
            tick_intervals: 0,
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

    /// Draws equally spaced ticks in the visual range without changing the Base value scale.
    /// Pass the number of intervals, including the two endpoints.
    pub fn tick_intervals(mut self, intervals: u16) -> Self {
        self.tick_intervals = intervals;
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

    fn point_at(self, bounds: Bounds<Pixels>, offset: Pixels) -> Point<Pixels> {
        if self.axis.is_vertical() {
            point(bounds.center().x, bounds.bottom() - offset)
        } else if self.reversed() {
            point(bounds.right() - offset, bounds.center().y)
        } else {
            point(bounds.left() + offset, bounds.center().y)
        }
    }

    fn track_corners(self, start: Pixels, end: Pixels) -> gpui::Corners<Pixels> {
        let (left, right) = if self.reversed() {
            (end, start)
        } else {
            (start, end)
        };
        gpui::Corners {
            top_left: if self.axis.is_vertical() { end } else { left },
            top_right: if self.axis.is_vertical() { end } else { right },
            bottom_left: if self.axis.is_vertical() { start } else { left },
            bottom_right: if self.axis.is_vertical() {
                start
            } else {
                right
            },
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
            direction: window_material_theme(window, cx).layout.direction,
        };
        let horizontal = self.axis.is_horizontal();
        let state = self.state.read(cx);
        let value = state.value();
        let percentage = state.percentage();
        let tick_count = self.tick_intervals;
        let start = percentage.start;
        let end = percentage.end;
        let range = value.is_range();
        let centered = self.kind == SliderKind::Centered && !range;
        let center = if centered { 0.5 } else { start };
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
        let colors = window_material_theme(window, cx).colors.clone();
        let resolve = |role| color(colors.color(role));
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

        let active_icon = resolve(ColorRole::OnPrimary);
        let inactive_icon = resolve(ColorRole::OnSecondaryContainer);
        let disabled_icon = resolve(ColorRole::OnSurface).opacity(0.38);
        let disabled_control = self.disabled;
        let dragging = gesture.borrow().thumb;
        let reduced = cx.reduce_motion();
        let scheme = window_material_theme(window, cx).motion;
        let resize = scheme.spec(
            gpui_material_theme::MotionKind::Spatial,
            gpui_material_theme::MotionSpeed::Fast,
            reduced,
        );
        let reveal = scheme.spec(
            gpui_material_theme::MotionKind::Effects,
            gpui_material_theme::MotionSpeed::Fast,
            reduced,
        );
        let width = |which, window: &mut Window, cx: &mut App| {
            px(animate(
                (
                    self.id.clone(),
                    if which == RangeThumb::Start {
                        "start-width"
                    } else {
                        "end-width"
                    },
                ),
                if dragging == Some(which) { 2. } else { 4. },
                resize,
                window,
                cx,
            )
            .clamp(2., 4.))
        };
        let start_width = if range {
            width(RangeThumb::Start, window, cx)
        } else {
            px(4.)
        };
        let end_width = width(RangeThumb::End, window, cx);
        let handle_width = |which| {
            if which == RangeThumb::Start {
                start_width
            } else {
                end_width
            }
        };
        let label_opacity = |which, focus: &FocusHandle, window: &mut Window, cx: &mut App| {
            let visible = !self.disabled
                && (dragging == Some(which)
                    || focus.is_focused(window) && window.last_input_was_keyboard());
            animate(
                (
                    self.id.clone(),
                    if which == RangeThumb::Start {
                        "start-label"
                    } else {
                        "end-label"
                    },
                ),
                f32::from(visible),
                reveal,
                window,
                cx,
            )
            .clamp(0., 1.)
        };
        let start_label = if range {
            label_opacity(RangeThumb::Start, &start_focus, window, cx)
        } else {
            0.
        };
        let end_label = label_opacity(RangeThumb::End, &end_focus, window, cx);
        // Expressive tracks leave 6 dp between a handle's edge and each adjacent segment.
        let end_gap = px(6.) + handle_width(RangeThumb::End) / 2.;
        let center_gap = if range {
            px(6.) + handle_width(RangeThumb::Start) / 2.
        } else if centered {
            px(6.)
        } else {
            px(0.)
        };
        let (first, last) = if center <= end {
            ((center, center_gap), (end, end_gap))
        } else {
            ((end, end_gap), (center, center_gap))
        };
        let segments = move |length| track_segments(length, first, last);
        let track_icon = self.icon.zip(icon_size).map(|(icon, side)| {
            canvas(
                move |bounds, window, cx| {
                    let side = px(side);
                    let length = bounds.size.along(geometry.axis);
                    let (offset, active) = segments(length)
                        .into_iter()
                        .find_map(|segment| segment.icon_position(side, px(track_radius)))?;
                    let foreground = if disabled_control {
                        disabled_icon
                    } else if active {
                        active_icon
                    } else {
                        inactive_icon
                    };
                    let origin = geometry.point_at(bounds, offset) - point(side / 2., side / 2.);
                    let mut icon = div()
                        .size(side)
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(side)
                        .line_height(side)
                        .text_color(foreground)
                        .child(icon)
                        .into_any_element();
                    let space = gpui::size(
                        AvailableSpace::Definite(side),
                        AvailableSpace::Definite(side),
                    );
                    icon.prepaint_as_root(origin, space, window, cx);
                    Some(icon)
                },
                |_, icon, window, cx| {
                    if let Some(mut icon) = icon {
                        icon.paint(window, cx);
                    }
                },
            )
            .absolute()
            .inset_0()
            .size_full()
        });
        let track = SliderIndicator::new(&self.state)
            .relative()
            .size_full()
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, _| {
                        let length = bounds.size.along(geometry.axis);
                        let track_segments = segments(length);
                        let draw_indicator = |offset, tint, window: &mut Window| {
                            let center = geometry.point_at(bounds, offset);
                            let radius = px(2.);
                            window.paint_quad(gpui::quad(
                                Bounds::from_corners(
                                    center - point(radius, radius),
                                    center + point(radius, radius),
                                ),
                                radius,
                                tint,
                                px(0.),
                                gpui::transparent_black(),
                                gpui::BorderStyle::Solid,
                            ));
                        };
                        for segment in track_segments {
                            if segment.end <= segment.start {
                                continue;
                            }
                            let a = geometry.point_at(bounds, segment.start);
                            let b = geometry.point_at(bounds, segment.end);
                            let half = px(thickness / 2.);
                            let rect = if horizontal {
                                Bounds::from_corners(
                                    point(a.x.min(b.x), a.y - half),
                                    point(a.x.max(b.x), b.y + half),
                                )
                            } else {
                                Bounds::from_corners(
                                    point(a.x - half, a.y.min(b.y)),
                                    point(a.x + half, a.y.max(b.y)),
                                )
                            };
                            let tint = if segment.active { active } else { inactive };
                            let (start_corner, end_corner) = segment.corners(px(track_radius));
                            window.paint_quad(gpui::quad(
                                rect,
                                geometry.track_corners(start_corner, end_corner),
                                tint,
                                px(0.),
                                gpui::transparent_black(),
                                gpui::BorderStyle::Solid,
                            ));
                            // The stop indicator marks the outer edge of each inactive segment.
                            if !segment.active {
                                let radius = px(track_radius);
                                if segment.outer_start
                                    && segment.start + radius + px(2.) <= segment.end
                                {
                                    draw_indicator(segment.start + radius, active, window);
                                }
                                if segment.outer_end
                                    && segment.end - radius - px(2.) >= segment.start
                                {
                                    draw_indicator(segment.end - radius, active, window);
                                }
                            }
                        }
                        if tick_count >= 2 {
                            // Skip ticks when they would visually merge instead of iterating every
                            // possible value on a fine-grained scale.
                            let spacing = f32::from(length) / f32::from(tick_count);
                            if spacing >= 6. {
                                for index in 1..tick_count {
                                    let offset =
                                        length * (f32::from(index) / f32::from(tick_count));
                                    if let Some(segment) = track_segments.iter().find(|segment| {
                                        segment.start + px(2.) <= offset
                                            && offset <= segment.end - px(2.)
                                    }) {
                                        let tint = if disabled_control {
                                            disabled_icon
                                        } else if segment.active {
                                            active_icon
                                        } else {
                                            inactive_icon
                                        };
                                        let on_endpoint = !segment.active
                                            && (segment.outer_start
                                                && (offset - segment.start - px(track_radius))
                                                    .abs()
                                                    < px(4.)
                                                || segment.outer_end
                                                    && (segment.end - px(track_radius) - offset)
                                                        .abs()
                                                        < px(4.));
                                        if !on_endpoint {
                                            draw_indicator(offset, tint, window);
                                        }
                                    }
                                }
                            }
                        }
                    },
                )
                .absolute()
                .inset_0()
                .size_full(),
            )
            .children(track_icon);
        let thumb = |which, position, name, focus: &FocusHandle| {
            let keyboard_state = self.state.clone();
            let handle_width = handle_width(which);
            let half_handle = handle_width / 2.;
            let (width, height) = if horizontal {
                (handle_width, px(thumb_extent))
            } else {
                (px(thumb_extent), handle_width)
            };
            let opacity = if which == RangeThumb::Start {
                start_label
            } else {
                end_label
            };
            let label = (opacity > 0.).then(|| {
                let text =
                    value_label(&self.state, which, cx, |value| format!("{value:.2}").into());
                let theme = window_material_theme(window, cx);
                let foreground = resolve(ColorRole::InverseOnSurface).opacity(opacity);
                let background = resolve(ColorRole::InverseSurface).opacity(opacity);
                let line = shape_text(
                    text,
                    TypeRole::LabelSmall.style(TypographyEmphasis::Baseline),
                    &theme.typography,
                    1.,
                    foreground,
                    window,
                    cx,
                )
                .expect("numeric slider label");
                canvas(
                    |_, _, _| (),
                    move |thumb, _, window, cx| {
                        let size = gpui::size(
                            line.line().width.ceil() + px(16.),
                            line.line_height() + px(8.),
                        );
                        let visible = window.content_mask().bounds;
                        if !visible.contains(&thumb.center()) {
                            return;
                        }
                        let origin = value_label_origin(
                            thumb,
                            size,
                            visible,
                            horizontal,
                            geometry.direction,
                        );
                        let bounds = Bounds::new(origin, size);
                        window.paint_quad(gpui::quad(
                            bounds,
                            px(6.),
                            background,
                            px(0.),
                            background,
                            gpui::BorderStyle::Solid,
                        ));
                        let _ = line.paint(
                            origin + point(px(8.), px(4.)),
                            gpui::TextAlign::Left,
                            None,
                            window,
                            cx,
                        );
                    },
                )
                .absolute()
                .inset_0()
                .size_full()
            });
            let element = geometry
                .place(
                    div()
                        .absolute()
                        .when(horizontal, |thumb| {
                            thumb
                                .top((px(target) - height) / 2.)
                                .ml(-half_handle)
                                .when(geometry.reversed(), |thumb| {
                                    thumb.ml(px(0.)).mr(-half_handle)
                                })
                        })
                        .when(!horizontal, |thumb| {
                            thumb.left((px(target) - width) / 2.).mb(-half_handle)
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
                .w(width)
                .h(height)
                .rounded_full()
                .bg(active)
                .focus_visible(move |style| style.bg(ring))
                .children(label);

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
        // The indicator records these bounds for Base input. Absolute thumbs are children
        // of the same unpadded coordinate space, so percentages use those bounds as well.
        let track_and_handles = div()
            .relative()
            .size_full()
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
        let root = div()
            .id(self.id.clone())
            .relative()
            .when(horizontal, |root| root.w_full().h(px(target)).px_2())
            .when(!horizontal, |root| root.w(px(target)).h(px(200.)).py_2())
            .child(track_and_handles);
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

        let touch = gesture.clone();
        let touch_state = self.state.clone();
        let touch_focus = [start_focus.clone(), end_focus.clone()];
        let touch_layer = canvas(
            |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
            move |_, hitbox, window, _| {
                let gesture = touch.clone();
                let state = touch_state.clone();
                let focus = touch_focus.clone();
                window.on_mouse_event(move |event: &TouchDragEvent, phase, window, cx| {
                    if !phase.bubble() {
                        return;
                    }
                    if event.phase == TouchPhase::Started {
                        if window.default_prevented()
                            || !hitbox.is_hovered_at(event.start_position, window)
                        {
                            return;
                        }
                        let bounds = state.read(cx).bounds();
                        if bounds.size.along(geometry.axis) <= px(0.) {
                            return;
                        }
                        let fraction = geometry.fraction(bounds, event.start_position);
                        let current = state.read(cx).percentage();
                        let which = if range && current.start == current.end {
                            choose_thumb(
                                true,
                                fraction,
                                fraction,
                                geometry.fraction(bounds, event.position),
                            )
                        } else {
                            choose_thumb(range, current.start, current.end, fraction)
                        };
                        window.prevent_default();
                        let mut gesture = gesture.borrow_mut();
                        gesture.origin = Some(event.start_position);
                        gesture.thumb = Some(which);
                        focus[usize::from(which == RangeThumb::End)].focus(window, cx);
                    }
                    let which = {
                        let gesture = gesture.borrow();
                        if gesture.origin != Some(event.start_position) || gesture.capture.is_some()
                        {
                            return;
                        }
                        let Some(which) = gesture.thumb else {
                            return;
                        };
                        which
                    };
                    match event.phase {
                        TouchPhase::Started | TouchPhase::Moved => {
                            update(&state, geometry, which, event.position, window, cx);
                        }
                        TouchPhase::Ended | TouchPhase::Cancelled => {
                            finish(&state, &gesture, window, cx, false);
                        }
                    }
                    cx.stop_propagation();
                });
            },
        )
        .absolute()
        .inset_0()
        .size_full();
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
        root.child(touch_layer)
            .on_mouse_down(MouseButton::Left, move |event, window, cx| {
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
                begin.borrow_mut().capture =
                    Some(DragCapture::watch(window, cx, move |window, cx| {
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

fn value_label_origin(
    thumb: Bounds<Pixels>,
    label: gpui::Size<Pixels>,
    visible: Bounds<Pixels>,
    horizontal: bool,
    direction: Direction,
) -> Point<Pixels> {
    let gap = px(8.);
    let x = if horizontal {
        thumb.center().x - label.width / 2.
    } else if direction == Direction::Ltr && thumb.right() + gap + label.width <= visible.right()
        || thumb.left() - gap - label.width < visible.left()
    {
        thumb.right() + gap
    } else {
        thumb.left() - gap - label.width
    };
    let y = if horizontal && thumb.top() - gap - label.height >= visible.top() {
        thumb.top() - gap - label.height
    } else if horizontal {
        thumb.bottom() + gap
    } else {
        thumb.center().y - label.height / 2.
    };
    point(
        x.clamp(
            visible.left(),
            (visible.right() - label.width).max(visible.left()),
        ),
        y.clamp(
            visible.top(),
            (visible.bottom() - label.height).max(visible.top()),
        ),
    )
}

#[derive(Clone, Copy)]
struct TrackSegment {
    start: Pixels,
    end: Pixels,
    outer_start: bool,
    outer_end: bool,
    active: bool,
}

impl TrackSegment {
    fn corners(self, outer: Pixels) -> (Pixels, Pixels) {
        let limit = (self.end - self.start).max(px(0.)) / 2.;
        let radius = |outside| if outside { outer } else { px(2.) }.min(limit);
        (radius(self.outer_start), radius(self.outer_end))
    }

    fn icon_position(self, side: Pixels, outer: Pixels) -> Option<(Pixels, bool)> {
        let (start, end) = self.corners(outer);
        let start = start.max(px(4.));
        let end = end.max(px(4.));
        (self.end - self.start >= start + side + end)
            .then_some((self.start + start + side / 2., self.active))
    }
}

fn track_segments(
    length: Pixels,
    (first, first_gap): (f32, Pixels),
    (last, last_gap): (f32, Pixels),
) -> [TrackSegment; 3] {
    let first = length * first;
    let last = length * last;
    [
        TrackSegment {
            start: px(0.),
            end: (first - first_gap).min(last - last_gap),
            outer_start: true,
            outer_end: false,
            active: false,
        },
        TrackSegment {
            start: first + first_gap,
            end: last - last_gap,
            outer_start: first_gap == px(0.),
            outer_end: last_gap == px(0.),
            active: true,
        },
        TrackSegment {
            start: (last + last_gap).max(first + first_gap),
            end: length,
            outer_start: false,
            outer_end: true,
            active: false,
        },
    ]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_segments_stop_the_same_distance_from_each_handle() {
        for percentage in [0.1, 0.5, 0.9] {
            let length = px(200.);
            let center = length * percentage;
            let [_, active, inactive] = track_segments(length, (0., px(0.)), (percentage, px(8.)));
            assert_eq!(center - active.end, px(8.));
            assert_eq!(inactive.start - center, px(8.));
            assert_eq!(active.corners(px(12.)).1, px(2.));
            assert_eq!(inactive.corners(px(12.)).0, px(2.));
        }

        let [before, active, after] = track_segments(px(200.), (0.5, px(7.)), (0.5, px(8.)));
        assert_eq!(before.end, px(92.));
        assert_eq!(after.start, px(108.));
        assert!(active.end < active.start);
    }
}
