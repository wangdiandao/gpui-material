use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    Anchor, AnyElement, App, Axis, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    Role, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{OverlayMotion, current_material_theme, move_roving_focus};
use gpui_material_theme::{
    ColorRole, CornerRole, CornerShape, Direction, LogicalEdges, StateLayer, TypeRole,
};

use crate::menu_appearance::{MenuColor, MenuItemStyle, Paint, Palette};
use crate::{Divider, SemanticTone};

use super::Menu;
use super::item::{Action, Kind, MenuItem};
use super::state::{MenuPresentation, PopupBounds};

pub(super) fn menu_divider(paint: Paint<'_>) -> gpui::Div {
    div()
        .w_full()
        .py(paint.motion.px(4.))
        .child(Divider::horizontal().thickness(paint.motion.px(1.).max(px(1.))))
}

pub(super) fn menu_heading(label: AnyElement, paint: Paint<'_>, palette: Palette) -> gpui::Div {
    div()
        .px(paint.motion.px(12.))
        .py(paint.motion.px(8.))
        .text_color(paint.color(palette.muted))
        .child(label)
}

pub(super) fn row_shape(
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

pub(super) fn row_layout<E: Styled + FluentBuilder>(
    element: E,
    style: MenuItemStyle,
    shape: CornerShape,
    paint: Paint<'_>,
) -> E {
    let min_height = paint.theme.layout.density.visual_size(48.);
    let element = element
        .justify_start()
        .w_full()
        .min_h(paint.motion.px(min_height))
        .line_height(paint.motion.px(20. * paint.theme.typography.text_scale()))
        .gap(paint.length(style.spacing.unwrap_or(px(8.))))
        .when(paint.direction == Direction::Rtl, Styled::flex_row_reverse);
    let element = paint.padding(
        element,
        style.padding.unwrap_or(LogicalEdges::symmetric(12., 0.)),
    );
    paint.shape(element, shape)
}

fn row_color_roles(
    style: MenuItemStyle,
    tone: SemanticTone,
    palette: Palette,
    marked: bool,
    enabled: bool,
) -> (ColorRole, ColorRole, ColorRole, ColorRole) {
    let shortcut = if enabled {
        ColorRole::OnSurfaceVariant
    } else {
        palette.ink
    };
    if marked {
        let (default_bg, default_ink) = if enabled {
            tone.roles().map_or(
                (palette.selected, palette.selected_ink),
                |(_, _, container, on_container)| (container, on_container),
            )
        } else {
            (palette.selected, palette.selected_ink)
        };
        let ink = style.selected_content.unwrap_or(default_ink);
        (
            style.selected_container.unwrap_or(default_bg),
            ink,
            ink,
            shortcut,
        )
    } else {
        let default_ink = if enabled {
            tone.role().unwrap_or(palette.ink)
        } else {
            palette.ink
        };
        (
            style.container.unwrap_or(palette.container),
            style.content.unwrap_or(default_ink),
            style.supporting.unwrap_or(palette.muted),
            shortcut,
        )
    }
}

pub(super) fn row_colors(
    style: MenuItemStyle,
    tone: SemanticTone,
    paint: Paint<'_>,
    palette: Palette,
    marked: bool,
    enabled: bool,
) -> (gpui::Hsla, gpui::Hsla, gpui::Hsla, gpui::Hsla) {
    let (background, foreground, muted, shortcut) =
        row_color_roles(style, tone, palette, marked, enabled);
    let factor = if enabled { 1. } else { 0.38 };
    (
        paint.color(background),
        paint.color(foreground).opacity(factor),
        paint.color(muted).opacity(factor),
        paint.color(shortcut).opacity(factor),
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
                Kind::Heading | Kind::Divider => {}
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
                if matches!(item.kind, Kind::Divider) {
                    return menu_divider(self.paint)
                        .id(item.id)
                        .test_support()
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
        let (background, ink, supporting, shortcut) =
            row_colors(item.style, item.tone, self.paint, palette, marked, enabled);
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
            .children(item.slots.children(
                item.label,
                marked,
                submenu,
                self.paint,
                (supporting, shortcut),
                true,
            ));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_row_color_roles_honor_tone_disabled_and_explicit_style_precedence() {
        use ColorRole::*;
        let palette = MenuColor::Standard.palette(false, None);

        assert_eq!(
            row_color_roles(
                MenuItemStyle::default(),
                SemanticTone::Default,
                palette,
                false,
                true
            ),
            (
                SurfaceContainer,
                OnSurface,
                OnSurfaceVariant,
                OnSurfaceVariant
            )
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
            let (bg, fg, _, shortcut) =
                row_color_roles(MenuItemStyle::default(), tone, palette, false, true);
            assert_eq!(bg, SurfaceContainer);
            assert_eq!(fg, role);
            assert_eq!(shortcut, OnSurfaceVariant);

            let (sel_bg, sel_fg, _, _) =
                row_color_roles(MenuItemStyle::default(), tone, palette, true, true);
            assert_eq!(sel_bg, container);
            assert_eq!(sel_fg, on_container);
        }

        let (_, disabled_fg, _, disabled_shortcut) = row_color_roles(
            MenuItemStyle::default(),
            SemanticTone::Danger,
            palette,
            false,
            false,
        );
        assert_eq!(disabled_fg, OnSurface);
        assert_eq!(disabled_shortcut, OnSurface);

        let style = MenuItemStyle {
            content: Some(Primary),
            selected_container: Some(TertiaryContainer),
            selected_content: Some(OnTertiaryContainer),
            ..Default::default()
        };
        let (_, overridden_fg, _, _) =
            row_color_roles(style, SemanticTone::Danger, palette, false, true);
        assert_eq!(overridden_fg, Primary);
        let (overridden_sel_bg, overridden_sel_fg, _, _) =
            row_color_roles(style, SemanticTone::Danger, palette, true, true);
        assert_eq!(overridden_sel_bg, TertiaryContainer);
        assert_eq!(overridden_sel_fg, OnTertiaryContainer);
    }
}
