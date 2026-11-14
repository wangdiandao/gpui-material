use gpui::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, Pixels, Radians,
    RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled, Transformation, Window,
    div, px, radians, size, svg,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{color, window_material_theme};
use gpui_material_theme::{ColorRole, Direction};

macro_rules! symbols {
    ($($variant:ident => $name:literal),+ $(,)?) => {
        /// Bundled official Material Symbols Outlined, weight 400, grade 0, optical size 24.
        /// Both fill variants are embedded unchanged, without an application AssetSource.
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum IconName { $($variant),+ }

        impl IconName {
            /// All bundled symbols, useful for icon pickers and asset validation.
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            /// Unchanged upstream SVG bytes for FILL=0 or FILL=1.
            pub fn svg_data(self, filled: bool) -> &'static [u8] {
                match (self, filled) {
                    $((Self::$variant, false) => include_bytes!(concat!("../assets/material-symbols/", $name, "_24px.svg")),
                      (Self::$variant, true) => include_bytes!(concat!("../assets/material-symbols/", $name, "_fill1_24px.svg"))),+
                }
            }
        }
    };
}

symbols! {
    Add => "add",
    Remove => "remove",
    Check => "check",
    Close => "close",
    ChevronRight => "chevron_right",
    ArrowForward => "arrow_forward",
    ArrowDropDown => "arrow_drop_down",
    Favorite => "favorite",
    Archive => "archive",
    Inbox => "inbox",
    Description => "description",
    Notifications => "notifications",
    MotionPhotosOn => "motion_photos_on",
    Contrast => "contrast",
    Search => "search",
    Error => "error",
    Info => "info",
    Edit => "edit",
    Exposure => "exposure",
}

/// A decorative icon by default; `.label` exposes an independently meaningful image.
/// Named symbols inherit text size and foreground, including disabled colors, from
/// their control's icon slot. Outside a slot, use `.size(px(24.))` for a 24dp icon.
/// Custom SVG paths use the application's AssetSource. Mirroring is opt-in.
#[derive(IntoElement)]
pub struct Icon {
    content: IconContent,
    name: Option<(ElementId, SharedString)>,
    side: Option<Pixels>,
    color: Option<ColorRole>,
    mirror: bool,
    rotation: Radians,
    filled: bool,
}

enum IconContent {
    Symbol(IconName),
    Svg(SharedString),
    Element(AnyElement),
}

impl Icon {
    pub fn new(symbol: IconName) -> Self {
        Self {
            content: IconContent::Symbol(symbol),
            name: None,
            side: None,
            color: None,
            mirror: false,
            rotation: radians(0.),
            filled: false,
        }
    }

    /// Caller-provided SVG; retains the existing 24dp/OnSurfaceVariant defaults.
    pub fn svg(path: impl Into<SharedString>) -> Self {
        Self {
            content: IconContent::Svg(path.into()),
            side: Some(px(24.)),
            color: Some(ColorRole::OnSurfaceVariant),
            ..Self::new(IconName::Info)
        }
    }

    pub fn custom(element: impl IntoElement) -> Self {
        Self {
            content: IconContent::Element(element.into_any_element()),
            ..Self::svg("")
        }
    }

    /// Select FILL=1 for a named symbol; does not alter caller-provided content.
    pub fn filled(mut self, filled: bool) -> Self {
        self.filled = filled;
        self
    }

    pub fn label(mut self, id: impl Into<ElementId>, name: impl Into<SharedString>) -> Self {
        self.name = Some((id.into(), name.into()));
        self
    }

    pub fn size(mut self, side: Pixels) -> Self {
        self.side = Some(side.max(px(0.)));
        self
    }

    pub fn color(mut self, color: ColorRole) -> Self {
        self.color = Some(color);
        self
    }

    pub fn mirror_in_rtl(mut self, mirror: bool) -> Self {
        self.mirror = mirror;
        self
    }

    /// Paint-time rotation composed with RTL mirroring; layout is unchanged.
    pub fn rotate(mut self, angle: Radians) -> Self {
        self.rotation = angle;
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let tint = self.color.map_or_else(
            || window.text_style().color,
            |role| color(theme.colors.color(role)),
        );
        let side = self
            .side
            .unwrap_or_else(|| window.text_style().font_size.to_pixels(window.rem_size()));
        let rtl = self.mirror && theme.layout.direction == Direction::Rtl;
        let transform = Transformation::rotate(self.rotation)
            .with_scaling(size(if rtl { -1. } else { 1. }, 1.));
        let icon = match self.content {
            IconContent::Symbol(symbol) => svg()
                .data(symbol.svg_data(self.filled))
                .size_full()
                .text_color(tint)
                .with_transformation(transform)
                .into_any_element(),
            IconContent::Svg(path) => svg()
                .path(path)
                .size_full()
                .text_color(tint)
                .with_transformation(transform)
                .into_any_element(),
            IconContent::Element(element) => element,
        };
        let icon = div()
            .size(side)
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .text_color(tint)
            .child(icon);
        match self.name {
            Some((id, label)) => icon
                .id(id)
                .test_support()
                .role(Role::Image)
                .aria_label(label)
                .into_any_element(),
            None => icon.into_any_element(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn every_bundled_symbol_rasterizes_without_fonts_or_application_assets() {
        let renderer = gpui::SvgRenderer::new(Arc::new(()));
        for name in IconName::ALL {
            for filled in [false, true] {
                let data = name.svg_data(filled);
                // Symbols must stay vector-only: no fonts, images or linked resources.
                let source = std::str::from_utf8(data).unwrap();
                assert!(!source.contains("<text") && !source.contains("<image"));
                assert!(!source.contains("href="));
                for scale in [1., 2.] {
                    let image = renderer
                        .render_single_frame(data, scale)
                        .unwrap_or_else(|error| panic!("{name:?} fill={filled}: {error}"));
                    assert!(
                        image
                            .as_bytes(0)
                            .unwrap()
                            .as_chunks::<4>()
                            .0
                            .iter()
                            .any(|pixel| pixel[3] != 0),
                        "{name:?} fill={filled} must paint visible pixels"
                    );
                }
            }
        }
        assert_ne!(
            IconName::Favorite.svg_data(false),
            IconName::Favorite.svg_data(true)
        );
    }
}
