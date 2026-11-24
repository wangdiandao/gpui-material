use std::{cell::RefCell, rc::Rc};

use gpui::{
    AnyElement, App, ElementId, Entity, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Pixels, Role, Size, StatefulInteractiveElement, Styled, Subscription,
    WeakFocusHandle, Window, anchored, div, point, px,
};
use gpui_material_foundation::{Scrim, ThemeScope};
use gpui_material_theme::MotionSpec;

use super::dialog::DialogState;

/// Unified z-order priority table for Material overlay surfaces.
///
/// Prevents layering collisions between modal sheets, dialogs, snackbars,
/// menus (including nested submenus), and tooltips.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ZLayer(pub(crate) usize);

impl ZLayer {
    /// Base deferred priority for modal sheets (`10..40`).
    pub(crate) const SHEET_BASE: usize = 10;
    /// Base deferred priority for modal dialogs (`40..80`).
    pub(crate) const DIALOG_BASE: usize = 40;
    /// Deferred priority for snackbars (`80`).
    pub(crate) const SNACKBAR: Self = Self(80);
    /// Base deferred priority for menus and nested submenus (`100..160`).
    pub(crate) const MENU_BASE: usize = 100;
    /// Deferred priority for tooltips (`160`), above menus and modals.
    pub(crate) const TOOLTIP: Self = Self(160);

    /// Resolves the z-layer for a modal sheet at nesting `layer`.
    pub(crate) const fn sheet(layer: usize) -> Self {
        Self(Self::SHEET_BASE.saturating_add(layer))
    }

    /// Resolves the z-layer for a modal dialog at nesting `layer`.
    pub(crate) const fn dialog(layer: usize) -> Self {
        Self(Self::DIALOG_BASE.saturating_add(layer))
    }

    /// Resolves the z-layer for a menu or submenu at `depth`.
    pub(crate) const fn menu(depth: usize) -> Self {
        Self(Self::MENU_BASE.saturating_add(depth))
    }

    /// Returns the raw GPUI `deferred` priority value.
    pub(crate) const fn priority(self) -> usize {
        self.0
    }
}

/// Derives a finite presence duration in milliseconds from a Material [`MotionSpec`].
pub(crate) fn motion_duration_ms(spec: MotionSpec) -> u64 {
    match spec {
        MotionSpec::Instant => 0,
        MotionSpec::Tween { duration_ms, .. } => u64::from(duration_ms),
        MotionSpec::Spring(spring) => {
            if !spring.stiffness.is_finite()
                || !spring.damping_ratio.is_finite()
                || spring.stiffness <= 0.
                || spring.damping_ratio <= 0.
            {
                return 200;
            }
            let decay = spring.damping_ratio * spring.stiffness.sqrt();
            let secs = 4.6 / decay;
            if secs.is_finite() {
                (secs * 1000.).round().clamp(80., 800.) as u64
            } else {
                200
            }
        }
    }
}

/// Shared deferred host for modal scrims and exit-animation shells.
pub(crate) struct ModalHost {
    id: ElementId,
    z_layer: ZLayer,
    viewport: Size<Pixels>,
    centered: bool,
    clip: bool,
}

impl ModalHost {
    pub(crate) fn new(id: ElementId, z_layer: ZLayer, viewport: Size<Pixels>) -> Self {
        Self {
            id,
            z_layer,
            viewport,
            centered: false,
            clip: false,
        }
    }

    pub(crate) fn centered(mut self) -> Self {
        self.centered = true;
        self
    }

    pub(crate) fn clip(mut self) -> Self {
        self.clip = true;
        self
    }

    pub(crate) fn render_exiting<E: StatefulInteractiveElement + ParentElement + IntoElement>(
        self,
        suffix: &'static str,
        state: &Entity<DialogState>,
        scrim: Scrim,
        panel: E,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let mut container = div()
            .id((self.id.clone(), suffix))
            .absolute()
            .top_0()
            .left_0()
            .w(self.viewport.width)
            .h(self.viewport.height);
        if self.centered {
            container = container.flex().items_center().justify_center();
        }
        if self.clip {
            container = container.overflow_hidden();
        }
        gpui::deferred(
            anchored().position(point(px(0.), px(0.))).child(
                container
                    .child(ThemeScope::inherit(scrim, window, cx))
                    .child(ThemeScope::inherit(
                        exiting_panel(panel, &self.id, state, window, cx),
                        window,
                        cx,
                    )),
            ),
        )
        .with_priority(self.z_layer.priority())
        .into_any_element()
    }
}

