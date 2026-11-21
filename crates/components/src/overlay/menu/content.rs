use std::cell::Cell;
use std::rc::Rc;

use crate::menu_appearance::{
    MenuColor, MenuItemStyle, MenuSurfaceStyle, Paint, Palette, axis_padding,
};
use crate::motion::{OverlayMotion, ScaledOverlay, menu_pivot};
use gpui::{
    Anchor, AnyElement, App, Axis, ElementId, FocusHandle, InteractiveElement, IntoElement,
    ParentElement, Role, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use gpui_base::{Positioner, TestSupportExt};
use gpui_material_foundation::{ThemeScope, current_material_theme, move_roving_focus};
use gpui_material_theme::{CornerRole, CornerShape, Direction, StateLayer, TypeRole};

use super::{
    Action, ItemSlots, Kind, Menu, MenuItem, MenuPresentation, PopupBounds, PresentedMenu,
};

pub(super) struct MenuExitRow {
    label: SharedString,
    disabled: bool,
    heading: bool,
    section: Option<(MenuSurfaceStyle, Rc<[MenuExitRow]>)>,
    selected: bool,
    submenu: bool,
    slots: ItemSlots,
    style: MenuItemStyle,
}

impl From<&MenuItem> for MenuExitRow {
    fn from(item: &MenuItem) -> Self {
        Self {
            label: item.label.clone(),
            disabled: item.disabled,
            heading: matches!(item.kind, Kind::Heading),
            section: match &item.kind {
                Kind::Section { rows, style, .. } => {
                    Some((*style, rows.iter().map(Self::from).collect()))
                }
                _ => None,
            },
            selected: matches!(item.kind, Kind::Check(true, _) | Kind::Select(true, _)),
            submenu: matches!(item.kind, Kind::Submenu(_)),
            slots: item.slots.clone(),
            style: item.style,
        }
    }
}

pub(super) fn menu_exit(
    (id, depth): (&ElementId, usize),
    bounds: &PopupBounds,
    motion: OverlayMotion,
    direction: Direction,
    window: &Window,
    cx: &App,
) -> Option<AnyElement> {
    let exit = bounds.popup.get();
    if exit.size.width <= px(0.) {
        return None;
    }
    let presented = bounds.presented.borrow();
    let PresentedMenu {
        items,
        presentation,
    } = presented.as_ref()?;
    let trigger = bounds.trigger.get();
    let theme = current_material_theme(window, cx);
    let paint = Paint {
        theme: &theme,
        motion,
        direction,
    };
    let palette = presentation
        .color
        .palette(false, presentation.appearance.container);
    let surface = div()
        .id(match depth {
            0 => ElementId::from((id.clone(), "menu-exit")),
            1 => ElementId::from((id.clone(), "submenu-exit-1")),
            2 => ElementId::from((id.clone(), "submenu-exit-2")),
            3 => ElementId::from((id.clone(), "submenu-exit-3")),
            _ => ElementId::from((id.clone(), format!("submenu-exit-{depth}"))),
        })
        .test_support()
        .h(exit.size.height * motion.scale)
        .w(exit.size.width * motion.scale)
        .opacity(motion.alpha)
        .overflow_y_scroll()
        .track_scroll(&presentation.scroll)
        .child(
            paint
                .surface(presentation.appearance, palette, false)
                .w_full()
                .children(presentation.decoration.as_ref().map(|build| build()))
                .children(exit_rows(items, paint, presentation.color, palette, false)),
        )
        .child(div().absolute().inset_0().occlude());
    Some(
        gpui::deferred(Positioner::corner(Anchor::TopLeft, exit.origin).child(
            ThemeScope::inherit(
                ScaledOverlay::new(
                    super::super::text_style(
                        surface,
                        TypeRole::LabelLarge,
                        motion.scale,
                        window,
                        cx,
                    ),
                    motion,
                    move |bounds| menu_pivot(bounds, trigger),
                ),
                window,
                cx,
            ),
        ))
        .with_priority(100 + depth)
        .into_any_element(),
    )
}

fn exit_rows(
    items: &[MenuExitRow],
    paint: Paint<'_>,
    preset: MenuColor,
    palette: Palette,
    grouped: bool,
) -> Vec<AnyElement> {
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            if let Some((style, children)) = &item.section {
                let palette = preset.palette(true, style.container);
                return paint
                    .surface(*style, palette, true)
                    .child(menu_heading(
                        item.slots.label(item.label.clone()),
                        paint,
                        palette,
                    ))
                    .children(exit_rows(children, paint, preset, palette, true))
                    .into_any_element();
            }
            if item.heading {
                return menu_heading(item.slots.label(item.label.clone()), paint, palette)
                    .into_any_element();
            }
            let style = row_shape(item.style, grouped, index, items.len(), item.selected);
            let (background, ink, supporting) =
                row_colors(item.style, paint, palette, item.selected, !item.disabled);
            row_layout(div().flex().items_center(), item.style, style, paint)
                .text_color(ink)
                .when(item.selected || item.style.container.is_some(), |row| {
                    row.bg(background)
                })
                .children(item.slots.children(
                    item.label.clone(),
                    item.selected,
                    item.submenu,
                    paint,
                    supporting,
                ))
                .into_any_element()
        })
        .collect()
}

