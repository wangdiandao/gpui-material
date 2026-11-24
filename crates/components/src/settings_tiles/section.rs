use gpui::{
    AnyElement, App, ElementId, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::current_material_theme;
use gpui_material_theme::{ColorRole, CornerShape, Direction, TypeRole};

use crate::{
    ListItem, ListVariant, Text,
    list::{compute_roving_tab_stop, handle_vertical_roving_key, segmented_corners},
};

#[cfg(feature = "settings-tiles")]
use super::{CheckboxTile, ChoiceTile, RadioTile, SliderTile, SwitchTile};

/// Any list row or settings tile that can participate in a [`ListSection`].
pub enum ListSectionItem {
    /// A standard [`ListItem`] row.
    Row(ListItem),
    /// A [`SwitchTile`] settings row.
    #[cfg(feature = "settings-tiles")]
    Switch(SwitchTile),
    /// A [`CheckboxTile`] settings row.
    #[cfg(feature = "settings-tiles")]
    Checkbox(CheckboxTile),
    /// A [`RadioTile`] settings row.
    #[cfg(feature = "settings-tiles")]
    Radio(RadioTile),
    /// A [`SliderTile`] settings row.
    #[cfg(feature = "settings-tiles")]
    Slider(SliderTile),
    /// A [`ChoiceTile`] settings row.
    #[cfg(feature = "settings-tiles")]
    Choice(ChoiceTile),
}

impl From<ListItem> for ListSectionItem {
    fn from(value: ListItem) -> Self {
        Self::Row(value)
    }
}

#[cfg(feature = "settings-tiles")]
impl From<SwitchTile> for ListSectionItem {
    fn from(value: SwitchTile) -> Self {
        Self::Switch(value)
    }
}

#[cfg(feature = "settings-tiles")]
impl From<CheckboxTile> for ListSectionItem {
    fn from(value: CheckboxTile) -> Self {
        Self::Checkbox(value)
    }
}

#[cfg(feature = "settings-tiles")]
impl From<RadioTile> for ListSectionItem {
    fn from(value: RadioTile) -> Self {
        Self::Radio(value)
    }
}

#[cfg(feature = "settings-tiles")]
impl From<SliderTile> for ListSectionItem {
    fn from(value: SliderTile) -> Self {
        Self::Slider(value)
    }
}

#[cfg(feature = "settings-tiles")]
impl From<ChoiceTile> for ListSectionItem {
    fn from(value: ChoiceTile) -> Self {
        Self::Choice(value)
    }
}

enum PreparedItem {
    Row(Box<ListItem>),
    #[cfg(feature = "settings-tiles")]
    Choice(Box<ChoiceTile>),
}

impl PreparedItem {
    fn from_section_item(item: ListSectionItem, window: &Window, cx: &App) -> Self {
        #[cfg(not(feature = "settings-tiles"))]
        let _ = (window, cx);
        match item {
            ListSectionItem::Row(row) => Self::Row(Box::new(row)),
            #[cfg(feature = "settings-tiles")]
            ListSectionItem::Switch(tile) => Self::Row(Box::new(tile.into_list_item())),
            #[cfg(feature = "settings-tiles")]
            ListSectionItem::Checkbox(tile) => Self::Row(Box::new(tile.into_list_item())),
            #[cfg(feature = "settings-tiles")]
            ListSectionItem::Radio(tile) => Self::Row(Box::new(tile.into_list_item())),
            #[cfg(feature = "settings-tiles")]
            ListSectionItem::Slider(tile) => Self::Row(Box::new(tile.into_list_item(window, cx))),
            #[cfg(feature = "settings-tiles")]
            ListSectionItem::Choice(tile) => Self::Choice(Box::new(tile)),
        }
    }

    fn id(&self) -> &ElementId {
        match self {
            Self::Row(row) => row.id(),
            #[cfg(feature = "settings-tiles")]
            Self::Choice(tile) => tile.id(),
        }
    }

    fn custom_focus(&self) -> Option<FocusHandle> {
        match self {
            Self::Row(row) => row.custom_focus(),
            #[cfg(feature = "settings-tiles")]
            Self::Choice(tile) => tile.custom_focus(),
        }
    }

    fn takes_focus(&self, cx: &App) -> bool {
        match self {
            Self::Row(row) => row.takes_focus(cx),
            #[cfg(feature = "settings-tiles")]
            Self::Choice(tile) => tile.takes_focus(),
        }
    }

    fn is_selected(&self) -> bool {
        match self {
            Self::Row(row) => row.is_selected(),
            #[cfg(feature = "settings-tiles")]
            Self::Choice(_) => false,
        }
    }

    fn render(
        self,
        resting: CornerShape,
        selected_shape: CornerShape,
        focus: Option<(FocusHandle, bool)>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        match self {
            Self::Row(row) => (*row).row(resting, selected_shape, focus, window, cx),
            #[cfg(feature = "settings-tiles")]
            Self::Choice(tile) => {
                (*tile).render_in_section(resting, selected_shape, focus, window, cx)
            }
        }
    }
}

/// Titled list section supporting [`ListVariant::Standard`] and Material 3 Expressive
/// [`ListVariant::Segmented`] grouped corner geometry and unified vertical roving focus across
/// [`ListItem`] and settings tiles.
#[derive(IntoElement)]
pub struct ListSection {
    id: ElementId,
    title: Option<SharedString>,
    variant: ListVariant,
    items: Vec<ListSectionItem>,
}

impl ListSection {
    /// Creates an empty [`ListVariant::Standard`] list section with the given element ID.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            title: None,
            variant: ListVariant::default(),
            items: Vec::new(),
        }
    }

    /// Shorthand for a Material 3 Expressive [`ListVariant::Segmented`] section.
    pub fn segmented(id: impl Into<ElementId>) -> Self {
        Self::new(id).variant(ListVariant::Segmented)
    }

    /// Sets an optional section heading displayed above the list and used as the list's accessible name.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the section layout variant (`Standard` or `Segmented`).
    pub fn variant(mut self, variant: ListVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Appends a single [`ListItem`] or settings tile to the section.
    pub fn item(mut self, item: impl Into<ListSectionItem>) -> Self {
        self.items.push(item.into());
        self
    }

    /// Appends multiple rows or settings tiles from an iterator.
    pub fn items<I>(mut self, items: impl IntoIterator<Item = I>) -> Self
    where
        I: Into<ListSectionItem>,
    {
        self.items.extend(items.into_iter().map(Into::into));
        self
    }
}

