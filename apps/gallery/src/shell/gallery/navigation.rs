use std::{cell::Cell, rc::Rc};

use gpui::{AnyView, App, AppContext, Context, Focusable, ListOffset, Pixels, Window, point, px};
use gpui_material::{
    foundation::{set_material_theme, set_window_material_theme, window_material_theme},
    theme::{Appearance, ThemePreset},
};

use super::Gallery;
use crate::{
    playground::ComponentPageView,
    playgrounds::playground_for,
    shell::{
        catalog::{CATALOG, CatalogRow, GalleryPage, StylePage, entry_index, filter_catalog_rows},
        overview::OverviewView,
        styles_examples::StylesExamples,
    },
    story::Story,
};

impl Gallery {
    pub(super) fn apply_theme_colors(
        &mut self,
        preset: ThemePreset,
        appearance: Appearance,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.preset = preset;
        let mut theme = window_material_theme(window, cx).clone();
        theme.colors = preset.colors(appearance);
        if self.local_theme {
            set_window_material_theme(Some(theme), window, cx);
        } else {
            set_material_theme(theme, cx);
        }
        cx.notify();
    }

    pub(super) fn select_preset(
        &mut self,
        preset: ThemePreset,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let appearance = window_material_theme(window, cx).colors.appearance();
        self.apply_theme_colors(preset, appearance, window, cx);
    }

    pub(super) fn set_appearance(
        &mut self,
        appearance: Appearance,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let preset = self.preset;
        self.apply_theme_colors(preset, appearance, window, cx);
    }

    pub(super) fn overview_view(&mut self, cx: &mut Context<Self>) -> AnyView {
        if let Some(view) = &self.overview {
            return view.clone().into();
        }
        let owner = cx.weak_entity();
        let on_navigate = {
            let owner = owner.clone();
            Rc::new(
                move |page: GalleryPage, window: &mut Window, cx: &mut App| {
                    let _ = owner.update(cx, |this, cx| this.navigate(page, window, cx));
                },
            )
        };
        let active_preset = {
            let owner = owner.clone();
            Rc::new(move |cx: &App| {
                owner
                    .upgrade()
                    .map(|gallery| gallery.read(cx).preset)
                    .unwrap_or_default()
            })
        };
        let on_select_preset = {
            let owner = owner.clone();
            Rc::new(
                move |preset: ThemePreset, window: &mut Window, cx: &mut App| {
                    let _ = owner.update(cx, |this, cx| this.select_preset(preset, window, cx));
                },
            )
        };
        let on_set_appearance = Rc::new(
            move |appearance: Appearance, window: &mut Window, cx: &mut App| {
                let _ = owner.update(cx, |this, cx| this.set_appearance(appearance, window, cx));
            },
        );
        let available_width = self.available_content_width.clone();
        let view = cx.new(|cx| {
            OverviewView::new(
                on_navigate,
                active_preset,
                on_select_preset,
                on_set_appearance,
                available_width,
                cx,
            )
        });
        self.overview = Some(view.clone());
        view.into()
    }

    pub(super) fn style_view(&mut self, style: StylePage, cx: &mut Context<Self>) -> AnyView {
        if let Some((_, view)) = self.styles.iter().find(|(entry, _)| *entry == style) {
            return view.clone();
        }
        let owner = cx.weak_entity();
        let active_preset = {
            let owner = owner.clone();
            Rc::new(move |cx: &App| {
                owner
                    .upgrade()
                    .map(|gallery| gallery.read(cx).preset)
                    .unwrap_or_default()
            })
        };
        let on_select_preset = Rc::new(
            move |preset: ThemePreset, window: &mut Window, cx: &mut App| {
                let _ = owner.update(cx, |this, cx| this.select_preset(preset, window, cx));
            },
        );
        let view: AnyView = cx
            .new(|_| StylesExamples::new(style, active_preset, on_select_preset))
            .into();
        self.styles.push((style, view.clone()));
        view
    }

