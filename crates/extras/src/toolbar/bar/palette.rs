use std::{cell::Cell, rc::Rc};

use gpui::{AnyElement, App, Hsla, IntoElement, Pixels, Point, Size, Window, px};
use gpui_material_components::{ControlElevations, FabColor, Icon};
use gpui_material_foundation::animate;
use gpui_material_theme::{ColorRole, Elevation, MotionScheme};

use super::{
    ActionIcon, Toolbar, ToolbarColor, ToolbarExitDirection, ToolbarPlacement,
    ToolbarScrollBehavior, ToolbarScrollState,
};

impl ActionIcon {
    pub(super) fn render_inline(&self) -> AnyElement {
        match self {
            Self::Symbol(icon) => Icon::new(*icon).size(px(24.)).into_any_element(),
            Self::Custom(build) => build(),
        }
    }

    pub(super) fn render_overflow(&self) -> AnyElement {
        match self {
            Self::Symbol(icon) => Icon::new(*icon).size(px(20.)).into_any_element(),
            Self::Custom(build) => build(),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct ToolbarPalette {
    pub(super) surface: Hsla,
    pub(super) ink: Hsla,
    pub(super) selected_fill: Hsla,
    pub(super) selected_ink: Hsla,
    pub(super) ring: Hsla,
    pub(super) shadow: Hsla,
    pub(super) default_fab_color: FabColor,
}

impl ToolbarPalette {
    pub(super) fn resolve(toolbar: &Toolbar, tone: impl Fn(ColorRole) -> Hsla) -> Self {
        let (surface_role, content_role, selected_role, selected_ink_role, default_fab_color) =
            match toolbar.color {
                ToolbarColor::Standard => (
                    ColorRole::SurfaceContainer,
                    ColorRole::OnSurfaceVariant,
                    ColorRole::SecondaryContainer,
                    ColorRole::OnSecondaryContainer,
                    FabColor::Primary,
                ),
                ToolbarColor::Vibrant => (
                    ColorRole::PrimaryContainer,
                    ColorRole::OnPrimaryContainer,
                    ColorRole::SurfaceContainer,
                    ColorRole::OnSurface,
                    FabColor::Tertiary,
                ),
            };
        Self {
            surface: toolbar
                .colors
                .container
                .unwrap_or_else(|| tone(surface_role)),
            ink: toolbar.colors.content.unwrap_or_else(|| tone(content_role)),
            selected_fill: toolbar
                .colors
                .selected_container
                .unwrap_or_else(|| tone(selected_role)),
            selected_ink: toolbar
                .colors
                .selected_content
                .unwrap_or_else(|| tone(selected_ink_role)),
            ring: tone(ColorRole::Secondary),
            shadow: tone(ColorRole::Shadow),
            default_fab_color,
        }
    }
}

/// Resolves the target Material elevation for a toolbar strip.
///
/// Floating toolbars default to `Elevation::Level3` when expanded and
/// `Elevation::Level0` when collapsed. Docked toolbars default to `Elevation::Level0`.
/// `ControlElevations::resting` overrides the expanded elevation and
/// `ControlElevations::disabled` overrides the collapsed elevation.
pub(super) fn resolve_target_elevation(
    placement: ToolbarPlacement,
    expanded: bool,
    elevations: ControlElevations,
) -> Elevation {
    if expanded {
        let default = match placement {
            ToolbarPlacement::Floating => Elevation::Level3,
            ToolbarPlacement::Docked => Elevation::Level0,
        };
        elevations.resting.unwrap_or(default)
    } else {
        elevations.disabled.unwrap_or(Elevation::Level0)
    }
}

pub(super) struct ScrollPresentation {
    pub(super) slide: Point<Pixels>,
    pub(super) hidden_strip: bool,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_scroll_presentation(
    toolbar: &mut Toolbar,
    footprint: &Rc<Cell<Size<Pixels>>>,
    behavior: ToolbarScrollBehavior,
    scheme: MotionScheme,
    reduced: bool,
    exposed: bool,
    rtl: bool,
    window: &mut Window,
    cx: &mut App,
) -> ScrollPresentation {
    let limit = behavior.exit_direction.map_or(0., |edge| {
        let size = footprint.get();
        f32::from(match edge {
            ToolbarExitDirection::Start | ToolbarExitDirection::End => size.width,
            ToolbarExitDirection::Top | ToolbarExitDirection::Bottom => size.height,
        })
    });
    let scroll_state = toolbar.scroll_state.take().or_else(|| {
        (toolbar.scroll.is_some() || toolbar.scroll_behavior.is_some()).then(|| {
            window.use_keyed_state((toolbar.id.clone(), "scroll-expansion"), cx, |_, _| {
                ToolbarScrollState::new()
            })
        })
    });
    let Some(state) = scroll_state else {
        return ScrollPresentation {
            slide: Point::default(),
            hidden_strip: false,
        };
    };
    let linked = toolbar.scroll.as_ref().map(|scroll| -scroll.offset().y);
    let position = state.update(cx, |state, _| state.prepare(linked, limit, behavior));
    let fraction = animate(
        (toolbar.id.clone(), "toolbar-scroll"),
        position.target,
        position.motion.spec(behavior, scheme, reduced),
        window,
        cx,
    )
    .clamp(0., 1.);
    toolbar.expanded &= position.expanded;
    state.update(cx, |state, cx| {
        if state.display(position, fraction, limit, exposed) {
            cx.notify();
        }
    });
    let distance = px(if exposed { 0. } else { fraction * limit });
    let hidden_strip = limit > 0. && distance >= px(limit);
    let slide = match behavior.exit_direction {
        Some(ToolbarExitDirection::Top) => gpui::point(px(0.), -distance),
        Some(ToolbarExitDirection::Bottom) => gpui::point(px(0.), distance),
        Some(ToolbarExitDirection::Start) if rtl => gpui::point(distance, px(0.)),
        Some(ToolbarExitDirection::End) if !rtl => gpui::point(distance, px(0.)),
        Some(ToolbarExitDirection::Start | ToolbarExitDirection::End) => {
            gpui::point(-distance, px(0.))
        }
        None => Point::default(),
    };
    ScrollPresentation {
        slide,
        hidden_strip,
    }
}
