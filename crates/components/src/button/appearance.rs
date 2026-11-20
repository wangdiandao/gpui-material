use gpui_material_theme::{ColorRole, CornerRole, Elevation, MotionKind, MotionSpeed};

use crate::action::{ActionStyle, DisabledStyle};

use super::{ButtonVariant, IconButtonVariant, Metrics};

pub(super) struct Appearance {
    pub(super) background: Option<ColorRole>,
    pub(super) foreground: ColorRole,
    pub(super) outlined: bool,
    pub(super) disabled: DisabledStyle,
    pub(super) elevation: Elevation,
    pub(super) hover_elevation: Elevation,
}

impl Appearance {
    pub(super) fn style(self, metrics: Metrics, corner: CornerRole, min_width: f32) -> ActionStyle {
        ActionStyle {
            background: self.background,
            background_color: None,
            foreground: self.foreground,
            disabled: self.disabled,
            height: metrics.height,
            min_width,
            corner,
            pressed_corner: metrics.pressed,
            selected_corner: None,
            corner_motion: (MotionKind::Spatial, MotionSpeed::Fast),
            outline: if self.outlined { metrics.outline } else { 0. },
            elevation: self.elevation,
            hover_elevation: self.hover_elevation,
        }
    }
}

pub(super) fn button_appearance(variant: ButtonVariant, selected: Option<bool>) -> Appearance {
    use ColorRole::*;
    use Elevation::*;
    let (background, foreground, outlined, elevation, hover_elevation) = match (variant, selected) {
        (ButtonVariant::Elevated, Some(true)) => (Some(Primary), OnPrimary, false, Level1, Level2),
        (ButtonVariant::Elevated, _) => (Some(SurfaceContainerLow), Primary, false, Level1, Level2),
        (ButtonVariant::Filled, Some(false)) => (
            Some(SurfaceContainer),
            OnSurfaceVariant,
            false,
            Level0,
            Level1,
        ),
        (ButtonVariant::Filled, _) => (Some(Primary), OnPrimary, false, Level0, Level1),
        (ButtonVariant::FilledTonal, Some(true)) => {
            (Some(Secondary), OnSecondary, false, Level0, Level1)
        }
        (ButtonVariant::FilledTonal, _) => (
            Some(SecondaryContainer),
            OnSecondaryContainer,
            false,
            Level0,
            Level1,
        ),
        (ButtonVariant::Outlined, Some(true)) => (
            Some(InverseSurface),
            InverseOnSurface,
            false,
            Level0,
            Level0,
        ),
        (ButtonVariant::Outlined, _) => (None, OnSurfaceVariant, true, Level0, Level0),
        // Compose overrides the outdated TextButtonTokens.LabelColor to Primary.
        (ButtonVariant::Text, _) => (None, Primary, false, Level0, Level0),
    };
    Appearance {
        background,
        foreground,
        outlined,
        disabled: DisabledStyle {
            // FilledTonalButton retains the baseline tokens; the expressive
            // toggle uses TonalButtonTokens instead.
            foreground: if variant == ButtonVariant::FilledTonal && selected.is_none() {
                OnSurface
            } else {
                OnSurfaceVariant
            },
            background: match (variant, selected) {
                (ButtonVariant::Outlined, Some(_)) => Some((OutlineVariant, 0.10)),
                (ButtonVariant::Outlined | ButtonVariant::Text, _) => None,
                (ButtonVariant::FilledTonal, None) => Some((OnSurface, 0.12)),
                _ => Some((OnSurface, 0.10)),
            },
            outline: (OutlineVariant, 0.10),
        },
        elevation,
        hover_elevation,
    }
}

pub(super) fn icon_appearance(variant: IconButtonVariant, selected: Option<bool>) -> Appearance {
    let mut appearance = match variant {
        IconButtonVariant::Standard => button_appearance(ButtonVariant::Text, selected),
        IconButtonVariant::Filled => button_appearance(ButtonVariant::Filled, selected),
        IconButtonVariant::FilledTonal => button_appearance(ButtonVariant::FilledTonal, selected),
        IconButtonVariant::Outlined => button_appearance(ButtonVariant::Outlined, selected),
    };
    if variant == IconButtonVariant::Standard {
        appearance.foreground = if selected == Some(true) {
            ColorRole::Primary
        } else {
            ColorRole::OnSurfaceVariant
        };
    }
    appearance.disabled = DisabledStyle {
        foreground: ColorRole::OnSurface,
        // Compose icon toggle disabled colors do not depend on selection.
        background: match variant {
            IconButtonVariant::Filled | IconButtonVariant::FilledTonal => {
                Some((ColorRole::OnSurface, 0.10))
            }
            IconButtonVariant::Standard | IconButtonVariant::Outlined => None,
        },
        outline: (ColorRole::OutlineVariant, 0.38),
    };
    appearance.hover_elevation = Elevation::Level0;
    appearance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_action_and_standard_icon_keep_distinct_compose_colors() {
        // Button.kt corrects TextButtonTokens.LabelColor, whereas a standard
        // icon remains neutral until selected (IconButtonDefaults.kt).
        assert_eq!(
            button_appearance(ButtonVariant::Text, None).foreground,
            ColorRole::Primary
        );
        for selected in [None, Some(false)] {
            assert_eq!(
                icon_appearance(IconButtonVariant::Standard, selected).foreground,
                ColorRole::OnSurfaceVariant
            );
        }
        assert_eq!(
            icon_appearance(IconButtonVariant::Standard, Some(true)).foreground,
            ColorRole::Primary
        );
    }

    #[test]
    fn disabled_tokens_are_not_inferred_from_enabled_selection_colors() {
        use ColorRole::*;
        for selected in [Some(false), Some(true)] {
            let outlined = button_appearance(ButtonVariant::Outlined, selected);
            assert_eq!(outlined.disabled.background, Some((OutlineVariant, 0.10)));
            assert_eq!(outlined.disabled.foreground, OnSurfaceVariant);
            assert_eq!(outlined.disabled.outline, (OutlineVariant, 0.10));
            let icon = icon_appearance(IconButtonVariant::Outlined, selected);
            assert_eq!(icon.disabled.background, None);
            assert_eq!(icon.disabled.foreground, OnSurface);
            assert_eq!(icon.disabled.outline, (OutlineVariant, 0.38));
        }
        assert_eq!(
            button_appearance(ButtonVariant::Outlined, None)
                .disabled
                .background,
            None
        );
        assert_eq!(
            button_appearance(ButtonVariant::Filled, None)
                .disabled
                .background,
            Some((OnSurface, 0.10))
        );
        // FilledTonalButton and its expressive toggle reference different token
        // generations in the pinned Compose release.
        assert_eq!(
            button_appearance(ButtonVariant::FilledTonal, None)
                .disabled
                .background,
            Some((OnSurface, 0.12))
        );
        assert_eq!(
            button_appearance(ButtonVariant::FilledTonal, Some(false))
                .disabled
                .background,
            Some((OnSurface, 0.10))
        );
        assert_eq!(
            icon_appearance(IconButtonVariant::FilledTonal, None)
                .disabled
                .background,
            Some((OnSurface, 0.10))
        );
    }
}
