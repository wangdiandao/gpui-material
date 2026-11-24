use gpui::{AnyElement, App, ElementId, Entity, FocusHandle, SharedString, Window};
use gpui_material_foundation::InteractionSource;
use gpui_material_theme::{CornerRole, TypeRole};

use crate::SemanticTone;
use crate::action::{Action, ActionControl, ActionLayout, ActionStyle};
use crate::control_style::{ButtonStyle, ControlSize};

#[derive(Clone, Copy)]
pub(super) struct Metrics {
    pub(super) height: f32,
    pub(super) icon: f32,
    pub(super) gap: f32,
    pub(super) padding: f32,
    pub(super) outline: f32,
    pub(super) square: CornerRole,
    pub(super) pressed: CornerRole,
    pub(super) label: TypeRole,
}

impl ControlSize {
    // Button{XSmall,Small,Medium,Large,XLarge}Tokens in Compose Material 3 1.5.0-rc01.
    pub(super) fn metrics(self) -> Metrics {
        use CornerRole::*;
        use TypeRole::*;
        let (height, icon, gap, padding, outline, square, pressed, label) = match self {
            Self::ExtraSmall => (32., 20., 8., 16., 1., Medium, Small, LabelLarge),
            Self::Small => (40., 20., 8., 16., 1., Medium, Small, LabelLarge),
            Self::Medium => (56., 24., 8., 24., 1., Large, Medium, TitleMedium),
            Self::Large => (96., 32., 12., 48., 2., ExtraLarge, Large, HeadlineSmall),
            Self::ExtraLarge => (136., 40., 16., 64., 3., ExtraLarge, Large, HeadlineLarge),
        };
        Metrics {
            height,
            icon,
            gap,
            padding,
            outline,
            square,
            pressed,
            label,
        }
    }

    pub(super) fn icon_metrics(self, width: IconButtonWidth) -> (Metrics, f32) {
        // Expressive icon buttons have their own icon and horizontal space.
        let mut metrics = self.metrics();
        let (icon, narrow, uniform, wide) = match self {
            Self::ExtraSmall => (20., 4., 6., 10.),
            Self::Small => (24., 4., 8., 14.),
            Self::Medium => (24., 12., 16., 24.),
            Self::Large => (32., 16., 32., 48.),
            Self::ExtraLarge => (40., 32., 48., 72.),
        };
        metrics.icon = icon;
        let space = match width {
            IconButtonWidth::Narrow => narrow,
            IconButtonWidth::Uniform => uniform,
            IconButtonWidth::Wide => wide,
        };
        (metrics, icon + 2. * space)
    }
}

/// Resting corner shape preset for Material buttons.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonShape {
    /// Pill-shaped full corner radius (default).
    #[default]
    Round,
    /// Size-dependent rounded-square corner radius.
    Square,
}

impl ButtonShape {
    pub(super) fn corner(self, square: CornerRole, invert: bool) -> CornerRole {
        if (self == Self::Round) != invert {
            CornerRole::Full
        } else {
            square
        }
    }
}

/// Visual emphasis style for [`super::Button`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonVariant {
    /// Raised low-emphasis surface with drop shadow.
    Elevated,
    /// High-emphasis solid filled container (default).
    #[default]
    Filled,
    /// Medium-emphasis tonal container fill.
    FilledTonal,
    /// Medium-emphasis outlined container with transparent fill.
    Outlined,
    /// Low-emphasis borderless text action.
    Text,
}

/// Visual emphasis style for [`super::ToggleButton`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToggleButtonVariant {
    /// Raised surface with drop shadow.
    Elevated,
    /// High-emphasis solid filled toggle (default).
    #[default]
    Filled,
    /// Medium-emphasis tonal container toggle.
    FilledTonal,
    /// Medium-emphasis outlined toggle.
    Outlined,
}

/// Visual emphasis style for [`super::IconButton`] and [`super::IconToggleButton`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum IconButtonVariant {
    /// Borderless, transparent icon button (default).
    #[default]
    Standard,
    /// High-emphasis solid filled icon button.
    Filled,
    /// Medium-emphasis tonal container icon button.
    FilledTonal,
    /// Medium-emphasis outlined icon button.
    Outlined,
}

/// Horizontal width preset for Expressive icon buttons.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum IconButtonWidth {
    /// Compact horizontal padding.
    Narrow,
    /// Balanced square-like horizontal padding (default).
    #[default]
    Uniform,
    /// Expanded pill-like horizontal padding.
    Wide,
}

pub(super) struct Common {
    pub(super) id: ElementId,
    pub(super) label: SharedString,
    pub(super) disabled: bool,
    pub(super) loading: bool,
    pub(super) focus: Option<FocusHandle>,
    pub(super) interaction_source: Option<Entity<InteractionSource>>,
    pub(super) appearance: ButtonStyle,
    pub(super) size: ControlSize,
    pub(super) shape: ButtonShape,
    pub(super) tone: SemanticTone,
}

impl Common {
    pub(super) fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            disabled: false,
            loading: false,
            focus: None,
            interaction_source: None,
            appearance: ButtonStyle::default(),
            size: ControlSize::default(),
            shape: ButtonShape::default(),
            tone: SemanticTone::default(),
        }
    }

    pub(super) fn render(
        self,
        content: AnyElement,
        style: ActionStyle,
        action: Action,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        self.render_scaled(content, style, action, 1., window, cx)
    }

    pub(super) fn render_scaled(
        self,
        content: AnyElement,
        style: ActionStyle,
        action: Action,
        scale: f32,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        ActionControl {
            id: self.id,
            label: self.label,
            focus: self.focus,
            disabled: self.disabled,
            action,
            style,
            content,
        }
        .render_layout(
            scale,
            ActionLayout {
                tab_stop: true,
                loading: self.loading,
                appearance: self.appearance,
                ..Default::default()
            },
            self.interaction_source.as_ref(),
            window,
            cx,
        )
    }
}
