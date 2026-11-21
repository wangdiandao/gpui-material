use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    Role, SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
    px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{
    Surface, SurfaceBorder, color, current_material_theme, wrapped_text,
};
use gpui_material_theme::{
    ColorRole, CornerRole, CornerShape, Direction, TypeRole, TypographyEmphasis,
};

use crate::action::Activate;
use crate::{Button, ButtonColors, ControlSize, Icon, IconButton, IconName, SemanticTone};

/// Optional per-region colors for [`Banner`].
///
/// Explicit overrides always take precedence over [`SemanticTone`] defaults.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BannerColors {
    pub container: Option<Hsla>,
    pub title: Option<Hsla>,
    pub message: Option<Hsla>,
    pub icon: Option<Hsla>,
    pub border: Option<Hsla>,
    pub dismiss: Option<Hsla>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BannerToneRoles {
    container: ColorRole,
    content: ColorRole,
    icon: ColorRole,
    border: ColorRole,
    dismiss: ColorRole,
}

const fn banner_tone_roles(tone: SemanticTone) -> BannerToneRoles {
    use ColorRole::*;
    match tone.roles() {
        Some((role, _, container, on_container)) => BannerToneRoles {
            container,
            content: on_container,
            icon: role,
            border: role,
            dismiss: on_container,
        },
        None => BannerToneRoles {
            container: SurfaceContainerLow,
            content: OnSurface,
            icon: Primary,
            border: OutlineVariant,
            dismiss: OnSurfaceVariant,
        },
    }
}

/// An in-flow, non-blocking Material 3 banner/callout surface.
///
/// Visibility is owned by the caller; `Banner` keeps no internal hidden state.
#[derive(IntoElement)]
pub struct Banner {
    id: ElementId,
    message: SharedString,
    title: Option<SharedString>,
    tone: SemanticTone,
    icon: Option<AnyElement>,
    primary_action: Option<(SharedString, Activate)>,
    secondary_action: Option<(SharedString, Activate)>,
    actions_content: Option<AnyElement>,
    on_dismiss: Option<Activate>,
    dismiss_label: SharedString,
    shape: CornerShape,
    colors: BannerColors,
    alert: Option<bool>,
}

impl Banner {
    /// Creates an in-flow banner with a stable element ID and message text.
    pub fn new(id: impl Into<ElementId>, message: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            message: message.into(),
            title: None,
            tone: SemanticTone::Default,
            icon: None,
            primary_action: None,
            secondary_action: None,
            actions_content: None,
            on_dismiss: None,
            dismiss_label: "Dismiss".into(),
            shape: CornerRole::Medium.shape(),
            colors: BannerColors::default(),
            alert: None,
        }
    }

    /// Sets an optional bold title line above the message.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Applies a Material 3 semantic tone palette.
    ///
    /// [`SemanticTone::Danger`] defaults to `Role::Alert` unless overridden via
    /// [`Self::alert`]. Explicit [`Self::colors`] overrides always take precedence.
    pub fn tone(mut self, tone: SemanticTone) -> Self {
        self.tone = tone;
        self
    }

    /// Sets an optional leading decorative icon tinted to the banner's icon color.
    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }

    /// Sets the primary banner action button, inheriting the banner's semantic tone.
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.primary_action = Some((label.into(), Rc::new(handler)));
        self
    }

    /// Sets an optional secondary banner action button, inheriting the banner's semantic tone.
    pub fn secondary_action(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.secondary_action = Some((label.into(), Rc::new(handler)));
        self
    }

    /// Supplies optional custom action controls in the banner's action row.
    pub fn actions_content(mut self, element: impl IntoElement) -> Self {
        self.actions_content = Some(element.into_any_element());
        self
    }

    /// Adds a trailing close [`IconButton`] that invokes `handler` when activated.
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }

    /// Sets the accessible label for the optional dismiss button. Defaults to `"Dismiss"`.
    pub fn dismiss_label(mut self, label: impl Into<SharedString>) -> Self {
        self.dismiss_label = label.into();
        self
    }

    /// Sets the container corner shape. Defaults to `CornerRole::Medium`.
    pub fn shape(mut self, shape: CornerShape) -> Self {
        self.shape = shape;
        self
    }

    /// Overrides banner region colors. Explicit values always take precedence over `.tone(...)`.
    pub fn colors(mut self, colors: BannerColors) -> Self {
        self.colors = colors;
        self
    }

    /// Explicitly chooses `Role::Alert` (`true`) or `Role::Status` (`false`).
    ///
    /// When unset, [`SemanticTone::Danger`] uses `Role::Alert` and other tones use `Role::Status`.
    pub fn alert(mut self, alert: bool) -> Self {
        self.alert = Some(alert);
        self
    }

    fn resolved_role(&self) -> Role {
        let is_alert = self.alert.unwrap_or(self.tone == SemanticTone::Danger);
        if is_alert { Role::Alert } else { Role::Status }
    }
}

impl RenderOnce for Banner {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let role = self.resolved_role();
        let roles = banner_tone_roles(self.tone);
        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let rtl = direction == Direction::Rtl;
        let theme_color = |r| color(theme.colors.color(r));