    pub(super) fn build_story_view(
        story: Story,
        available_width: Rc<Cell<Pixels>>,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<AnyView> {
        if let Some(playground) = playground_for(story) {
            return Some(
                cx.new(|cx| ComponentPageView::new(story, playground, available_width, window, cx))
                    .into(),
            );
        }
        story.new_view(window, cx)
    }

    pub(super) fn story_view(
        &mut self,
        story: Story,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<AnyView> {
        if let Some((_, view)) = self.stories.iter().find(|(entry, _)| *entry == story) {
            return Some(view.clone());
        }
        let view = Self::build_story_view(story, self.available_content_width.clone(), window, cx)?;
        self.stories.push((story, view.clone()));
        Some(view)
    }

    pub(super) fn query(&self, cx: &App) -> String {
        self.search.read(cx).value().trim().to_ascii_lowercase()
    }

    pub(super) fn refresh_filtered_rows(&mut self, cx: &mut Context<Self>) {
        let has_value = !self.search.read(cx).value().is_empty();
        let query = self.query(cx);
        let next = filter_catalog_rows(&query);
        let list_changed = self.filtered_rows.as_ref() != next.as_slice();
        let clear_button_changed = self.search_has_value != has_value;
        self.search_has_value = has_value;
        if list_changed {
            self.filtered_rows = next.into();
            self.catalog_list.reset(self.filtered_rows.len());
            self.catalog_list.scroll_to(ListOffset {
                item_ix: 0,
                offset_in_item: px(0.),
            });
        }
        if list_changed || clear_button_changed {
            cx.notify();
        }
    }

    pub(super) fn open_search_result(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let query = self.query(cx);
        if query.is_empty() {
            return;
        }
        let result = GalleryPage::find(&query).or_else(|| {
            CATALOG
                .iter()
                .find(|entry| entry.matches(&query))
                .map(|entry| entry.page)
        });
        if let Some(page) = result {
            self.navigate(page, window, cx);
            self.content_focus.focus(window, cx);
        }
    }

    pub(super) fn reveal_page_in_catalog(&self, page: GalleryPage) {
        let Some(target_entry) = entry_index(page) else {
            return;
        };
        let Some(row_ix) = self
            .filtered_rows
            .iter()
            .position(|row| *row == CatalogRow::Entry(target_entry))
        else {
            return;
        };
        if self.catalog_list.bounds_for_item(row_ix).is_some() {
            self.catalog_list.scroll_to_reveal_item(row_ix);
        } else {
            self.catalog_list.scroll_to(ListOffset {
                item_ix: row_ix.saturating_sub(1),
                offset_in_item: px(0.),
            });
        }
    }

    pub(super) fn navigate(
        &mut self,
        page: GalleryPage,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let page_changed = self.current != page;
        if page_changed {
            self.current = page;
            self.content_scroll.set_offset(point(px(0.), px(0.)));
            match page {
                GalleryPage::Story(Story::Introduction) => {
                    self.overview_view(cx);
                }
                GalleryPage::Story(story) => {
                    self.story_view(story, window, cx);
                }
                GalleryPage::Style(style) => {
                    self.style_view(style, cx);
                }
            }
            self.reveal_page_in_catalog(page);
        }
        let closed_catalog = std::mem::take(&mut self.single_pane_catalog_open);
        if Self::single_pane(Self::width_class(window)) {
            self.content_focus.focus(window, cx);
        }
        if page_changed || closed_catalog {
            cx.notify();
        }
    }

    pub(super) fn toggle_catalog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let open = if Self::single_pane(Self::width_class(window)) {
            self.single_pane_catalog_open = !self.single_pane_catalog_open;
            self.single_pane_catalog_open
        } else {
            self.catalog_collapsed = !self.catalog_collapsed;
            !self.catalog_collapsed
        };
        if open {
            self.search.read(cx).focus_handle(cx).focus(window, cx);
        } else {
            self.content_focus.focus(window, cx);
        }
        cx.notify();
    }
}
