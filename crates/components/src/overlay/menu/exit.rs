use std::rc::Rc;

use gpui::{
    Anchor, AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement,
    SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::{Positioner, TestSupportExt};
use gpui_material_foundation::{OverlayMotion, ScaledOverlay, ThemeScope, current_material_theme};
use gpui_material_theme::{Direction, TypeRole};

use crate::SemanticTone;
use crate::menu_appearance::{
    MenuColor, MenuItemStyle, MenuSurfaceStyle, Paint, Palette, menu_pivot,
};

use super::super::modal::ZLayer;
use super::content::{menu_divider, menu_heading, row_colors, row_layout, row_shape};
use super::item::{ItemSlots, Kind, MenuItem};
use super::state::{PopupBounds, PresentedMenu};

pub(super) struct MenuExitRow {
    label: SharedString,
    disabled: bool,
    heading: bool,
    divider: bool,
    section: Option<(MenuSurfaceStyle, Rc<[MenuExitRow]>)>,
    selected: bool,
    submenu: bool,
    slots: ItemSlots,
    tone: SemanticTone,
    style: MenuItemStyle,
}

impl From<&MenuItem> for MenuExitRow {
    fn from(item: &MenuItem) -> Self {
        Self {
            label: item.label.clone(),
            disabled: item.disabled,
            heading: matches!(item.kind, Kind::Heading),
            divider: matches!(item.kind, Kind::Divider),
            section: match &item.kind {
                Kind::Section { rows, style, .. } => {
                    Some((*style, rows.iter().map(Self::from).collect()))
                }
                _ => None,
            },
            selected: matches!(item.kind, Kind::Check(true, _) | Kind::Select(true, _)),
            submenu: matches!(item.kind, Kind::Submenu(_)),
            slots: item.slots.clone(),
            tone: item.tone,
            style: item.style,
        }
    }
}

impl MenuExitRow {
    pub(super) fn matches_slice(rows: &[Self], items: &[MenuItem]) -> bool {
        rows.len() == items.len()
            && rows
                .iter()
                .zip(items.iter())
                .all(|(row, item)| row.matches_item(item))
    }

    fn matches_item(&self, item: &MenuItem) -> bool {
        let selected = matches!(item.kind, Kind::Check(true, _) | Kind::Select(true, _));
        let heading = matches!(item.kind, Kind::Heading);
        let divider = matches!(item.kind, Kind::Divider);
        let submenu = matches!(item.kind, Kind::Submenu(_));
        if self.label != item.label
            || self.disabled != item.disabled
            || self.selected != selected
            || self.heading != heading
            || self.divider != divider
            || self.submenu != submenu
            || self.tone != item.tone
            || self.slots.shortcut != item.slots.shortcut
        {
            return false;
        }
        match (&self.section, &item.kind) {
            (Some((_, section_rows)), Kind::Section { rows, .. }) => {
                Self::matches_slice(section_rows, rows)
            }
            (None, Kind::Section { .. }) | (Some(_), _) => false,
            (None, _) => true,
        }
    }
}

pub(super) fn build_exit_branch(
    id: &ElementId,
    bounds: &Rc<PopupBounds>,
    motion: OverlayMotion,
    direction: Direction,
    window: &Window,
    cx: &App,
) -> gpui::Div {
    // Closing the parent removes Base's nested popup hosts immediately.
    // Use the last presented branch to finish their own visual exits.
    let mut exits = div();
    let mut level = Some(bounds.clone());
    let mut depth = 0;
    while let Some(bounds) = level {
        let Some(previous) = bounds.motion.get() else {
            break;
        };
        let motion = if depth == 0 {
            motion
        } else {
            OverlayMotion {
                scale: 0.8 + (motion.scale - 0.8) * (previous.scale - 0.8) / 0.2,
                alpha: motion.alpha * previous.alpha,
            }
        };
        exits = exits.children(menu_exit(
            (id, depth),
            &bounds,
            motion,
            direction,
            window,
            cx,
        ));
        level = bounds.child.borrow().clone();
        depth += 1;
    }
    exits
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
        .with_priority(ZLayer::menu(depth).priority())
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
            if item.divider {
                return menu_divider(paint).into_any_element();
            }
            let style = row_shape(item.style, grouped, index, items.len(), item.selected);
            let (background, ink, supporting, shortcut) = row_colors(
                item.style,
                item.tone,
                paint,
                palette,
                item.selected,
                !item.disabled,
            );
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
                    (supporting, shortcut),
                    false,
                ))
                .into_any_element()
        })
        .collect()
}