        let container_color = self
            .colors
            .container
            .unwrap_or_else(|| theme_color(roles.container));
        let title_color = self
            .colors
            .title
            .unwrap_or_else(|| theme_color(roles.content));
        let message_color = self
            .colors
            .message
            .unwrap_or_else(|| theme_color(roles.content));
        let icon_color = self.colors.icon.unwrap_or_else(|| theme_color(roles.icon));
        let border_color = self
            .colors
            .border
            .unwrap_or_else(|| theme_color(roles.border));
        let dismiss_color = self
            .colors
            .dismiss
            .unwrap_or_else(|| theme_color(roles.dismiss));

        let summary: SharedString = match &self.title {
            Some(title) if !title.is_empty() => format!("{title}. {}", self.message).into(),
            _ => self.message.clone(),
        };

        let icon_slot = self.icon.map(|icon| {
            div()
                .id((self.id.clone(), "icon"))
                .test_support()
                .flex_none()
                .pt(px(2.))
                .text_color(icon_color)
                .text_size(px(20.))
                .child(icon)
        });

        let title_slot = self.title.map(|title| {
            wrapped_text(
                title.clone(),
                TypeRole::TitleSmall.style(TypographyEmphasis::Emphasized),
                1.,
                title_color,
                window,
                cx,
            )
            .w_full()
            .when(rtl, Styled::text_right)
            .id((self.id.clone(), "title"))
            .test_support()
            .role(Role::Label)
            .aria_label(title)
        });

        let message_slot = wrapped_text(
            self.message.clone(),
            TypeRole::BodyMedium.style(TypographyEmphasis::Baseline),
            1.,
            message_color,
            window,
            cx,
        )
        .w_full()
        .when(rtl, Styled::text_right)
        .id((self.id.clone(), "message"))
        .test_support()
        .role(Role::Label)
        .aria_label(self.message);

        let primary_button = self.primary_action.map(|(label, handler)| {
            Button::text((self.id.clone(), "action"), label)
                .size(ControlSize::ExtraSmall)
                .tone(self.tone)
                .on_activate(move |window, cx| handler(window, cx))
        });
        let secondary_button = self.secondary_action.map(|(label, handler)| {
            Button::text((self.id.clone(), "secondary-action"), label)
                .size(ControlSize::ExtraSmall)
                .tone(self.tone)
                .on_activate(move |window, cx| handler(window, cx))
        });

        let has_actions = primary_button.is_some()
            || secondary_button.is_some()
            || self.actions_content.is_some();
        let actions_slot = has_actions.then(|| {
            div()
                .id((self.id.clone(), "actions"))
                .test_support()
                .mt(px(8.))
                .flex()
                .flex_row()
                .flex_wrap()
                .items_center()
                .gap(px(8.))
                .when(rtl, Styled::flex_row_reverse)
                .children(primary_button)
                .children(secondary_button)
                .children(self.actions_content)
        });

        let dismiss_slot = self.on_dismiss.map(|handler| {
            div().flex_none().child(
                IconButton::standard(
                    (self.id.clone(), "dismiss"),
                    self.dismiss_label,
                    Icon::new(IconName::Close),
                )
                .size(ControlSize::ExtraSmall)
                .colors(ButtonColors {
                    content: Some(dismiss_color),
                    ..Default::default()
                })
                .on_activate(move |window, cx| handler(window, cx)),
            )
        });

        let surface = Surface::new(roles.container)
            .container_color(container_color)
            .content_color(message_color)
            .shape(self.shape)
            .border(SurfaceBorder {
                width: px(1.),
                color: border_color,
            })
            .w_full()
            .px(px(16.))
            .py(px(12.))
            .flex()
            .flex_row()
            .items_start()
            .gap(px(12.))
            .when(rtl, Styled::flex_row_reverse)
            .children(icon_slot)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .when(rtl, Styled::items_end)
                    .children(title_slot)
                    .child(message_slot)
                    .children(actions_slot),
            )
            .children(dismiss_slot);

        div()
            .id(self.id)
            .test_support()
            .role(role)
            .aria_label(summary)
            .w_full()
            .child(surface)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn banner_tone_roles_and_alert_defaults_match_specification() {
        use ColorRole::*;
        assert_eq!(
            banner_tone_roles(SemanticTone::Default),
            BannerToneRoles {
                container: SurfaceContainerLow,
                content: OnSurface,
                icon: Primary,
                border: OutlineVariant,
                dismiss: OnSurfaceVariant,
            }
        );
        for (tone, role, container, on_container) in [
            (
                SemanticTone::Danger,
                Error,
                ErrorContainer,
                OnErrorContainer,
            ),
            (
                SemanticTone::Success,
                Success,
                SuccessContainer,
                OnSuccessContainer,
            ),
            (
                SemanticTone::Warning,
                Warning,
                WarningContainer,
                OnWarningContainer,
            ),
            (SemanticTone::Info, Info, InfoContainer, OnInfoContainer),
        ] {
            assert_eq!(
                banner_tone_roles(tone),
                BannerToneRoles {
                    container,
                    content: on_container,
                    icon: role,
                    border: role,
                    dismiss: on_container,
                }
            );
        }

        assert_eq!(Banner::new("b1", "Info").resolved_role(), Role::Status);
        assert_eq!(
            Banner::new("b2", "Error")
                .tone(SemanticTone::Danger)
                .resolved_role(),
            Role::Alert
        );
        assert_eq!(
            Banner::new("b3", "Error")
                .tone(SemanticTone::Danger)
                .alert(false)
                .resolved_role(),
            Role::Status
        );
        assert_eq!(
            Banner::new("b4", "Notice").alert(true).resolved_role(),
            Role::Alert
        );
    }
}
