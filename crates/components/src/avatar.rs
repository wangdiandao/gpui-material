use std::{cell::RefCell, rc::Rc};

use gpui::{
    AnyElement, App, ElementId, Hsla, ImageSource, InteractiveElement, IntoElement, ObjectFit,
    ParentElement, Pixels, RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled,
    StyledImage, Window, div, img, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{color, current_material_theme, text_style};
use gpui_material_theme::{ColorRole, CornerShape, TypeRole, TypographyEmphasis};

/// Material avatar diameter presets and custom logical-pixel sizing.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum AvatarSize {
    ExtraSmall,
    Small,
    #[default]
    Medium,
    Large,
    ExtraLarge,
    Custom(Pixels),
}

impl AvatarSize {
    /// Resolves the avatar diameter in logical pixels. Invalid custom lengths fall back to `Medium` (40 px).
    pub fn pixels(self) -> Pixels {
        match self {
            Self::ExtraSmall => px(24.),
            Self::Small => px(32.),
            Self::Medium => px(40.),
            Self::Large => px(48.),
            Self::ExtraLarge => px(64.),
            Self::Custom(size) => {
                let value = f32::from(size);
                if value.is_finite() && value > 0. {
                    size
                } else {
                    px(40.)
                }
            }
        }
    }

    fn typography_role(self) -> TypeRole {
        let side = f32::from(self.pixels());
        if side < 28. {
            TypeRole::LabelSmall
        } else if side < 36. {
            TypeRole::LabelMedium
        } else if side < 44. {
            TypeRole::TitleSmall
        } else if side < 56. {
            TypeRole::TitleMedium
        } else {
            TypeRole::HeadlineSmall
        }
    }

    fn icon_size(self) -> Pixels {
        let side = f32::from(self.pixels());
        px((side * 0.55).round().clamp(12., side))
    }
}

/// Outer clipping shape for an [`Avatar`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum AvatarShape {
    #[default]
    Circle,
    Rounded(CornerShape),
}

/// Optional per-instance pigments for an [`Avatar`]. Omitted fields use the configured color roles.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AvatarColors {
    pub container: Option<Hsla>,
    pub content: Option<Hsla>,
    pub border: Option<Hsla>,
}

enum AvatarContent {
    Initials(SharedString),
    Icon(AnyElement),
    Image {
        source: ImageSource,
        fallback: Option<AnyElement>,
    },
    Custom(AnyElement),
}

/// A circular or rounded Material avatar displaying initials, an icon, an image, or custom content.
#[derive(IntoElement)]
pub struct Avatar {
    content: AvatarContent,
    size: AvatarSize,
    shape: AvatarShape,
    container_role: ColorRole,
    content_role: ColorRole,
    colors: AvatarColors,
    border_width: Pixels,
    accessible_label: Option<(ElementId, SharedString)>,
}

impl Avatar {
    /// Displays trimmed, uppercased 1–2 character initials with typography scaled to the avatar size.
    pub fn initials(text: impl Into<SharedString>) -> Self {
        Self::with_content(AvatarContent::Initials(format_initials(&text.into())))
    }

    /// Displays a decorative icon sized proportionally to the avatar.
    pub fn icon(icon: impl IntoElement) -> Self {
        Self::with_content(AvatarContent::Icon(icon.into_any_element()))
    }

    /// Displays an image cropped with `ObjectFit::Cover` and clipped to the avatar shape.
    pub fn image(source: impl Into<ImageSource>) -> Self {
        Self::with_content(AvatarContent::Image {
            source: source.into(),
            fallback: None,
        })
    }

    /// Displays arbitrary caller-supplied visual content clipped to the avatar shape.
    pub fn custom(content: impl IntoElement) -> Self {
        Self::with_content(AvatarContent::Custom(content.into_any_element()))
    }

    fn with_content(content: AvatarContent) -> Self {
        Self {
            content,
            size: AvatarSize::default(),
            shape: AvatarShape::default(),
            container_role: ColorRole::PrimaryContainer,
            content_role: ColorRole::OnPrimaryContainer,
            colors: AvatarColors::default(),
            border_width: px(0.),
            accessible_label: None,
        }
    }

