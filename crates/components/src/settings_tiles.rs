#[cfg(all(feature = "surfaces", feature = "selection"))]
mod switch_tile {
    use std::rc::Rc;

    use gpui::{AnyElement, App, ElementId, IntoElement, RenderOnce, Role, SharedString, Window};
    use gpui_material_theme::CornerRole;

    use crate::{
        ListItem, ListItemColors, ListItemShapes, ListVariant, Switch, SwitchColors, action::Change,
    };

    /// Settings convenience row wrapping [`ListItem`] with a synchronized decorative [`Switch`].
    ///
    /// Clicking anywhere on the row or pressing Enter or Space toggles once with a single focus
    /// stop and `Role::Switch` semantics.
    #[derive(IntoElement)]
    pub struct SwitchTile {
        id: ElementId,
        label: SharedString,
        checked: bool,
        supporting: Option<SharedString>,
        overline: Option<SharedString>,
        leading: Option<AnyElement>,
        disabled: bool,
        variant: ListVariant,
        colors: ListItemColors,
        switch_colors: SwitchColors,
        on_change: Option<Change>,
    }

    impl SwitchTile {
        pub fn new(
            id: impl Into<ElementId>,
            label: impl Into<SharedString>,
            checked: bool,
        ) -> Self {
            Self {
                id: id.into(),
                label: label.into(),
                checked,
                supporting: None,
                overline: None,
                leading: None,
                disabled: false,
                variant: ListVariant::default(),
                colors: ListItemColors::default(),
                switch_colors: SwitchColors::default(),
                on_change: None,
            }
        }

        pub fn supporting(mut self, text: impl Into<SharedString>) -> Self {
            self.supporting = Some(text.into());
            self
        }

        pub fn overline(mut self, text: impl Into<SharedString>) -> Self {
            self.overline = Some(text.into());
            self
        }

        pub fn leading(mut self, leading: impl IntoElement) -> Self {
            self.leading = Some(leading.into_any_element());
            self
        }

        pub fn disabled(mut self, disabled: bool) -> Self {
            self.disabled = disabled;
            self
        }

        pub fn variant(mut self, variant: ListVariant) -> Self {
            self.variant = variant;
            self
        }

        pub fn colors(mut self, colors: ListItemColors) -> Self {
            self.colors = colors;
            self
        }

        pub fn switch_colors(mut self, colors: SwitchColors) -> Self {
            self.switch_colors = colors;
            self
        }

        pub fn on_change(
            mut self,
            handler: impl Fn(bool, &mut Window, &mut App) + 'static,
        ) -> Self {
            self.on_change = Some(Rc::new(handler));
            self
        }
    }

    impl RenderOnce for SwitchTile {
        fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
            let switch = Switch::new(
                (self.id.clone(), "switch"),
                self.label.clone(),
                self.checked,
            )
            .disabled(self.disabled)
            .colors(self.switch_colors)
            .decorative(true);

            let mut item = ListItem::new(self.id, self.label)
                .checked(self.checked)
                .checked_role(Role::Switch)
                .disabled(self.disabled)
                .colors(self.colors)
                .trailing(switch);

            if let Some(supporting) = self.supporting {
                item = item.supporting_text(supporting);
            }
            if let Some(overline) = self.overline {
                item = item.overline(overline);
            }
            if let Some(leading) = self.leading {
                item = item.leading(leading);
            }
            if self.variant == ListVariant::Segmented {
                item = item.shapes(ListItemShapes {
                    resting: Some(CornerRole::Large.shape()),
                    selected: Some(CornerRole::Large.shape()),
                    ..Default::default()
                });
            }
            if let Some(on_change) = self.on_change {
                item = item.on_checked_change(move |next, window, cx| on_change(next, window, cx));
            }
            item
        }
    }
}

#[cfg(all(feature = "surfaces", feature = "selection"))]
pub use switch_tile::SwitchTile;

#[cfg(all(feature = "surfaces", feature = "feedback"))]
mod choice_tile {
    use std::rc::Rc;

    use gpui::{
        Anchor, AnyElement, App, ElementId, Entity, IntoElement, ParentElement, RenderOnce,
        SharedString, Styled, Window, div, prelude::FluentBuilder, px,
    };
    use gpui_material_foundation::{color, current_material_theme};
    use gpui_material_theme::{ColorRole, CornerRole, Direction, TypeRole};

