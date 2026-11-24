use std::rc::Rc;

use gpui::{
    Anchor, AnyElement, App, ElementId, Entity, FocusHandle, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_material_foundation::{InteractionSource, color, current_material_theme};
use gpui_material_theme::{ColorRole, CornerRole, CornerShape, Direction, TypeRole};

use crate::{
    DialogState, Icon, IconName, ListItemColors, ListItemShapes, ListVariant, Menu, MenuItem, Text,
};

use super::common::TileFields;

type LeadingFactory = Rc<dyn Fn() -> AnyElement>;
type SelectChoice = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

/// One selectable option in a [`ChoiceTile`] anchored menu.
pub struct ChoiceOption {
    id: SharedString,
    label: SharedString,
    supporting: Option<SharedString>,
    leading: Option<LeadingFactory>,
    disabled: bool,
}

impl ChoiceOption {
    /// Creates a selectable choice option with a stable identifier and visible label.
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            supporting: None,
            leading: None,
            disabled: false,
        }
    }

    /// Sets optional supporting text shown below the option label in the popup menu.
    pub fn supporting(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting = Some(text.into());
        self
    }

    /// Supplies a repeatable leading icon or graphic for the option row in the popup menu.
    pub fn leading<E: IntoElement>(mut self, build: impl Fn() -> E + 'static) -> Self {
        self.leading = Some(Rc::new(move || build().into_any_element()));
        self
    }

    /// Disables selecting this option in the popup menu when `true`.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Settings convenience row combining [`crate::ListItem`] with an anchored Material [`Menu`] of
/// [`ChoiceOption`]s.
#[derive(IntoElement)]
pub struct ChoiceTile {
    fields: TileFields,
    options: Vec<ChoiceOption>,
    selected: SharedString,
    placeholder: Option<SharedString>,
    menu_state: Option<Entity<DialogState>>,
    on_select: Option<SelectChoice>,
}

