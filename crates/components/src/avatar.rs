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

type OverflowContent = Box<dyn FnOnce(usize) -> AnyElement>;

const DEFAULT_RING_WIDTH: Pixels = px(2.);

/// An overlapping horizontal stack of [`Avatar`] items with surface-matched cutout rings
/// and an optional `+N` overflow indicator.
#[derive(IntoElement)]
pub struct AvatarGroup {
    id: ElementId,
    avatars: Vec<Avatar>,
    size: AvatarSize,
    max_visible: Option<usize>,
    overlap: Option<Pixels>,
    ring_width: Pixels,
    ring_role: ColorRole,
    ring_color: Option<Hsla>,
    overflow_colors: AvatarColors,
    overflow_content: Option<OverflowContent>,
    accessible_label: Option<SharedString>,
}

impl AvatarGroup {
    /// Creates an overlapping stack from an iterator of [`Avatar`] items.
    pub fn new(id: impl Into<ElementId>, avatars: impl IntoIterator<Item = Avatar>) -> Self {
        Self {
            id: id.into(),
            avatars: avatars.into_iter().collect(),
            size: AvatarSize::default(),
            max_visible: None,
            overlap: None,
            ring_width: DEFAULT_RING_WIDTH,
            ring_role: ColorRole::Surface,
            ring_color: None,
            overflow_colors: AvatarColors::default(),
            overflow_content: None,
            accessible_label: None,
        }
    }

    /// Applies a uniform diameter across all member avatars and the overflow indicator.
    /// Defaults to `AvatarSize::Medium` (40 px).
    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = size;
        self
    }

    /// Limits the number of displayed member avatars before appending a `+N` overflow indicator.
    pub fn max_visible(mut self, count: usize) -> Self {
        self.max_visible = Some(count);
        self
    }

    /// Sets the positive overlap distance between adjacent avatars in logical pixels.
    /// Defaults to 25% of the resolved diameter and is clamped to `[0, diameter - 1]`.
    pub fn overlap(mut self, pixels: Pixels) -> Self {
        self.overlap = Some(pixels);
        self
    }

    /// Sets the surface-matched cutout ring width around each avatar in logical pixels.
    /// Defaults to `2 px`. Set to `0 px` to omit the ring.
    pub fn ring_width(mut self, pixels: Pixels) -> Self {
        let value = f32::from(pixels);
        self.ring_width = if value.is_finite() && value >= 0. {
            pixels
        } else {
            px(0.)
        };
        self
    }

    /// Overrides the cutout ring with an explicit color. Takes precedence over `.ring_role`.
    pub fn ring_color(mut self, color: impl Into<Hsla>) -> Self {
        self.ring_color = Some(color.into());
        self
    }

    /// Resolves the cutout ring color from the window theme's semantic color role.
    /// Defaults to `ColorRole::Surface`.
    pub fn ring_role(mut self, role: ColorRole) -> Self {
        self.ring_role = role;
        self.ring_color = None;
        self
    }

    /// Overrides container, content, and border pigments on the `+N` overflow indicator.
    /// Omitted fields use `SurfaceContainerHighest` / `OnSurfaceVariant`.
    pub fn overflow_colors(mut self, colors: AvatarColors) -> Self {
        self.overflow_colors = colors;
        self
    }

    /// Replaces the default `+N` text inside the overflow avatar with custom visual content.
    pub fn overflow_content<E: IntoElement>(
        mut self,
        content: impl FnOnce(usize) -> E + 'static,
    ) -> Self {
        self.overflow_content = Some(Box::new(move |count| content(count).into_any_element()));
        self
    }

    /// Exposes a summary `Role::Group` accessible label for the avatar stack without adding focus stops.
    pub fn accessible_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }
}

fn max_overlap_for_diameter(diameter: f32) -> f32 {
    if diameter > 1. { diameter - 1. } else { 0. }
}

fn default_overlap(size: AvatarSize) -> Pixels {
    let diameter = f32::from(size.pixels());
    px((diameter * 0.25)
        .round()
        .clamp(0., max_overlap_for_diameter(diameter)))
}

fn resolve_overlap(size: AvatarSize, overlap: Option<Pixels>) -> Pixels {
    let diameter = f32::from(size.pixels());
    let upper = max_overlap_for_diameter(diameter);
    match overlap {
        Some(value) => {
            let raw = f32::from(value);
            if raw.is_finite() {
                px(raw.clamp(0., upper))
            } else {
                default_overlap(size)
            }
        }
        None => default_overlap(size),
    }
}

