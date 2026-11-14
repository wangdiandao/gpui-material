use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, FocusHandle, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, anchored, deferred, div, px,
};
use gpui_material_foundation::{animate, color, logical_anchor, window_material_theme};
use gpui_material_theme::{
    ColorRole, CornerRole, Direction, Elevation, MotionKind, MotionSpeed, TypeRole,
    TypographyEmphasis,
};

use crate::action::{Action, ActionControl, ActionStyle, Activate, DisabledStyle, icon_slot};

/// Expressive FAB sizes. Small is the 56 dp FAB and Small Extended FAB.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FabSize {
    #[default]
    Small,
    Medium,
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FabColor {
    #[default]
    Primary,
    Secondary,
    Tertiary,
    PrimaryStrong,
    SecondaryStrong,
    TertiaryStrong,
}

struct FabContent {
    id: ElementId,
    name: SharedString,
    icon: AnyElement,
    size: FabSize,
    color: FabColor,
    disabled: bool,
    focus: Option<FocusHandle>,
    on_activate: Option<Activate>,
}

impl FabContent {
    fn new(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        icon: impl IntoElement,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            icon: icon.into_any_element(),
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
            elevation: Elevation::Level3,
            hover_elevation: Elevation::Level4,
        }
    }
}

macro_rules! fab_builders {
    () => {
        pub fn size(mut self, size: FabSize) -> Self {
            self.content.size = size;
            self
        }

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

        pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
            self.content.focus = Some(focus.clone());
            self
        }

        pub fn on_activate(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
            self.content.on_activate = Some(Rc::new(handler));
            self
        }
    };
}

/// An elevated icon action with a required accessibility name. Mount it beside
/// the scroll viewport to keep its position fixed while content scrolls.
#[derive(IntoElement)]
pub struct Fab {
    content: FabContent,
}

impl Fab {
    pub fn new(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        icon: impl IntoElement,
    ) -> Self {
        Self {
            content: FabContent::new(id, name, icon),
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
        ActionControl {
            id: self.content.id,
            label: self.content.name,
            focus: self.content.focus,
            disabled: self.content.disabled,
            action: Action::Invoke(self.content.on_activate),
            style,
            content: icon_slot(self.content.icon, icon_size).into_any_element(),
        }
        .render(window, cx)
    }
}

/// An elevated icon and label action. Expansion is controlled by the caller.
/// Both presentations share the same focus handle, action and accessible name.
#[derive(IntoElement)]
pub struct ExtendedFab {
    content: FabContent,
    expanded: bool,
}

impl ExtendedFab {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        icon: impl IntoElement,
    ) -> Self {
        Self {
            content: FabContent::new(id, label, icon),
            expanded: true,
        }
    }

    fab_builders!();

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }
}

impl RenderOnce for ExtendedFab {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let style = self.content.style();
        let (_, icon_size, _, padding, gap, role) = self.content.size.metrics();
        let theme = window_material_theme(window, cx);
        let direction = theme.layout.direction;
        let foreground = if self.content.disabled {
            color(theme.colors.color(ColorRole::OnSurface)).opacity(0.38)
        } else {
            color(theme.colors.color(style.foreground))
        };
        let reduced = cx.reduce_motion();
        let motion = theme
            .motion
            .spec(MotionKind::Spatial, MotionSpeed::Fast, reduced);
        let effects = theme
            .motion
            .spec(MotionKind::Effects, MotionSpeed::Fast, reduced);
        let progress = animate(
            ElementId::from((self.content.id.clone(), "expansion")),
            f32::from(self.expanded),
            motion,
            window,
            cx,
        )
        .clamp(0., 1.);
        let opacity = animate(
            (self.content.id.clone(), "label-opacity"),
            f32::from(self.expanded),
            effects,
            window,
            cx,
        )
        .clamp(0., 1.);
        // Interpolate from a square FAB with its icon centered to the label layout.
        let collapsed_padding = (style.height - icon_size) / 2.;
        let start_padding = collapsed_padding + (padding - collapsed_padding) * progress;
        let typography = role.style(TypographyEmphasis::Baseline);
        let mut row = div()
            .flex()
            .items_center()
            .min_w_0()
            .max_w_full()
            .px(px(start_padding))
            .gap(px(gap * progress));
        if direction == Direction::Rtl {
            row = row.flex_row_reverse();
        }
        row = row.child(icon_slot(self.content.icon, icon_size));
        if progress > 0. {
            row = row.child(crate::text::single_line(
                self.content.name.clone(),
                typography,
                1.,
                foreground,
                (None, progress, opacity),
                window,
                cx,
            ));
        }
        ActionControl {
            id: self.content.id,
            label: self.content.name,
            focus: self.content.focus,
            disabled: self.content.disabled,
            action: Action::Invoke(self.content.on_activate),
            style,
            content: row.into_any_element(),
        }
        .render(window, cx)
    }
}

/// Places a FAB inside the current window viewport at logical bottom-end.
/// Mount this element beside the scroll owner so it stays visible while scrolling.
#[derive(IntoElement)]
pub struct FloatingAction {
    action: AnyElement,
    margin: gpui::Pixels,
}

impl FloatingAction {
    pub fn new(action: impl IntoElement) -> Self {
        Self {
            action: action.into_any_element(),
            margin: px(16.),
        }
    }

    pub fn margin(mut self, margin: gpui::Pixels) -> Self {
        self.margin = margin.max(px(0.));
        self
    }
}

impl RenderOnce for FloatingAction {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let viewport = window.viewport_size();
        let direction = window_material_theme(window, cx).layout.direction;
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
                .child(
                    div()
                        .max_w((viewport.width - margin * 2.).max(px(0.)))
                        .child(self.action),
                ),
        )
    }
}
