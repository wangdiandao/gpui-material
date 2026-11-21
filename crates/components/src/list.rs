use std::rc::Rc;

use gpui::{
    AnyElement, App, Axis, ElementId, Entity, FocusHandle, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled, Window,
    accesskit::Toggled, div, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{InteractionSource, current_material_theme, move_roving_focus};
use gpui_material_theme::{CornerRole, CornerShape};

use crate::{SwipeToDismiss, SwipeToDismissState, action::Activate};

mod long_press;
mod render;
mod style;

pub use style::{
    ListItemAlignment, ListItemColors, ListItemElevations, ListItemPadding, ListItemShapes,
    ListItemStateColors,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ListVariant {
    #[default]
    Standard,
    Segmented,
}

type ValueChange = Rc<dyn Fn(bool, &mut Window, &mut App)>;
type SwipeConfig = Box<dyn FnOnce(SwipeToDismiss) -> SwipeToDismiss>;

enum Slot {
    Text(SharedString),
    Custom(AnyElement),
}

impl Slot {
    fn has_line_break(&self) -> bool {
        matches!(self, Self::Text(value) if value.contains('\n'))
    }
}

struct Expansion {
    expanded: bool,
    content: AnyElement,
}

struct SecondaryAction {
    label: SharedString,
    handler: Activate,
}

struct RowSwipe {
    state: Entity<SwipeToDismissState>,
    configure: SwipeConfig,
}

/// A Standard or Segmented list row. Values and data changes belong to the caller.
#[derive(IntoElement)]
pub struct ListItem {
    id: ElementId,
    label: SharedString,
    content: Option<AnyElement>,
    supporting: Option<Slot>,
    overline: Option<Slot>,
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
    activation: Option<Activate>,
    checked_change: Option<ValueChange>,
    expanded_change: Option<ValueChange>,
    long_press: Option<SecondaryAction>,
    long_press_button: Option<ElementId>,
    expansion: Option<Expansion>,
    swipe: Option<RowSwipe>,
    checked: Option<Toggled>,
    checked_role: Role,
    checked_selects_row: bool,
    invalid: bool,
    footer: Option<AnyElement>,
    #[cfg(feature = "feedback")]
    menu_trigger: Option<(bool, Activate)>,
    selected: bool,
    disabled: bool,
    dragged: bool,
    focus: Option<(FocusHandle, bool)>,
    shapes: ListItemShapes,
    colors: ListItemColors,
    elevations: ListItemElevations,
    alignment: ListItemAlignment,
    padding: Option<ListItemPadding>,
    source: Option<Entity<InteractionSource>>,
}

impl ListItem {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            content: None,
            supporting: None,
            overline: None,
            leading: None,
            trailing: None,
            activation: None,
            checked_change: None,
            expanded_change: None,
            long_press: None,
            long_press_button: None,
            expansion: None,
            swipe: None,
            checked: None,
            checked_role: Role::CheckBox,
            checked_selects_row: true,
            invalid: false,
            footer: None,
            #[cfg(feature = "feedback")]
            menu_trigger: None,
            selected: false,
            disabled: false,
            dragged: false,
            focus: None,
            shapes: ListItemShapes::default(),
            colors: ListItemColors::default(),
            elevations: ListItemElevations::default(),
            alignment: ListItemAlignment::Auto,
            padding: None,
            source: None,
        }
    }

    /// Replaces the visual headline. The constructor's label still names the row action.
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting = Some(Slot::Text(text.into()));
        self
    }

    pub fn supporting_content(mut self, content: impl IntoElement) -> Self {
        self.supporting = Some(Slot::Custom(content.into_any_element()));
        self
    }

    pub fn overline(mut self, text: impl Into<SharedString>) -> Self {
        self.overline = Some(Slot::Text(text.into()));
        self
    }

    pub fn overline_content(mut self, content: impl IntoElement) -> Self {
        self.overline = Some(Slot::Custom(content.into_any_element()));
        self
    }

    pub fn leading(mut self, leading: impl IntoElement) -> Self {
        self.leading = Some(leading.into_any_element());
        self
    }

    pub fn trailing(mut self, trailing: impl IntoElement) -> Self {
        self.trailing = Some(trailing.into_any_element());
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// The controlled checked value, separate from the row's selection state.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(if checked {
            Toggled::True
        } else {
            Toggled::False
        });
        self
    }

    /// Requests the opposite checked value on activation. The current value defaults to false.
    pub fn on_checked_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.checked.get_or_insert(Toggled::False);
        self.checked_change = Some(Rc::new(handler));
        self
    }

    /// Supplies controlled disclosure content. It is mounted below the row while expanded.
    pub fn expanded_content(mut self, expanded: bool, content: impl IntoElement) -> Self {
        self.expansion = Some(Expansion {
            expanded,
            content: content.into_any_element(),
        });
        self
    }

    pub fn on_expanded_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.expanded_change = Some(Rc::new(handler));
        self
    }

    /// Names the secondary command invoked by a pointer hold or Shift+F10.
    pub fn on_long_press(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.long_press = Some(SecondaryAction {
            label: label.into(),
            handler: Rc::new(handler),
        });
        self
    }

    /// Adds a visible button for the named secondary command in the trailing area.
    pub fn long_press_button(mut self, id: impl Into<ElementId>) -> Self {
        self.long_press_button = Some(id.into());
        self
    }

    /// Composes the row with a retained SwipeToDismiss instance. Use its configuration
    /// callback for direction, background and dismissal, keeping the state's owner unchanged.
    pub fn swipe_to_dismiss(
        mut self,
        state: &Entity<SwipeToDismissState>,
        configure: impl FnOnce(SwipeToDismiss) -> SwipeToDismiss + 'static,
    ) -> Self {
        self.swipe = Some(RowSwipe {
            state: state.clone(),
            configure: Box::new(configure),
        });
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_activate(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.activation = Some(Rc::new(handler));
        self
    }

    pub fn shapes(mut self, shapes: ListItemShapes) -> Self {
        self.shapes = shapes;
        self
    }

    pub fn colors(mut self, colors: ListItemColors) -> Self {
        self.colors = colors;
        self
    }

    pub fn elevations(mut self, elevations: ListItemElevations) -> Self {
        self.elevations = elevations;
        self
    }

    pub fn vertical_alignment(mut self, alignment: ListItemAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn content_padding(mut self, padding: ListItemPadding) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Reports a caller-owned drag. Clear the flag after completion or cancellation.
    pub fn dragged(mut self, dragged: bool) -> Self {
        self.dragged = dragged;
        self
    }

    pub fn interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
        self.source = Some(source.clone());
        self
    }

    /// Uses a retained focus handle and roving tab stop supplied by a virtual
    /// collection. The collection owns navigation across unrendered rows.
    pub fn track_focus(mut self, focus: &FocusHandle, tab_stop: bool) -> Self {
        self.focus = Some((focus.clone(), tab_stop));
        self
    }

    #[cfg(feature = "selection")]
    pub(crate) fn checked_role(mut self, role: Role) -> Self {
        self.checked_role = role;
        self
    }

    #[cfg(feature = "selection")]
    pub(crate) fn checked_toggled(mut self, state: Toggled) -> Self {
        self.checked = Some(state);
        self
    }

    #[cfg(feature = "selection")]
    pub(crate) fn checked_selects_row(mut self, selects: bool) -> Self {
        self.checked_selects_row = selects;
        self
    }

    #[cfg(feature = "selection")]
    pub(crate) fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    #[cfg(feature = "selection")]
    pub(crate) fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }

    #[cfg(feature = "feedback")]
    pub(crate) fn menu_trigger(mut self, open: bool, a11y_toggle: Activate) -> Self {
        self.menu_trigger = Some((open, a11y_toggle));
        self
    }

    pub(crate) fn id(&self) -> &ElementId {
        &self.id
    }

    pub(crate) fn is_selected(&self) -> bool {
        self.selected
    }

    fn has_primary_action(&self) -> bool {
        self.activation.is_some()
            || self.checked_change.is_some()
            || self.expansion.is_some() && self.expanded_change.is_some()
    }

    fn has_menu_trigger(&self) -> bool {
        #[cfg(feature = "feedback")]
        {
            self.menu_trigger.is_some()
        }
        #[cfg(not(feature = "feedback"))]
        {
            false
        }
    }

    pub(crate) fn takes_focus(&self, cx: &App) -> bool {
        !self.disabled
            && (self.has_primary_action() || self.has_menu_trigger() || self.long_press.is_some())
            && self
                .swipe
                .as_ref()
                .is_none_or(|swipe| swipe.state.read(cx).shows_foreground())
    }

    fn primary_action(&self) -> Option<Activate> {
        if !self.has_primary_action() {
            return None;
        }
        let action = self.activation.clone();
        let check = self.checked.zip(self.checked_change.clone());
        let expansion = self
            .expansion
            .as_ref()
            .map(|expansion| expansion.expanded)
            .zip(self.expanded_change.clone());
        Some(Rc::new(move |window, cx| {
            if let Some((checked, change)) = &check {
                change(*checked != Toggled::True, window, cx);
            }
            if let Some((expanded, change)) = &expansion {
                change(!expanded, window, cx);
            }
            if let Some(action) = &action {
                action(window, cx);
            }
        }))
    }

    pub(crate) fn row(
        self,
        resting: CornerShape,
        selected_shape: CornerShape,
        focus: Option<(FocusHandle, bool)>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        window.with_element_namespace((self.id.clone(), "material-list-row"), |window| {
            self.render_row(resting, selected_shape, focus, window, cx)
        })
    }
}

