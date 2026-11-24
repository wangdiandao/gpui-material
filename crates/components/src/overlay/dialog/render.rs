use gpui::{
    AnyElement, App, ElementId, Entity, FocusHandle, Hsla, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, Role, ScrollHandle, SharedString, Size,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::{DialogChangeReason, TestSupportExt};
use gpui_material_foundation::{
    Scrim, Surface, ThemeScope, color, current_material_theme, isolated_scroll,
};
use gpui_material_theme::{
    ColorRole, CornerRole, CornerShape, Direction, Elevation, MotionKind, MotionSpeed, TypeRole,
};

use crate::{Button, ButtonVariant, IconButton, SemanticTone};

use super::super::modal::{ModalHost, ZLayer, modal_backdrop, motion_duration_ms};
use super::{
    ActionSlot, Confirm, Dialog, DialogAction, DialogCloseReason, DialogColors, DialogProperties,
    DialogState, DialogVariant, ModalSession,
};

#[allow(clippy::too_many_arguments)]
fn build_surface_shell(
    max_height: Pixels,
    text_color: Hsla,
    colors: DialogColors,
    tonal_elevation: Option<Elevation>,
    shadow_elevation: Option<Elevation>,
    shape: Option<CornerShape>,
    full_screen: bool,
    unstructured: bool,
    native: bool,
) -> Surface {
    let role = if tonal_elevation.is_some() {
        ColorRole::Surface
    } else {
        ColorRole::SurfaceContainerHigh
    };
    let mut surface = Surface::new(role)
        .flex()
        .flex_col()
        .max_h(max_height)
        .content_color(text_color);
    if let Some(container) = colors.container {
        surface = surface.container_color(container);
    } else if unstructured && tonal_elevation.is_none() {
        surface = surface.container_color(gpui::transparent_black());
    }
    if let Some(tonal) = tonal_elevation {
        surface = surface.tonal_elevation(tonal);
    }
    if let Some(shape) = shape.or_else(|| {
        (!full_screen && !unstructured && !native).then(|| CornerRole::ExtraLarge.shape())
    }) {
        surface = surface.shape(shape);
    }
    let shadow = shadow_elevation.unwrap_or(if full_screen || unstructured || native {
        Elevation::Level0
    } else {
        Elevation::Level3
    });
    surface.shadow_elevation(shadow)
}

#[allow(clippy::too_many_arguments)]
fn build_actions(
    confirm: Option<(SharedString, Confirm)>,
    confirm_tone: SemanticTone,
    confirm_disabled: bool,
    confirm_loading: bool,
    confirm_content: Option<ActionSlot>,
    confirm_action: DialogAction,
    confirm_focus: &FocusHandle,
    dismiss_content: Option<ActionSlot>,
    dismiss_action: DialogAction,
    cancel_focus: &FocusHandle,
    cancel_label: SharedString,
    show_cancel: bool,
    full_screen: bool,
) -> (Option<AnyElement>, Option<AnyElement>) {
    let confirm_element = match confirm_content {
        Some(content) => Some(content(confirm_action)),
        None => confirm.map(|(label, _)| {
            Button::new("confirm", label)
                .track_focus(confirm_focus)
                .variant(ButtonVariant::Text)
                .tone(confirm_tone)
                .disabled(confirm_disabled)
                .loading(confirm_loading)
                .on_activate(move |window, cx| confirm_action.activate(window, cx))
                .into_any_element()
        }),
    };
    let cancel_element = match dismiss_content {
        Some(content) => Some(content(dismiss_action)),
        None if show_cancel && full_screen => Some(
            IconButton::new(
                "cancel",
                cancel_label,
                crate::Icon::new(crate::IconName::Close),
            )
            .track_focus(cancel_focus)
            .on_activate(move |window, cx| dismiss_action.activate(window, cx))
            .into_any_element(),
        ),
        None if show_cancel => Some(
            Button::new("cancel", cancel_label)
                .track_focus(cancel_focus)
                .variant(ButtonVariant::Text)
                .on_activate(move |window, cx| dismiss_action.activate(window, cx))
                .into_any_element(),
        ),
        None => None,
    };
    (cancel_element, confirm_element)
}

struct StructuredDialogParts {
    icon: Option<AnyElement>,
    heading: Option<gpui::Div>,
    content: AnyElement,
    cancel: Option<AnyElement>,
    confirm: Option<AnyElement>,
}

fn populate_structured_surface(
    surface: Surface,
    parts: StructuredDialogParts,
    width: Pixels,
    viewport_height: Pixels,
    full_screen: bool,
    rtl: bool,
) -> Surface {
    let surface = surface.w(width);
    if full_screen {
        surface
            .h(viewport_height)
            .child(
                div()
                    .id("dialog-app-bar")
                    .test_support()
                    .flex()
                    .flex_none()
                    .items_center()
                    .h(px(56.))
                    .px(px(4.))
                    .gap(px(8.))
                    .when(rtl, Styled::flex_row_reverse)
                    .children(parts.cancel)
                    .children(parts.icon)
                    .child(div().flex_1().min_w_0().children(parts.heading))
                    .children(parts.confirm),
            )
            .child(parts.content)
    } else {
        surface
            .p(px(24.))
            .children(parts.icon.map(|icon| {
                div()
                    .flex_none()
                    .flex()
                    .justify_center()
                    .mb(px(16.))
                    .child(icon)
            }))
            .children(
                parts
                    .heading
                    .map(|heading| div().flex_none().mb(px(16.)).child(heading)),
            )
            .child(parts.content)
            .children(
                (parts.cancel.is_some() || parts.confirm.is_some()).then(|| {
                    div()
                        .flex()
                        .flex_none()
                        .mt(px(24.))
                        .when(rtl, Styled::flex_row_reverse)
                        .justify_end()
                        .flex_wrap()
                        .gap(px(8.))
                        .children(parts.cancel)
                        .children(parts.confirm)
                }),
            )
    }
}

#[allow(clippy::too_many_arguments)]
fn build_structured_parts(
    icon: Option<AnyElement>,
    icon_color: Hsla,
    show_title: bool,
    title: SharedString,
    title_content: Option<AnyElement>,
    title_color: Hsla,
    description: Option<SharedString>,
    body: Option<AnyElement>,
    scroll: &ScrollHandle,
    cancel: Option<AnyElement>,
    confirm: Option<AnyElement>,
    full_screen: bool,
    window: &Window,
    cx: &App,
) -> StructuredDialogParts {
    let icon = icon.map(|icon| div().text_color(icon_color).child(icon).into_any_element());
    let has_icon = icon.is_some();
    let heading = show_title.then(|| {
        let heading = title_content.unwrap_or_else(|| title.into_any_element());
        super::super::text_style(
            div()
                .min_w_0()
                .text_color(title_color)
                .when(has_icon && !full_screen, Styled::text_center)
                .child(heading),
            if full_screen {
                TypeRole::TitleLarge
            } else {
                TypeRole::HeadlineSmall
            },
            1.,
            window,
            cx,
        )
    });
    let supporting = description.map(|description| {
        super::super::text_style(
            div().id("description").role(Role::Label).child(description),
            TypeRole::BodyMedium,
            1.,
            window,
            cx,
        )
    });
    let content = isolated_scroll("dialog-content", scroll)
        .test_support()
        .min_h_0()
        .flex()
        .flex_col()
        .gap(px(16.))
        .when(full_screen, |content| content.flex_1().p(px(24.)))
        .children(supporting)
        .children(body)
        .into_any_element();
    StructuredDialogParts {
        icon,
        heading,
        content,
        cancel,
        confirm,
    }
}

#[allow(clippy::too_many_arguments)]
fn render_exiting_dialog<E: StatefulInteractiveElement + ParentElement + IntoElement>(
    id: ElementId,
    state: &Entity<DialogState>,
    panel: E,
    backdrop: Scrim,
    viewport: Size<Pixels>,
    layer: usize,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    ModalHost::new(id, ZLayer::dialog(layer), viewport)
        .centered()
        .render_exiting("dialog-exit", state, backdrop, panel, window, cx)
}

#[allow(clippy::too_many_arguments)]
fn render_active_dialog<E: IntoElement>(
    title: SharedString,
    panel: E,
    backdrop: Scrim,
    focus: FocusHandle,
    session: ModalSession,
    confirm_action: DialogAction,
    keyboard_confirmation: bool,
    layer: usize,
    native: bool,
    properties: DialogProperties,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    if native {
        let session = session.clone();
        window.on_window_should_close(cx, move |window, cx| {
            session.dismiss(DialogCloseReason::WindowClose, window, cx)
        });
    }
    let outside = session.clone();
    let keyboard_focus = focus.clone();
    gpui_base::Dialog::new(cx)
        .focus_handle(focus)
        .layer(ZLayer::dialog(layer).priority(), true)
        .accessibility_label(title)
        .close_on_backdrop_press(false)
        .backdrop(ThemeScope::inherit(
            modal_backdrop(backdrop, move |window, cx| {
                if !native && properties.dismiss_on_outside_click {
                    outside.dismiss(DialogCloseReason::OutsideClick, window, cx);
                }
            }),
            window,
            cx,
        ))
        .on_ok(move |_, window, cx| {
            if keyboard_confirmation && keyboard_focus.is_focused(window) {
                confirm_action.activate(window, cx);
            } else {
                // Let a focused control handle Enter through its own key listener.
                cx.propagate();
            }
            false
        })
        .on_open_change(move |_, reason, window, cx| {
            if reason == DialogChangeReason::Cancel && properties.dismiss_on_escape {
                session.dismiss(DialogCloseReason::Escape, window, cx);
            }
        })
        .popup(ThemeScope::inherit(panel, window, cx))
        .into_any_element()
}

impl RenderOnce for Dialog {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = self.state.read(cx);
        let native = state.native.is_some();
        if let Some(native) = &state.native
            && native.child.window_id() != window.window_handle().window_id()
        {
            return div().into_any_element();
        }
        let open = state.open;
        let focus = state.focus.clone();
        let scroll = state.scroll.clone();
        let cancel_focus = state.cancel_focus.clone();
        let confirm_focus = state.confirm_focus.clone();
        let session = ModalSession::new(
            &self.state,
            self.dismiss_request.take(),
            self.on_closed.take(),
            cx,
        );
        let theme = current_material_theme(window, cx);
        let duration_ms = motion_duration_ms(theme.motion.spec(
            MotionKind::Spatial,
            if open {
                MotionSpeed::Default
            } else {
                MotionSpeed::Fast
            },
            cx.reduce_motion(),
        ));
        let presence = crate::overlay::presence(&self.id, open, duration_ms, window, cx);
        if !presence.should_render() {
            return div().into_any_element();
        }
        let progress = if native { 1. } else { presence.progress };
        let viewport = window.viewport_size();
        let full_screen = self.variant == DialogVariant::FullScreen && viewport.width < px(600.);
        let window_filling = full_screen || native;
        let margin = if window_filling { px(0.) } else { px(48.) };
        let available_width = (viewport.width - margin).max(px(0.));
        let width = if full_screen {
            available_width
        } else {
            self.width.min(available_width)
        };
        let max_height = (viewport.height - margin).max(px(0.));
        let motion_offset = px((1. - progress) * if full_screen { 32. } else { 8. });
        let rtl = theme.layout.direction == Direction::Rtl;
        let colors = &theme.colors;
        let icon_color = self
            .colors
            .icon
            .unwrap_or_else(|| color(colors.color(ColorRole::Secondary)));
        let title_color = self
            .colors
            .title
            .unwrap_or_else(|| color(colors.color(ColorRole::OnSurface)));
        let text_color = self
            .colors
            .text
            .unwrap_or_else(|| color(colors.color(ColorRole::OnSurfaceVariant)));

        let confirm_action = DialogAction {
            session: session.clone(),
            reason: DialogCloseReason::Confirm,
            validate: self
                .confirm
                .as_ref()
                .map(|(_, validator)| validator.clone()),
        };
        let dismiss_action = DialogAction {
            session: session.clone(),
            reason: DialogCloseReason::Cancel,
            validate: None,
        };
        let keyboard_confirmation = self.confirm.is_some()
            && self.confirm_content.is_none()
            && !self.unstructured
            && !self.confirm_disabled
            && !self.confirm_loading;
        let body = self
            .body
            .map(|body| body(confirm_action.clone(), dismiss_action.clone()));
        let mut surface = build_surface_shell(
            max_height,
            text_color,
            self.colors,
            self.tonal_elevation,
            self.shadow_elevation,
            self.shape,
            full_screen,
            self.unstructured,
            native,
        );
        if self.unstructured {
            surface = surface
                .min_w(px(280.).min(width))
                .max_w(width)
                .children(body);
        } else {
            let (cancel, confirm) = build_actions(
                self.confirm,
                self.confirm_tone,
                self.confirm_disabled,
                self.confirm_loading,
                self.confirm_content,
                confirm_action.clone(),
                &confirm_focus,
                self.dismiss_content,
                dismiss_action,
                &cancel_focus,
                self.cancel,
                self.show_cancel,
                full_screen,
            );
            let parts = build_structured_parts(
                self.icon,
                icon_color,
                self.show_title,
                self.title.clone(),
                self.title_content,
                title_color,
                self.description,
                body,
                &scroll,
                cancel,
                confirm,
                full_screen,
                window,
                cx,
            );
            surface = populate_structured_surface(
                surface,
                parts,
                width,
                viewport.height,
                full_screen,
                rtl,
            );
        }

        let panel = div()
            .id(self.id.clone())
            .test_support()
            .relative()
            .max_w(available_width)
            .max_h(max_height)
            .opacity(progress)
            .mt(motion_offset)
            .when(open, |panel| panel.occlude())
            .child(surface);
        let backdrop = Scrim::new((self.id.clone(), "scrim"))
            .opacity(if native {
                0.
            } else {
                self.scrim_opacity * progress
            })
            .when_some(self.scrim_color, Scrim::color);
        if !open {
            return render_exiting_dialog(
                self.id,
                &self.state,
                panel,
                backdrop,
                viewport,
                self.layer,
                window,
                cx,
            );
        }
        render_active_dialog(
            self.title,
            panel,
            backdrop,
            focus,
            session,
            confirm_action,
            keyboard_confirmation,
            self.layer,
            native,
            self.properties,
            window,
            cx,
        )
    }
}
