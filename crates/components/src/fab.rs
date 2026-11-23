use std::rc::Rc;

use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, App, AvailableSpace, ElementId, Empty, Entity, FocusHandle, IntoElement,
    ParentElement, Pixels, Point, RenderOnce, SharedString, Styled, Window, anchored, canvas,
    deferred, div, point, px, size,
};
use gpui_material_foundation::{
    InteractionSource, ThemeScope, animate, current_material_theme, logical_anchor,
};
use gpui_material_theme::{
    ColorRole, CornerRole, CornerShape, Direction, Elevation, MotionKind, MotionSpec, MotionSpeed,
    TypeRole, TypographyEmphasis,
};

use crate::action::{
    Action, ActionControl, ActionLayout, ActionStyle, Activate, DisabledStyle, icon_slot,
};
use crate::button_style::{ButtonColors, ButtonElevations, ButtonStyle};
use crate::motion::OverlayMotion;

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
    fn metrics(self) -> (f32, f32, CornerRole, f32, f32, TypeRole) {
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

/// Material color mapping for [`Fab`] and [`ExtendedFab`].
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
    fn pivot(self, direction: Direction) -> Point<f32> {
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

    fn sample(
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

struct FabContent {
    id: ElementId,
    name: SharedString,
    size: FabSize,
    color: FabColor,
    disabled: bool,
    focus: Option<FocusHandle>,
    on_activate: Option<Activate>,
}

impl FabContent {
    fn new(id: impl Into<ElementId>, name: impl Into<SharedString>) -> Self {
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

    fn style(&self) -> ActionStyle {
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
            corner_motion: (
                gpui_material_theme::MotionKind::Spatial,
                gpui_material_theme::MotionSpeed::Fast,
            ),
            outline: 0.,
            outline_role: ColorRole::OutlineVariant,
            elevation: Elevation::Level3,
            hover_elevation: Elevation::Level4,
        }
    }

    fn action(self, content: impl IntoElement) -> ActionControl {
        ActionControl {
            style: self.style(),
            id: self.id,
            label: self.name,
            focus: self.focus,
            disabled: self.disabled,
            action: Action::Invoke(self.on_activate),
            content: content.into_any_element(),
        }
    }
}

#[derive(Default)]
struct FabAppearance {
    style: ButtonStyle,
    interaction_source: Option<Entity<InteractionSource>>,
    visible: Option<bool>,
    animation: FabVisibilityAnimation,
}

impl FabAppearance {
    fn motion(
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

    fn interactive(
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

    fn control(
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

macro_rules! fab_builders {
    () => {
        /// Sets the Expressive FAB size tier. Defaults to [`FabSize::Small`].
        pub fn size(mut self, size: FabSize) -> Self {
            self.content.size = size;
            self
        }

        /// Sets the FAB color palette preset. Defaults to [`FabColor::Primary`].
        pub fn color(mut self, color: FabColor) -> Self {
            self.content.color = color;
            self
        }

        /// Desktop compatibility extension. Material recommends hiding a FAB
        /// when its primary action is unavailable instead of disabling it.
        pub fn disabled(mut self, disabled: bool) -> Self {
            self.content.disabled = disabled;
            self
        }

        /// Uses a caller-owned focus handle for keyboard traversal and focus state.
        pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
            self.content.focus = Some(focus.clone());
            self
        }

        /// Invoked once when the FAB is activated via pointer, keyboard, or assistive technology.
        pub fn on_activate(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
            self.content.on_activate = Some(Rc::new(handler));
            self
        }

        /// Overrides the container shape for all interaction states.
        pub fn shape(mut self, shape: CornerShape) -> Self {
            self.appearance.style.shapes.resting = Some(shape);
            self.appearance.style.shapes.pressed = Some(shape);
            self
        }

        /// Sets the enabled and disabled container and content colors.
        pub fn colors(mut self, colors: ButtonColors) -> Self {
            self.appearance.style.colors = colors;
            self
        }

        /// Overrides the elevation for each interaction state.
        pub fn elevations(mut self, elevations: ButtonElevations) -> Self {
            self.appearance.style.elevations = elevations;
            self
        }

        /// Reports interaction state through a caller-owned source.
        pub fn interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
            self.appearance.interaction_source = Some(source.clone());
            self
        }

        /// Applies the caller's visibility state. Use a stable ID across updates.
        pub fn visible(mut self, visible: bool) -> Self {
            self.appearance.visible = Some(visible);
            self
        }

        /// Sets the pivot, hidden scale, and independent scale and alpha motions.
        pub fn visibility_animation(mut self, animation: FabVisibilityAnimation) -> Self {
            self.appearance.animation = animation;
            self
        }
    };
}

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

/// An elevated action with a label or custom content and an optional icon.
/// All presentations share one action, accessible name, and focus handle.
#[derive(IntoElement)]
pub struct ExtendedFab {
    content: FabContent,
    icon: Option<AnyElement>,
    custom_content: Option<AnyElement>,
    expanded: bool,
    appearance: FabAppearance,
}

impl ExtendedFab {
    /// Creates an action with a leading icon and a label.
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        icon: impl IntoElement,
    ) -> Self {
        Self {
            icon: Some(icon.into_any_element()),
            ..Self::text(id, label)
        }
    }

    /// Creates a text-only action. The label also supplies its accessible name.
    pub fn text(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            content: FabContent::new(id, label),
            icon: None,
            custom_content: None,
            expanded: true,
            appearance: FabAppearance::default(),
        }
    }

    fab_builders!();

    /// Replaces the rendered label with decorative content. The constructor's
    /// accessible name and optional icon still belong to the action.
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.custom_content = Some(content.into_any_element());
        self
    }

    /// Applies the caller's expansion state. An icon-bearing action collapses
    /// to a square. Iconless actions retain their content in both states.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }
}

impl RenderOnce for ExtendedFab {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (height, icon, _, padding, gap, role) = self.content.size.metrics();
        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let extent = theme.layout.density.interactive_size(height);
        let visual_height = theme.layout.density.visual_size(height);
        let scheme = theme.motion;
        let motion = self.appearance.motion(
            &self.content.id,
            extent.max(f32::from(window.viewport_size().width)),
            window,
            cx,
        );
        let action = self.content.action(Empty);
        let interactive = self.appearance.interactive(&action, motion, window, cx);
        if motion.alpha == 0. {
            return Empty.into_any_element();
        }
        let (progress, opacity) = if self.icon.is_some() {
            let reduced = cx.reduce_motion();
            let mut sample = |channel, kind| {
                animate(
                    (action.id.clone(), channel),
                    f32::from(self.expanded),
                    scheme.spec(kind, MotionSpeed::Fast, reduced),
                    window,
                    cx,
                )
                .clamp(0., 1.)
            };
            (
                sample("expansion", MotionKind::Spatial),
                sample("label-opacity", MotionKind::Effects),
            )
        } else {
            (1., 1.)
        };
        let (icon_size, padding, gap) = if self.icon.is_some() {
            let collapsed_padding = (height - icon) / 2.;
            (
                icon,
                collapsed_padding + (padding - collapsed_padding) * progress,
                gap * progress,
            )
        } else {
            (0., padding, 0.)
        };
        let ink = self
            .appearance
            .style
            .colors
            .content(action.disabled, false)
            .unwrap_or_else(|| action.style.foreground(action.disabled, window, cx));
        let typography = role.style(TypographyEmphasis::Baseline);
        let mut custom_content = self.custom_content;
        let mut pre_shaped = None;
        let label_width = if progress == 0. {
            px(0.)
        } else if let Some(content) = &mut custom_content {
            window
                .with_text_style(
                    Some(gpui::TextStyleRefinement {
                        color: Some(ink),
                        font_size: Some(px(typography.size).into()),
                        line_height: Some(px(typography.line_height).into()),
                        ..Default::default()
                    }),
                    |window| {
                        content.layout_as_root(
                            size(AvailableSpace::MaxContent, AvailableSpace::MaxContent),
                            window,
                            cx,
                        )
                    },
                )
                .width
        } else {
            let (clean, shaped) = crate::text::shape_single_line(
                action.label.clone(),
                typography,
                1.,
                ink,
                window,
                cx,
            );
            let width = shaped.line().width.ceil();
            pre_shaped = Some((clean, shaped));
            width
        };
        let fixed_width = px(icon_size + gap + padding * 2.);
        let width = (fixed_width + label_width * progress).max(px(extent));
        // Keep the nominal width through visibility scaling. Only the resolved
        // action is prepainted, so measurement creates no hitboxes or semantics.
        let control = canvas(
            move |bounds, window, cx| {
                if motion.scale == 0. {
                    return None;
                }
                let scale = if f32::from(bounds.size.width * motion.scale).is_finite() {
                    motion.scale
                } else {
                    1.
                };
                let motion = OverlayMotion { scale, ..motion };
                let pivot = self.appearance.animation.alignment.pivot(direction);
                let assigned_width = bounds.size.width * scale;
                let mut row = div()
                    .flex()
                    .items_center()
                    .w(assigned_width)
                    .px(px(padding * scale))
                    .gap(px(gap * scale))
                    .when(direction == Direction::Rtl, Styled::flex_row_reverse);
                if let Some(icon) = self.icon {
                    row = row.child(icon_slot(icon, icon_size * scale));
                }
                let mut action = action;
                let available = (bounds.size.width - fixed_width).max(px(0.));
                let label_width = (label_width * progress).min(available) * scale;
                if label_width > px(0.) {
                    if let Some(content) = custom_content {
                        row = row.child(custom_label(
                            content,
                            label_width,
                            px(visual_height * scale),
                            opacity,
                        ));
                    } else if scale == 1.
                        && let Some(shaped) = pre_shaped
                    {
                        row = row.child(crate::text::single_line_with_shaped(
                            shaped,
                            typography,
                            1.,
                            ink,
                            (Some(label_width / progress), progress, opacity),
                            window,
                            cx,
                        ));
                    } else {
                        row = row.child(crate::text::single_line(
                            action.label.clone(),
                            typography,
                            scale,
                            ink,
                            (Some(label_width / progress), progress, opacity),
                            window,
                            cx,
                        ));
                    }
                }
                action.content = row.into_any_element();
                let action = self
                    .appearance
                    .control(action, scale, interactive, window, cx);
                let mut target = div()
                    .w(assigned_width)
                    .opacity(motion.alpha)
                    .child(action)
                    .into_any_element();
                target.prepaint_as_root(
                    motion.origin(bounds, pivot),
                    bounds
                        .size
                        .map(|side| AvailableSpace::Definite(side * scale)),
                    window,
                    cx,
                );
                Some(target)
            },
            |_, target, window, cx| {
                if let Some(mut target) = target {
                    target.paint(window, cx);
                }
            },
        );
        div()
            .w(width)
            .h(px(extent))
            .min_w(px(extent))
            .min_h(px(extent))
            .max_w_full()
            .child(control.size_full())
            .into_any_element()
    }
}

fn custom_label(mut content: AnyElement, width: Pixels, height: Pixels, opacity: f32) -> gpui::Div {
    div()
        .w(width)
        .h(height)
        .min_w_0()
        .overflow_hidden()
        .opacity(opacity)
        .child(
            canvas(
                move |bounds, window, cx| {
                    let actual = content.layout_as_root(
                        bounds.size.map(AvailableSpace::Definite),
                        window,
                        cx,
                    );
                    let origin = point(
                        bounds.origin.x,
                        bounds.origin.y + ((bounds.size.height - actual.height) / 2.).max(px(0.)),
                    );
                    content.prepaint_at(origin, window, cx);
                    content
                },
                |_, mut content, window, cx| content.paint(window, cx),
            )
            .size_full(),
        )
}

/// Places a FAB inside the current window viewport at logical bottom-end.
/// Mount this element beside the scroll owner so it stays visible while scrolling.
#[derive(IntoElement)]
pub struct FloatingAction {
    action: AnyElement,
    margin: gpui::Pixels,
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
    pub fn margin(mut self, margin: gpui::Pixels) -> Self {
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
                .position(gpui::point(x, viewport.height - margin))
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
