use std::{cell::RefCell, rc::Rc};

use gpui::{
    AnyElement, App, Edges, ElementId, Entity, FocusHandle, Hsla, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, Role, ScrollHandle, SharedString, Size,
    StatefulInteractiveElement, Styled, Window, anchored, div, point, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt as _;
use gpui_material_foundation::{
    Scrim, Surface, ThemeScope, animate, color, current_material_theme, isolated_scroll,
};
use gpui_material_theme::{
    ColorRole, CornerRole, CornerShape, Direction, Elevation, MaterialTheme, MotionKind,
    MotionSpec, MotionSpeed, TypeRole,
};

use crate::{Button, ButtonVariant, Divider, Icon, IconButton, IconName, ScaffoldInsets, Text};

use super::{
    super::{
        bottom_sheet::{
            BottomHandle, BottomSheetLayout, BottomSheetState, BottomSheetThresholds,
            BottomSheetValue,
        },
        dialog::{
            DialogCloseReason, DialogProperties, DialogState, ModalClosed, ModalDismissRequest,
            ModalSession, exiting_panel, modal_backdrop,
        },
    },
    Change, CloseAction, CloseSlot, Sheet, SheetBinding, SheetClose, SheetColors, SheetEdge,
    SheetMargins, SheetMode, SheetPlacement,
    gesture::{Gesture, drag_handle, drag_handle_mark},
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
    on_close: &mut Option<ModalClosed>,
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
    if !modal && on_close.is_none() {
        *on_close = Some(Rc::new(|_, _, _| {}));
    }
    (height, partial.max(px(1.)))
}

struct SheetGeometry {
    left: bool,
    detached: bool,
    rtl: bool,
    margins: Edges<Pixels>,
    extent: Pixels,
    collapsed: Pixels,
    travel: Pixels,
    height: Pixels,
    offscreen: Pixels,
}

#[allow(clippy::too_many_arguments)]
fn resolve_geometry(
    id: &ElementId,
    theme: &MaterialTheme,
    edge: SheetEdge,
    placement: SheetPlacement,
    margins: SheetMargins,
    modal: bool,
    bottom: bool,
    viewport: Size<Pixels>,
    available: Size<Pixels>,
    requested_extent: Pixels,
    side_max_width: Pixels,
    expanded: bool,
    has_expansion: bool,
    retained_layout: Option<(Pixels, Pixels)>,
    visibility: f32,
    gesture: &Rc<RefCell<Gesture>>,
    window: &mut Window,
    cx: &mut App,
) -> SheetGeometry {
    let left = match edge {
        SheetEdge::Start => theme.layout.direction == Direction::Ltr,
        SheetEdge::End => theme.layout.direction == Direction::Rtl,
        SheetEdge::Bottom => false,
    };
    let detached = !bottom && placement == SheetPlacement::Detached;
    let rtl = theme.layout.direction == Direction::Rtl;
    let margins = if detached {
        margins.fit(viewport, rtl)
    } else {
        Edges::default()
    };
    let available_width = (viewport.width - margins.left - margins.right).max(px(0.));
    let target = if bottom {
        requested_extent.min(available.height)
    } else {
        requested_extent.min(side_max_width).min(available_width)
    };
    let extent = if !bottom && !modal {
        px(animate(
            (id.clone(), "side-sheet-width"),
            f32::from(target),
            theme.motion.spec(
                MotionKind::Spatial,
                MotionSpeed::Default,
                cx.reduce_motion(),
            ),
            window,
            cx,
        ))
        .clamp(px(0.), available_width)
    } else {
        target
    };
    // The 48dp handle and 48dp header actions must remain operable at rest.
    let collapsed = px(48. + 8. + 48. + 16.).min(extent);
    let travel = extent - collapsed;
    let height = if let Some((height, _)) = retained_layout {
        height
    } else if has_expansion && bottom {
        let drag = gesture.borrow();
        let tracked = drag.capture.is_some();
        let target = if expanded { extent } else { collapsed } - drag.offset;
        let policy = if tracked {
            MotionSpec::Instant
        } else {
            theme.motion.spec(
                MotionKind::Spatial,
                MotionSpeed::Default,
                cx.reduce_motion(),
            )
        };
        px(animate(
            (id.clone(), "sheet-height"),
            f32::from(target),
            policy,
            window,
            cx,
        ))
        .clamp(collapsed, extent)
    } else {
        extent
    };
    let offscreen = if retained_layout.is_some() {
        px(0.)
    } else {
        -(if bottom { height } else { extent }) * (1. - visibility)
    };
    SheetGeometry {
        left,
        detached,
        rtl,
        margins,
        extent,
        collapsed,
        travel,
        height,
        offscreen,
    }
}

#[allow(clippy::too_many_arguments)]
fn build_handle(
    id: &ElementId,
    bottom: bool,
    open: bool,
    bottom_state: Option<&Entity<BottomSheetState>>,
    has_retained_layout: bool,
    expansion: Option<&(bool, Change)>,
    gesture: Rc<RefCell<Gesture>>,
    travel: Pixels,
    close_action: &SheetClose,
    handle_content: Option<AnyElement>,
    handle_label: SharedString,
    show_handle: bool,
    thresholds: BottomSheetThresholds,
    tint: Hsla,
    window: &mut Window,
    cx: &mut App,
) -> Option<AnyElement> {
    if let Some(state) =
        bottom_state.filter(|state| show_handle || state.read(cx).available_values().len() > 1)
    {
        return Some(
            BottomHandle {
                id: id.clone(),
                state: state.clone(),
                close: close_action.clone(),
                decoration: handle_content,
                label: handle_label,
                grip: show_handle,
                thresholds,
                tint,
                active: open,
            }
            .render(window, cx),
        );
    }
    expansion
        .filter(|_| bottom && !has_retained_layout)
        .map(|(expanded, change)| {
            if open {
                drag_handle(id, gesture, travel, *expanded, change.clone(), window, cx)
            } else {
                div()
                    .w_full()
                    .h(px(48.))
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(drag_handle_mark(tint))
                    .into_any_element()
            }
        })
}

fn build_close_control(
    show_close: bool,
    show_default_close: bool,
    bottom: bool,
    close_label: SharedString,
    close_content: Option<CloseSlot>,
    close_action: SheetClose,
) -> Option<AnyElement> {
    if !show_close {
        return None;
    }
    match close_content {
        Some(content) => Some(content(close_action)),
        None if show_default_close && bottom => Some(
            Button::new("close", close_label)
                .variant(ButtonVariant::Text)
                .on_activate(move |window, cx| close_action.activate(window, cx))
                .into_any_element(),
        ),
        None if show_default_close => Some(
            IconButton::new("close", close_label, Icon::new(IconName::Close))
                .on_activate(move |window, cx| close_action.activate(window, cx))
                .into_any_element(),
        ),
        None => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn build_header(
    title: SharedString,
    header_override: Option<AnyElement>,
    back: Option<AnyElement>,
    expansion: Option<(bool, Change)>,
    close: Option<AnyElement>,
    bottom: bool,
    rtl: bool,
    open: bool,
    header_color: Option<Hsla>,
    side_color: Hsla,
) -> AnyElement {
    let default_title = if bottom {
        Text::new(title)
            .id("sheet-heading")
            .role(TypeRole::TitleMedium)
    } else {
        Text::new(title)
            .id("sheet-heading")
            .role(TypeRole::TitleMedium)
            .wrap()
    };
    div()
        .id("sheet-header")
        .test_support()
        .flex()
        .flex_none()
        .items_center()
        .justify_between()
        .flex_wrap()
        .gap(if bottom { px(8.) } else { px(12.) })
        .when(!bottom && rtl, Styled::flex_row_reverse)
        .children(back)
        .child(
            div()
                .flex_1()
                .min_w(px(56.))
                .max_w_full()
                .when_some(
                    header_color.or((!bottom).then_some(side_color)),
                    Styled::text_color,
                )
                .child(header_override.unwrap_or_else(|| default_title.into_any_element())),
        )
        .when_some(
            expansion.filter(|_| bottom),
            |header, (expanded, change)| {
                header.child(
                    Button::new("expand", if expanded { "Collapse" } else { "Expand" })
                        .variant(ButtonVariant::Text)
                        .on_activate(move |window, cx| {
                            if open {
                                change(!expanded, window, cx);
                            }
                        }),
                )
            },
        )
        .children(close)
        .into_any_element()
}

fn build_footer(
    footer: Option<AnyElement>,
    footer_divider: bool,
    divider_color: Option<Hsla>,
    rtl: bool,
) -> Option<AnyElement> {
    footer.map(|footer| {
        div()
            .id("sheet-footer")
            .test_support()
            .flex()
            .flex_col()
            .flex_none()
            .min_h(px(72.))
            .when(footer_divider, |footer| {
                footer.child(
                    div()
                        .id("sheet-footer-divider")
                        .test_support()
                        .child(styled_divider(Divider::horizontal(), divider_color)),
                )
            })
            .child(
                div()
                    .pt(px(16.))
                    .flex_1()
                    .flex()
                    .items_center()
                    .when(rtl, Styled::justify_end)
                    .child(footer),
            )
            .into_any_element()
    })
}

fn styled_divider(divider: Divider, color_override: Option<Hsla>) -> Divider {
    match color_override {
        Some(color) => divider.color_override(color),
        None => divider,
    }
}

struct SurfaceParts {
    handle: Option<AnyElement>,
    header: Option<AnyElement>,
    content: AnyElement,
    footer: Option<AnyElement>,
    scroll: ScrollHandle,
}

#[allow(clippy::too_many_arguments)]
fn build_surface(
    parts: SurfaceParts,
    theme: &MaterialTheme,
    edge: SheetEdge,
    bottom: bool,
    modal: bool,
    detached: bool,
    expanded: bool,
    show_chrome: bool,
    has_retained_layout: bool,
    height: Pixels,
    collapsed: Pixels,
    content_insets: ScaffoldInsets,
    shape: Option<CornerShape>,
    container_role: Option<ColorRole>,
    content_role: Option<ColorRole>,
    colors: SheetColors,
    tonal_elevation: Elevation,
    shadow_elevation: Option<Elevation>,
) -> Surface {
    let container_role = container_role.unwrap_or(if !bottom && !modal {
        ColorRole::Surface
    } else {
        ColorRole::SurfaceContainerLow
    });
    let shade = |role| color(theme.colors.color(role));
    let content_color = colors.content.or_else(|| content_role.map(shade));
    let shape = shape.unwrap_or_else(|| {
        if bottom {
            CornerRole::ExtraLargeTop.shape()
        } else if detached {
            CornerRole::Large.shape()
        } else if modal {
            if edge == SheetEdge::Start {
                CornerRole::LargeEnd.shape()
            } else {
                CornerRole::LargeStart.shape()
            }
        } else {
            CornerRole::None.shape()
        }
    });
    let elevation = shadow_elevation.unwrap_or(if bottom || modal || detached {
        Elevation::Level1
    } else {
        Elevation::Level0
    });
    let body_insets = content_insets.resolve(theme.layout.direction);
    let has_handle = parts.handle.is_some();
    Surface::new(container_role)
        .shape(shape)
        .shadow_elevation(elevation)
        .tonal_elevation(tonal_elevation)
        .when_some(content_color, Surface::content_color)
        .when_some(colors.container, Surface::container_color)
        .flex()
        .flex_col()
        .gap(px(8.))
        .px(if bottom { px(16.) } else { px(24.) })
        .pb(if bottom { px(16.) } else { px(24.) })
        .when(!has_handle, |surface| surface.pt(px(16.)))
        .children(parts.handle)
        .when_else(
            bottom,
            |surface| surface.w_full().h(height).overflow_hidden(),
            |surface| surface.h_full().flex_1().min_w_0().overflow_hidden(),
        )
        .children(show_chrome.then_some(parts.header).flatten())
        .when(show_chrome && (expanded || height > collapsed), |surface| {
            surface.child(
                isolated_scroll("sheet-content", &parts.scroll)
                    .test_support()
                    .flex_1()
                    .child(if content_insets == ScaffoldInsets::default() {
                        parts.content
                    } else {
                        div()
                            .pt(body_insets.top)
                            .pr(body_insets.right)
                            .pb(body_insets.bottom)
                            .pl(body_insets.left)
                            .child(parts.content)
                            .into_any_element()
                    }),
            )
        })
        .children(
            parts
                .footer
                .filter(|_| !has_retained_layout || height >= px(192.)),
        )
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
    let layer = 9usize.saturating_add(layer);
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
        .with_priority(layer)
        .into_any_element()
    } else {
        gpui::deferred(
            anchored().position(point(px(0.), px(0.))).child(
                div()
                    .id((id.clone(), "sheet-exit"))
                    .absolute()
                    .top_0()
                    .left_0()
                    .w(viewport.width)
                    .h(viewport.height)
                    .overflow_hidden()
                    .child(ThemeScope::inherit(scrim, window, cx))
                    .child(ThemeScope::inherit(
                        exiting_panel(panel, &id, dialog, window, cx),
                        window,
                        cx,
                    )),
            ),
        )
        .with_priority(layer)
        .into_any_element()
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
        let mut on_close = self.on_close;
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
                &mut on_close,
                window,
                cx,
            )
        });
        let open = !modal || dialog.read(cx).is_open();
        let presence = (modal && retained_layout.is_none()).then(|| {
            self.presence
                .unwrap_or_else(|| crate::motion::presence(&self.id, open, 240, window, cx))
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
        let theme = current_material_theme(window, cx).clone();
        let expanded = !bottom
            || self
                .expansion
                .as_ref()
                .is_none_or(|(expanded, _)| *expanded);
        let focus = dialog.read(cx).focus.clone();
        let scroll = dialog.read(cx).scroll.clone();
        let session = modal
            .then(|| ModalSession::new(&dialog, on_dismiss_request.clone(), on_close.clone(), cx));
        let gesture = window
            .use_keyed_state((self.id.clone(), "drag"), cx, |_, _| {
                Rc::new(RefCell::new(Gesture::default()))
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
        let show_default_close = (modal || on_close.is_some())
            && bottom_state.is_none_or(|state| {
                state
                    .read(cx)
                    .available_values()
                    .contains(&BottomSheetValue::Hidden)
            });
        let close_action = SheetClose(match session.clone() {
            Some(session) => CloseAction::Modal(session),
            None => CloseAction::Standard(on_dismiss_request, on_close),
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