    use crate::{
        DialogState, Icon, IconName, ListItem, ListItemColors, ListItemShapes, ListVariant, Menu,
        MenuItem, Text,
    };

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
        pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
            Self {
                id: id.into(),
                label: label.into(),
                supporting: None,
                leading: None,
                disabled: false,
            }
        }

        pub fn supporting(mut self, text: impl Into<SharedString>) -> Self {
            self.supporting = Some(text.into());
            self
        }

        /// Supplies a repeatable leading icon or graphic for the option row in the popup menu.
        pub fn leading<E: IntoElement>(mut self, build: impl Fn() -> E + 'static) -> Self {
            self.leading = Some(Rc::new(move || build().into_any_element()));
            self
        }

        pub fn disabled(mut self, disabled: bool) -> Self {
            self.disabled = disabled;
            self
        }
    }

    /// Settings convenience row combining [`ListItem`] with an anchored Material [`Menu`] of
    /// [`ChoiceOption`]s.
    #[derive(IntoElement)]
    pub struct ChoiceTile {
        id: ElementId,
        label: SharedString,
        options: Vec<ChoiceOption>,
        selected: SharedString,
        supporting: Option<SharedString>,
        overline: Option<SharedString>,
        leading: Option<AnyElement>,
        placeholder: Option<SharedString>,
        disabled: bool,
        variant: ListVariant,
        colors: ListItemColors,
        menu_state: Option<Entity<DialogState>>,
        on_select: Option<SelectChoice>,
    }

    impl ChoiceTile {
        pub fn new(
            id: impl Into<ElementId>,
            label: impl Into<SharedString>,
            options: impl IntoIterator<Item = ChoiceOption>,
            selected: impl Into<SharedString>,
        ) -> Self {
            Self {
                id: id.into(),
                label: label.into(),
                options: options.into_iter().collect(),
                selected: selected.into(),
                supporting: None,
                overline: None,
                leading: None,
                placeholder: None,
                disabled: false,
                variant: ListVariant::default(),
                colors: ListItemColors::default(),
                menu_state: None,
                on_select: None,
            }
        }

        pub fn supporting(mut self, text: impl Into<SharedString>) -> Self {
            self.supporting = Some(text.into());
            self
        }

        pub fn overline(mut self, text: impl Into<SharedString>) -> Self {
            self.overline = Some(text.into());
            self
        }

        pub fn leading(mut self, leading: impl IntoElement) -> Self {
            self.leading = Some(leading.into_any_element());
            self
        }

        pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
            self.placeholder = Some(text.into());
            self
        }

        pub fn disabled(mut self, disabled: bool) -> Self {
            self.disabled = disabled;
            self
        }

        pub fn variant(mut self, variant: ListVariant) -> Self {
            self.variant = variant;
            self
        }

        pub fn colors(mut self, colors: ListItemColors) -> Self {
            self.colors = colors;
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
    }

    impl RenderOnce for ChoiceTile {
        fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
            let theme = current_material_theme(window, cx);
            let rtl = theme.layout.direction == Direction::Rtl;
            let value_color = if self.disabled {
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

            let trailing = div()
                .flex()
                .items_center()
                .gap(px(4.))
                .text_color(value_color)
                .when(rtl, Styled::flex_row_reverse)
                .when(!current_text.is_empty(), |row| {
                    row.child(
                        Text::new(current_text)
                            .id((self.id.clone(), "choice-value"))
                            .role(TypeRole::BodyMedium),
                    )
                })
                .child(Icon::new(IconName::ArrowDropDown).size(px(20.)));

            let build_row = |id: ElementId,
                             label: SharedString,
                             supporting: Option<SharedString>,
                             overline: Option<SharedString>,
                             leading: Option<AnyElement>,
                             disabled: bool,
                             variant: ListVariant,
                             colors: ListItemColors| {
                let mut item = ListItem::new(id, label)
                    .disabled(disabled)
                    .colors(colors)
                    .trailing(trailing);
                if let Some(supporting) = supporting {
                    item = item.supporting_text(supporting);
                }
                if let Some(overline) = overline {
                    item = item.overline(overline);
                }
                if let Some(leading) = leading {
                    item = item.leading(leading);
                }
                if variant == ListVariant::Segmented {
                    item = item.shapes(ListItemShapes {
                        resting: Some(CornerRole::Large.shape()),
                        selected: Some(CornerRole::Large.shape()),
                        ..Default::default()
                    });
                }
                item
            };

            if self.disabled {
                return build_row(
                    self.id,
                    self.label,
                    self.supporting,
                    self.overline,
                    self.leading,
                    true,
                    self.variant,
                    self.colors,
                )
                .into_any_element();
            }

            let state = self.menu_state.unwrap_or_else(|| {
                window.use_keyed_state((self.id.clone(), "choice-tile-menu-state"), cx, |_, cx| {
                    DialogState::new(cx)
                })
            });
            let open = state.read(cx).is_open();
            let view = window.current_view();
            let trigger_focus = window
                .use_keyed_state((self.id.clone(), "choice-tile-focus"), cx, |_, cx| {
                    cx.focus_handle()
                })
                .read(cx)
                .clone();

            let tile_id = self.id.clone();
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

            let row_id = self.id.clone();
            let label = self.label;
            let supporting = self.supporting;
            let overline = self.overline;
            let leading = self.leading;
            let variant = self.variant;
            let colors = self.colors;
            let row_focus = trigger_focus.clone();

            Menu::composed(
                (self.id.clone(), "menu"),
                menu_items,
                move |is_open, toggle| {
                    build_row(
                        row_id, label, supporting, overline, leading, false, variant, colors,
                    )
                    .track_focus(&row_focus, true)
                    .menu_trigger(is_open, toggle)
                    .into_any_element()
                },
            )
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
}

#[cfg(all(feature = "surfaces", feature = "feedback"))]
pub use choice_tile::{ChoiceOption, ChoiceTile};
