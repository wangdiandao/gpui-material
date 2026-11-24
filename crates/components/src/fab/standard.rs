use gpui::{
    AnyElement, App, ElementId, Empty, IntoElement, ParentElement, Pixels, RenderOnce,
    SharedString, Styled, Window, anchored, deferred, div, point, px,
};
use gpui_material_foundation::{ThemeScope, current_material_theme, logical_anchor};
use gpui_material_theme::Direction;

use crate::action::icon_slot;

use super::{
    fab_builders,
    types::{FabAppearance, FabContent, FabSize},
};

/// An elevated icon action with a required accessibility name. Mount it beside
/// the scroll viewport to keep its position fixed while content scrolls.
#[derive(IntoElement)]
pub struct Fab {
    content: FabContent,
    icon: AnyElement,
    appearance: FabAppearance,
}

impl Fab {
    /// Creates a floating action button with an element ID, required accessible name, and icon.
    pub fn new(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        icon: impl IntoElement,
    ) -> Self {
        Self {
            content: FabContent::new(id, name),
            icon: icon.into_any_element(),
            appearance: FabAppearance::default(),
        }
    }

    fab_builders!();
}

impl RenderOnce for Fab {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let style = self.content.style();
        let (_, icon_size, _, _, _, _) = self.content.size.metrics();
        let icon_size = if self.content.size == FabSize::Large {
            36.
        } else {
            icon_size
        };
        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let extent = theme.layout.density.interactive_size(style.height);
        let motion = self.appearance.motion(&self.content.id, extent, window, cx);
        let action = self
            .content
            .action(icon_slot(self.icon, icon_size * motion.scale));
        if self.appearance.visible.is_none() {
            return self.appearance.control(action, 1., true, window, cx);
        }
        let interactive = self.appearance.interactive(&action, motion, window, cx);
        if motion.alpha == 0. {
            return Empty.into_any_element();
        }
        let extent = px(extent);
        let mut footprint = div().relative().size(extent).min_w(extent).min_h(extent);
        if motion.scale > 0. {
            let pivot = self.appearance.animation.alignment.pivot(direction);
            let distance = extent * (1. - motion.scale);
            let control = self
                .appearance
                .control(action, motion.scale, interactive, window, cx);
            footprint = footprint.child(
                div()
                    .absolute()
                    .left(distance * pivot.x)
                    .top(distance * pivot.y)
                    .opacity(motion.alpha)
                    .child(control),
            );
        }
        footprint.into_any_element()
    }
}

/// Places a FAB inside the current window viewport at logical bottom-end.
/// Mount this element beside the scroll owner so it stays visible while scrolling.
#[derive(IntoElement)]
pub struct FloatingAction {
    action: AnyElement,
    margin: Pixels,
}

impl FloatingAction {
    /// Anchors `action` to the viewport's logical bottom-end corner with a default 16 dp margin.
    pub fn new(action: impl IntoElement) -> Self {
        Self {
            action: action.into_any_element(),
            margin: px(16.),
        }
    }

    /// Sets the inset from the viewport edges in logical pixels (clamped to non-negative values).
    pub fn margin(mut self, margin: Pixels) -> Self {
        self.margin = margin.max(px(0.));
        self
    }
}

impl RenderOnce for FloatingAction {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let viewport = window.viewport_size();
        let direction = current_material_theme(window, cx).layout.direction;
        let margin = self.margin.min(viewport.width.min(viewport.height) / 4.);
        let x = if direction == Direction::Rtl {
            margin
        } else {
            viewport.width - margin
        };
        deferred(
            anchored()
                .anchor(logical_anchor(gpui::Anchor::BottomRight, direction))
                .position(point(x, viewport.height - margin))
                .snap_to_window_with_margin(margin)
                .child(ThemeScope::inherit(
                    div()
                        .max_w((viewport.width - margin * 2.).max(px(0.)))
                        .child(self.action),
                    window,
                    cx,
                )),
        )
    }
}