/// Keeps a closing modal's layout mounted while redirecting focus outside and occluding hits.
pub(crate) fn exiting_panel<E: StatefulInteractiveElement + ParentElement>(
    panel: E,
    id: &ElementId,
    state: &Entity<DialogState>,
    window: &mut Window,
    cx: &mut App,
) -> E {
    let focus = state.read(cx).focus.clone();
    // The body can contain caller-owned controls. Keep their layout through
    // exit, but send focus back outside the panel and cover its hit targets.
    let guard = window.use_keyed_state((id.clone(), "exiting-focus"), cx, |_, _| {
        let previous = Rc::new(RefCell::new(None::<WeakFocusHandle>));
        (previous, None::<Subscription>)
    });
    let target = window
        .focused(cx)
        .filter(|handle| !focus.contains(handle, window));
    guard.update(cx, |(previous, guard), cx| {
        if let Some(target) = target {
            *previous.borrow_mut() = Some(target.downgrade());
        }
        if guard.is_none() {
            let previous = previous.clone();
            let state = state.downgrade();
            *guard = Some(window.on_focus_in(&focus, cx, move |window, cx| {
                // A new session can focus the container before the exit tree is redrawn.
                if state
                    .upgrade()
                    .is_some_and(|state| state.read(cx).is_open())
                {
                    return;
                }
                if let Some(handle) = previous
                    .borrow()
                    .as_ref()
                    .and_then(WeakFocusHandle::upgrade)
                {
                    handle.focus(window, cx);
                } else {
                    window.blur(cx);
                }
            }));
        }
    });
    panel
        .role(Role::Group)
        .track_focus(&focus)
        .capture_key_down(|_, _, cx| cx.stop_propagation())
        .a11y_synthetic_children(|tree| tree.parent_node().set_hidden())
        .child(div().absolute().inset_0().occlude())
}

/// Supplies the hit target shared by modal sheet and dialog scrims.
pub(crate) fn modal_backdrop(
    scrim: Scrim,
    dismiss: impl Fn(&mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .absolute()
        .inset_0()
        .occlude()
        .on_any_mouse_down(move |event, window, cx| {
            cx.stop_propagation();
            if event.button == MouseButton::Left {
                dismiss(window, cx);
            }
        })
        .on_scroll(|_, _, cx| cx.stop_propagation())
        .child(scrim)
}

#[cfg(test)]
mod tests {
    use gpui_material_theme::{MotionKind, MotionScheme, MotionSpeed, Spring};

    use super::*;

    #[test]
    fn z_layers_are_strictly_ordered_without_collisions() {
        assert!(ZLayer::sheet(0) < ZLayer::sheet(1));
        assert!(ZLayer::sheet(10) < ZLayer::dialog(0));
        assert!(ZLayer::dialog(0) < ZLayer::dialog(1));
        assert!(ZLayer::dialog(10) < ZLayer::SNACKBAR);
        assert!(ZLayer::SNACKBAR < ZLayer::menu(0));
        assert!(ZLayer::menu(0) < ZLayer::menu(1));
        assert!(ZLayer::menu(10) < ZLayer::TOOLTIP);
    }

    #[test]
    fn motion_duration_respects_reduced_motion_and_spring_schemes() {
        assert_eq!(motion_duration_ms(MotionSpec::Instant), 0);
        assert_eq!(
            motion_duration_ms(MotionSpec::Tween {
                duration_ms: 175,
                easing: gpui_material_theme::CubicBezier {
                    x1: 0.,
                    y1: 0.,
                    x2: 1.,
                    y2: 1.,
                },
            }),
            175
        );
        let fast = motion_duration_ms(MotionScheme::Standard.spec(
            MotionKind::Spatial,
            MotionSpeed::Fast,
            false,
        ));
        let slow = motion_duration_ms(MotionScheme::Standard.spec(
            MotionKind::Spatial,
            MotionSpeed::Slow,
            false,
        ));
        assert!(fast >= 80 && slow > fast && slow <= 800);
        assert_eq!(
            motion_duration_ms(MotionSpec::Spring(Spring {
                damping_ratio: f32::NAN,
                stiffness: 100.,
            })),
            200
        );
    }
}