impl RenderOnce for ListItem {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus = self.focus.clone();
        self.row(
            CornerRole::None.shape(),
            CornerRole::None.shape(),
            focus,
            window,
            cx,
        )
    }
}

/// A list of identified rows. Applications can supply a visible window of rows
/// from their own virtualized collection.
#[derive(IntoElement)]
pub struct List {
    id: ElementId,
    items: Vec<ListItem>,
    variant: ListVariant,
}

impl List {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            variant: ListVariant::default(),
        }
    }

    pub fn variant(mut self, variant: ListVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ListItem>) -> Self {
        self.items.extend(items);
        self
    }
}

impl RenderOnce for List {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        window.with_element_namespace((self.id.clone(), "material-list"), |window| {
            let last = self.items.len().saturating_sub(1);
            let direction = current_material_theme(window, cx).layout.direction;
            let focus: Vec<_> = self
                .items
                .iter()
                .map(|item| {
                    window
                        .use_keyed_state((item.id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
                        .read(cx)
                        .clone()
                })
                .collect();
            let enabled: Vec<_> = self.items.iter().map(|item| item.takes_focus(cx)).collect();
            let tab_stop = focus
                .iter()
                .enumerate()
                .position(|(ix, focus)| enabled[ix] && focus.is_focused(window))
                .or_else(|| {
                    self.items
                        .iter()
                        .enumerate()
                        .position(|(ix, item)| enabled[ix] && item.selected)
                })
                .or_else(|| enabled.iter().position(|enabled| *enabled));
            let navigation = focus.clone();
            div()
                .id(self.id)
                .role(Role::List)
                .test_support()
                .flex()
                .flex_col()
                .w_full()
                .on_key_down(move |event, window, cx| {
                    // Nested controls keep their own arrow-key behavior.
                    if !navigation.iter().any(|focus| focus.is_focused(window)) {
                        return;
                    }
                    let items: Vec<_> = navigation
                        .iter()
                        .zip(&enabled)
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
                        window.refresh();
                    }
                })
                .when(self.variant == ListVariant::Segmented, |list| {
                    list.gap(px(2.))
                })
                .children(self.items.into_iter().zip(focus).enumerate().map(
                    |(ix, (item, focus))| {
                        let (resting, selected) = match self.variant {
                            ListVariant::Standard => {
                                (CornerRole::None.shape(), CornerRole::None.shape())
                            }
                            ListVariant::Segmented => {
                                let top = if ix == 0 { 16. } else { 4. };
                                let bottom = if ix == last { 16. } else { 4. };
                                (
                                    CornerShape::Radii([top, top, bottom, bottom]),
                                    CornerRole::Large.shape(),
                                )
                            }
                        };
                        item.row(
                            resting,
                            selected,
                            Some((focus, Some(ix) == tab_stop)),
                            window,
                            cx,
                        )
                    },
                ))
        })
    }
}
