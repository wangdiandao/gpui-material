use gpui::{
    Context, ElementId, InteractiveElement, IntoElement, ListOffset, ParentElement, Pixels, Role,
    StatefulInteractiveElement, Styled, Window, div, list, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material::{
    ControlSize, Icon, IconButton, IconName, ListItem, ListItemShapes, Text, TextField,
    TextFieldVariant,
    foundation::{color, window_material_theme},
    theme::{ColorRole, CornerRole, CornerShape, Direction, TypeRole},
};

use super::{CloseCatalog, Gallery};
use crate::{
    shell::catalog::{CATALOG, CatalogRow},
    story::stack,
};

impl Gallery {
    pub(super) fn render_catalog(
        &self,
        single_pane: bool,
        inset: Pixels,
        radius: Pixels,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let rows = self.filtered_rows.clone();
        let handles = self.catalog_focus_handles.clone();
        let no_results = rows.is_empty();
        let current = self.current;
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let surface = color(theme.colors.color(ColorRole::Surface));
        let owner = cx.weak_entity();
        let mut search = TextField::new("gallery-search", "Search catalog", &self.search)
            .variant(TextFieldVariant::Outlined);
        if self.search_has_value {
            let state = self.search.downgrade();
            search = search.trailing(
                IconButton::new("clear-search", "Clear search", Icon::new(IconName::Close))
                    .size(ControlSize::ExtraSmall)
                    .on_activate(move |window, cx| {
                        let _ = state.update(cx, |search, cx| {
                            search.focus(window, cx);
                            search.replace_all("", window, cx);
                        });
                    }),
            );
        }

        let tab_stop = rows
            .iter()
            .enumerate()
            .find_map(|(row_ix, row)| match *row {
                CatalogRow::Entry(entry_ix) if handles[entry_ix].is_focused(window) => Some(row_ix),
                _ => None,
            })
            .or_else(|| {
                rows.iter()
                    .enumerate()
                    .find_map(|(row_ix, row)| match *row {
                        CatalogRow::Entry(entry_ix) if CATALOG[entry_ix].page == current => {
                            Some(row_ix)
                        }
                        _ => None,
                    })
            })
            .or_else(|| {
                rows.iter()
                    .position(|row| matches!(row, CatalogRow::Entry(_)))
            });

        let nav_rows = rows.clone();
        let nav_handles = handles.clone();
        let nav_list = self.catalog_list.clone();

        div()
            .id("gallery-catalog-pane")
            .key_context("GalleryCatalog")
            .test_support()
            .track_focus(&self.catalog_focus)
            .on_action(cx.listener(move |this, _: &CloseCatalog, window, cx| {
                if single_pane {
                    this.toggle_catalog(window, cx);
                } else {
                    cx.propagate();
                }
            }))
            .flex_1()
            .min_w_0()
            .min_h_0()
            .flex()
            .flex_col()
            .rounded(radius)
            .bg(surface)
            .child(
                stack()
                    .gap_3()
                    .flex_none()
                    .px(inset)
                    .py_4()
                    .when(rtl, Styled::items_end)
                    .when(single_pane, |header| {
                        header.child(self.render_section_tabs("drawer-section", window, cx))
                    })
                    .child(search),
            )
            .child(
                div()
                    .id("gallery-catalog")
                    .tab_group()
                    .min_h_0()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .child(if no_results {
                        div()
                            .px(inset)
                            .pb_4()
                            .child(
                                div()
                                    .id("no-components")
                                    .role(Role::Status)
                                    .aria_label("No matching components")
                                    .test_support()
                                    .p_2()
                                    .child(Text::new("No matching components")),
                            )
                            .into_any_element()
                    } else {
                        let render_rows = rows;
                        let render_handles = handles;
                        div()
                            .id("component-catalog")
                            .role(Role::List)
                            .test_support()
                            .flex_1()
                            .min_h_0()
                            .w_full()
                            .on_key_down(move |event, window, cx| {
                                let entry_positions: Vec<(usize, usize)> = nav_rows
                                    .iter()
                                    .enumerate()
                                    .filter_map(|(row_ix, row)| match *row {
                                        CatalogRow::Entry(entry_ix) => Some((row_ix, entry_ix)),
                                        CatalogRow::GroupHeader(_) => None,
                                    })
                                    .collect();
                                if entry_positions.is_empty() {
                                    return;
                                }
                                let Some(active) =
                                    entry_positions.iter().position(|&(_, entry_ix)| {
                                        nav_handles[entry_ix].is_focused(window)
                                    })
                                else {
                                    return;
                                };
                                let count = entry_positions.len();
                                let next = match event.keystroke.key.as_str() {
                                    "up" => Some((active + count - 1) % count),
                                    "down" => Some((active + 1) % count),
                                    "home" => Some(0),
                                    "end" => Some(count - 1),
                                    _ => None,
                                };
                                if let Some(next) = next {
                                    let (row_ix, entry_ix) = entry_positions[next];
                                    nav_handles[entry_ix].focus(window, cx);
                                    if nav_list.bounds_for_item(row_ix).is_some() {
                                        nav_list.scroll_to_reveal_item(row_ix);
                                    } else {
                                        nav_list.scroll_to(ListOffset {
                                            item_ix: row_ix,
                                            offset_in_item: px(0.),
                                        });
                                    }
                                    cx.stop_propagation();
                                    window.refresh();
                                }
                            })
                            .child(
                                list(self.catalog_list.clone(), move |row_ix, _, _| {
                                    match render_rows[row_ix] {
                                        CatalogRow::GroupHeader(group) => div()
                                            .id((ElementId::from("catalog-group"), group.id()))
                                            .w_full()
                                            .px(inset)
                                            .pt(if row_ix == 0 { px(4.) } else { px(14.) })
                                            .pb(px(6.))
                                            .flex()
                                            .when(rtl, Styled::justify_end)
                                            .child(
                                                Text::new(group.title())
                                                    .role(TypeRole::LabelMedium)
                                                    .color(ColorRole::OnSurfaceVariant),
                                            )
                                            .into_any_element(),
                                        CatalogRow::Entry(entry_ix) => {
                                            let entry = CATALOG[entry_ix];
                                            let owner = owner.clone();
                                            let is_first_in_group = row_ix == 0
                                                || matches!(
                                                    render_rows.get(row_ix.saturating_sub(1)),
                                                    Some(CatalogRow::GroupHeader(_))
                                                );
                                            let is_last_in_group = row_ix + 1 == render_rows.len()
                                                || matches!(
                                                    render_rows.get(row_ix + 1),
                                                    Some(CatalogRow::GroupHeader(_))
                                                );
                                            let top = if is_first_in_group { 16. } else { 4. };
                                            let bottom = if is_last_in_group { 16. } else { 4. };
                                            let shapes = ListItemShapes {
                                                resting: Some(CornerShape::Radii([
                                                    top, top, bottom, bottom,
                                                ])),
                                                selected: Some(CornerRole::Large.shape()),
                                                ..ListItemShapes::default()
                                            };
                                            let is_last_row = row_ix + 1 == render_rows.len();
                                            div()
                                                .w_full()
                                                .px(inset)
                                                .pb(if is_last_row { px(16.) } else { px(2.) })
                                                .child(
                                                    ListItem::new(
                                                        (
                                                            ElementId::from("gallery-story"),
                                                            entry.page.id(),
                                                        ),
                                                        entry.page.name(),
                                                    )
                                                    .shapes(shapes)
                                                    .selected(current == entry.page)
                                                    .track_focus(&render_handles[entry_ix])
                                                    .tab_stop(Some(row_ix) == tab_stop)
                                                    .on_activate(move |window, cx| {
                                                        let _ = owner.update(cx, |this, cx| {
                                                            this.navigate(entry.page, window, cx);
                                                        });
                                                    }),
                                                )
                                                .into_any_element()
                                        }
                                    }
                                })
                                .size_full(),
                            )
                            .into_any_element()
                    }),
            )
    }
}
