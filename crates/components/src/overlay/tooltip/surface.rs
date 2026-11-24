use gpui::{
    AccessibleAction, AnyElement, App, InteractiveElement, IntoElement, ParentElement,
    SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
};
use gpui_base::{Link, TestSupportExt};
use gpui_material_foundation::{OverlayMotion, current_material_theme};
use gpui_material_theme::{Direction, TypeRole};

use crate::{Button, ButtonColors, ButtonVariant};

use super::appearance::SurfaceStyle;
use super::{ActivateTooltipAction, Tooltip, TooltipAction};

pub(super) fn tooltip_surface(
    tooltip: &mut Tooltip,
    open: bool,
    style: SurfaceStyle,
    motion: OverlayMotion,
    window: &Window,
    cx: &App,
) -> gpui::Div {
    // Preserve custom rich content during exit animation to avoid layout/content flicker.
    let title = tooltip
        .title_content
        .take()
        .or_else(|| tooltip.title.take().map(IntoElement::into_any_element));
    let body = tooltip
        .body_content
        .take()
        .unwrap_or_else(|| tooltip.text.clone().into_any_element());
    let action = if open {
        tooltip.action_content.take().or_else(|| {
            tooltip
                .action
                .take()
                .map(|action| render_action(action, true, style, motion, window, cx))
        })
    } else {
        tooltip
            .action
            .take()
            .map(|action| render_action(action, false, style, motion, window, cx))
    };
    let link = tooltip.link.take().map(|(label, callback)| {
        if open {
            let activate = callback.clone();
            Link::new("tooltip-link")
                .accessibility_label(label.clone())
                .text_color(style.action)
                .underline()
                .on_activate(move |_, window, cx| callback(window, cx))
                .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                    activate(window, cx)
                })
                .child(label)
                .into_any_element()
        } else {
            div()
                .text_color(style.action)
                .child(label)
                .into_any_element()
        }
    });
    let text_role = if tooltip.rich {
        TypeRole::BodyMedium
    } else {
        TypeRole::BodySmall
    };
    let direction = current_material_theme(window, cx).layout.direction;
    let shortcut = tooltip.shortcut.take().filter(|text| !text.is_empty());
    let shortcut_pill = |text: SharedString| {
        super::super::text_style(
            div()
                .id("tooltip-shortcut")
                .test_support()
                .flex_none()
                .px(motion.px(4.))
                .rounded(motion.px(4.))
                .bg(style.content.opacity(0.12))
                .text_color(style.content.opacity(0.78))
                .child(text),
            TypeRole::LabelSmall,
            motion.scale,
            window,
            cx,
        )
    };
    let (title_row, body_shortcut) = match (title, shortcut) {
        (Some(title), Some(shortcut)) => (
            Some(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(motion.px(8.))
                    .when(direction == Direction::Rtl, Styled::flex_row_reverse)
                    .child(super::super::text_style(
                        div().min_w_0().text_color(style.title).child(title),
                        TypeRole::TitleSmall,
                        motion.scale,
                        window,
                        cx,
                    ))
                    .child(shortcut_pill(shortcut)),
            ),
            None,
        ),
        (Some(title), None) => (
            Some(super::super::text_style(
                div().text_color(style.title).child(title),
                TypeRole::TitleSmall,
                motion.scale,
                window,
                cx,
            )),
            None,
        ),
        (None, shortcut) => (None, shortcut),
    };
    let body_row = if let Some(shortcut) = body_shortcut {
        div()
            .flex()
            .items_center()
            .gap(motion.px(6.))
            .when(direction == Direction::Rtl, Styled::flex_row_reverse)
            .child(super::super::text_style(
                div().min_w_0().text_color(style.content).child(body),
                text_role,
                motion.scale,
                window,
                cx,
            ))
            .child(shortcut_pill(shortcut))
    } else {
        super::super::text_style(
            div().text_color(style.content).child(body),
            text_role,
            motion.scale,
            window,
            cx,
        )
    };
    style
        .apply(div())
        .children(title_row)
        .child(body_row)
        .children(action.map(|action| div().text_color(style.action).child(action)))
        .children(link)
}

fn render_action(
    action: TooltipAction,
    open: bool,
    style: SurfaceStyle,
    motion: OverlayMotion,
    window: &Window,
    cx: &App,
) -> AnyElement {
    if open {
        let activate = action.callback.clone();
        return div()
            .id("tooltip-action-keys")
            .key_context("MaterialTooltipAction")
            .on_action(move |_: &ActivateTooltipAction, window, cx| activate(window, cx))
            .child(
                Button::new("tooltip-action", action.label)
                    .track_focus(&action.focus)
                    .variant(ButtonVariant::Text)
                    .scale(motion.scale)
                    .colors(ButtonColors {
                        content: style.action_override,
                        ..Default::default()
                    })
                    .on_activate(move |window, cx| (action.callback)(window, cx)),
            )
            .into_any_element();
    }
    let density = current_material_theme(window, cx).layout.density;
    div()
        .min_h(motion.px(density.interactive_size(40.)))
        .flex()
        .items_center()
        .justify_center()
        .child(
            super::super::text_style(
                div().px(motion.px(16.)).min_w_0().truncate(),
                TypeRole::LabelLarge,
                motion.scale,
                window,
                cx,
            )
            .text_color(style.action)
            .child(action.label),
        )
        .into_any_element()
}
