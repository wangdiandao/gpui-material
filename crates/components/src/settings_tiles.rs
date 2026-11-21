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

        pub(super) fn into_list_item(self) -> ListItem {
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
                .checked_selects_row(false)
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
            if let Some(on_change) = self.on_change {
                item = item.on_checked_change(move |next, window, cx| on_change(next, window, cx));
            }
            item
        }
    }

    impl RenderOnce for SwitchTile {
        fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
            let segmented = self.variant == ListVariant::Segmented;
            let mut item = self.into_list_item();
            if segmented {
                item = item.shapes(ListItemShapes {
                    resting: Some(CornerRole::Large.shape()),
                    selected: Some(CornerRole::Large.shape()),
                    ..Default::default()
                });
            }
            item
        }
    }
}

#[cfg(all(feature = "surfaces", feature = "selection"))]
pub use switch_tile::SwitchTile;

#[cfg(all(feature = "surfaces", feature = "selection"))]
mod checkbox_tile {
    use std::rc::Rc;

    use gpui::{
        AnyElement, App, ElementId, IntoElement, RenderOnce, Role, SharedString, Window,
        accesskit::Toggled,
    };
    use gpui_material_theme::CornerRole;

    use crate::{
        Checkbox, CheckboxColors, CheckboxState, ListItem, ListItemColors, ListItemShapes,
        ListVariant,
    };

    type CheckboxTileChange = Rc<dyn Fn(CheckboxState, &mut Window, &mut App)>;

    /// Settings convenience row wrapping [`ListItem`] with a synchronized decorative [`Checkbox`].
    ///
    /// Clicking anywhere on the row or pressing Enter or Space requests the next [`CheckboxState`]
    /// with a single focus stop and `Role::CheckBox` semantics.
    #[derive(IntoElement)]
    pub struct CheckboxTile {
        id: ElementId,
        label: SharedString,
        state: CheckboxState,
        supporting: Option<SharedString>,
        overline: Option<SharedString>,
        leading: Option<AnyElement>,
        trailing_checkbox: bool,
        error: bool,
        disabled: bool,
        variant: ListVariant,
        colors: ListItemColors,
        checkbox_colors: CheckboxColors,
        on_change: Option<CheckboxTileChange>,
    }

    impl CheckboxTile {
        pub fn new(
            id: impl Into<ElementId>,
            label: impl Into<SharedString>,
            state: CheckboxState,
        ) -> Self {
            Self {
                id: id.into(),
                label: label.into(),
                state,
                supporting: None,
                overline: None,
                leading: None,
                trailing_checkbox: true,
                error: false,
                disabled: false,
                variant: ListVariant::default(),
                colors: ListItemColors::default(),
                checkbox_colors: CheckboxColors::default(),
                on_change: None,
            }
        }

        /// Convenience constructor for a two-state boolean checkbox tile.
        pub fn bool(
            id: impl Into<ElementId>,
            label: impl Into<SharedString>,
            checked: bool,
        ) -> Self {
            Self::new(
                id,
                label,
                if checked {
                    CheckboxState::Checked
                } else {
                    CheckboxState::Unchecked
                },
            )
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

        /// Places the decorative checkbox in the trailing slot (`true`, default) or leading slot (`false`).
        pub fn trailing_checkbox(mut self, trailing: bool) -> Self {
            self.trailing_checkbox = trailing;
            self
        }

        pub fn error(mut self, error: bool) -> Self {
            self.error = error;
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

        pub fn checkbox_colors(mut self, colors: CheckboxColors) -> Self {
            self.checkbox_colors = colors;
            self
        }

        pub fn on_change(
            mut self,
            handler: impl Fn(CheckboxState, &mut Window, &mut App) + 'static,
        ) -> Self {
            self.on_change = Some(Rc::new(handler));
            self
        }

        pub(super) fn into_list_item(self) -> ListItem {
            let checkbox = Checkbox::new(
                (self.id.clone(), "checkbox"),
                self.label.clone(),
                self.state,
            )
            .error(self.error)
            .disabled(self.disabled)
            .colors(self.checkbox_colors)
            .decorative(true);

            let toggled = match self.state {
                CheckboxState::Unchecked => Toggled::False,
                CheckboxState::Checked => Toggled::True,
                CheckboxState::Indeterminate => Toggled::Mixed,
            };

            let mut item = ListItem::new(self.id, self.label)
                .checked_toggled(toggled)
                .checked_role(Role::CheckBox)
                .checked_selects_row(false)
                .invalid(self.error)
                .disabled(self.disabled)
                .colors(self.colors);

            if self.trailing_checkbox {
                item = item.trailing(checkbox);
                if let Some(leading) = self.leading {
                    item = item.leading(leading);
                }
            } else {
                item = item.leading(checkbox);
            }

            if let Some(supporting) = self.supporting {
                item = item.supporting_text(supporting);
            }
            if let Some(overline) = self.overline {
                item = item.overline(overline);
            }
            if let Some(on_change) = self.on_change {
                let next = if self.state == CheckboxState::Checked {
                    CheckboxState::Unchecked
                } else {
                    CheckboxState::Checked
                };
                item = item.on_activate(move |window, cx| on_change(next, window, cx));
            }
            item
        }
    }

    impl RenderOnce for CheckboxTile {
        fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
            let segmented = self.variant == ListVariant::Segmented;
            let mut item = self.into_list_item();
            if segmented {
                item = item.shapes(ListItemShapes {
                    resting: Some(CornerRole::Large.shape()),
                    selected: Some(CornerRole::Large.shape()),
                    ..Default::default()
                });
            }
            item
        }
    }
}

#[cfg(all(feature = "surfaces", feature = "selection"))]
pub use checkbox_tile::CheckboxTile;

#[cfg(all(feature = "surfaces", feature = "selection"))]
mod radio_tile {
    use std::rc::Rc;

