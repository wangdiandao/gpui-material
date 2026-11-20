use std::{cell::RefCell, rc::Rc};

use gpui::{
    Along, AnyElement, App, AppContext, AvailableSpace, Axis, Bounds, DragMoveEvent, ElementId,
    Entity, FocusHandle, HitboxBehavior, Hsla, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Pixels, Point, Render, RenderOnce, Role, SharedString, Size,
    StatefulInteractiveElement, Styled, TouchDragEvent, TouchPhase, Window, canvas, div, point,
    prelude::FluentBuilder, px, relative,
};
use gpui_base::{
    AxisExt, TestSupportExt,
    slider::{Slider as BaseSlider, SliderIndicator},
};
use gpui_material_foundation::{
    DragCapture, InteractionSource, InteractionState, InteractionTracker, animate, color,
    current_material_theme, range_semantics, shape_text,
};
use gpui_material_theme::{ColorRole, Direction, TypeRole, TypographyEmphasis};

use crate::ControlSize;

pub use gpui_base::slider::{SliderEvent, SliderScale, SliderState, SliderValue};
pub use gpui_material_foundation::RangeThumb;

/// Optional track dimensions in logical pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SliderTrackStyle {
    pub thickness: Option<Pixels>,
    pub outer_radius: Option<Pixels>,
    pub inner_radius: Option<Pixels>,
    pub thumb_gap: Option<Pixels>,
}

/// Optional dimensions and colors for one handle.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SliderThumbStyle {
    /// The first dimension lies along the slider axis, the second across it.
    pub size: Option<Size<Pixels>>,
    /// The dimension along the slider axis while the handle is pressed.
    pub pressed_width: Option<Pixels>,
    pub color: Option<Hsla>,
    pub disabled_color: Option<Hsla>,
    pub focused_color: Option<Hsla>,
}

/// Optional slider colors. Unspecified fields use the window theme.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SliderColors {
    pub thumb: Option<Hsla>,
    pub active_track: Option<Hsla>,
    pub inactive_track: Option<Hsla>,
    pub active_tick: Option<Hsla>,
    pub inactive_tick: Option<Hsla>,
    pub disabled_thumb: Option<Hsla>,
    pub disabled_active_track: Option<Hsla>,
    pub disabled_inactive_track: Option<Hsla>,
    pub disabled_active_tick: Option<Hsla>,
    pub disabled_inactive_tick: Option<Hsla>,
}

/// The window position and default paint of a track tick or stop.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct SliderMarker {
    pub center: Point<Pixels>,
    pub diameter: Pixels,
    pub color: Hsla,
}

type DrawMarker = Box<dyn Fn(SliderMarker, &mut Window, &mut App)>;
type FormatValue = Box<dyn Fn(f32) -> SharedString>;

#[derive(Default)]
struct ThumbOptions {
    style: SliderThumbStyle,
    content: Option<AnyElement>,
    source: Option<Entity<InteractionSource>>,
}

fn thumb_index(which: RangeThumb) -> usize {
    usize::from(which == RangeThumb::End)
}

