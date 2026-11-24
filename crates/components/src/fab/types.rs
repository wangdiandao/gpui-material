use gpui::{
    AnyElement, App, ElementId, Entity, FocusHandle, IntoElement, Point, SharedString, Window,
    point,
};
use gpui_material_foundation::{InteractionSource, OverlayMotion, animate, current_material_theme};
use gpui_material_theme::{
    ColorRole, CornerRole, Direction, Elevation, MotionKind, MotionSpec, MotionSpeed, TypeRole,
};

use crate::action::{
    Action, ActionControl, ActionLayout, ActionStyle, Activate, DisabledStyle, InvokeAction,
};
use crate::control_style::ButtonStyle;

/// Expressive FAB sizes. Small is the 56 dp FAB and Small Extended FAB.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FabSize {
    /// 56 dp height (default).
    #[default]
    Small,
    /// 80 dp height.
    Medium,
    /// 96 dp height.
    Large,
}

impl FabSize {
    pub(super) fn metrics(self) -> (f32, f32, CornerRole, f32, f32, TypeRole) {
        // FabBaseline, FabMedium/Large, and ExtendedFabSmall/Medium/Large.
        match self {
            Self::Small => (56., 24., CornerRole::Large, 16., 8., TypeRole::TitleMedium),
            Self::Medium => (
                80.,
                28.,
                CornerRole::LargeIncreased,
                26.,
                12.,
                TypeRole::TitleLarge,
            ),
            Self::Large => (
                96.,
                32.,
                CornerRole::ExtraLarge,
                28.,
                16.,
                TypeRole::HeadlineSmall,
            ),
        }
    }
}

/// Material color mapping for [`super::Fab`] and [`super::ExtendedFab`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FabColor {
    /// `PrimaryContainer` / `OnPrimaryContainer` (default).
    #[default]
    Primary,
    /// `SecondaryContainer` / `OnSecondaryContainer`.
    Secondary,
    /// `TertiaryContainer` / `OnTertiaryContainer`.
    Tertiary,
    /// High-contrast `Primary` / `OnPrimary`.
    PrimaryStrong,
    /// High-contrast `Secondary` / `OnSecondary`.
    SecondaryStrong,
    /// High-contrast `Tertiary` / `OnTertiary`.
    TertiaryStrong,
}

/// Logical pivot for the FAB's visibility transition.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FabAlignment {
    /// Top-leading corner (top-left in LTR, top-right in RTL).
    TopStart,
    /// Top-center edge.
    TopCenter,
    /// Top-trailing corner (top-right in LTR, top-left in RTL).
    TopEnd,
    /// Vertical center, leading edge.
    CenterStart,
    /// Center of the FAB footprint (default).
    #[default]
    Center,
    /// Vertical center, trailing edge.
    CenterEnd,
    /// Bottom-leading corner (bottom-left in LTR, bottom-right in RTL).
    BottomStart,
    /// Bottom-center edge.
    BottomCenter,
    /// Bottom-trailing corner (bottom-right in LTR, bottom-left in RTL).
    BottomEnd,
}

impl FabAlignment {
    pub(super) fn pivot(self, direction: Direction) -> Point<f32> {
        use FabAlignment::*;
        let x = match self {
            TopStart | CenterStart | BottomStart => 0.,
            TopCenter | Center | BottomCenter => 0.5,
            TopEnd | CenterEnd | BottomEnd => 1.,
        };
        let y = match self {
            TopStart | TopCenter | TopEnd => 0.,
            CenterStart | Center | CenterEnd => 0.5,
            BottomStart | BottomCenter | BottomEnd => 1.,
        };
        point(
            if direction == Direction::Rtl {
                1. - x
            } else {
                x
            },
            y,
        )
    }
}

/// Appearance and disappearance motion for a controlled FAB.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FabVisibilityAnimation {
    /// Logical anchor point about which the FAB scales during show/hide transitions.
    pub alignment: FabAlignment,
    /// The ending scale of a hide and the starting scale of a show.
    /// Negative, non-finite, or overflowing values use the default of 0.2.
    pub target_scale: f32,
    /// Defaults to the window theme's Fast Spatial motion.
    pub scale_motion: Option<MotionSpec>,
    /// Defaults to the window theme's Fast Effects motion.
    pub alpha_motion: Option<MotionSpec>,
}

impl Default for FabVisibilityAnimation {
    fn default() -> Self {
        Self {
            alignment: FabAlignment::Center,
            target_scale: 0.2,
            scale_motion: None,
            alpha_motion: None,
        }
    }
}

impl FabVisibilityAnimation {
    /// Resolves scale, alpha, and the physical pivot for related FAB components.
    #[doc(hidden)]
    pub fn frame(
        self,
        id: &ElementId,
        visible: bool,
        extent: f32,
        window: &mut Window,
        cx: &mut App,
    ) -> (f32, f32, Point<f32>) {
        let sample = self.sample(id, visible, extent, window, cx);
        let direction = current_material_theme(window, cx).layout.direction;
        (sample.scale, sample.alpha, self.alignment.pivot(direction))
    }