    use gpui::{
        AnyElement, App, ElementId, IntoElement, RenderOnce, Role, SharedString, Window,
        accesskit::Toggled,
    };
    use gpui_material_theme::CornerRole;

    use crate::{
        ListItem, ListItemColors, ListItemShapes, ListVariant, Radio, RadioColors, action::Activate,
    };

    /// Settings convenience row wrapping [`ListItem`] with a synchronized decorative [`Radio`].
    ///
    /// Clicking anywhere on an unselected row or pressing Enter or Space invokes `.on_select`
    /// with a single focus stop and `Role::RadioButton` semantics.
    #[derive(IntoElement)]
    pub struct RadioTile {
        id: ElementId,
        label: SharedString,
        selected: bool,
        supporting: Option<SharedString>,
        overline: Option<SharedString>,
        leading: Option<AnyElement>,
        trailing_radio: bool,
        disabled: bool,
        variant: ListVariant,
        colors: ListItemColors,
        radio_colors: RadioColors,
        on_select: Option<Activate>,
    }

    impl RadioTile {
        pub fn new(
            id: impl Into<ElementId>,
            label: impl Into<SharedString>,
            selected: bool,
        ) -> Self {
            Self {
                id: id.into(),
                label: label.into(),
                selected,
                supporting: None,
                overline: None,
                leading: None,
                trailing_radio: true,
                disabled: false,
                variant: ListVariant::default(),
                colors: ListItemColors::default(),
                radio_colors: RadioColors::default(),
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

        /// Places the decorative radio indicator in the trailing slot (`true`, default) or leading slot (`false`).
        pub fn trailing_radio(mut self, trailing: bool) -> Self {
            self.trailing_radio = trailing;
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

        pub fn radio_colors(mut self, colors: RadioColors) -> Self {
            self.radio_colors = colors;
            self
        }

        pub fn on_select(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
            self.on_select = Some(Rc::new(handler));
            self
        }

        pub(super) fn into_list_item(self) -> ListItem {
            let radio = Radio::new((self.id.clone(), "radio"), self.label.clone())
                .selected(self.selected)
                .disabled(self.disabled)
                .colors(self.radio_colors)
                .decorative(true);

            let toggled = if self.selected {
                Toggled::True
            } else {
                Toggled::False
            };

            let mut item = ListItem::new(self.id, self.label)
                .checked_toggled(toggled)
                .checked_role(Role::RadioButton)
                .checked_selects_row(false)
                .disabled(self.disabled)
                .colors(self.colors);

            if self.trailing_radio {
                item = item.trailing(radio);
                if let Some(leading) = self.leading {
                    item = item.leading(leading);
                }
            } else {
                item = item.leading(radio);
            }

            if let Some(supporting) = self.supporting {
                item = item.supporting_text(supporting);
            }
            if let Some(overline) = self.overline {
                item = item.overline(overline);
            }
            if let Some(on_select) = self.on_select {
                let already_selected = self.selected;
                item = item.on_activate(move |window, cx| {
                    if !already_selected {
                        on_select(window, cx);
                    }
                });
            }
            item
        }
    }

    impl RenderOnce for RadioTile {
        fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
            let segmented = self.variant == ListVariant::Segmented;
            let mut item = self.into_list_item();
            if segmented {
                item = item.shapes(ListItemShapes {
                    resting: Some(CornerRole::Large.shape()),
                    selected: Some(CornerRole::Large.shape()),
                    ..Default::default()
                });
            }
            item
        }
    }
}

#[cfg(all(feature = "surfaces", feature = "selection"))]
pub use radio_tile::RadioTile;

#[cfg(all(feature = "surfaces", feature = "selection"))]
mod slider_tile {
    use std::rc::Rc;