impl RenderOnce for ListSection {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        window.with_element_namespace((self.id.clone(), "material-list-section"), |window| {
            let direction = current_material_theme(window, cx).layout.direction;
            let rtl = direction == Direction::Rtl;
            let prepared: Vec<_> = self
                .items
                .into_iter()
                .map(|item| PreparedItem::from_section_item(item, window, cx))
                .collect();
            let last = prepared.len().saturating_sub(1);
            let focus: Vec<_> = prepared
                .iter()
                .map(|item| {
                    item.custom_focus().unwrap_or_else(|| {
                        window
                            .use_keyed_state((item.id().clone(), "focus"), cx, |_, cx| {
                                cx.focus_handle()
                            })
                            .read(cx)
                            .clone()
                    })
                })
                .collect();
            let enabled: Vec<_> = prepared.iter().map(|item| item.takes_focus(cx)).collect();
            let selected: Vec<_> = prepared.iter().map(PreparedItem::is_selected).collect();
            let tab_stop = compute_roving_tab_stop(&focus, &enabled, &selected, window);
            let navigation = focus.clone();
            let list_label = self.title.clone();
            let heading = self.title.map(|title| {
                div()
                    .id((self.id.clone(), "section-heading"))
                    .test_support()
                    .w_full()
                    .flex()
                    .when(rtl, Styled::justify_end)
                    .px(px(16.))
                    .pt(px(8.))
                    .pb(px(6.))
                    .child(
                        Text::new(title)
                            .id((self.id.clone(), "section-title"))
                            .role(TypeRole::LabelMedium)
                            .color(ColorRole::OnSurfaceVariant),
                    )
            });
            let list = div()
                .id((self.id.clone(), "list"))
                .role(Role::List)
                .when_some(list_label, |list, label| list.aria_label(label))
                .test_support()
                .flex()
                .flex_col()
                .w_full()
                .on_key_down(move |event, window, cx| {
                    // Nested controls (e.g. SliderTile slider or open ChoiceTile menu)
                    // keep their own arrow-key behavior.
                    handle_vertical_roving_key(&navigation, &enabled, event, direction, window, cx);
                })
                .when(self.variant == ListVariant::Segmented, |list| {
                    list.gap(px(2.))
                })
                .children(prepared.into_iter().zip(focus).enumerate().map(
                    |(ix, (item, focus))| {
                        let (resting, selected_shape) = segmented_corners(self.variant, ix, last);
                        item.render(
                            resting,
                            selected_shape,
                            Some((focus, Some(ix) == tab_stop)),
                            window,
                            cx,
                        )
                    },
                ));
            div()
                .id(self.id)
                .test_support()
                .flex()
                .flex_col()
                .w_full()
                .children(heading)
                .child(list)
        })
    }
}