impl ChoiceTile {
    /// Creates a choice settings row with an element ID, headline label, available options, and currently selected option ID.
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        options: impl IntoIterator<Item = ChoiceOption>,
        selected: impl Into<SharedString>,
    ) -> Self {
        Self {
            fields: TileFields::new(id, label),
            options: options.into_iter().collect(),
            selected: selected.into(),
            placeholder: None,
            menu_state: None,
            on_select: None,
        }
    }

    /// Sets secondary supporting text displayed below the headline.
    pub fn supporting(mut self, text: impl Into<SharedString>) -> Self {
        self.fields.supporting = Some(text.into());
        self
    }

    /// Sets overline text displayed above the headline.
    pub fn overline(mut self, text: impl Into<SharedString>) -> Self {
        self.fields.overline = Some(text.into());
        self
    }

    /// Sets an optional leading icon or graphic.
    pub fn leading(mut self, leading: impl IntoElement) -> Self {
        self.fields.leading = Some(leading.into_any_element());
        self
    }

    /// Sets fallback text shown on the trailing side when `selected` matches no option.
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// Disables opening the menu and dims the tile when `true`.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.fields.disabled = disabled;
        self
    }

    /// Sets standalone row styling (`Standard` or `Segmented`).
    pub fn variant(mut self, variant: ListVariant) -> Self {
        self.fields.variant = variant;
        self
    }

    /// Overrides container and slot colors on the underlying [`crate::ListItem`].
    pub fn colors(mut self, colors: ListItemColors) -> Self {
        self.fields.colors = colors;
        self
    }

    /// Uses a caller-owned focus handle for this tile.
    pub fn track_focus(mut self, focus: &FocusHandle) -> Self {
        self.fields.focus = Some(focus.clone());
        self
    }

    /// Reports hover, focus, press, and disabled states through a caller-owned source.
    pub fn interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
        self.fields.source = Some(source.clone());
        self
    }

    /// Supplies an external `Entity<DialogState>` to observe or control popup menu visibility.
    pub fn menu_state(mut self, state: &Entity<DialogState>) -> Self {
        self.menu_state = Some(state.clone());
        self
    }

    /// Invoked with the selected option's ID when the user picks an option from the menu.
    pub fn on_select(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    pub(super) fn id(&self) -> &ElementId {
        &self.fields.id
    }

    pub(super) fn custom_focus(&self) -> Option<FocusHandle> {
        self.fields.focus.clone()
    }

    pub(super) fn takes_focus(&self) -> bool {
        !self.fields.disabled
    }

    pub(super) fn render_in_section(
        self,
        resting: CornerShape,
        selected_shape: CornerShape,
        focus: Option<(FocusHandle, bool)>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let theme = current_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let value_color = if self.fields.disabled {
            color(theme.colors.color(ColorRole::OnSurface)).opacity(0.38)
        } else {
            color(theme.colors.color(ColorRole::OnSurfaceVariant))
        };

        let current_text = self
            .options
            .iter()
            .find(|option| option.id == self.selected)
            .map(|option| option.label.clone())
            .or(self.placeholder)
            .unwrap_or_default();
        let value_description = (!current_text.is_empty()).then(|| current_text.clone());

        let trailing = div()
            .flex()
            .items_center()
            .gap(px(4.))
            .text_color(value_color)
            .when(rtl, Styled::flex_row_reverse)
            .when(!current_text.is_empty(), |row| {
                row.child(
                    Text::new(current_text)
                        .id((self.fields.id.clone(), "choice-value"))
                        .role(TypeRole::BodyMedium),
                )
            })
            .child(Icon::new(IconName::ArrowDropDown).size(px(20.)));

        let shapes = ListItemShapes {
            resting: Some(resting),
            selected: Some(selected_shape),
            ..Default::default()
        };

        let build_row = move |fields: TileFields| {
            let mut item = fields.into_list_item().shapes(shapes).trailing(trailing);
            if let Some(description) = value_description {
                item = item.aria_description(description);
            }
            item
        };

        if self.fields.disabled {
            return build_row(self.fields).row(resting, selected_shape, None, window, cx);
        }

        let tile_id = self.fields.id.clone();
        let state = self.menu_state.unwrap_or_else(|| {
            window.use_keyed_state((tile_id.clone(), "choice-tile-menu-state"), cx, |_, cx| {
                DialogState::new(cx)
            })
        });
        let open = state.read(cx).is_open();
        let view = window.current_view();
        let (trigger_focus, tab_stop) = focus
            .or_else(|| self.fields.focus.clone().map(|handle| (handle, true)))
            .unwrap_or_else(|| {
                let handle = window
                    .use_keyed_state((tile_id.clone(), "choice-tile-focus"), cx, |_, cx| {
                        cx.focus_handle()
                    })
                    .read(cx)
                    .clone();
                (handle, true)
            });

        let selected_id = self.selected;
        let on_select = self.on_select;
        let menu_items: Vec<MenuItem> = self
            .options
            .into_iter()
            .map(|option| {
                let is_selected = option.id == selected_id;
                let option_id = option.id.clone();
                let mut entry = MenuItem::new((tile_id.clone(), option.id), option.label)
                    .selected(is_selected)
                    .disabled(option.disabled);
                if let Some(supporting) = option.supporting {
                    entry = entry.supporting(move || supporting.clone());
                }
                if let Some(leading) = option.leading {
                    entry = entry.leading(move || leading());
                }
                if let Some(callback) = on_select.clone() {
                    entry = entry.on_activate(move |window, cx| {
                        callback(option_id.clone(), window, cx);
                    });
                }
                entry
            })
            .collect();

        let mut fields = self.fields;
        fields.focus = Some(trigger_focus.clone());

        Menu::composed((tile_id, "menu"), menu_items, move |is_open, toggle| {
            build_row(fields)
                .tab_stop(tab_stop)
                .menu_trigger(is_open, toggle)
                .into_any_element()
        })
        // `Menu` mirrors `Anchor` internally via `gpui_material_foundation::logical_anchor`,
        // so `Anchor::TopRight` aligns to the logical trailing edge in both LTR and RTL.
        .anchor(Anchor::TopRight)
        .open(open)
        .on_open_change(move |requested, window, cx| {
            state.update(cx, |dialog, cx| {
                if requested {
                    dialog.open(window, cx);
                } else {
                    dialog.close(window, cx);
                }
            });
            if requested {
                trigger_focus.focus(window, cx);
            }
            cx.notify(view);
        })
        .into_any_element()
    }
}

impl RenderOnce for ChoiceTile {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (resting, selected_shape) = if self.fields.variant == ListVariant::Segmented {
            (CornerRole::Large.shape(), CornerRole::Large.shape())
        } else {
            (CornerRole::None.shape(), CornerRole::None.shape())
        };
        self.render_in_section(resting, selected_shape, None, window, cx)
    }
}
