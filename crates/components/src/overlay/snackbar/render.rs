use gpui::{
    App, ClickEvent, Context, Hsla, InteractiveElement, IntoElement, KeyDownEvent, ParentElement,
    Pixels, Role, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
};
use gpui_base::TestSupportExt as _;
use gpui_material_foundation::{OverlayMotion, color, current_material_theme, elevation_shadows};
use gpui_material_theme::{ColorRole, CornerShape, Direction, Elevation, TypeRole};

use crate::SemanticTone;

use super::host::{Entry, SnackbarHost};
use super::{HostRenderer, Snackbar, SnackbarParts, SnackbarResult};

fn snackbar_tone_roles(tone: SemanticTone) -> (ColorRole, ColorRole, ColorRole) {
    tone.roles().map_or(
        (
            ColorRole::InverseSurface,
            ColorRole::InverseOnSurface,
            ColorRole::InversePrimary,
        ),
        |(role, _, container, on_container)| (container, on_container, role),
    )
}

pub(super) fn snackbar_surface(
    entry: &Entry,
    motion: OverlayMotion,
    width: Pixels,
    active: bool,
    renderer: Option<&HostRenderer>,
    window: &mut Window,
    cx: &mut Context<SnackbarHost>,
) -> impl IntoElement {
    let id = entry.id;
    let serial = id.sequence;
    let message = &entry.message;
    let theme = current_material_theme(window, cx);
    let tone = |role| color(theme.colors.color(role));
    let (_, content_role, action_role) = snackbar_tone_roles(message.tone);
    let default_content = tone(content_role);
    let default_action = tone(action_role);
    let content_color = message.colors.content.unwrap_or(default_content);
    let action_color = message.colors.action.unwrap_or(default_action);
    let dismiss_color = message.colors.dismiss.unwrap_or(default_content);
    let dismiss_feedback = message.colors.dismiss.unwrap_or(default_action);
    let spacing = motion.px(16.);
    let message_content = message.message_content.as_ref().map_or_else(
        || message.message.clone().into_any_element(),
        |content| content(window, cx),
    );
    let message_part = div()
        .id(("snackbar-message", serial))
        .test_support()
        .when(active, |label| {
            label
                .role(Role::Status)
                .aria_label(message.message.clone())
                .a11y_synthetic_children(|tree| {
                    let node = tree.parent_node();
                    node.set_live(gpui::accesskit::Live::Polite);
                    node.set_live_atomic();
                })
        })
        .min_w_0()
        .flex_1()
        .py(motion.px(14.))
        .text_color(content_color)
        .child(message_content)
        .into_any_element();

    let render_control = |index,
                          id_name,
                          label,
                          content,
                          foreground: Hsla,
                          feedback: Hsla,
                          result,
                          cx: &Context<SnackbarHost>| {
        if active {
            gpui_base::Button::new(id_name)
                .min_h(motion.px(48.))
                .px(spacing * 0.5)
                .rounded(spacing * 0.375)
                .text_color(foreground)
                .font_weight(gpui::FontWeight::MEDIUM)
                .hover(move |style| {
                    style.bg(feedback.opacity(gpui_material_theme::StateLayer::Hover.opacity()))
                })
                .active(move |style| {
                    style.bg(feedback.opacity(gpui_material_theme::StateLayer::Pressed.opacity()))
                })
                .focus_visible(move |style| style.border(motion.px(2.)).border_color(feedback))
                .track_focus(&entry.controls[index])
                .accessibility_label(label)
                .on_click(cx.listener(move |host, _: &ClickEvent, window, cx| {
                    host.finish(id, result, window, cx);
                }))
                .child(content)
                .into_any_element()
        } else {
            div()
                .min_h(motion.px(48.))
                .px(spacing * 0.5)
                .flex()
                .items_center()
                .line_height(gpui::relative(1.))
                .text_color(foreground)
                .font_weight(gpui::FontWeight::MEDIUM)
                .child(content)
                .into_any_element()
        }
    };
    let action_part = message.action_label().map(|label| {
        let content = message.action_content.as_ref().map_or_else(
            || label.clone().into_any_element(),
            |content| content(window, cx),
        );
        let control = render_control(
            0,
            "snackbar-action",
            label.clone(),
            content,
            action_color,
            action_color,
            SnackbarResult::ActionPerformed,
            cx,
        );
        if active {
            div()
                .id("snackbar-action-region")
                .test_support()
                .child(control)
                .into_any_element()
        } else {
            control
        }
    });
    let dismiss_part = message.show_dismiss.then(|| {
        let (label, content) = message.dismiss_content.as_ref().map_or_else(
            || {
                (
                    message
                        .dismiss_label
                        .clone()
                        .unwrap_or_else(|| "Dismiss message".into()),
                    crate::Icon::new(crate::IconName::Close)
                        .size(motion.px(24.))
                        .into_any_element(),
                )
            },
            |(label, content)| (label.clone(), content(window, cx)),
        );
        render_control(
            1,
            "snackbar-dismiss",
            label,
            content,
            dismiss_color,
            dismiss_feedback,
            SnackbarResult::Dismissed,
            cx,
        )
    });
    let parts = SnackbarParts {
        message: message_part,
        action: action_part,
        dismiss: dismiss_part,
        scale: motion.scale,
    };
    let rendered = match renderer {
        Some(render) => render(message, parts, window, cx),
        None => default_layout(message, parts, motion, window, cx).into_any_element(),
    };
    let surface = div()
        .id((if active { "snackbar" } else { "snackbar-exit" }, serial))
        .test_support()
        .w(width * motion.scale)
        .opacity(motion.alpha)
        .when(active, |surface| {
            surface
                .on_hover(cx.listener(move |host, hovering: &bool, window, cx| {
                    if host.active.as_ref().is_some_and(|entry| entry.id == id) {
                        host.set_hovered(*hovering, window, cx);
                    }
                }))
                .capture_key_down(cx.listener(move |host, event: &KeyDownEvent, window, cx| {
                    if event.keystroke.key == "escape" && host.dismiss_message(id, window, cx) {
                        cx.stop_propagation();
                    }
                }))
        })
        .child(rendered);
    super::super::text_style(surface, TypeRole::BodyMedium, motion.scale, window, cx)
}

