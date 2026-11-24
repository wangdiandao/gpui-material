use std::{cell::RefCell, rc::Rc};

use gpui::{
    AnyElement, App, ElementId, Entity, Hsla, InteractiveElement, IntoElement, ParentElement,
    Pixels, ScrollHandle, SharedString, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt as _;
use gpui_material_foundation::{Surface, color, isolated_scroll, resolve_edges};
use gpui_material_theme::{
    ColorRole, CornerRole, CornerShape, Elevation, LogicalEdges, MaterialTheme, TypeRole,
};

use crate::{Button, ButtonVariant, Divider, Icon, IconButton, IconName, Text};

use super::super::bottom_sheet::{
    BottomHandle, BottomSheetState, BottomSheetThresholds, ExpandableGesture,
};
use super::{Change, CloseSlot, SheetClose, SheetColors, SheetEdge};

#[allow(clippy::too_many_arguments)]
pub(super) fn build_handle(
    id: &ElementId,
    bottom: bool,
    open: bool,
    bottom_state: Option<&Entity<BottomSheetState>>,
    has_retained_layout: bool,
    expansion: Option<&(bool, Change)>,
    gesture: Rc<RefCell<ExpandableGesture>>,
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
            BottomHandle::render_expandable(
                id,
                gesture,
                travel,
                *expanded,
                change.clone(),
                tint,
                open,
                window,
                cx,
            )
        })
}

pub(super) fn build_close_control(
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
pub(super) fn build_header(
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

pub(super) fn build_footer(
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

pub(super) fn styled_divider(divider: Divider, color_override: Option<Hsla>) -> Divider {
    match color_override {
        Some(color) => divider.color_override(color),
        None => divider,
    }
}

pub(super) struct SurfaceParts {
    pub(super) handle: Option<AnyElement>,
    pub(super) header: Option<AnyElement>,
    pub(super) content: AnyElement,
    pub(super) footer: Option<AnyElement>,
    pub(super) scroll: ScrollHandle,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_surface(
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
    content_insets: LogicalEdges,
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
    let body_insets = resolve_edges(content_insets, theme.layout.direction);
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
                    .child(if content_insets == LogicalEdges::ZERO {
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
