use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, Pixels, RenderOnce,
    Role, SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
    px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{
    ShapePolygon, Surface, color, current_material_theme, wrapped_text,
};
use gpui_material_theme::{
    ColorRole, CornerShape, Direction, MaterialShape, TypeRole, TypographyEmphasis,
};

const DEFAULT_MAX_WIDTH: Pixels = px(360.);
const ICON_CONTAINER_SIZE: Pixels = px(72.);
const ICON_SIZE: Pixels = px(32.);
const ACTION_GAP: Pixels = px(12.);
const ACTIONS_TOP_MARGIN: Pixels = px(24.);

enum EmptyStateVisual {
    Icon(AnyElement),
    Custom(AnyElement),
}

enum SupportingSlot {
    Text(SharedString),
    Custom(AnyElement),
}

enum IconContainerShape {
    Polygon(ShapePolygon),
    Corners(CornerShape),
}

/// A purely presentational Material 3 Expressive empty-state layout shell.
///
/// Displays an optional decorative icon backdrop (or custom visual), title, optional
/// supporting text or content, and optional centered primary/secondary actions.
#[derive(IntoElement)]
pub struct EmptyState {
    id: ElementId,
    title: SharedString,
    title_role: TypeRole,
    supporting: Option<SupportingSlot>,
    visual: Option<EmptyStateVisual>,
    icon_container_shape: IconContainerShape,
    icon_container_role: ColorRole,
    icon_content_role: ColorRole,
    primary_action: Option<AnyElement>,
    secondary_action: Option<AnyElement>,
    max_width: Pixels,
}

impl EmptyState {
    /// Creates an empty-state layout with a stable element ID and required title.
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            title_role: TypeRole::TitleLarge,
            supporting: None,
            visual: None,
            icon_container_shape: IconContainerShape::Polygon(MaterialShape::Cookie7Sided.into()),
            icon_container_role: ColorRole::SecondaryContainer,
            icon_content_role: ColorRole::OnSecondaryContainer,
            primary_action: None,
            secondary_action: None,
            max_width: DEFAULT_MAX_WIDTH,
        }
    }

    /// Sets wrapped `BodyMedium` supporting copy below the title.
    pub fn supporting(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting = Some(SupportingSlot::Text(text.into()));
        self
    }

    /// Replaces the supporting text slot with arbitrary visual content.
    pub fn supporting_content(mut self, element: impl IntoElement) -> Self {
        self.supporting = Some(SupportingSlot::Custom(element.into_any_element()));
        self
    }

    /// Renders a decorative icon inside an expressive `Surface` container above the title.
    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.visual = Some(EmptyStateVisual::Icon(icon.into_any_element()));
        self
    }

    /// Sets the polygon backdrop shape for `.icon(...)` using any [`ShapePolygon`]
    /// (including [`MaterialShape`] or [`PolygonRole`](gpui_material_theme::PolygonRole)).
    /// Defaults to `MaterialShape::Cookie7Sided`.
    pub fn icon_container_shape(mut self, shape: impl Into<ShapePolygon>) -> Self {
        self.icon_container_shape = IconContainerShape::Polygon(shape.into());
        self
    }

    /// Switches the `.icon(...)` container backdrop to a rounded [`CornerShape`].
    pub fn shape(mut self, shape: CornerShape) -> Self {
        self.icon_container_shape = IconContainerShape::Corners(shape);
        self
    }

    /// Sets the semantic color roles for the decorative `.icon(...)` container and foreground.
    /// Defaults to `SecondaryContainer` / `OnSecondaryContainer`.
    pub fn icon_container_color_role(mut self, container: ColorRole, content: ColorRole) -> Self {
        self.icon_container_role = container;
        self.icon_content_role = content;
        self
    }

    /// Replaces the decorative icon container with a custom visual illustration element.
    pub fn visual(mut self, element: impl IntoElement) -> Self {
        self.visual = Some(EmptyStateVisual::Custom(element.into_any_element()));
        self
    }

    /// Adds a caller-provided primary action control to the centered action row.
    pub fn primary_action(mut self, action: impl IntoElement) -> Self {
        self.primary_action = Some(action.into_any_element());
        self
    }

    /// Adds a caller-provided secondary action control to the centered action row.
    pub fn secondary_action(mut self, action: impl IntoElement) -> Self {
        self.secondary_action = Some(action.into_any_element());
        self
    }

    /// Constrains the centered content column's maximum width in logical pixels.
    /// Defaults to `360 dp`. Non-positive or non-finite values fall back to the default.
    pub fn max_width(mut self, pixels: Pixels) -> Self {
        self.max_width = resolve_max_width(pixels);
        self
    }

    /// Sets the Material typography role used for the title. Defaults to `TypeRole::TitleLarge`.
    pub fn title_role(mut self, role: TypeRole) -> Self {
        self.title_role = role;
        self
    }
}

