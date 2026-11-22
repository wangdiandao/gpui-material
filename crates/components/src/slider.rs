mod geometry;
mod render;

use gpui::{
    AnyElement, App, Axis, ElementId, Entity, FocusHandle, Hsla, IntoElement, Pixels, Point,
    SharedString, Size, Window,
};
use gpui_material_foundation::InteractionSource;

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
    focus: Option<FocusHandle>,
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

    /// Uses the caller's focus handle for one handle instead of an internal keyed handle.
    /// Use `RangeThumb::End` for a single-value slider.
    pub fn track_focus(mut self, which: RangeThumb, focus: &FocusHandle) -> Self {
        self.thumbs[thumb_index(which)].focus = Some(focus.clone());
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