fn menu_heading(label: AnyElement, paint: Paint<'_>, palette: Palette) -> gpui::Div {
    div()
        .px(paint.motion.px(12.))
        .py(paint.motion.px(8.))
        .text_color(paint.color(palette.muted))
        .child(label)
}

fn row_shape(
    style: MenuItemStyle,
    grouped: bool,
    index: usize,
    count: usize,
    selected: bool,
) -> CornerShape {
    if selected {
        if let Some(shape) = style.selected_shape {
            return shape;
        }
        if grouped {
            return CornerRole::Medium.shape();
        }
    }
    if let Some(shape) = style.shape {
        return shape;
    }
    if !grouped {
        return CornerRole::None.shape();
    }
    match (index, count) {
        (0, 2..) => CornerShape::Radii([12., 12., 4., 4.]),
        (index, 2..) if index + 1 == count => CornerShape::Radii([4., 4., 12., 12.]),
        _ => CornerRole::ExtraSmall.shape(),
    }
}

fn row_layout<E: Styled + FluentBuilder>(
    element: E,
    style: MenuItemStyle,
    shape: CornerShape,
    paint: Paint<'_>,
) -> E {
    let element = element
        .justify_start()
        .w_full()
        .min_h(paint.motion.px(48.))
        .line_height(paint.motion.px(20. * paint.theme.typography.text_scale()))
        .gap(paint.length(style.spacing.unwrap_or(px(8.))))
        .when(paint.direction == Direction::Rtl, Styled::flex_row_reverse);
    let element = paint.padding(
        element,
        style
            .padding
            .unwrap_or_else(|| axis_padding(px(12.), px(0.))),
    );
    paint.shape(element, shape)
}

fn row_colors(
    style: MenuItemStyle,
    paint: Paint<'_>,
    palette: Palette,
    marked: bool,
    enabled: bool,
) -> (gpui::Hsla, gpui::Hsla, gpui::Hsla) {
    let (background, foreground, muted) = if marked {
        let ink = style.selected_content.unwrap_or(palette.selected_ink);
        (
            style.selected_container.unwrap_or(palette.selected),
            ink,
            ink,
        )
    } else {
        (
            style.container.unwrap_or(palette.container),
            style.content.unwrap_or(palette.ink),
            style.supporting.unwrap_or(palette.muted),
        )
    };
    let factor = if enabled { 1. } else { 0.38 };
    (
        paint.color(background),
        paint.color(foreground).opacity(factor),
        paint.color(muted).opacity(factor),
    )
}

