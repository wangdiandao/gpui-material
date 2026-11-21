use gpui_material_theme::{ColorRole, CornerRole, Elevation, MotionKind, MotionSpeed};

use crate::action::{ActionStyle, DisabledStyle};

use super::{ButtonTone, ButtonVariant, IconButtonVariant, Metrics};

pub(super) struct Appearance {
    pub(super) background: Option<ColorRole>,
    pub(super) foreground: ColorRole,
    pub(super) outlined: bool,
    pub(super) outline_role: ColorRole,
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
            outline_role: self.outline_role,
            elevation: self.elevation,
            hover_elevation: self.hover_elevation,
        }
    }
}

fn tone_roles(tone: ButtonTone) -> Option<(ColorRole, ColorRole, ColorRole, ColorRole)> {
    use ColorRole::*;
    match tone {
        ButtonTone::Default => None,
        ButtonTone::Danger => Some((Error, OnError, ErrorContainer, OnErrorContainer)),
        ButtonTone::Success => Some((Success, OnSuccess, SuccessContainer, OnSuccessContainer)),
        ButtonTone::Warning => Some((Warning, OnWarning, WarningContainer, OnWarningContainer)),
        ButtonTone::Info => Some((Info, OnInfo, InfoContainer, OnInfoContainer)),
    }
}