    use gpui::{
        AnyElement, App, ElementId, Entity, IntoElement, ParentElement, RenderOnce, SharedString,
        Styled, Window, div,
    };
    use gpui_material_foundation::{color, current_material_theme};
    use gpui_material_theme::{ColorRole, CornerRole, TypeRole};

    use crate::{
        ListItem, ListItemColors, ListItemShapes, ListVariant, Slider, SliderState, SliderValue,
        Text,
    };

    type FormatValue = Rc<dyn Fn(f32) -> SharedString>;
    type SliderConfig = Box<dyn FnOnce(Slider) -> Slider>;

    pub(super) fn format_slider_scalar(value: f32) -> SharedString {
        if !value.is_finite() {
            return "0".into();
        }
        let rounded = value.round();
        if (value - rounded).abs() < 1e-4 {
            format!("{}", rounded as i64).into()
        } else {
            format!("{value:.2}").into()
        }
    }

    pub(super) fn format_slider_value(
        value: SliderValue,
        formatter: Option<&FormatValue>,
    ) -> SharedString {
        let format_one = |v: f32| match formatter {
            Some(f) => f(v),
            None => format_slider_scalar(v),
        };
        if value.is_range() {
            format!(
                "{} – {}",
                format_one(value.start()),
                format_one(value.end())
            )
            .into()
        } else {
            format_one(value.end())
        }
    }

    /// Settings convenience row combining a [`ListItem`] header, live formatted value display,
    /// and an embedded [`Slider`] bound to a caller-owned [`SliderState`].
    #[derive(IntoElement)]
    pub struct SliderTile {
        id: ElementId,
        label: SharedString,
        state: Entity<SliderState>,
        supporting: Option<SharedString>,
        overline: Option<SharedString>,
        leading: Option<AnyElement>,
        disabled: bool,
        variant: ListVariant,
        colors: ListItemColors,
        format_value: Option<FormatValue>,
        configure_slider: Option<SliderConfig>,
    }

    impl SliderTile {
        pub fn new(
            id: impl Into<ElementId>,
            label: impl Into<SharedString>,
            state: &Entity<SliderState>,
        ) -> Self {
            Self {
                id: id.into(),
                label: label.into(),
                state: state.clone(),
                supporting: None,
                overline: None,
                leading: None,
                disabled: false,
                variant: ListVariant::default(),
                colors: ListItemColors::default(),
                format_value: None,
                configure_slider: None,
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

        /// Formats the live numeric value displayed on the trailing side of the header row.
        pub fn format_value(mut self, formatter: impl Fn(f32) -> SharedString + 'static) -> Self {
            self.format_value = Some(Rc::new(formatter));
            self
        }

        /// Customizes the embedded [`Slider`] (colors, tick intervals, size, value indicator, etc.).
        pub fn slider(mut self, configure: impl FnOnce(Slider) -> Slider + 'static) -> Self {
            self.configure_slider = Some(Box::new(configure));
            self
        }

        pub(super) fn into_list_item(self, window: &Window, cx: &App) -> ListItem {
            let theme = current_material_theme(window, cx);
            let value_color = if self.disabled {
                color(theme.colors.color(ColorRole::OnSurface)).opacity(0.38)
            } else {
                color(theme.colors.color(ColorRole::OnSurfaceVariant))
            };
            let current = self.state.read(cx).value();
            let value_text = format_slider_value(current, self.format_value.as_ref());

            let mut slider =
                Slider::new((self.id.clone(), "slider"), self.label.clone(), &self.state)
                    .disabled(self.disabled);
            if let Some(formatter) = self.format_value {
                slider = slider.format_value(move |value| formatter(value));
            }
            if let Some(configure) = self.configure_slider {
                slider = configure(slider);
            }

            let trailing = div().text_color(value_color).child(
                Text::new(value_text)
                    .id((self.id.clone(), "slider-value"))
                    .role(TypeRole::BodyMedium),
            );

            let mut item = ListItem::new(self.id, self.label)
                .disabled(self.disabled)
                .colors(self.colors)
                .trailing(trailing)
                .footer(slider);

            if let Some(supporting) = self.supporting {
                item = item.supporting_text(supporting);
            }
            if let Some(overline) = self.overline {
                item = item.overline(overline);
            }
            if let Some(leading) = self.leading {
                item = item.leading(leading);
            }
            item
        }
    }

    impl RenderOnce for SliderTile {
        fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
            let segmented = self.variant == ListVariant::Segmented;
            let mut item = self.into_list_item(window, cx);
            if segmented {
                item = item.shapes(ListItemShapes {
                    resting: Some(CornerRole::Large.shape()),
                    selected: Some(CornerRole::Large.shape()),
                    ..Default::default()
                });
            }
            item
        }
    }
}

#[cfg(all(feature = "surfaces", feature = "selection"))]
pub use slider_tile::SliderTile;

#[cfg(all(feature = "surfaces", feature = "feedback"))]
mod choice_tile {
    use std::rc::Rc;