fn thumb_id(id: &ElementId, which: RangeThumb) -> ElementId {
    (
        id.clone(),
        if which == RangeThumb::Start {
            "start"
        } else {
            "end"
        },
    )
        .into()
}

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
    reverse_vertical: bool,
    size: ControlSize,
    disabled: bool,
    icon: Option<AnyElement>,
    tick_intervals: u16,
    range_end_name: Option<SharedString>,
    track_style: SliderTrackStyle,
    track: Option<AnyElement>,
    thumbs: [ThumbOptions; 2],
    colors: SliderColors,
    draw_tick: Option<DrawMarker>,
    draw_stop: Option<DrawMarker>,
    show_stop_indicators: bool,
    value_indicator: bool,
    format_value: Option<FormatValue>,
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
            reverse_vertical: false,
            size: ControlSize::ExtraSmall,
            disabled: false,
            icon: None,
            tick_intervals: 0,
            range_end_name: None,
            track_style: SliderTrackStyle::default(),
            track: None,
            thumbs: Default::default(),
            colors: SliderColors::default(),
            draw_tick: None,
            draw_stop: None,
            show_stop_indicators: true,
            value_indicator: true,
            format_value: None,
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

    /// Increases vertical sliders from top to bottom. Horizontal sliders follow the window direction.
    pub fn reverse_vertical(mut self, reverse: bool) -> Self {
        self.reverse_vertical = reverse;
        self
    }

    /// Selects the expressive track size. ExtraSmall (16 dp) is the default.
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

    /// Replaces the built-in track, icon, and marker artwork. The slider retains its input region.
    pub fn track(mut self, element: impl IntoElement) -> Self {
        self.track = Some(element.into_any_element());
        self
    }

    /// Sets the built-in track dimensions. Thickness also sizes a custom track's slot.
    pub fn track_style(mut self, style: SliderTrackStyle) -> Self {
        self.track_style = style;
        self
    }

    /// Replaces the handle artwork. Use `RangeThumb::End` for a single value.
    pub fn thumb(mut self, which: RangeThumb, element: impl IntoElement) -> Self {
        self.thumbs[thumb_index(which)].content = Some(element.into_any_element());
        self
    }

    /// Sets per-handle size and colors. Use `RangeThumb::End` for a single value.
    pub fn thumb_style(mut self, which: RangeThumb, style: SliderThumbStyle) -> Self {
        self.thumbs[thumb_index(which)].style = style;
        self
    }

    pub fn colors(mut self, colors: SliderColors) -> Self {
        self.colors = colors;
        self
    }

    /// Observes one handle under the slider ID paired with `"start"` or `"end"`.
    pub fn interaction_source(
        mut self,
        which: RangeThumb,
        source: &Entity<InteractionSource>,
    ) -> Self {
        self.thumbs[thumb_index(which)].source = Some(source.clone());
        self
    }

    /// Replaces the default dot painter for visual ticks.
    pub fn draw_ticks(
        mut self,
        draw: impl Fn(SliderMarker, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.draw_tick = Some(Box::new(draw));
        self
    }

    /// Replaces the default dot painter for track stops.
    pub fn draw_stop_indicators(
        mut self,
        draw: impl Fn(SliderMarker, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.draw_stop = Some(Box::new(draw));
        self
    }

    pub fn show_stop_indicators(mut self, show: bool) -> Self {
        self.show_stop_indicators = show;
        self
    }

    /// Enables the value indicator during dragging and keyboard focus. It is enabled by default.
    pub fn value_indicator(mut self, show: bool) -> Self {
        self.value_indicator = show;
        self
    }

    /// Formats the current value of the handle whose indicator is visible.
    pub fn format_value(mut self, format: impl Fn(f32) -> SharedString + 'static) -> Self {
        self.format_value = Some(Box::new(format));
        self
    }
}

#[derive(Default)]
struct Gesture {
    thumb: Option<RangeThumb>,
    origin: Option<Point<Pixels>>,
    capture: Option<DragCapture>,
    dragged: bool,
    disabled: bool,
    feedback: [ThumbFeedback; 2],
}

#[derive(Default)]
struct ThumbFeedback {
    focus: Option<FocusHandle>,
    tracker: Option<InteractionTracker>,
    hovered: bool,
}

impl Gesture {
    fn record(&self, window: &Window, cx: &mut App) {
        for (which, feedback) in [RangeThumb::Start, RangeThumb::End]
            .into_iter()
            .zip(&self.feedback)
        {
            let Some(tracker) = &feedback.tracker else {
                continue;
            };
            let mut snapshot = InteractionState::default();
            snapshot.disabled = self.disabled;
            if !self.disabled && window.is_window_active() {
                snapshot.hovered = feedback.hovered;
                snapshot.focus_visible = window.last_input_was_keyboard()
                    && feedback
                        .focus
                        .as_ref()
                        .is_some_and(|focus| focus.is_focused(window));
                snapshot.pressed = self.thumb == Some(which);
                snapshot.dragged = snapshot.pressed && self.dragged;
            }
            tracker.record(snapshot, cx);
        }
    }
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
    reverse_vertical: bool,
}

impl Geometry {
    fn reversed(self) -> bool {
        if self.axis.is_horizontal() {
            self.direction == Direction::Rtl
        } else {
            self.reverse_vertical
        }
    }

    fn position(self, bounds: Bounds<Pixels>, pointer: Point<Pixels>) -> Point<Pixels> {
        if !self.reversed() {
            pointer
        } else if self.axis.is_horizontal() {
            point(bounds.left() + (bounds.right() - pointer.x), pointer.y)
        } else {
            point(pointer.x, bounds.top() + (bounds.bottom() - pointer.y))
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
        if self.axis.is_vertical() && self.reversed() {
            point(bounds.center().x, bounds.top() + offset)
        } else if self.axis.is_vertical() {
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
            top_left: if self.axis.is_vertical() { right } else { left },
            top_right: right,
            bottom_left: left,
            bottom_right: if self.axis.is_vertical() { left } else { right },
        }
    }

    fn place(self, element: gpui::Div, position: f32) -> gpui::Div {
        if self.axis.is_vertical() && self.reversed() {
            element.top(relative(position))
        } else if self.axis.is_vertical() {
            element.bottom(relative(position))
        } else if self.reversed() {
            element.right(relative(position))
        } else {
            element.left(relative(position))
        }
    }

    fn reversed_key_delta(self, key: &str) -> Option<f32> {
        if !self.reversed() {
            return None;
        }
        match (self.axis, key) {
            (Axis::Horizontal, "left") | (Axis::Vertical, "down") => Some(1.),
            (Axis::Horizontal, "right") | (Axis::Vertical, "up") => Some(-1.),
            _ => None,
        }
    }

    fn choose_at(
        self,
        range: bool,
        values: std::ops::Range<f32>,
        handle_sizes: [Size<Pixels>; 2],
        bounds: Bounds<Pixels>,
        pointer: Point<Pixels>,
    ) -> RangeThumb {
        let fraction = self.fraction(bounds, pointer);
        if range && values.start != values.end {
            let hit = |position, handle: Size<Pixels>| {
                let center = self.point_at(bounds, bounds.size.along(self.axis) * position);
                let offset = pointer - center;
                let (along, across) = if self.axis.is_horizontal() {
                    (offset.x, offset.y)
                } else {
                    (offset.y, offset.x)
                };
                along.abs() <= handle.width / 2. && across.abs() <= handle.height / 2.
            };
            match (
                hit(values.start, handle_sizes[0]),
                hit(values.end, handle_sizes[1]),
            ) {
                (true, false) => return RangeThumb::Start,
                (false, true) => return RangeThumb::End,
                _ => {}
            }
        }
        choose_thumb(range, values.start, values.end, fraction)
    }
}

fn dimension(value: Option<Pixels>, default: f32, zero_allowed: bool) -> Pixels {
    value
        .filter(|value| {
            f32::from(*value).is_finite() && (*value > px(0.) || zero_allowed && *value == px(0.))
        })
        .unwrap_or(px(default))
}

impl RenderOnce for Slider {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
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
            direction: current_material_theme(window, cx).layout.direction,
            reverse_vertical: self.reverse_vertical,
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
        let focus_handles = [&start_focus, &end_focus];
        {
            let mut gesture = gesture.borrow_mut();
            gesture.disabled = self.disabled;
            for which in [RangeThumb::Start, RangeThumb::End] {
                let index = thumb_index(which);
                let feedback = &mut gesture.feedback[index];
                feedback.focus = Some(focus_handles[index].clone());
                feedback.tracker = self.thumbs[index]
                    .source
                    .as_ref()
                    .filter(|_| range || which == RangeThumb::End)
                    .map(|source| {
                        InteractionSource::track(source, &thumb_id(&self.id, which), window, cx)
                    });
                if self.disabled
                    || !window.is_window_active()
                    || !range && which == RangeThumb::Start
                {
                    feedback.hovered = false;
                }
            }
            gesture.record(window, cx);
        }
        // Material slider specs, expressive track and handle sizes.
        let (thickness, thumb_extent, track_radius, icon_size): (f32, f32, f32, Option<f32>) =
            match self.size {
                ControlSize::ExtraSmall => (16., 44., 8., None),
                ControlSize::Small => (24., 44., 8., None),
                ControlSize::Medium => (40., 52., 12., Some(24.)),
                ControlSize::Large => (56., 68., 16., Some(24.)),
                ControlSize::ExtraLarge => (96., 108., 28., Some(32.)),
            };
        let [start_content, end_content] = self.thumbs.each_mut().map(|thumb| thumb.content.take());
        let thickness = dimension(self.track_style.thickness, thickness, false);
        let track_radius =
            dimension(self.track_style.outer_radius, track_radius, true).min(thickness / 2.);
        let inner_radius = dimension(self.track_style.inner_radius, 2., true).min(thickness / 2.);
        let thumb_gap = dimension(self.track_style.thumb_gap, 6., true);
        let cross_extent = |which| {
            dimension(
                self.thumbs[thumb_index(which)]
                    .style
                    .size
                    .map(|size| size.height),
                thumb_extent,
                false,
            )
        };
        let target = thickness.max(cross_extent(RangeThumb::End)).max(px(48.));
        let target = if range {
            target.max(cross_extent(RangeThumb::Start))
        } else {
            target
        };
        let colors = current_material_theme(window, cx).colors.clone();
        let resolve = |role| color(colors.color(role));
        let disabled = resolve(ColorRole::OnSurface);
        let (active, inactive, active_tick, inactive_tick, thumb_color) = if self.disabled {
            (
                self.colors
                    .disabled_active_track
                    .unwrap_or_else(|| disabled.opacity(0.38)),
                self.colors
                    .disabled_inactive_track
                    .unwrap_or_else(|| disabled.opacity(0.12)),
                self.colors
                    .disabled_active_tick
                    .unwrap_or_else(|| disabled.opacity(0.38)),
                self.colors
                    .disabled_inactive_tick
                    .unwrap_or_else(|| disabled.opacity(0.38)),
                self.colors
                    .disabled_thumb
                    .unwrap_or_else(|| disabled.opacity(0.38)),
            )
        } else {
            (
                self.colors
                    .active_track
                    .unwrap_or_else(|| resolve(ColorRole::Primary)),
                self.colors
                    .inactive_track
                    .unwrap_or_else(|| resolve(ColorRole::SecondaryContainer)),
                self.colors
                    .active_tick
                    .unwrap_or_else(|| resolve(ColorRole::OnPrimary)),
                self.colors
                    .inactive_tick
                    .unwrap_or_else(|| resolve(ColorRole::OnSecondaryContainer)),
                self.colors
                    .thumb
                    .unwrap_or_else(|| resolve(ColorRole::Primary)),
            )
        };
        let ring = resolve(ColorRole::Secondary);

        let active_icon = resolve(ColorRole::OnPrimary);
        let inactive_icon = resolve(ColorRole::OnSecondaryContainer);
        let disabled_icon = resolve(ColorRole::OnSurface).opacity(0.38);
        let disabled_control = self.disabled;
        let dragging = gesture
            .borrow()
            .thumb
            .filter(|_| !self.disabled && window.is_window_active());
        let reduced = cx.reduce_motion();
        let scheme = current_material_theme(window, cx).motion;
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
        let rest_width = |which| {
            dimension(
                self.thumbs[thumb_index(which)]
                    .style
                    .size
                    .map(|size| size.width),
                4.,
                false,
            )
        };
        let pressed_width = |which| {
            dimension(
                self.thumbs[thumb_index(which)].style.pressed_width,
                2.,
                false,
            )
        };
        let width = |which, window: &mut Window, cx: &mut App| {
            let rest = f32::from(rest_width(which));
            let pressed = f32::from(pressed_width(which));
            px(animate(
                (
                    self.id.clone(),
                    if which == RangeThumb::Start {
                        "start-width"
                    } else {
                        "end-width"
                    },
                ),
                if dragging == Some(which) {
                    pressed
                } else {
                    rest
                },
                resize,
                window,
                cx,
            )
            .clamp(rest.min(pressed), rest.max(pressed)))
        };
        let start_width = if range {
            width(RangeThumb::Start, window, cx)
        } else {
            px(4.)
        };
        let end_width = width(RangeThumb::End, window, cx);
        let handle_sizes = [
            gpui::size(start_width, cross_extent(RangeThumb::Start)),
            gpui::size(end_width, cross_extent(RangeThumb::End)),
        ];
        let handle_width = |which| {
            if which == RangeThumb::Start {
                start_width
            } else {
                end_width
            }
        };
        let axis_padding = |which| rest_width(which).max(pressed_width(which)) / 2.;
        let padding = px(8.).max(axis_padding(RangeThumb::End));
        let padding = if range {
            padding.max(axis_padding(RangeThumb::Start))
        } else {
            padding
        };
        let label_opacity = |which, focus: &FocusHandle, window: &mut Window, cx: &mut App| {
            if !self.value_indicator || self.disabled {
                return 0.;
            }
            let visible = window.is_window_active()
                && (dragging == Some(which)
                    || dragging.is_none()
                        && focus.is_focused(window)
                        && window.last_input_was_keyboard());
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
        let end_gap = thumb_gap + handle_width(RangeThumb::End) / 2.;
        let center_gap = if range {
            thumb_gap + handle_width(RangeThumb::Start) / 2.
        } else if centered {
            thumb_gap
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
                    let (offset, active) = segments(length).into_iter().find_map(|segment| {
                        segment.icon_position(side, track_radius, inner_radius)
                    })?;
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
        let track_artwork = if let Some(custom) = self.track.take() {
            div()
                .absolute()
                .when(horizontal, |track| {
                    track.w_full().h(thickness).top((target - thickness) / 2.)
                })
                .when(!horizontal, |track| {
                    track.h_full().w(thickness).left((target - thickness) / 2.)
                })
                .child(custom)
                .into_any_element()
        } else {
            let show_stops = self.show_stop_indicators;
            let draw_tick = self.draw_tick.take();
            let draw_stop = self.draw_stop.take();
            div()
                .absolute()
                .inset_0()
                .child(
                    canvas(
                        |_, _, _| (),
                        move |bounds, _, window, cx| {
                            let length = bounds.size.along(geometry.axis);
                            let track_segments = segments(length);
                            let stop_offset = track_radius.max(px(2.));
                            let marker = |offset, color| SliderMarker {
                                center: geometry.point_at(bounds, offset),
                                diameter: px(4.),
                                color,
                            };
                            for segment in track_segments {
                                if segment.end <= segment.start {
                                    continue;
                                }
                                let a = geometry.point_at(bounds, segment.start);
                                let b = geometry.point_at(bounds, segment.end);
                                let half = thickness / 2.;
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
                                let (start_corner, end_corner) =
                                    segment.corners(track_radius, inner_radius);
                                window.paint_quad(gpui::quad(
                                    rect,
                                    geometry.track_corners(start_corner, end_corner),
                                    tint,
                                    px(0.),
                                    gpui::transparent_black(),
                                    gpui::BorderStyle::Solid,
                                ));
                                // The stop indicator marks the outer edge of each inactive segment.
                                if show_stops && !segment.active {
                                    if segment.outer_start
                                        && segment.start + stop_offset + px(2.) <= segment.end
                                    {
                                        draw_marker(
                                            draw_stop.as_ref(),
                                            marker(segment.start + stop_offset, active),
                                            window,
                                            cx,
                                        );
                                    }
                                    if segment.outer_end
                                        && segment.end - stop_offset - px(2.) >= segment.start
                                    {
                                        draw_marker(
                                            draw_stop.as_ref(),
                                            marker(segment.end - stop_offset, active),
                                            window,
                                            cx,
                                        );
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
                                        if let Some(segment) =
                                            track_segments.iter().find(|segment| {
                                                segment.start + px(2.) <= offset
                                                    && offset <= segment.end - px(2.)
                                            })
                                        {
                                            let tint = if segment.active {
                                                active_tick
                                            } else {
                                                inactive_tick
                                            };
                                            let on_endpoint = show_stops
                                                && !segment.active
                                                && (segment.outer_start
                                                    && (offset - segment.start - stop_offset)
                                                        .abs()
                                                        < px(4.)
                                                    || segment.outer_end
                                                        && (segment.end - stop_offset - offset)
                                                            .abs()
                                                            < px(4.));
                                            if !on_endpoint {
                                                draw_marker(
                                                    draw_tick.as_ref(),
                                                    marker(offset, tint),
                                                    window,
                                                    cx,
                                                );
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
                .children(track_icon)
                .into_any_element()
        };
        let track = SliderIndicator::new(&self.state)
            .relative()
            .size_full()
            .child(track_artwork);
        let thumb = |which, position, name, focus: &FocusHandle, content: Option<AnyElement>| {
            let keyboard_state = self.state.clone();
            let handle_width = handle_width(which);
            let half_handle = handle_width / 2.;
            let cross = cross_extent(which);
            let (width, height) = if horizontal {
                (handle_width, cross)
            } else {
                (cross, handle_width)
            };
            let style = self.thumbs[thumb_index(which)].style;
            let tint = if self.disabled {
                style.disabled_color
            } else {
                style.color
            }
            .unwrap_or(thumb_color);
            let focus_color = style.focused_color.unwrap_or(ring);
            let opacity = if which == RangeThumb::Start {
                start_label
            } else {
                end_label
            };
            let label = (opacity > 0.)
                .then(|| {
                    let numeric_value = if which == RangeThumb::Start {
                        value.start()
                    } else {
                        value.end()
                    };
                    let text = self.format_value.as_ref().map_or_else(
                        || format!("{numeric_value:.2}").into(),
                        |format| format(numeric_value),
                    );
                    let theme = current_material_theme(window, cx);
                    let foreground = resolve(ColorRole::InverseOnSurface).opacity(opacity);
                    let background = resolve(ColorRole::InverseSurface).opacity(opacity);
                    let line = shape_text(
                        text,
                        TypeRole::LabelLarge.style(TypographyEmphasis::Baseline),
                        &theme.typography,
                        1.,
                        foreground,
                        window,
                        cx,
                    )?;
                    Some(
                        canvas(
                            |_, _, _| (),
                            move |thumb, _, window, cx| {
                                let size = gpui::size(
                                    (line.line().width.ceil() + px(16.)).max(px(48.)),
                                    line.line_height().max(px(44.)),
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
                                    origin
                                        + point(
                                            (size.width - line.line().width) / 2.,
                                            (size.height - line.line_height()) / 2.,
                                        ),
                                    gpui::TextAlign::Left,
                                    None,
                                    window,
                                    cx,
                                );
                            },
                        )
                        .absolute()
                        .inset_0()
                        .size_full(),
                    )
                })
                .flatten();
            let custom = content.is_some();
            let element = geometry
                .place(
                    div()
                        .absolute()
                        .when(horizontal, |thumb| {
                            thumb
                                .top((target - height) / 2.)
                                .ml(-half_handle)
                                .when(geometry.reversed(), |thumb| {
                                    thumb.ml(px(0.)).mr(-half_handle)
                                })
                        })
                        .when(!horizontal, |thumb| {
                            thumb.left((target - width) / 2.).when_else(
                                geometry.reversed(),
                                |thumb| thumb.mt(-half_handle),
                                |thumb| thumb.mb(-half_handle),
                            )
                        }),
                    position,
                )
                .id(thumb_id(&self.id, which))
                .when(!self.disabled, |thumb| thumb.role(Role::Slider))
                .when(!self.disabled, |thumb| thumb.aria_label(name))
                .w(width)
                .h(height)
                .when_else(
                    custom,
                    |thumb| {
                        thumb
                            .text_color(tint)
                            .focus_visible(move |style| style.border_1().border_color(focus_color))
                            .children(content.map(|art| div().size_full().child(art)))
                    },
                    |thumb| {
                        thumb
                            .rounded_full()
                            .bg(tint)
                            .focus_visible(move |style| style.bg(focus_color))
                    },
                )
                .children(label);

            let element = if self.disabled {
                element
            } else {
                let hover = gesture.clone();
                range_semantics(element, &self.state, which, self.axis, cx)
                    .capture_key_down(move |event, window, cx| {
                        let Some(sign) = geometry.reversed_key_delta(&event.keystroke.key) else {
                            return;
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
                    })
                    .on_hover(move |hovered, window, cx| {
                        let mut hover = hover.borrow_mut();
                        hover.feedback[thumb_index(which)].hovered = *hovered;
                        hover.record(window, cx);
                    })
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
                    start_content,
                ))
            })
            .child(thumb(
                RangeThumb::End,
                end,
                self.range_end_name.unwrap_or(self.name),
                &end_focus,
                end_content,
            ));
        let root = div()
            .id(self.id.clone())
            .relative()
            .when(horizontal, |root| root.w_full().h(target).px(padding))
            .when(!horizontal, |root| root.w(target).h(px(200.)).py(padding))
            .child(track_and_handles);
        let active_thumb = gesture.borrow().thumb;
        let inactive = !window.is_window_active();
        if active_thumb
            .is_some_and(|which| self.disabled || inactive || !range && which == RangeThumb::Start)
        {
            let state = self.state.clone();
            let gesture = gesture.clone();
            window.defer(cx, move |window, cx| {
                finish(&state, &gesture, window, cx, true)
            });
        }
        if self.disabled || inactive && active_thumb.is_some() {
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
                        let current = state.read(cx).percentage();
                        let which = if range && current.start == current.end {
                            let fraction = geometry.fraction(bounds, event.start_position);
                            choose_thumb(
                                true,
                                fraction,
                                fraction,
                                geometry.fraction(bounds, event.position),
                            )
                        } else {
                            geometry.choose_at(
                                range,
                                current,
                                handle_sizes,
                                bounds,
                                event.start_position,
                            )
                        };
                        window.prevent_default();
                        {
                            let mut gesture = gesture.borrow_mut();
                            gesture.origin = Some(event.start_position);
                            gesture.thumb = Some(which);
                        }
                        focus[thumb_index(which)].focus(window, cx);
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
                            {
                                let mut gesture = gesture.borrow_mut();
                                gesture.dragged = event.position != event.start_position;
                                gesture.record(window, cx);
                            }
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
                let which = geometry.choose_at(
                    range,
                    down_state.read(cx).percentage(),
                    handle_sizes,
                    bounds,
                    event.position,
                );
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
                down.borrow().record(window, cx);
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
                {
                    let mut begin = begin.borrow_mut();
                    begin.dragged = true;
                    begin.record(window, cx);
                }
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
                    drag_focus[thumb_index(which)].focus(window, cx);
                }
                gesture.record(window, cx);
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

fn draw_marker(draw: Option<&DrawMarker>, marker: SliderMarker, window: &mut Window, cx: &mut App) {
    if let Some(draw) = draw {
        draw(marker, window, cx);
    } else {
        let radius = marker.diameter / 2.;
        let offset = point(radius, radius);
        window.paint_quad(gpui::quad(
            Bounds::from_corners(marker.center - offset, marker.center + offset),
            radius,
            marker.color,
            px(0.),
            gpui::transparent_black(),
            gpui::BorderStyle::Solid,
        ));
    }
}

fn value_label_origin(
    thumb: Bounds<Pixels>,
    label: gpui::Size<Pixels>,
    visible: Bounds<Pixels>,
    horizontal: bool,
    direction: Direction,
) -> Point<Pixels> {
    let gap = px(12.);
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
    fn corners(self, outer: Pixels, inner: Pixels) -> (Pixels, Pixels) {
        let limit = (self.end - self.start).max(px(0.)) / 2.;
        let radius = |outside| if outside { outer } else { inner }.min(limit);
        (radius(self.outer_start), radius(self.outer_end))
    }

    fn icon_position(self, side: Pixels, outer: Pixels, inner: Pixels) -> Option<(Pixels, bool)> {
        let (start, end) = self.corners(outer, inner);
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
            outer_start: first == px(0.) && first_gap == px(0.),
            outer_end: last == length && last_gap == px(0.),
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
        let bounds = state.bounds();
        if bounds.size.along(geometry.axis) <= px(0.) {
            return;
        }
        let position = geometry.position(bounds, pointer);
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
    let capture = {
        let mut gesture = gesture.borrow_mut();
        if gesture.thumb.take().is_none() {
            return;
        }
        gesture.origin = None;
        gesture.dragged = false;
        gesture.record(window, cx);
        gesture.capture.take()
    };
    if let Some(capture) = capture
        && cancel
    {
        capture.cancel(window, cx);
    }
    state.update(cx, |state, cx| state.handle_release(cx));
    window.refresh();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_labels_keep_material_gap_and_flip_inside_the_viewport() {
        let visible = Bounds::new(point(px(0.), px(0.)), gpui::size(px(200.), px(200.)));
        let label = gpui::size(px(48.), px(44.));
        let thumb = Bounds::new(point(px(98.), px(100.)), gpui::size(px(4.), px(44.)));
        let above = value_label_origin(thumb, label, visible, true, Direction::Ltr);
        assert_eq!(above, point(px(76.), px(44.)));
        assert_eq!(thumb.top() - above.y - label.height, px(12.));

        let top = Bounds::new(point(px(0.), px(0.)), thumb.size);
        let below = value_label_origin(top, label, visible, true, Direction::Rtl);
        assert_eq!(below, point(px(0.), px(56.)));
        let vertical = value_label_origin(thumb, label, visible, false, Direction::Rtl);
        assert_eq!(vertical.x + label.width + px(12.), thumb.left());
    }

    #[test]
    fn track_segments_stop_the_same_distance_from_each_handle() {
        for percentage in [0.1, 0.5, 0.9] {
            let length = px(200.);
            let center = length * percentage;
            let [_, active, inactive] = track_segments(length, (0., px(0.)), (percentage, px(8.)));
            assert_eq!(center - active.end, px(8.));
            assert_eq!(inactive.start - center, px(8.));
            assert_eq!(active.corners(px(12.), px(2.)).1, px(2.));
            assert_eq!(inactive.corners(px(12.), px(2.)).0, px(2.));
        }

        let [before, active, after] = track_segments(px(200.), (0.5, px(7.)), (0.5, px(8.)));
        assert_eq!(before.end, px(92.));
        assert_eq!(after.start, px(108.));
        assert!(active.end < active.start);
    }
}
