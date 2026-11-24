use std::{cell::RefCell, rc::Rc};

use gpui::{
    AnyElement, App, ElementId, Entity, FocusHandle, Hsla, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, Role, Size, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt as _;
use gpui_material_foundation::{Scrim, ThemeScope, color, current_material_theme};
use gpui_material_theme::{ColorRole, MotionKind, MotionSpeed};

use crate::Divider;

use super::{
    super::{
        bottom_sheet::{BottomSheetLayout, BottomSheetState, BottomSheetValue, ExpandableGesture},
        dialog::{
            DialogCloseReason, DialogProperties, DialogState, ModalClosed, ModalDismissRequest,
            ModalSession,
        },
        modal::{ModalHost, ZLayer, modal_backdrop, motion_duration_ms},
    },
    CloseAction, Sheet, SheetBinding, SheetClose, SheetEdge, SheetMode,
    chrome::{
        SurfaceParts, build_close_control, build_footer, build_handle, build_header, build_surface,
        styled_divider,
    },
    geometry::resolve_geometry,
};

struct RetainedConfig {
    available: Size<Pixels>,
    extent: Pixels,
    peek_height: Option<Pixels>,
    mode: SheetMode,
    drag_enabled: bool,
    show_handle: bool,
}

fn configure_retained_bottom(
    state: &Entity<BottomSheetState>,
    config: RetainedConfig,
    on_dismiss_request: &mut Option<ModalDismissRequest>,
    on_closed: &mut Option<ModalClosed>,
    window: &mut Window,
    cx: &mut App,
) -> (Pixels, Pixels) {
    let modal = config.mode == SheetMode::Modal;
    let expanded = config.extent.min(config.available.height);
    let partial = config
        .peek_height
        .unwrap_or_else(|| {
            if modal {
                config.available.height / 2.
            } else {
                px(56.)
            }
        })
        .min(expanded);
    let layout = BottomSheetLayout {
        available: config.available,
        expanded,
        partial,
        mode: config.mode,
        drag_enabled: config.drag_enabled && config.show_handle,
    };
    let height = state.update(cx, |state, cx| {
        state.configure(layout, window, cx);
        state.visible_height()
    });
    let weak = state.downgrade();
    let request = on_dismiss_request.take();
    *on_dismiss_request = Some(Rc::new(move |reason, window, cx| {
        request
            .as_ref()
            .is_none_or(|request| request(reason, window, cx))
            && weak
                .update(cx, |state, cx| state.accept_user_close(window, cx))
                .unwrap_or(false)
    }));
    if !modal && on_closed.is_none() {
        *on_closed = Some(Rc::new(|_, _, _| {}));
    }
    (height, partial.max(px(1.)))
}

#[allow(clippy::too_many_arguments)]
fn render_modal<E: StatefulInteractiveElement + ParentElement + IntoElement>(
    id: ElementId,
    panel: E,
    dialog: &Entity<DialogState>,
    focus: FocusHandle,
    session: Option<ModalSession>,
    open: bool,
    layer: usize,
    viewport: Size<Pixels>,
    properties: DialogProperties,
    scrim_opacity: f32,
    scrim_color: Option<Hsla>,
    visibility: f32,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let z_layer = ZLayer::sheet(layer);
    let scrim = Scrim::new((id.clone(), "scrim"))
        .opacity(scrim_opacity * visibility)
        .when_some(scrim_color, Scrim::color);
    if let Some(session) = session.filter(|_| open) {
        let outside = session.clone();
        // Scope Base's host and focus trap IDs to this sheet, including
        // when nested sheets render as siblings.
        gpui::deferred(
            div().id((id, "sheet-root")).child(
                gpui_base::Sheet::new(cx)
                    .focus_handle(focus)
                    .overlay_interactive(false)
                    .overlay(ThemeScope::inherit(
                        modal_backdrop(scrim, move |window, cx| {
                            if properties.dismiss_on_outside_click {
                                outside.dismiss(DialogCloseReason::OutsideClick, window, cx);
                            }
                        }),
                        window,
                        cx,
                    ))
                    .surface(ThemeScope::inherit(
                        div().absolute().inset_0().overflow_hidden().child(panel),
                        window,
                        cx,
                    ))
                    .request_close(move |window, cx| {
                        cx.stop_propagation();
                        if properties.dismiss_on_escape {
                            session.dismiss(DialogCloseReason::Escape, window, cx);
                        }
                    }),
            ),
        )
        .with_priority(z_layer.priority())
        .into_any_element()
    } else {
        ModalHost::new(id, z_layer, viewport).clip().render_exiting(
            "sheet-exit",
            dialog,
            scrim,
            panel,
            window,
            cx,
        )
    }
}

impl RenderOnce for Sheet {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let modal = self.mode == SheetMode::Modal;
        let viewport = window.viewport_size();
        let available = self.layout_size.unwrap_or(viewport);
        let bottom = self.edge == SheetEdge::Bottom;
        let bottom_width = available.width.min(self.max_width.unwrap_or(px(640.)));
        let side_max_width = self.max_width.unwrap_or(px(400.));
        let bottom_left = (viewport.width - bottom_width) / 2.;
        let (dialog, retained) = match &self.state {
            SheetBinding::Immediate(dialog) => (dialog.clone(), None),
            SheetBinding::Retained(state) => (state.read(cx).dialog.clone(), Some(state.clone())),
        };
        let bottom_state = retained.as_ref().filter(|_| bottom);
        let mut on_dismiss_request = self.on_dismiss_request;
        let mut on_closed = self.on_closed;
        let retained_layout = bottom_state.map(|state| {
            configure_retained_bottom(
                state,
                RetainedConfig {
                    available,
                    extent: self.extent,
                    peek_height: self.peek_height,
                    mode: self.mode,
                    drag_enabled: self.drag_enabled,
                    show_handle: self.show_handle,
                },
                &mut on_dismiss_request,
                &mut on_closed,
                window,
                cx,
            )
        });
        let open = !modal || dialog.read(cx).is_open();
        let theme = current_material_theme(window, cx).clone();
        let duration_ms = motion_duration_ms(theme.motion.spec(
            MotionKind::Spatial,
            if open {
                MotionSpeed::Slow
            } else {
                MotionSpeed::Default
            },
            cx.reduce_motion(),
        ));
        let presence = (modal && retained_layout.is_none()).then(|| {
            self.presence.unwrap_or_else(|| {
                crate::overlay::presence(&self.id, open, duration_ms, window, cx)
            })
        });
        let hidden = bottom_state.is_some_and(|state| {
            let state = state.read(cx);
            state.target_value() == BottomSheetValue::Hidden && state.visible_height() <= px(0.)
        });
        if hidden || presence.is_some_and(|presence| !presence.should_render()) {
            return div().into_any_element();
        }
        let visibility = if let Some((height, peek)) = retained_layout {
            (height / peek).clamp(0., 1.)
        } else {
            presence.map_or(1., |presence| presence.progress)
        };
        let expanded = !bottom
            || self
                .expansion
                .as_ref()
                .is_none_or(|(expanded, _)| *expanded);
        let focus = dialog.read(cx).focus.clone();
        let scroll = dialog.read(cx).scroll.clone();
        let session = modal
            .then(|| ModalSession::new(&dialog, on_dismiss_request.clone(), on_closed.clone(), cx));
        let gesture = window
            .use_keyed_state((self.id.clone(), "drag"), cx, |_, _| {
                Rc::new(RefCell::new(ExpandableGesture::default()))
            })
            .read(cx)
            .clone();
        let geometry = resolve_geometry(
            &self.id,
            &theme,
            self.edge,
            self.placement,
            self.margins,
            modal,
            bottom,
            viewport,
            available,
            self.extent,
            side_max_width,
            expanded,
            self.expansion.is_some(),
            retained_layout,
            visibility,
            &gesture,
            window,
            cx,
        );
        let show_default_close = (modal || on_closed.is_some())
            && bottom_state.is_none_or(|state| {
                state
                    .read(cx)
                    .available_values()
                    .contains(&BottomSheetValue::Hidden)
            });
        let close_action = SheetClose(match session.clone() {
            Some(session) => CloseAction::Modal(session),
            None => CloseAction::Standard(on_dismiss_request, on_closed),
        });
        let side_color = color(theme.colors.color(ColorRole::OnSurfaceVariant));
        let handle = build_handle(
            &self.id,
            bottom,
            open,
            bottom_state,
            retained_layout.is_some(),
            self.expansion.as_ref(),
            gesture,
            geometry.travel,
            &close_action,
            self.handle_content,
            self.handle_label,
            self.show_handle,
            self.thresholds,
            side_color,
            window,
            cx,
        );
        let show_chrome =
            retained_layout.is_none() || handle.is_none() || geometry.height >= px(120.);
        let close = build_close_control(
            self.show_close,
            show_default_close,
            bottom,
            self.close_label,
            self.close_content,
            close_action,
        );
        let header = self.show_header.then(|| {
            build_header(
                self.title.clone(),
                self.header,
                self.back,
                self.expansion,
                close,
                bottom,
                geometry.rtl,
                open,
                self.colors.header,
                side_color,
            )
        });
        let footer = build_footer(
            self.footer,
            self.footer_divider,
            self.colors.divider,
            geometry.rtl,
        );
        let surface = build_surface(
            SurfaceParts {
                handle,
                header,
                content: self.content,
                footer,
                scroll,
            },
            &theme,
            self.edge,
            bottom,
            modal,
            geometry.detached,
            expanded,
            show_chrome,
            retained_layout.is_some(),
            geometry.height,
            geometry.collapsed,
            self.content_insets,
            self.shape,
            self.container_role,
            self.content_role,
            self.colors,
            self.tonal_elevation,
            self.shadow_elevation,
        );
        let edge_separator =
            (!bottom && !modal && !geometry.detached && self.edge_divider).then(|| {
                div()
                    .id("sheet-edge-divider")
                    .test_support()
                    .h_full()
                    .flex_none()
                    .child(styled_divider(Divider::vertical(), self.colors.divider))
            });
        let panel = div()
            .id(self.id.clone())
            .test_support()
            .when(open, |panel| {
                panel
                    .role(if modal { Role::Dialog } else { Role::Group })
                    .aria_label(self.title)
                    .when(modal, |panel| {
                        panel.a11y_synthetic_children(|tree| tree.parent_node().set_modal())
                    })
                    .occlude()
            })
            .when(bottom && !modal, |panel| {
                panel.w_full().max_w(bottom_width).mx_auto()
            })
            .when(!bottom, |panel| {
                panel
                    .flex()
                    .w_full()
                    .min_w_0()
                    .when(geometry.left, Styled::flex_row_reverse)
                    .when(!modal, Styled::h_full)
            })
            .when(modal, |panel| {
                panel.absolute().when_else(
                    bottom,
                    |panel| {
                        panel
                            .bottom(geometry.offscreen)
                            .left(bottom_left)
                            .w(bottom_width)
                    },
                    |panel| {
                        panel
                            .w(geometry.extent)
                            .top(geometry.margins.top)
                            .h(
                                (viewport.height - geometry.margins.top - geometry.margins.bottom)
                                    .max(px(0.)),
                            )
                            .when_else(
                                geometry.left,
                                |panel| panel.left(geometry.margins.left + geometry.offscreen),
                                |panel| panel.right(geometry.margins.right + geometry.offscreen),
                            )
                    },
                )
            })
            .children(edge_separator)
            .child(surface);
        if !modal {
            return if bottom {
                panel.into_any_element()
            } else {
                div()
                    .w(geometry.extent + geometry.margins.left + geometry.margins.right)
                    .max_w_full()
                    .min_w_0()
                    .h_full()
                    .pt(geometry.margins.top)
                    .pb(geometry.margins.bottom)
                    .pl(geometry.margins.left)
                    .pr(geometry.margins.right)
                    .child(panel)
                    .into_any_element()
            };
        }
        render_modal(
            self.id,
            panel,
            &dialog,
            focus,
            session,
            open,
            self.layer,
            viewport,
            self.properties,
            self.scrim_opacity,
            self.colors.scrim,
            visibility,
            window,
            cx,
        )
    }
}