    use gpui::{
        Anchor, AnyElement, App, ElementId, Entity, FocusHandle, IntoElement, ParentElement,
        RenderOnce, SharedString, Styled, Window, div, prelude::FluentBuilder, px,
    };
    use gpui_material_foundation::{color, current_material_theme};
    use gpui_material_theme::{ColorRole, CornerRole, CornerShape, Direction, TypeRole};

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

        pub(super) fn id(&self) -> &ElementId {
            &self.id
        }

        pub(super) fn takes_focus(&self) -> bool {
            !self.disabled
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

            let shapes = ListItemShapes {
                resting: Some(resting),
                selected: Some(selected_shape),
                ..Default::default()
            };

            let build_row = move |id: ElementId,
                                  label: SharedString,
                                  supporting: Option<SharedString>,
                                  overline: Option<SharedString>,
                                  leading: Option<AnyElement>,
                                  disabled: bool,
                                  colors: ListItemColors| {
                let mut item = ListItem::new(id, label)
                    .disabled(disabled)
                    .colors(colors)
                    .shapes(shapes)
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
                    self.colors,
                )
                .row(resting, selected_shape, None, window, cx);
            }

            let state = self.menu_state.unwrap_or_else(|| {
                window.use_keyed_state((self.id.clone(), "choice-tile-menu-state"), cx, |_, cx| {
                    DialogState::new(cx)
                })
            });
            let open = state.read(cx).is_open();
            let view = window.current_view();
            let (trigger_focus, tab_stop) = focus.unwrap_or_else(|| {
                let handle = window
                    .use_keyed_state((self.id.clone(), "choice-tile-focus"), cx, |_, cx| {
                        cx.focus_handle()
                    })
                    .read(cx)
                    .clone();
                (handle, true)
            });

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
            let colors = self.colors;
            let row_focus = trigger_focus.clone();

            Menu::composed(
                (self.id.clone(), "menu"),
                menu_items,
                move |is_open, toggle| {
                    build_row(row_id, label, supporting, overline, leading, false, colors)
                        .track_focus(&row_focus, tab_stop)
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

    impl RenderOnce for ChoiceTile {
        fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
            let (resting, selected_shape) = if self.variant == ListVariant::Segmented {
                (CornerRole::Large.shape(), CornerRole::Large.shape())
            } else {
                (CornerRole::None.shape(), CornerRole::None.shape())
            };
            self.render_in_section(resting, selected_shape, None, window, cx)
        }
    }
}

#[cfg(all(feature = "surfaces", feature = "feedback"))]
pub use choice_tile::{ChoiceOption, ChoiceTile};

#[cfg(feature = "surfaces")]
mod list_section {
    use gpui::{
        AnyElement, App, Axis, ElementId, FocusHandle, InteractiveElement, IntoElement,
        ParentElement, RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled, Window,
        div, prelude::FluentBuilder, px,
    };
    use gpui_base::TestSupportExt;
    use gpui_material_foundation::{current_material_theme, move_roving_focus};
    use gpui_material_theme::{ColorRole, CornerRole, CornerShape, Direction, TypeRole};