    /// Supplies fallback visual content for `Avatar::image` while loading or if loading fails.
    pub fn fallback(mut self, fallback: impl IntoElement) -> Self {
        if let AvatarContent::Image { fallback: slot, .. } = &mut self.content {
            *slot = Some(fallback.into_any_element());
        }
        self
    }

    /// Sets the avatar diameter preset or custom pixel size. Defaults to `AvatarSize::Medium` (40 px).
    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = size;
        self
    }

    /// Sets the avatar shape. Defaults to `AvatarShape::Circle`.
    pub fn shape(mut self, shape: AvatarShape) -> Self {
        self.shape = shape;
        self
    }

    /// Resolves container and content pigments from the window theme's semantic color roles.
    pub fn color_role(mut self, container: ColorRole, content: ColorRole) -> Self {
        self.container_role = container;
        self.content_role = content;
        self
    }

    /// Overrides container, content, and border pigments directly.
    pub fn colors(mut self, colors: AvatarColors) -> Self {
        self.colors = colors;
        self
    }

    /// Draws an inner border stroke around the avatar perimeter in logical pixels.
    pub fn border(mut self, width: Pixels) -> Self {
        let value = f32::from(width);
        self.border_width = if value.is_finite() && value >= 0. {
            width
        } else {
            px(0.)
        };
        self
    }

    /// Exposes the avatar as a named `Role::Image` for assistive technology without adding a focus stop.
    pub fn accessible_label(
        mut self,
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
    ) -> Self {
        self.accessible_label = Some((id.into(), label.into()));
        self
    }
}

fn format_initials(text: &str) -> SharedString {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return SharedString::default();
    }
    let mut words = trimmed.split_whitespace();
    let first = words.next().unwrap_or_default();
    if let Some(second) = words.next() {
        let mut out = String::new();
        if let Some(ch) = first.chars().next() {
            out.extend(ch.to_uppercase());
        }
        if let Some(ch) = second.chars().next() {
            out.extend(ch.to_uppercase());
        }
        return out.into();
    }
    let mut out = String::new();
    for ch in first.chars().take(2) {
        out.extend(ch.to_uppercase());
    }
    out.into()
}

fn resolve_radii(
    shape: AvatarShape,
    side: Pixels,
    direction: gpui_material_theme::Direction,
) -> [Pixels; 4] {
    match shape {
        AvatarShape::Circle => [side * 0.5; 4],
        AvatarShape::Rounded(corners) => {
            let extent = f32::from(side);
            corners.radii(extent, extent, direction).map(px)
        }
    }
}