pub(super) fn button_appearance(
    variant: ButtonVariant,
    selected: Option<bool>,
    tone: ButtonTone,
) -> Appearance {
    use ColorRole::*;
    use Elevation::*;
    let (background, foreground, outlined, outline_role, elevation, hover_elevation) =
        if let Some((role, on_role, container, on_container)) = tone_roles(tone) {
            match (variant, selected) {
                (ButtonVariant::Elevated, Some(true)) => {
                    (Some(role), on_role, false, OutlineVariant, Level1, Level2)
                }
                (ButtonVariant::Elevated, _) => (
                    Some(SurfaceContainerLow),
                    role,
                    false,
                    OutlineVariant,
                    Level1,
                    Level2,
                ),
                (ButtonVariant::Filled, Some(false)) => (
                    Some(SurfaceContainer),
                    role,
                    false,
                    OutlineVariant,
                    Level0,
                    Level1,
                ),
                (ButtonVariant::Filled, _) => {
                    (Some(role), on_role, false, OutlineVariant, Level0, Level1)
                }
                (ButtonVariant::FilledTonal, Some(true)) => {
                    (Some(role), on_role, false, OutlineVariant, Level0, Level1)
                }
                (ButtonVariant::FilledTonal, _) => (
                    Some(container),
                    on_container,
                    false,
                    OutlineVariant,
                    Level0,
                    Level1,
                ),
                (ButtonVariant::Outlined, Some(true)) => {
                    (Some(role), on_role, false, OutlineVariant, Level0, Level0)
                }
                (ButtonVariant::Outlined, _) => (None, role, true, role, Level0, Level0),
                (ButtonVariant::Text, _) => (None, role, false, OutlineVariant, Level0, Level0),
            }
        } else {
            match (variant, selected) {
                (ButtonVariant::Elevated, Some(true)) => (
                    Some(Primary),
                    OnPrimary,
                    false,
                    OutlineVariant,
                    Level1,
                    Level2,
                ),
                (ButtonVariant::Elevated, _) => (
                    Some(SurfaceContainerLow),
                    Primary,
                    false,
                    OutlineVariant,
                    Level1,
                    Level2,
                ),
                (ButtonVariant::Filled, Some(false)) => (
                    Some(SurfaceContainer),
                    OnSurfaceVariant,
                    false,
                    OutlineVariant,
                    Level0,
                    Level1,
                ),
                (ButtonVariant::Filled, _) => (
                    Some(Primary),
                    OnPrimary,
                    false,
                    OutlineVariant,
                    Level0,
                    Level1,
                ),
                (ButtonVariant::FilledTonal, Some(true)) => (
                    Some(Secondary),
                    OnSecondary,
                    false,
                    OutlineVariant,
                    Level0,
                    Level1,
                ),
                (ButtonVariant::FilledTonal, _) => (
                    Some(SecondaryContainer),
                    OnSecondaryContainer,
                    false,
                    OutlineVariant,
                    Level0,
                    Level1,
                ),
                (ButtonVariant::Outlined, Some(true)) => (
                    Some(InverseSurface),
                    InverseOnSurface,
                    false,
                    OutlineVariant,
                    Level0,
                    Level0,
                ),
                (ButtonVariant::Outlined, _) => {
                    (None, OnSurfaceVariant, true, OutlineVariant, Level0, Level0)
                }
                // Compose overrides the outdated TextButtonTokens.LabelColor to Primary.
                (ButtonVariant::Text, _) => (None, Primary, false, OutlineVariant, Level0, Level0),
            }
        };
    Appearance {
        background,
        foreground,
        outlined,
        outline_role,
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

pub(super) fn icon_appearance(
    variant: IconButtonVariant,
    selected: Option<bool>,
    tone: ButtonTone,
) -> Appearance {
    let mut appearance = match variant {
        IconButtonVariant::Standard => button_appearance(ButtonVariant::Text, selected, tone),
        IconButtonVariant::Filled => button_appearance(ButtonVariant::Filled, selected, tone),
        IconButtonVariant::FilledTonal => {
            button_appearance(ButtonVariant::FilledTonal, selected, tone)
        }
        IconButtonVariant::Outlined => button_appearance(ButtonVariant::Outlined, selected, tone),
    };
    if variant == IconButtonVariant::Standard && tone == ButtonTone::Default {
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
            button_appearance(ButtonVariant::Text, None, ButtonTone::Default).foreground,
            ColorRole::Primary
        );
        for selected in [None, Some(false)] {
            assert_eq!(
                icon_appearance(IconButtonVariant::Standard, selected, ButtonTone::Default)
                    .foreground,
                ColorRole::OnSurfaceVariant
            );
        }
        assert_eq!(
            icon_appearance(IconButtonVariant::Standard, Some(true), ButtonTone::Default)
                .foreground,
            ColorRole::Primary
        );
    }

    #[test]
    fn disabled_tokens_are_not_inferred_from_enabled_selection_colors() {
        use ColorRole::*;
        for selected in [Some(false), Some(true)] {
            let outlined =
                button_appearance(ButtonVariant::Outlined, selected, ButtonTone::Default);
            assert_eq!(outlined.disabled.background, Some((OutlineVariant, 0.10)));
            assert_eq!(outlined.disabled.foreground, OnSurfaceVariant);
            assert_eq!(outlined.disabled.outline, (OutlineVariant, 0.10));
            let icon = icon_appearance(IconButtonVariant::Outlined, selected, ButtonTone::Default);
            assert_eq!(icon.disabled.background, None);
            assert_eq!(icon.disabled.foreground, OnSurface);
            assert_eq!(icon.disabled.outline, (OutlineVariant, 0.38));
        }
        assert_eq!(
            button_appearance(ButtonVariant::Outlined, None, ButtonTone::Default)
                .disabled
                .background,
            None
        );
        assert_eq!(
            button_appearance(ButtonVariant::Filled, None, ButtonTone::Default)
                .disabled
                .background,
            Some((OnSurface, 0.10))
        );
        // FilledTonalButton and its expressive toggle reference different token
        // generations in the pinned Compose release.
        assert_eq!(
            button_appearance(ButtonVariant::FilledTonal, None, ButtonTone::Default)
                .disabled
                .background,
            Some((OnSurface, 0.12))
        );
        assert_eq!(
            button_appearance(ButtonVariant::FilledTonal, Some(false), ButtonTone::Default)
                .disabled
                .background,
            Some((OnSurface, 0.10))
        );
        assert_eq!(
            icon_appearance(IconButtonVariant::FilledTonal, None, ButtonTone::Default)
                .disabled
                .background,
            Some((OnSurface, 0.10))
        );
    }

    #[test]
    fn button_tones_map_all_variants_and_selection_states_to_semantic_roles() {
        use ColorRole::*;
        let cases = [
            (
                ButtonTone::Danger,
                Error,
                OnError,
                ErrorContainer,
                OnErrorContainer,
            ),
            (
                ButtonTone::Success,
                Success,
                OnSuccess,
                SuccessContainer,
                OnSuccessContainer,
            ),
            (
                ButtonTone::Warning,
                Warning,
                OnWarning,
                WarningContainer,
                OnWarningContainer,
            ),
            (
                ButtonTone::Info,
                Info,
                OnInfo,
                InfoContainer,
                OnInfoContainer,
            ),
        ];

        for (tone, role, on_role, container, on_container) in cases {
            // Filled: non-toggle & selected use role/on_role; unselected toggle uses neutral surface + tone fg.
            for selected in [None, Some(true)] {
                let filled = button_appearance(ButtonVariant::Filled, selected, tone);
                assert_eq!(filled.background, Some(role));
                assert_eq!(filled.foreground, on_role);
                assert!(!filled.outlined);
            }
            let filled_off = button_appearance(ButtonVariant::Filled, Some(false), tone);
            assert_eq!(filled_off.background, Some(SurfaceContainer));
            assert_eq!(filled_off.foreground, role);

            // FilledTonal: non-toggle & unselected use container/on_container; selected uses role/on_role.
            for selected in [None, Some(false)] {
                let tonal = button_appearance(ButtonVariant::FilledTonal, selected, tone);
                assert_eq!(tonal.background, Some(container));
                assert_eq!(tonal.foreground, on_container);
            }
            let tonal_on = button_appearance(ButtonVariant::FilledTonal, Some(true), tone);
            assert_eq!(tonal_on.background, Some(role));
            assert_eq!(tonal_on.foreground, on_role);

            // Elevated: non-toggle & unselected use SurfaceContainerLow + tone fg; selected uses role/on_role.
            for selected in [None, Some(false)] {
                let elevated = button_appearance(ButtonVariant::Elevated, selected, tone);
                assert_eq!(elevated.background, Some(SurfaceContainerLow));
                assert_eq!(elevated.foreground, role);
                assert_eq!(elevated.elevation, Elevation::Level1);
            }
            let elevated_on = button_appearance(ButtonVariant::Elevated, Some(true), tone);
            assert_eq!(elevated_on.background, Some(role));
            assert_eq!(elevated_on.foreground, on_role);

            // Outlined: non-toggle & unselected use transparent + tone fg & tone outline; selected uses role/on_role.
            for selected in [None, Some(false)] {
                let outlined = button_appearance(ButtonVariant::Outlined, selected, tone);
                assert_eq!(outlined.background, None);
                assert_eq!(outlined.foreground, role);
                assert!(outlined.outlined);
                assert_eq!(outlined.outline_role, role);
            }
            let outlined_on = button_appearance(ButtonVariant::Outlined, Some(true), tone);
            assert_eq!(outlined_on.background, Some(role));
            assert_eq!(outlined_on.foreground, on_role);
            assert!(!outlined_on.outlined);

            // Text and Standard icon buttons use tone foreground.
            let text = button_appearance(ButtonVariant::Text, None, tone);
            assert_eq!(text.background, None);
            assert_eq!(text.foreground, role);

            for selected in [None, Some(false), Some(true)] {
                let standard_icon = icon_appearance(IconButtonVariant::Standard, selected, tone);
                assert_eq!(standard_icon.background, None);
                assert_eq!(standard_icon.foreground, role);
                assert_eq!(standard_icon.hover_elevation, Elevation::Level0);
            }

            // IconButton variants match tone mapping with icon disabled tokens preserved.
            let icon_filled = icon_appearance(IconButtonVariant::Filled, None, tone);
            assert_eq!(icon_filled.background, Some(role));
            assert_eq!(icon_filled.foreground, on_role);
            assert_eq!(icon_filled.disabled.foreground, OnSurface);

            let icon_outlined = icon_appearance(IconButtonVariant::Outlined, None, tone);
            assert_eq!(icon_outlined.foreground, role);
            assert_eq!(icon_outlined.outline_role, role);
            assert_eq!(icon_outlined.disabled.outline, (OutlineVariant, 0.38));
        }
    }
}