fn default_layout(
    message: &Snackbar,
    parts: SnackbarParts,
    motion: OverlayMotion,
    window: &Window,
    cx: &App,
) -> gpui::Div {
    let SnackbarParts {
        message: content,
        action,
        dismiss,
        ..
    } = parts;
    let theme = current_material_theme(window, cx);
    let direction = theme.layout.direction;
    let rtl = direction == Direction::Rtl;
    let tone = |role| color(theme.colors.color(role));
    let spacing = motion.px(16.);
    let (container_role, _, _) = snackbar_tone_roles(message.tone);
    let mut surface = div()
        .bg(message
            .colors
            .container
            .unwrap_or_else(|| tone(container_role)))
        .shadow(elevation_shadows(
            Elevation::Level3.dp(),
            motion.scale,
            tone(ColorRole::Shadow),
        ))
        .flex()
        .min_h(motion.px(48.))
        .gap(spacing * 0.5)
        .px(spacing)
        .when(rtl, Styled::flex_row_reverse);
    if message.shape == CornerShape::Full {
        surface = surface.rounded_full();
    } else {
        // GPUI further clamps these radii against the actual laid-out quad.
        let [top_left, top_right, bottom_right, bottom_left] =
            message.shape.radii(f32::MAX / 2., f32::MAX / 2., direction);
        let radii = &mut surface.style().corner_radii;
        radii.top_left = Some(motion.px(top_left).into());
        radii.top_right = Some(motion.px(top_right).into());
        radii.bottom_right = Some(motion.px(bottom_right).into());
        radii.bottom_left = Some(motion.px(bottom_left).into());
    }
    match (message.action_on_new_line, action) {
        (true, Some(action)) => surface
            .flex_col()
            .items_stretch()
            .gap_0()
            .child(content)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .gap(spacing * 0.5)
                    .when(rtl, Styled::flex_row_reverse)
                    .child(action)
                    .children(dismiss),
            ),
        (_, action) => surface
            .items_center()
            .child(content)
            .children(action)
            .children(dismiss),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snackbar_tone_roles_map_inverse_default_and_semantic_palettes() {
        use ColorRole::*;
        assert_eq!(
            snackbar_tone_roles(SemanticTone::Default),
            (InverseSurface, InverseOnSurface, InversePrimary)
        );
        assert_eq!(
            snackbar_tone_roles(SemanticTone::Danger),
            (ErrorContainer, OnErrorContainer, Error)
        );
        assert_eq!(
            snackbar_tone_roles(SemanticTone::Success),
            (SuccessContainer, OnSuccessContainer, Success)
        );
        assert_eq!(
            snackbar_tone_roles(SemanticTone::Warning),
            (WarningContainer, OnWarningContainer, Warning)
        );
        assert_eq!(
            snackbar_tone_roles(SemanticTone::Info),
            (InfoContainer, OnInfoContainer, Info)
        );
    }
}
