use std::rc::Rc;

use gpui::{
    App, ElementId, Entity, FocusHandle, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, div,
};
use gpui_material_foundation::{InteractionSource, color, current_material_theme};
use gpui_material_theme::{ColorRole, TypeRole};

use crate::{
    ListItem, ListItemColors, ListVariant, RangeThumb, Slider, SliderState, SliderValue, Text,
};

use super::common::{TileFields, apply_standalone_variant};

type FormatValue = Rc<dyn Fn(f32) -> SharedString>;
type SliderConfig = Box<dyn FnOnce(Slider) -> Slider>;

fn format_slider_scalar(value: f32) -> SharedString {
    if !value.is_finite() {
        return "0".into();
    }
    let rounded = value.round();
    if (value - rounded).abs() < 1e-4 {
        format!("{}", rounded as i64).into()
    } else {
        format!("{value:.2}").into()
    }
}

fn format_slider_value(value: SliderValue, formatter: Option<&FormatValue>) -> SharedString {
    let format_one = |v: f32| match formatter {
        Some(f) => f(v),
        None => format_slider_scalar(v),
    };
    if value.is_range() {
        format!(
            "{} – {}",
            format_one(value.start()),
            format_one(value.end())
        )
        .into()
    } else {
        format_one(value.end())
    }
}

/// Settings convenience row combining a [`ListItem`] header, live formatted value display,
/// and an embedded [`Slider`] bound to a caller-owned [`SliderState`].
#[derive(IntoElement)]
pub struct SliderTile {
    fields: TileFields,
    state: Entity<SliderState>,
    format_value: Option<FormatValue>,
    configure_slider: Option<SliderConfig>,
}

impl SliderTile {
    /// Creates a slider settings row with an element ID, headline label, and caller-owned [`SliderState`].
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<SliderState>,
    ) -> Self {
        Self {
            fields: TileFields::new(id, label),
            state: state.clone(),
            format_value: None,
            configure_slider: None,
        }
    }

    /// Sets secondary supporting text displayed below the headline.
    pub fn supporting(mut self, text: impl Into<SharedString>) -> Self {
        self.fields.supporting = Some(text.into());
        self
    }

    /// Sets overline text displayed above the headline.
    pub fn overline(mut self, text: impl Into<SharedString>) -> Self {
        self.fields.overline = Some(text.into());
        self
    }

    /// Sets an optional leading icon or graphic.
    pub fn leading(mut self, leading: impl IntoElement) -> Self {
        self.fields.leading = Some(leading.into_any_element());
        self
    }

    /// Disables the embedded slider and dims the tile when `true`.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.fields.disabled = disabled;
        self
    }

    /// Sets standalone row styling (`Standard` or `Segmented`).
    pub fn variant(mut self, variant: ListVariant) -> Self {
        self.fields.variant = variant;
        self
    }

    /// Overrides container and slot colors on the underlying [`ListItem`].
    pub fn colors(mut self, colors: ListItemColors) -> Self {
        self.fields.colors = colors;
        self
    }

    /// Binds a caller-owned focus handle to the embedded slider's primary handle.
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.fields.focus = Some(focus.clone());
        self
    }

    /// Observes the embedded slider's primary handle under the tile's ID (and `(id, "start")` for range sliders).
    pub fn interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
        self.fields.source = Some(source.clone());
        self
    }

    /// Formats the live numeric value displayed on the trailing side of the header row.
    pub fn format_value(mut self, formatter: impl Fn(f32) -> SharedString + 'static) -> Self {
        self.format_value = Some(Rc::new(formatter));
        self
    }

    /// Customizes the embedded [`Slider`] (colors, tick intervals, size, value indicator, etc.).
    pub fn slider(mut self, configure: impl FnOnce(Slider) -> Slider + 'static) -> Self {
        self.configure_slider = Some(Box::new(configure));
        self
    }

    pub(super) fn into_list_item(mut self, window: &Window, cx: &App) -> ListItem {
        let theme = current_material_theme(window, cx);
        let value_color = if self.fields.disabled {
            color(theme.colors.color(ColorRole::OnSurface)).opacity(0.38)
        } else {
            color(theme.colors.color(ColorRole::OnSurfaceVariant))
        };
        let current = self.state.read(cx).value();
        let value_text = format_slider_value(current, self.format_value.as_ref());

        let focus = self.fields.focus.take();
        let source = self.fields.source.take();
        let mut slider = Slider::new(
            (self.fields.id.clone(), "slider"),
            self.fields.label.clone(),
            &self.state,
        )
        .disabled(self.fields.disabled);
        if let Some(focus) = focus {
            slider = slider.track_focus(RangeThumb::End, &focus);
        }
        if let Some(source) = source {
            slider =
                slider.interaction_source_with_id(RangeThumb::End, self.fields.id.clone(), &source);
            if current.is_range() {
                slider = slider.interaction_source_with_id(
                    RangeThumb::Start,
                    (self.fields.id.clone(), "start").into(),
                    &source,
                );
            }
        }
        if let Some(formatter) = self.format_value {
            slider = slider.format_value(move |value| formatter(value));
        }
        if let Some(configure) = self.configure_slider {
            slider = configure(slider);
        }

        let trailing = div().text_color(value_color).child(
            Text::new(value_text)
                .id((self.fields.id.clone(), "slider-value"))
                .role(TypeRole::BodyMedium),
        );

        self.fields
            .into_list_item()
            .trailing(trailing)
            .footer(slider)
    }
}

impl RenderOnce for SliderTile {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let variant = self.fields.variant;
        apply_standalone_variant(self.into_list_item(window, cx), variant)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slider_tile_formats_integers_decimals_and_ranges_cleanly() {
        assert_eq!(format_slider_scalar(42.0), "42");
        assert_eq!(format_slider_scalar(3.25), "3.25");
        assert_eq!(format_slider_scalar(f32::NAN), "0");
        assert_eq!(format_slider_value(SliderValue::Single(75.0), None), "75");
        assert_eq!(
            format_slider_value(SliderValue::Range(10.0, 80.5), None),
            "10 – 80.50"
        );
    }
}