impl RenderOnce for Avatar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = current_material_theme(window, cx);
        let side = self.size.pixels();
        let direction = theme.layout.direction;
        let [tl, tr, br, bl] = resolve_radii(self.shape, side, direction);
        let container_color = self
            .colors
            .container
            .unwrap_or_else(|| color(theme.colors.color(self.container_role)));
        let content_color = self
            .colors
            .content
            .unwrap_or_else(|| color(theme.colors.color(self.content_role)));
        let border_color = self
            .colors
            .border
            .unwrap_or_else(|| color(theme.colors.color(ColorRole::OutlineVariant)));
        let typography = self
            .size
            .typography_role()
            .style(TypographyEmphasis::Baseline);
        let icon_size = self.size.icon_size();

        let base = div()
            .relative()
            .flex_none()
            .size(side)
            .overflow_hidden()
            .rounded_tl(tl)
            .rounded_tr(tr)
            .rounded_br(br)
            .rounded_bl(bl)
            .bg(container_color)
            .text_color(content_color)
            .flex()
            .items_center()
            .justify_center();

        let body = match self.content {
            AvatarContent::Initials(text) => base.child(crate::text::single_line(
                text,
                typography,
                1.,
                content_color,
                (None, 1., 1.),
                window,
                cx,
            )),
            AvatarContent::Icon(icon) => base.text_size(icon_size).child(icon),
            AvatarContent::Image { source, fallback } => {
                let mut image = img(source)
                    .size_full()
                    .object_fit(ObjectFit::Cover)
                    .rounded_tl(tl)
                    .rounded_tr(tr)
                    .rounded_br(br)
                    .rounded_bl(bl);
                if let Some(fallback) = fallback {
                    let slot = Rc::new(RefCell::new(Some(fallback)));
                    let loading_slot = slot.clone();
                    let error_slot = slot;
                    image = image
                        .with_loading(move || {
                            div()
                                .size_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .children(loading_slot.borrow_mut().take())
                                .into_any_element()
                        })
                        .with_fallback(move || {
                            div()
                                .size_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .children(error_slot.borrow_mut().take())
                                .into_any_element()
                        });
                }
                text_style(base, typography, 1., window, cx).child(image)
            }
            AvatarContent::Custom(content) => {
                text_style(base, typography, 1., window, cx).child(content)
            }
        };

        let body = body.when(self.border_width > px(0.), |avatar| {
            avatar.child(
                div()
                    .absolute()
                    .inset_0()
                    .rounded_tl(tl)
                    .rounded_tr(tr)
                    .rounded_br(br)
                    .rounded_bl(bl)
                    .border(self.border_width)
                    .border_color(border_color),
            )
        });

        if let Some((id, label)) = self.accessible_label {
            body.id(id)
                .test_support()
                .role(Role::Image)
                .aria_label(label)
                .into_any_element()
        } else {
            body.into_any_element()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_material_theme::Direction;

    #[test]
    fn initials_formatting_trims_uppercases_and_bounds_length() {
        assert_eq!(format_initials("Ada Lovelace"), "AL");
        assert_eq!(format_initials("  grace   hopper  "), "GH");
        assert_eq!(format_initials("al"), "AL");
        assert_eq!(format_initials("a"), "A");
        assert_eq!(format_initials("Alice"), "AL");
        assert_eq!(format_initials("   "), "");
    }

    #[test]
    fn avatar_size_presets_and_typography_scale_monotonically() {
        assert_eq!(AvatarSize::ExtraSmall.pixels(), px(24.));
        assert_eq!(AvatarSize::Small.pixels(), px(32.));
        assert_eq!(AvatarSize::Medium.pixels(), px(40.));
        assert_eq!(AvatarSize::Large.pixels(), px(48.));
        assert_eq!(AvatarSize::ExtraLarge.pixels(), px(64.));
        assert_eq!(AvatarSize::Custom(px(52.)).pixels(), px(52.));
        assert_eq!(AvatarSize::Custom(px(-4.)).pixels(), px(40.));
        assert_eq!(AvatarSize::Custom(px(f32::NAN)).pixels(), px(40.));

        assert_eq!(
            AvatarSize::ExtraSmall.typography_role(),
            TypeRole::LabelSmall
        );
        assert_eq!(AvatarSize::Small.typography_role(), TypeRole::LabelMedium);
        assert_eq!(AvatarSize::Medium.typography_role(), TypeRole::TitleSmall);
        assert_eq!(AvatarSize::Large.typography_role(), TypeRole::TitleMedium);
        assert_eq!(
            AvatarSize::ExtraLarge.typography_role(),
            TypeRole::HeadlineSmall
        );
    }

    #[test]
    fn rounded_avatar_shape_mirrors_asymmetric_corners_in_rtl() {
        let shape = AvatarShape::Rounded(CornerShape::Radii([12., 4., 8., 2.]));
        let ltr = resolve_radii(shape, px(40.), Direction::Ltr);
        let rtl = resolve_radii(shape, px(40.), Direction::Rtl);
        assert_eq!(ltr, [px(12.), px(4.), px(8.), px(2.)]);
        assert_eq!(rtl, [px(4.), px(12.), px(2.), px(8.)]);
    }
}
