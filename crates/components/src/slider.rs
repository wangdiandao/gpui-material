mod geometry;
mod gesture;
mod layout;
mod palette;
mod render;
mod thumb;
mod track;

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
    /// Cross-axis thickness of the slider track.
    pub thickness: Option<Pixels>,
    /// Corner radius at the outer ends of the track.
    pub outer_radius: Option<Pixels>,
    /// Corner radius at the inner track edges adjacent to the thumb gap.
    pub inner_radius: Option<Pixels>,
    /// Gap between the thumb handle and the active/inactive track segments.
    pub thumb_gap: Option<Pixels>,
}

/// Optional dimensions and colors for one handle.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SliderThumbStyle {
    /// The first dimension lies along the slider axis, the second across it.
    pub size: Option<Size<Pixels>>,
    /// The dimension along the slider axis while the handle is pressed.
    pub pressed_width: Option<Pixels>,
    /// Handle fill color when enabled.
    pub color: Option<Hsla>,
    /// Handle fill color when disabled.
    pub disabled_color: Option<Hsla>,
    /// Handle fill color when keyboard focus is visible.
    pub focused_color: Option<Hsla>,
}

/// Optional slider colors. Unspecified fields use the window theme.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SliderColors {
    /// Enabled handle fill color.
    pub thumb: Option<Hsla>,
    /// Enabled active track fill color.
    pub active_track: Option<Hsla>,
    /// Enabled inactive track fill color.
    pub inactive_track: Option<Hsla>,
    /// Enabled tick and stop marker color on the active track segment.
    pub active_tick: Option<Hsla>,
    /// Enabled tick and stop marker color on the inactive track segment.
    pub inactive_tick: Option<Hsla>,
    /// Disabled handle fill color.
    pub disabled_thumb: Option<Hsla>,
    /// Disabled active track fill color.
    pub disabled_active_track: Option<Hsla>,
    /// Disabled inactive track fill color.
    pub disabled_inactive_track: Option<Hsla>,
    /// Disabled tick and stop marker color on the active track segment.
    pub disabled_active_tick: Option<Hsla>,
    /// Disabled tick and stop marker color on the inactive track segment.
    pub disabled_inactive_tick: Option<Hsla>,
}

/// The window position and default paint of a track tick or stop.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct SliderMarker {
    /// Center point of the marker in window coordinates.
    pub center: Point<Pixels>,
    /// Default marker diameter in logical pixels.
    pub diameter: Pixels,
    /// Resolved marker color for the active or inactive track segment.
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
    source_key: Option<ElementId>,
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

/// Origin of the active track fill for a single-value [`Slider`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SliderKind {
    /// Active track extends from the start of the range to the handle (default).
    #[default]
    Standard,
    /// Active track extends symmetrically from the midpoint of the range to the handle.
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
    /// Creates a horizontal single-value slider bound to `state` with an element ID and accessible name.
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

    /// Selects standard start-anchored or centered active track geometry.
    pub fn kind(mut self, kind: SliderKind) -> Self {
        self.kind = kind;
        self
    }

    /// Sets the slider orientation (`Axis::Horizontal` by default).
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

    /// Disables pointer dragging, keyboard adjustment, and focus traversal when `true`.
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

    /// Places an optional decorative icon inside the active track on medium and larger sizes.
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

    /// Replaces the handle artwork of a single-value slider (`RangeThumb::End`).
    pub fn single_thumb(self, element: impl IntoElement) -> Self {
        self.thumb(RangeThumb::End, element)
    }

    /// Sets per-handle size and colors. Use `RangeThumb::End` for a single value.
    pub fn thumb_style(mut self, which: RangeThumb, style: SliderThumbStyle) -> Self {
        self.thumbs[thumb_index(which)].style = style;
        self
    }

    /// Sets the handle size and colors of a single-value slider (`RangeThumb::End`).
    pub fn single_thumb_style(self, style: SliderThumbStyle) -> Self {
        self.thumb_style(RangeThumb::End, style)
    }

    /// Overrides enabled and disabled track, thumb, and tick colors.
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

    /// Uses the caller's focus handle for a single-value slider (`RangeThumb::End`).
    pub fn single_track_focus(self, focus: &FocusHandle) -> Self {
        self.track_focus(RangeThumb::End, focus)
    }

    /// Observes one handle under the slider ID paired with `"start"` or `"end"`.
    pub fn interaction_source(
        mut self,
        which: RangeThumb,
        source: &Entity<InteractionSource>,
    ) -> Self {
        let entry = &mut self.thumbs[thumb_index(which)];
        entry.source = Some(source.clone());
        entry.source_key = None;
        self
    }

    /// Observes the handle of a single-value slider (`RangeThumb::End`).
    pub fn single_interaction_source(self, source: &Entity<InteractionSource>) -> Self {
        self.interaction_source(RangeThumb::End, source)
    }

    #[cfg(feature = "surfaces")]
    pub(crate) fn interaction_source_with_id(
        mut self,
        which: RangeThumb,
        id: ElementId,
        source: &Entity<InteractionSource>,
    ) -> Self {
        let entry = &mut self.thumbs[thumb_index(which)];
        entry.source = Some(source.clone());
        entry.source_key = Some(id);
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

    /// Controls whether endpoint stop dots are drawn on the track. Defaults to `true`.
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
