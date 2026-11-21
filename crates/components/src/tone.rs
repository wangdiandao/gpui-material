use gpui_material_theme::ColorRole;

/// Semantic palette shortcut shared across Material components.
///
/// Explicit per-instance color or border overrides always take precedence over
/// semantic tone defaults, and disabled states keep their disabled appearance.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SemanticTone {
    #[default]
    Default,
    Danger,
    Success,
    Warning,
    Info,
}

impl SemanticTone {
    /// Resolves `(role, on_role, container, on_container)` from
    /// [`gpui_material_theme::ColorRole`], returning `None` for
    /// [`SemanticTone::Default`].
    pub const fn roles(self) -> Option<(ColorRole, ColorRole, ColorRole, ColorRole)> {
        use ColorRole::*;
        match self {
            Self::Default => None,
            Self::Danger => Some((Error, OnError, ErrorContainer, OnErrorContainer)),
            Self::Success => Some((Success, OnSuccess, SuccessContainer, OnSuccessContainer)),
            Self::Warning => Some((Warning, OnWarning, WarningContainer, OnWarningContainer)),
            Self::Info => Some((Info, OnInfo, InfoContainer, OnInfoContainer)),
        }
    }

    /// Resolves the primary foreground/accent role for this tone.
    pub const fn role(self) -> Option<ColorRole> {
        match self.roles() {
            Some((role, _, _, _)) => Some(role),
            None => None,
        }
    }

    /// Resolves the foreground role paired with [`Self::role`].
    pub const fn on_role(self) -> Option<ColorRole> {
        match self.roles() {
            Some((_, on_role, _, _)) => Some(on_role),
            None => None,
        }
    }

    /// Resolves the tonal container role for this tone.
    pub const fn container_role(self) -> Option<ColorRole> {
        match self.roles() {
            Some((_, _, container, _)) => Some(container),
            None => None,
        }
    }

    /// Resolves the foreground role paired with [`Self::container_role`].
    pub const fn on_container_role(self) -> Option<ColorRole> {
        match self.roles() {
            Some((_, _, _, on_container)) => Some(on_container),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_tone_resolves_expected_color_roles() {
        use ColorRole::*;
        assert_eq!(SemanticTone::Default.roles(), None);
        assert_eq!(SemanticTone::Default.role(), None);
        assert_eq!(SemanticTone::Default.on_role(), None);
        assert_eq!(SemanticTone::Default.container_role(), None);
        assert_eq!(SemanticTone::Default.on_container_role(), None);

        let cases = [
            (
                SemanticTone::Danger,
                Error,
                OnError,
                ErrorContainer,
                OnErrorContainer,
            ),
            (
                SemanticTone::Success,
                Success,
                OnSuccess,
                SuccessContainer,
                OnSuccessContainer,
            ),
            (
                SemanticTone::Warning,
                Warning,
                OnWarning,
                WarningContainer,
                OnWarningContainer,
            ),
            (
                SemanticTone::Info,
                Info,
                OnInfo,
                InfoContainer,
                OnInfoContainer,
            ),
        ];

        for (tone, role, on_role, container, on_container) in cases {
            assert_eq!(tone.roles(), Some((role, on_role, container, on_container)));
            assert_eq!(tone.role(), Some(role));
            assert_eq!(tone.on_role(), Some(on_role));
            assert_eq!(tone.container_role(), Some(container));
            assert_eq!(tone.on_container_role(), Some(on_container));
        }
    }
}