    use crate::{ListItem, ListVariant, Text};

    #[cfg(feature = "feedback")]
    use super::ChoiceTile;
    #[cfg(feature = "selection")]
    use super::{CheckboxTile, RadioTile, SliderTile, SwitchTile};

    /// Any list row or settings tile that can participate in a [`ListSection`].
    pub enum ListSectionItem {
        Row(ListItem),
        #[cfg(feature = "selection")]
        Switch(SwitchTile),
        #[cfg(feature = "selection")]
        Checkbox(CheckboxTile),
        #[cfg(feature = "selection")]
        Radio(RadioTile),
        #[cfg(feature = "selection")]
        Slider(SliderTile),
        #[cfg(feature = "feedback")]
        Choice(ChoiceTile),
    }

    impl From<ListItem> for ListSectionItem {
        fn from(value: ListItem) -> Self {
            Self::Row(value)
        }
    }

    #[cfg(feature = "selection")]
    impl From<SwitchTile> for ListSectionItem {
        fn from(value: SwitchTile) -> Self {
            Self::Switch(value)
        }
    }

    #[cfg(feature = "selection")]
    impl From<CheckboxTile> for ListSectionItem {
        fn from(value: CheckboxTile) -> Self {
            Self::Checkbox(value)
        }
    }

    #[cfg(feature = "selection")]
    impl From<RadioTile> for ListSectionItem {
        fn from(value: RadioTile) -> Self {
            Self::Radio(value)
        }
    }

    #[cfg(feature = "selection")]
    impl From<SliderTile> for ListSectionItem {
        fn from(value: SliderTile) -> Self {
            Self::Slider(value)
        }
    }

    #[cfg(feature = "feedback")]
    impl From<ChoiceTile> for ListSectionItem {
        fn from(value: ChoiceTile) -> Self {
            Self::Choice(value)
        }
    }

    enum PreparedItem {
        Row(Box<ListItem>),
        #[cfg(feature = "feedback")]
        Choice(Box<ChoiceTile>),
    }

    impl PreparedItem {
        fn from_section_item(item: ListSectionItem, window: &Window, cx: &App) -> Self {
            #[cfg(not(feature = "selection"))]
            let _ = (window, cx);
            match item {
                ListSectionItem::Row(row) => Self::Row(Box::new(row)),
                #[cfg(feature = "selection")]
                ListSectionItem::Switch(tile) => Self::Row(Box::new(tile.into_list_item())),
                #[cfg(feature = "selection")]
                ListSectionItem::Checkbox(tile) => Self::Row(Box::new(tile.into_list_item())),
                #[cfg(feature = "selection")]
                ListSectionItem::Radio(tile) => Self::Row(Box::new(tile.into_list_item())),
                #[cfg(feature = "selection")]
                ListSectionItem::Slider(tile) => {
                    Self::Row(Box::new(tile.into_list_item(window, cx)))
                }
                #[cfg(feature = "feedback")]
                ListSectionItem::Choice(tile) => Self::Choice(Box::new(tile)),
            }
        }

        fn id(&self) -> &ElementId {
            match self {
                Self::Row(row) => row.id(),
                #[cfg(feature = "feedback")]
                Self::Choice(tile) => tile.id(),
            }
        }

        fn takes_focus(&self, cx: &App) -> bool {
            match self {
                Self::Row(row) => row.takes_focus(cx),
                #[cfg(feature = "feedback")]
                Self::Choice(tile) => tile.takes_focus(),
            }
        }

        fn is_selected(&self) -> bool {
            match self {
                Self::Row(row) => row.is_selected(),
                #[cfg(feature = "feedback")]
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
                #[cfg(feature = "feedback")]
                Self::Choice(tile) => {
                    (*tile).render_in_section(resting, selected_shape, focus, window, cx)
                }
            }
        }
    }