fn resolve_max_width(width: Pixels) -> Pixels {
    let raw = f32::from(width);
    if raw.is_finite() && raw > 0. {
        width
    } else {
        DEFAULT_MAX_WIDTH
    }
}

impl RenderOnce for EmptyState {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let on_surface = color(theme.colors.color(ColorRole::OnSurface));
        let on_surface_variant = color(theme.colors.color(ColorRole::OnSurfaceVariant));
        let icon_fg = color(theme.colors.color(self.icon_content_role));

        let visual_slot = self.visual.map(|visual| match visual {
            EmptyStateVisual::Icon(icon) => {
                let surface = Surface::new(self.icon_container_role).content_color(icon_fg);
                let surface = match self.icon_container_shape {
                    IconContainerShape::Polygon(polygon) => surface.polygon_shape(polygon),
                    IconContainerShape::Corners(corners) => surface.shape(corners),
                };
                div()
                    .id((self.id.clone(), "icon-container"))
                    .test_support()
                    .mb(px(20.))
                    .child(
                        surface
                            .size(ICON_CONTAINER_SIZE)
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(ICON_SIZE)
                            .child(icon),
                    )
                    .into_any_element()
            }
            EmptyStateVisual::Custom(element) => div()
                .id((self.id.clone(), "visual"))
                .test_support()
                .mb(px(20.))
                .flex()
                .items_center()
                .justify_center()
                .child(element)
                .into_any_element(),
        });

        let title = wrapped_text(
            self.title.clone(),
            self.title_role.style(TypographyEmphasis::Baseline),
            1.,
            on_surface,
            window,
            cx,
        )
        .w_full()
        .text_center()
        .id((self.id.clone(), "title"))
        .test_support()
        .role(Role::Label)
        .aria_label(self.title);

        let supporting_slot = self.supporting.map(|slot| match slot {
            SupportingSlot::Text(text) => div()
                .w_full()
                .mt(px(8.))
                .child(
                    wrapped_text(
                        text.clone(),
                        TypeRole::BodyMedium.style(TypographyEmphasis::Baseline),
                        1.,
                        on_surface_variant,
                        window,
                        cx,
                    )
                    .w_full()
                    .text_center()
                    .id((self.id.clone(), "supporting"))
                    .test_support()
                    .role(Role::Label)
                    .aria_label(text),
                )
                .into_any_element(),
            SupportingSlot::Custom(element) => div()
                .id((self.id.clone(), "supporting"))
                .test_support()
                .w_full()
                .mt(px(8.))
                .flex()
                .justify_center()
                .child(element)
                .into_any_element(),
        });

        let has_actions = self.primary_action.is_some() || self.secondary_action.is_some();
        let actions_slot = has_actions.then(|| {
            div()
                .id((self.id.clone(), "actions"))
                .test_support()
                .mt(ACTIONS_TOP_MARGIN)
                .flex()
                .flex_row()
                .flex_wrap()
                .items_center()
                .justify_center()
                .gap(ACTION_GAP)
                .when(direction == Direction::Rtl, Styled::flex_row_reverse)
                .children(self.primary_action)
                .children(self.secondary_action)
        });

        div()
            .id(self.id.clone())
            .test_support()
            .w_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .px(px(24.))
            .py(px(32.))
            .child(
                div()
                    .id((self.id, "content"))
                    .test_support()
                    .w_full()
                    .max_w(self.max_width)
                    .flex()
                    .flex_col()
                    .items_center()
                    .children(visual_slot)
                    .child(title)
                    .children(supporting_slot)
                    .children(actions_slot),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_state_max_width_clamps_invalid_values_to_default() {
        assert_eq!(resolve_max_width(px(420.)), px(420.));
        assert_eq!(resolve_max_width(px(0.)), DEFAULT_MAX_WIDTH);
        assert_eq!(resolve_max_width(px(-120.)), DEFAULT_MAX_WIDTH);
        assert_eq!(resolve_max_width(px(f32::NAN)), DEFAULT_MAX_WIDTH);
        assert_eq!(resolve_max_width(px(f32::INFINITY)), DEFAULT_MAX_WIDTH);
    }
}