pub(super) fn menu_content(
    items: Vec<MenuItem>,
    close: Action,
    close_level: Option<Action>,
    popup_focus: FocusHandle,
    (direction, bounds, presentation, motion): (
        Direction,
        Rc<PopupBounds>,
        MenuPresentation,
        OverlayMotion,
    ),
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    fn collect_focus(
        items: &[MenuItem],
        focus: &mut Vec<(FocusHandle, bool)>,
        window: &mut Window,
        cx: &mut App,
    ) {
        for item in items {
            match &item.kind {
                Kind::Heading => {}
                Kind::Section { rows, .. } => collect_focus(rows, focus, window, cx),
                _ => {
                    let handle = window
                        .use_keyed_state((item.id.clone(), "focus"), cx, |_, cx| cx.focus_handle());
                    focus.push((handle.read(cx).clone(), !item.disabled));
                }
            }
        }
    }
    let mut focus = Vec::new();
    collect_focus(&items, &mut focus, window, cx);
    let handles: Rc<[(FocusHandle, bool)]> = focus.into();
    if popup_focus.is_focused(window)
        && let Some((first, _)) = handles.iter().find(|(_, enabled)| *enabled)
    {
        let first = first.clone();
        window.defer(cx, move |window, cx| {
            if popup_focus.is_focused(window) {
                first.focus(window, cx);
            }
        });
    }
    let viewport = window.viewport_size();
    let limit = (viewport.height * 0.7).min(px(420.)) * motion.scale;
    let available_width = (viewport.width - px(16.)).max(px(0.)) * motion.scale;
    let theme = current_material_theme(window, cx).clone();
    let paint = Paint {
        theme: &theme,
        motion,
        direction,
    };
    let palette = presentation
        .color
        .palette(false, presentation.appearance.container);
    let rendered_rows = {
        let mut rows = ActiveRows {
            paint,
            focus: handles.iter(),
            close,
            bounds,
            preset: presentation.color,
        };
        rows.render(items, palette, false, window, cx)
    };
    let menu = div()
        .id(if close_level.is_some() {
            "submenu-content"
        } else {
            "menu"
        })
        .test_support()
        .role(Role::Menu)
        .overflow_y_scroll()
        .track_scroll(&presentation.scroll)
        .max_h(limit)
        .on_key_down(move |event, window, cx| {
            let backward = if direction == Direction::Rtl {
                "right"
            } else {
                "left"
            };
            if event.keystroke.key == backward
                && let Some(close_level) = &close_level
            {
                close_level(window, cx);
                cx.stop_propagation();
                return;
            }
            let items: Vec<_> = handles
                .iter()
                .map(|(focus, enabled)| (focus, *enabled))
                .collect();
            if move_roving_focus(
                &items,
                &event.keystroke.key,
                Axis::Vertical,
                direction,
                window,
                cx,
            )
            .is_some()
            {
                cx.stop_propagation();
            }
        })
        .child(
            paint
                .surface(presentation.appearance, palette, false)
                .min_w(motion.px(112.).min(available_width))
                .max_w(motion.px(280.).min(available_width))
                .children(presentation.decoration.as_ref().map(|build| build()))
                .children(rendered_rows),
        );
    super::super::text_style(menu, TypeRole::LabelLarge, motion.scale, window, cx)
        .into_any_element()
}

struct ActiveRows<'a> {
    paint: Paint<'a>,
    focus: std::slice::Iter<'a, (FocusHandle, bool)>,
    close: Action,
    bounds: Rc<PopupBounds>,
    preset: MenuColor,
}