    pub(super) fn sample(
        self,
        id: &ElementId,
        visible: bool,
        extent: f32,
        window: &mut Window,
        cx: &mut App,
    ) -> OverlayMotion {
        let scheme = current_material_theme(window, cx).motion;
        let reduced = cx.reduce_motion();
        let mut progress = |channel, kind, spec: Option<MotionSpec>| {
            let spec = if reduced {
                MotionSpec::Instant
            } else {
                spec.unwrap_or_else(|| scheme.spec(kind, MotionSpeed::Fast, false))
            };
            animate((id.clone(), channel), f32::from(visible), spec, window, cx)
        };
        let spatial = progress(
            "fab-visibility-scale",
            MotionKind::Spatial,
            self.scale_motion,
        );
        let alpha = progress(
            "fab-visibility-alpha",
            MotionKind::Effects,
            self.alpha_motion,
        );
        let target = if self.target_scale >= 0. && (self.target_scale * extent).is_finite() {
            self.target_scale
        } else {
            Self::default().target_scale
        };
        let scale = ((1. - spatial) * target + spatial).max(0.);
        OverlayMotion {
            scale: if (scale * extent).is_finite() {
                scale
            } else {
                1.
            },
            alpha: alpha.clamp(0., 1.),
        }
    }
}

pub(super) struct FabContent {
    pub(super) id: ElementId,
    pub(super) name: SharedString,
    pub(super) size: FabSize,
    pub(super) color: FabColor,
    pub(super) disabled: bool,
    pub(super) focus: Option<FocusHandle>,
    pub(super) on_activate: Option<Activate>,
}

impl FabContent {
    pub(super) fn new(id: impl Into<ElementId>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            size: FabSize::default(),
            color: FabColor::default(),
            disabled: false,
            focus: None,
            on_activate: None,
        }
    }

    pub(super) fn style(&self) -> ActionStyle {
        let (height, _, corner, _, _, _) = self.size.metrics();
        let (background, foreground) = match self.color {
            FabColor::Primary => (ColorRole::PrimaryContainer, ColorRole::OnPrimaryContainer),
            FabColor::Secondary => (
                ColorRole::SecondaryContainer,
                ColorRole::OnSecondaryContainer,
            ),
            FabColor::Tertiary => (ColorRole::TertiaryContainer, ColorRole::OnTertiaryContainer),
            FabColor::PrimaryStrong => (ColorRole::Primary, ColorRole::OnPrimary),
            FabColor::SecondaryStrong => (ColorRole::Secondary, ColorRole::OnSecondary),
            FabColor::TertiaryStrong => (ColorRole::Tertiary, ColorRole::OnTertiary),
        };
        ActionStyle {
            background: Some(background),
            background_color: None,
            foreground,
            // Disabled FABs are a desktop extension; preserve the established
            // Material disabled-container treatment for this extension.
            disabled: DisabledStyle {
                foreground: ColorRole::OnSurface,
                background: Some((ColorRole::OnSurface, 0.12)),
                outline: (ColorRole::OnSurface, 0.12),
            },
            height,
            min_width: height,
            corner,
            pressed_corner: corner,
            selected_corner: None,
            corner_motion: (MotionKind::Spatial, MotionSpeed::Fast),
            outline: 0.,
            outline_role: ColorRole::OutlineVariant,
            elevation: Elevation::Level3,
            hover_elevation: Elevation::Level4,
        }
    }

    pub(super) fn action(self, content: impl IntoElement) -> ActionControl {
        ActionControl {
            style: self.style(),
            id: self.id,
            label: self.name,
            focus: self.focus,
            disabled: self.disabled,
            action: Action::Invoke(self.on_activate.map(InvokeAction::Full)),
            content: content.into_any_element(),
        }
    }
}

#[derive(Default)]
pub(super) struct FabAppearance {
    pub(super) style: ButtonStyle,
    pub(super) interaction_source: Option<Entity<InteractionSource>>,
    pub(super) visible: Option<bool>,
    pub(super) animation: FabVisibilityAnimation,
}

impl FabAppearance {
    pub(super) fn motion(
        &self,
        id: &ElementId,
        extent: f32,
        window: &mut Window,
        cx: &mut App,
    ) -> OverlayMotion {
        self.visible.map_or(
            OverlayMotion {
                scale: 1.,
                alpha: 1.,
            },
            |visible| self.animation.sample(id, visible, extent, window, cx),
        )
    }

    pub(super) fn interactive(
        &self,
        action: &ActionControl,
        motion: OverlayMotion,
        window: &mut Window,
        cx: &mut App,
    ) -> bool {
        let interactive = self.visible != Some(false) && motion.alpha > 0. && motion.scale > 0.;
        if !interactive {
            action.clear_interaction(window, cx);
        }
        interactive
    }

    pub(super) fn control(
        &self,
        action: ActionControl,
        scale: f32,
        interactive: bool,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        action.render_layout(
            scale,
            ActionLayout {
                tab_stop: interactive,
                decorative: !interactive,
                appearance: self.style,
                ..Default::default()
            },
            self.interaction_source.as_ref().filter(|_| interactive),
            window,
            cx,
        )
    }
}