fn split_visible(total: usize, max_visible: Option<usize>) -> (usize, usize) {
    match max_visible {
        Some(limit) if total > limit => (limit, total - limit),
        _ => (total, 0),
    }
}

impl RenderOnce for AvatarGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let is_rtl = direction == gpui_material_theme::Direction::Rtl;
        let side = self.size.pixels();
        let overlap = resolve_overlap(self.size, self.overlap);
        let ring_width = self.ring_width;
        let ring_color = self
            .ring_color
            .unwrap_or_else(|| color(theme.colors.color(self.ring_role)));
        let (visible_count, overflow_count) = split_visible(self.avatars.len(), self.max_visible);
        let overflow_shape = self
            .avatars
            .first()
            .map_or(AvatarShape::Circle, |avatar| avatar.shape);

        let mut items: Vec<Avatar> = self
            .avatars
            .into_iter()
            .take(visible_count)
            .map(|mut avatar| {
                avatar.size = self.size;
                avatar
            })
            .collect();

        if overflow_count > 0 {
            let overflow_avatar = match self.overflow_content {
                Some(render) => Avatar::custom(render(overflow_count)),
                None => Avatar::with_content(AvatarContent::Initials(
                    format!("+{overflow_count}").into(),
                )),
            }
            .size(self.size)
            .shape(overflow_shape)
            .color_role(
                ColorRole::SurfaceContainerHighest,
                ColorRole::OnSurfaceVariant,
            )
            .colors(self.overflow_colors);
            items.push(overflow_avatar);
        }

        let mut row = div()
            .id(self.id)
            .test_support()
            .flex()
            .flex_row()
            .items_center()
            .when(is_rtl, Styled::flex_row_reverse)
            .when_some(self.accessible_label, |row, label| {
                row.role(Role::Group).aria_label(label)
            });

        for (index, avatar) in items.into_iter().enumerate() {
            let [tl, tr, br, bl] = resolve_radii(avatar.shape, side, direction);
            let slot = if ring_width > px(0.) {
                let outer_side = side + ring_width * 2.;
                let [otl, otr, obr, obl] = match avatar.shape {
                    AvatarShape::Circle => [outer_side * 0.5; 4],
                    AvatarShape::Rounded(_) => [
                        tl + ring_width,
                        tr + ring_width,
                        br + ring_width,
                        bl + ring_width,
                    ],
                };
                div()
                    .flex_none()
                    .size(outer_side)
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(ring_color)
                    .border(ring_width)
                    .border_color(ring_color)
                    .rounded_tl(otl)
                    .rounded_tr(otr)
                    .rounded_br(obr)
                    .rounded_bl(obl)
                    .child(avatar)
            } else {
                div().flex_none().size(side).child(avatar)
            };
            let slot = if index > 0 && overlap > px(0.) {
                if is_rtl {
                    slot.mr(-overlap)
                } else {
                    slot.ml(-overlap)
                }
            } else {
                slot
            };
            row = row.child(slot);
        }

        row
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

    #[test]
    fn avatar_group_overlap_defaults_and_clamps_within_diameter() {
        assert_eq!(resolve_overlap(AvatarSize::ExtraSmall, None), px(6.));
        assert_eq!(resolve_overlap(AvatarSize::Small, None), px(8.));
        assert_eq!(resolve_overlap(AvatarSize::Medium, None), px(10.));
        assert_eq!(resolve_overlap(AvatarSize::Large, None), px(12.));
        assert_eq!(resolve_overlap(AvatarSize::ExtraLarge, None), px(16.));

        assert_eq!(resolve_overlap(AvatarSize::Medium, Some(px(14.))), px(14.));
        assert_eq!(resolve_overlap(AvatarSize::Medium, Some(px(-5.))), px(0.));
        assert_eq!(resolve_overlap(AvatarSize::Medium, Some(px(80.))), px(39.));
        assert_eq!(
            resolve_overlap(AvatarSize::Medium, Some(px(f32::NAN))),
            px(10.)
        );
        assert_eq!(
            resolve_overlap(AvatarSize::Medium, Some(px(f32::INFINITY))),
            px(10.)
        );
    }

    #[test]
    fn avatar_group_split_visible_computes_overflow_counts() {
        assert_eq!(split_visible(5, None), (5, 0));
        assert_eq!(split_visible(5, Some(3)), (3, 2));
        assert_eq!(split_visible(3, Some(3)), (3, 0));
        assert_eq!(split_visible(2, Some(5)), (2, 0));
        assert_eq!(split_visible(4, Some(0)), (0, 4));
        assert_eq!(split_visible(0, Some(0)), (0, 0));
    }
}