impl ActiveRows<'_> {
    fn render(
        &mut self,
        items: Vec<MenuItem>,
        palette: Palette,
        grouped: bool,
        window: &mut Window,
        cx: &mut App,
    ) -> Vec<AnyElement> {
        let count = items.len();
        items
            .into_iter()
            .enumerate()
            .map(|(index, item)| {
                if let Kind::Section {
                    rows,
                    mut style,
                    inactive_shape,
                } = item.kind
                {
                    let palette = self.preset.palette(true, style.container);
                    let state = inactive_shape.map(|shape| {
                        let state = window.use_keyed_state(
                            (item.id.clone(), "section-hover"),
                            cx,
                            |_, _| Cell::new(false),
                        );
                        if state.read(cx).get() {
                            style.shape = Some(shape);
                        }
                        state
                    });
                    let children = self.render(rows, palette, true, window, cx);
                    return self
                        .paint
                        .surface(style, palette, true)
                        .id(item.id)
                        .test_support()
                        .role(Role::Group)
                        .aria_label(item.label.clone())
                        .when_some(state, |section, state| {
                            section.on_hover(move |hovered, window, cx| {
                                let state = state.read(cx);
                                if state.replace(!hovered) != !hovered {
                                    window.refresh();
                                }
                            })
                        })
                        .child(menu_heading(
                            item.slots.label(item.label),
                            self.paint,
                            palette,
                        ))
                        .children(children)
                        .into_any_element();
                }
                if matches!(item.kind, Kind::Heading) {
                    return menu_heading(item.slots.label(item.label), self.paint, palette)
                        .id(item.id)
                        .role(Role::Label)
                        .into_any_element();
                }
                let shape = row_shape(
                    item.style,
                    grouped,
                    index,
                    count,
                    matches!(item.kind, Kind::Check(true, _) | Kind::Select(true, _)),
                );
                self.row(item, palette, shape)
            })
            .collect()
    }

    fn row(&mut self, item: MenuItem, palette: Palette, shape: CornerShape) -> AnyElement {
        let (focus, enabled) = self
            .focus
            .next()
            .expect("each menu command has a focus entry");
        let enabled = *enabled;
        let action = item.activation();
        let (role, marked) = match item.kind {
            Kind::Check(checked, _) => (Role::MenuItemCheckBox, checked),
            Kind::Select(selected, _) => (Role::MenuItem, selected),
            _ => (Role::MenuItem, false),
        };
        let submenu = matches!(item.kind, Kind::Submenu(_));
        let (background, ink, supporting) =
            row_colors(item.style, self.paint, palette, marked, enabled);
        let row = gpui_base::Button::new(item.id.clone())
            .role(role)
            .track_focus(focus)
            .disabled(!enabled)
            .accessibility_label(item.label.clone())
            .flex()
            .text_color(ink);
        let row = row_layout(row, item.style, shape, self.paint)
            .when(marked || item.style.container.is_some(), |row| {
                row.bg(background)
            })
            .when(enabled, |row| {
                row.hover(move |style| {
                    style.bg(background.blend(ink.opacity(StateLayer::Hover.opacity())))
                })
                .focus_visible(move |style| {
                    style.bg(background.blend(ink.opacity(StateLayer::Focus.opacity())))
                })
                .active(move |style| {
                    style.bg(background.blend(ink.opacity(StateLayer::Pressed.opacity())))
                })
            })
            .when(role == Role::MenuItemCheckBox, |row| {
                row.aria_toggled(if marked {
                    gpui::accesskit::Toggled::True
                } else {
                    gpui::accesskit::Toggled::False
                })
            })
            .when(matches!(item.kind, Kind::Select(..)), |row| {
                row.aria_selected(marked)
            })
            .when(enabled && !submenu, |row| {
                let close = self.close.clone();
                let activate: Action = Rc::new(move |window, cx| {
                    close(window, cx);
                    if let Some(action) = &action {
                        action(window, cx);
                    }
                });
                let keyboard = activate.clone();
                let accessibility = activate.clone();
                row.capture_key_down(move |event, window, cx| {
                    if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                        keyboard(window, cx);
                        cx.stop_propagation();
                    }
                })
                .on_a11y_action(gpui::AccessibleAction::Click, move |_, window, cx| {
                    accessibility(window, cx)
                })
                .on_click(move |_, window, cx| activate(window, cx))
            })
            .children(
                item.slots
                    .children(item.label, marked, submenu, self.paint, supporting),
            );
        if let Kind::Submenu(children) = item.kind {
            Menu {
                close_parent: Some(self.close.clone()),
                enabled,
                ancestor: Some(self.bounds.clone()),
                ..Menu::new((item.id, "submenu"), row, children)
                    .anchor(Anchor::LeftCenter)
                    .color(self.preset)
            }
            .into_any_element()
        } else {
            row.into_any_element()
        }
    }
}