    pub(super) fn section_corners(
        variant: ListVariant,
        ix: usize,
        last: usize,
    ) -> (CornerShape, CornerShape) {
        match variant {
            ListVariant::Standard => (CornerRole::None.shape(), CornerRole::None.shape()),
            ListVariant::Segmented => {
                let top = if ix == 0 { 16. } else { 4. };
                let bottom = if ix == last { 16. } else { 4. };
                (
                    CornerShape::Radii([top, top, bottom, bottom]),
                    CornerRole::Large.shape(),
                )
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

        pub fn title(mut self, title: impl Into<SharedString>) -> Self {
            self.title = Some(title.into());
            self
        }

        pub fn variant(mut self, variant: ListVariant) -> Self {
            self.variant = variant;
            self
        }

        pub fn item(mut self, item: impl Into<ListSectionItem>) -> Self {
            self.items.push(item.into());
            self
        }

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
                        window
                            .use_keyed_state((item.id().clone(), "focus"), cx, |_, cx| {
                                cx.focus_handle()
                            })
                            .read(cx)
                            .clone()
                    })
                    .collect();
                let enabled: Vec<_> = prepared.iter().map(|item| item.takes_focus(cx)).collect();
                let tab_stop = focus
                    .iter()
                    .enumerate()
                    .position(|(ix, focus)| enabled[ix] && focus.is_focused(window))
                    .or_else(|| {
                        prepared
                            .iter()
                            .enumerate()
                            .position(|(ix, item)| enabled[ix] && item.is_selected())
                    })
                    .or_else(|| enabled.iter().position(|enabled| *enabled));
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
                        if !navigation.iter().any(|focus| focus.is_focused(window)) {
                            return;
                        }
                        let key = event.keystroke.key.as_str();
                        let moved = if key == "home" {
                            enabled.iter().position(|enabled| *enabled).inspect(|&ix| {
                                navigation[ix].focus(window, cx);
                            })
                        } else if key == "end" {
                            enabled.iter().rposition(|enabled| *enabled).inspect(|&ix| {
                                navigation[ix].focus(window, cx);
                            })
                        } else {
                            let items: Vec<_> = navigation
                                .iter()
                                .zip(&enabled)
                                .map(|(focus, enabled)| (focus, *enabled))
                                .collect();
                            move_roving_focus(&items, key, Axis::Vertical, direction, window, cx)
                        };
                        if moved.is_some() {
                            cx.stop_propagation();
                            window.refresh();
                        }
                    })
                    .when(self.variant == ListVariant::Segmented, |list| {
                        list.gap(px(2.))
                    })
                    .children(prepared.into_iter().zip(focus).enumerate().map(
                        |(ix, (item, focus))| {
                            let (resting, selected_shape) = section_corners(self.variant, ix, last);
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
}

#[cfg(feature = "surfaces")]
pub use list_section::{ListSection, ListSectionItem};

#[cfg(test)]
mod tests {
    use super::list_section::section_corners;
    use crate::ListVariant;
    use gpui_material_theme::{CornerRole, CornerShape};

    #[test]
    fn list_section_computes_m3e_segmented_and_standard_corners() {
        assert_eq!(
            section_corners(ListVariant::Standard, 0, 2),
            (CornerRole::None.shape(), CornerRole::None.shape())
        );
        // Single item gets 16 dp all around.
        assert_eq!(
            section_corners(ListVariant::Segmented, 0, 0),
            (
                CornerShape::Radii([16., 16., 16., 16.]),
                CornerRole::Large.shape()
            )
        );
        // Multi-item segmented list: first [16,16,4,4], middle [4,4,4,4], last [4,4,16,16].
        assert_eq!(
            section_corners(ListVariant::Segmented, 0, 2),
            (
                CornerShape::Radii([16., 16., 4., 4.]),
                CornerRole::Large.shape()
            )
        );
        assert_eq!(
            section_corners(ListVariant::Segmented, 1, 2),
            (
                CornerShape::Radii([4., 4., 4., 4.]),
                CornerRole::Large.shape()
            )
        );
        assert_eq!(
            section_corners(ListVariant::Segmented, 2, 2),
            (
                CornerShape::Radii([4., 4., 16., 16.]),
                CornerRole::Large.shape()
            )
        );
    }

    #[cfg(feature = "selection")]
    #[test]
    fn slider_tile_formats_integers_decimals_and_ranges_cleanly() {
        use super::slider_tile::{format_slider_scalar, format_slider_value};
        use crate::SliderValue;

        assert_eq!(format_slider_scalar(42.0), "42");
        assert_eq!(format_slider_scalar(3.25), "3.25");
        assert_eq!(format_slider_scalar(f32::NAN), "0");
        assert_eq!(format_slider_value(SliderValue::Single(75.0), None), "75");
        assert_eq!(
            format_slider_value(SliderValue::Range(10.0, 80.5), None),
            "10 – 80.50"
        );
    }
}
