use std::rc::Rc;

use gpui::{
    Anchor, App, Div, ElementId, Hsla, InteractiveElement, IntoElement, RenderOnce, Role,
    ScrollHandle, SharedString, Stateful, StatefulInteractiveElement, Styled, Window,
    accesskit::Action, div,
};
use gpui_material_theme::Direction;

use crate::{current_material_theme, scrim_color};

type ScrimClick = Rc<dyn Fn(&mut Window, &mut App)>;

/// A backdrop that fills its positioned parent. Add it after background content
/// and before the foreground surface. The caller owns visibility and modal focus.
#[derive(IntoElement)]
pub struct Scrim {
    id: ElementId,
    color: Option<Hsla>,
    opacity: f32,
    accessibility_label: Option<SharedString>,
    on_click: Option<ScrimClick>,
}

impl Scrim {
    const DEFAULT_OPACITY: f32 = 0.32;

    /// Creates a decorative backdrop with the current window's Scrim color role.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            color: None,
            opacity: Self::DEFAULT_OPACITY,
            accessibility_label: None,
            on_click: None,
        }
    }

    /// Uses a custom tint. The tint's own alpha multiplies the scrim opacity.
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Sets opacity in the range 0–1, clamping values outside it. NaN uses 0.32.
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = if opacity.is_nan() {
            Self::DEFAULT_OPACITY
        } else {
            opacity.clamp(0., 1.)
        };
        self
    }

    /// Names the clickable scrim for assistive technology. Defaults to “Dismiss overlay.”
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    /// Blocks pointer input behind the scrim and handles click or accessibility activation.
    /// Keep keyboard dismissal and focus management in the foreground surface.
    pub fn on_click(mut self, on_click: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(on_click));
        self
    }
}

impl RenderOnce for Scrim {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let fill = self.color.map_or_else(
            || scrim_color(&current_material_theme(window, cx), self.opacity),
            |color| color.opacity(self.opacity),
        );
        let backdrop = div().id(self.id).absolute().inset_0().bg(fill);
        match self.on_click {
            Some(on_click) => {
                let on_accessibility_click = on_click.clone();
                backdrop
                    .occlude()
                    .role(Role::Button)
                    .aria_label(
                        self.accessibility_label
                            .unwrap_or_else(|| "Dismiss overlay".into()),
                    )
                    // GPUI's default accessibility click targets the center,
                    // where the foreground surface may cover the scrim.
                    .on_a11y_action(Action::Click, move |_, window, cx| {
                        on_accessibility_click(window, cx);
                    })
                    .on_click(move |_, window, cx| {
                        cx.stop_propagation();
                        on_click(window, cx);
                    })
            }
            None => backdrop,
        }
    }
}

/// Mirrors the horizontal part of an overlay anchor for logical direction.
/// Base retains placement, fit, focus scope and dismissal.
pub fn logical_anchor(anchor: Anchor, direction: Direction) -> Anchor {
    if direction == Direction::Ltr {
        return anchor;
    }
    match anchor {
        Anchor::TopLeft => Anchor::TopRight,
        Anchor::TopRight => Anchor::TopLeft,
        Anchor::BottomLeft => Anchor::BottomRight,
        Anchor::BottomRight => Anchor::BottomLeft,
        Anchor::LeftCenter => Anchor::RightCenter,
        Anchor::RightCenter => Anchor::LeftCenter,
        Anchor::TopCenter | Anchor::BottomCenter => anchor,
    }
}

/// Creates an inner vertical scroll viewport that keeps wheel input from
/// reaching scroll owners behind it. The caller gives it a bounded height.
pub fn isolated_scroll(id: impl Into<ElementId>, handle: &ScrollHandle) -> Stateful<Div> {
    div()
        .id(id)
        .min_h_0()
        .overflow_y_scroll()
        .occlude()
        .track_scroll(handle)
}
