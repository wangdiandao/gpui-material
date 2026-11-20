use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

use crate::{
    MenuSurfaceStyle,
    motion::{OverlayMotion, ScaledOverlay, menu_pivot},
};
use gpui::{
    AccessibleAction, Anchor, AnyElement, App, AppContext, Bounds, Context, ElementId, Entity,
    EntityInputHandler, FocusHandle, Focusable, Hsla, InteractiveElement, IntoElement,
    KeyDownEvent, ListAlignment, ListOffset, ListSizingBehavior, ListState, ParentElement, Pixels,
    RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled, Subscription, Window,
    canvas, div, list, prelude::FluentBuilder, px,
};
use gpui_base::{Popup, TestSupportExt};
use gpui_material_foundation::{
    ThemeScope, animate, color, current_material_theme, logical_anchor,
};
use gpui_material_theme::{ColorRole, MotionKind, MotionSpeed, TypeRole, TypographyEmphasis};

use super::{TextField, TextFieldEvent, TextFieldState, TextFieldVariant};

/// A selectable dropdown value. IDs remain stable when options are reordered.
#[derive(Clone, PartialEq, Eq)]
pub struct DropdownOption {
    pub id: ElementId,
    pub label: SharedString,
}

impl DropdownOption {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

/// Focus and activation behavior for the primary dropdown anchor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropdownAnchor {
    /// Allows typing and keeps focus in the field when the menu opens.
    PrimaryEditable,
    /// Moves focus to the list when the read-only field opens the menu.
    PrimaryNotEditable,
}

type OpenChange = Rc<dyn Fn(bool, &mut Window, &mut App)>;
type Dismiss = Rc<dyn Fn(&mut Window, &mut App)>;
type Select = Rc<dyn Fn(&DropdownOption, &mut Window, &mut App)>;
type Editor = Box<dyn FnOnce(TextField) -> AnyElement>;
type AnchorContent = Box<dyn FnOnce(bool) -> AnyElement>;
type OptionContent = Rc<dyn Fn(&DropdownOption, bool, &mut Window, &mut App) -> AnyElement>;

#[derive(Default)]
struct Visibility {
    controlled: Option<bool>,
    change: Option<OpenChange>,
    dismiss: Option<Dismiss>,
}

#[derive(Clone, Copy)]
enum Opening {
    Editor,
    PopupFromPrimary,
    PopupFromSecondary,
}

/// Retains the filtered model independently of the visible rows. Comparisons
/// use stable IDs and labels; only a changed collection or query is filtered.
#[derive(Default)]
struct OptionSnapshot {
    source: Rc<[DropdownOption]>,
    query: Option<SharedString>,
    visible: Rc<[DropdownOption]>,
}

impl OptionSnapshot {
    fn reconcile(&mut self, source: Vec<DropdownOption>, query: Option<SharedString>) -> bool {
        if self.source.as_ref() == source.as_slice() && self.query == query {
            return false;
        }
        self.source = source.into();
        self.visible = if let Some(query) = &query {
            let query = query.to_lowercase();
            self.source
                .iter()
                .filter(|option| option.label.to_lowercase().contains(&query))
                .cloned()
                .collect()
        } else {
            self.source.clone()
        };
        self.query = query;
        true
    }
}

/// Keeps the editable query and pending keyboard choice across renders.
pub struct ExposedDropdownState {
    input: Entity<TextFieldState>,
    committed: SharedString,
    editable: bool,
    open: bool,
    presented_open: bool,
    visibility: Weak<Visibility>,
    primary_enabled: bool,
    opening: Option<Opening>,
    popup_focus: FocusHandle,
    secondary_focus: FocusHandle,
    restore_focus: Option<FocusHandle>,
    focus_epoch: u64,
    active: Option<ElementId>,
    selected: Option<ElementId>,
    scroll: ListState,
    presented: Rc<[DropdownOption]>,
    metrics: Option<(
        gpui::Pixels,
        f32,
        gpui::Pixels,
        gpui_material_theme::Typography,
        u64,
    )>,
    options: OptionSnapshot,
    _input: Subscription,
}

impl ExposedDropdownState {
    pub fn new(
        value: impl Into<SharedString>,
        editable: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let value = value.into();
        let input = cx.new(|cx| TextFieldState::new(window, cx).default_value(value));
        Self::with_editor(&input, editable, window, cx)
    }

    /// Uses a caller-retained single-line editor and its current value.
    pub fn with_editor(
        input: &Entity<TextFieldState>,
        editable: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let committed = input.read(cx).value();
        input.update(cx, |input, cx| input.set_readonly(!editable, cx));
        let subscription = cx.subscribe_in(input, window, |state, input, event, window, cx| {
            if !state.editable || !matches!(event, TextFieldEvent::Change) {
                return;
            }
            if input.update(cx, |input, cx| {
                input.marked_text_range(window, cx).is_some()
            }) {
                return;
            }
            if input.read(cx).value() != state.committed {
                state.active = None;
                if state.primary_enabled {
                    state.request_open(true, Some(Opening::Editor), window, cx);
                }
            }
            cx.notify();
        });
        Self {
            input: input.clone(),
            committed,
            editable,
            open: false,
            presented_open: false,
            visibility: Weak::new(),
            primary_enabled: true,
            opening: None,
            popup_focus: cx.focus_handle(),
            secondary_focus: cx.focus_handle(),
            restore_focus: None,
            focus_epoch: 0,
            active: None,
            selected: None,
            scroll: ListState::new(0, ListAlignment::Top, px(280.)),
            presented: Rc::default(),
            metrics: None,
            options: OptionSnapshot::default(),
            _input: subscription,
        }
    }

    /// Synchronizes the label and dismisses an internally controlled list.
    pub fn set_value(
        &mut self,
        value: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let value = value.into();
        if self.committed != value {
            self.selected = None;
        }
        self.committed = value;
        if self
            .visibility
            .upgrade()
            .is_none_or(|policy| policy.controlled.is_none())
        {
            self.open = false;
        }
        self.reset_draft(window, cx);
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn selected_option_id(&self) -> Option<&ElementId> {
        self.selected.as_ref()
    }

    /// The retained variable-height list position, including while closed.
    pub fn scroll_state(&self) -> &ListState {
        &self.scroll
    }

    /// Uses a caller-owned list position. The dropdown owns its item count.
    pub fn set_scroll_state(&mut self, scroll: &ListState, cx: &mut Context<Self>) {
        let anchor = scroll.logical_scroll_top();
        scroll.reset(self.presented.len());
        scroll.scroll_to(anchor);
        self.scroll = scroll.clone();
        self.metrics = None;
        cx.notify();
    }

    fn reset_draft(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.active = None;
        self.input.update(cx, |input, cx| {
            if input.value() != self.committed {
                input.set_value(self.committed.clone(), window, cx);
            }
        });
        cx.notify();
    }

    fn request_open(
        &mut self,
        open: bool,
        opening: Option<Opening>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.open == open {
            return;
        }
        self.opening = opening;
        let visibility = self.visibility.upgrade();
        if visibility
            .as_ref()
            .is_none_or(|policy| policy.controlled.is_none())
        {
            self.open = open;
            if !open {
                self.reset_draft(window, cx);
            }
        }
        if let Some(visibility) = visibility
            && (visibility.change.is_some() || !open && visibility.dismiss.is_some())
        {
            window.defer(cx, move |window, cx| {
                if let Some(change) = &visibility.change {
                    change(open, window, cx);
                }
                if !open && let Some(dismiss) = &visibility.dismiss {
                    dismiss(window, cx);
                }
            });
        }
        cx.notify();
    }

    fn present(
        &mut self,
        visibility: &Rc<Visibility>,
        primary_enabled: bool,
        secondary_enabled: bool,
        primary: Opening,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.primary_enabled = primary_enabled;
        self.open = visibility.controlled.unwrap_or(self.open);
        self.visibility = Rc::downgrade(visibility);
        let opening = self.opening.take();
        if self.presented_open == self.open {
            if !self.open {
                self.active = None;
            }
            return;
        }
        self.presented_open = self.open;
        self.focus_epoch = self.focus_epoch.wrapping_add(1);
        let epoch = self.focus_epoch;
        if self.open {
            let editor = self.input.read(cx).focus_handle(cx);
            let (focus, restore) = match opening.unwrap_or(primary) {
                Opening::Editor => (editor.clone(), None),
                Opening::PopupFromPrimary => (self.popup_focus.clone(), Some(editor)),
                Opening::PopupFromSecondary => {
                    (self.popup_focus.clone(), Some(self.secondary_focus.clone()))
                }
            };
            self.restore_focus = restore;
            if !focus.is_focused(window) {
                let state = cx.weak_entity();
                window.on_next_frame(move |window, cx| {
                    if state
                        .upgrade()
                        .is_some_and(|state| state.read(cx).focus_epoch == epoch)
                    {
                        focus.focus(window, cx);
                    }
                });
            }
        } else {
            self.reset_draft(window, cx);
            if let Some(mut restore) = self.restore_focus.take()
                && self.popup_focus.contains_focused(window, cx)
            {
                if !secondary_enabled && restore == self.secondary_focus {
                    restore = self.input.read(cx).focus_handle(cx);
                }
                let popup = self.popup_focus.clone();
                let state = cx.weak_entity();
                window.defer(cx, move |window, cx| {
                    if state
                        .upgrade()
                        .is_some_and(|state| state.read(cx).focus_epoch == epoch)
                        && popup.contains_focused(window, cx)
                    {
                        restore.focus(window, cx);
                    }
                });
            }
        }
    }

    fn navigate(&mut self, backwards: bool, cx: &mut Context<Self>) {
        let choices = &self.options.visible;
        let index = self
            .active
            .as_ref()
            .and_then(|id| choices.iter().position(|choice| &choice.id == id));
        let next = index.map_or_else(
            || {
                if backwards {
                    choices.len().saturating_sub(1)
                } else {
                    0
                }
            },
            |index| {
                if backwards {
                    index
                        .checked_sub(1)
                        .unwrap_or(choices.len().saturating_sub(1))
                } else {
                    (index + 1) % choices.len()
                }
            },
        );
        self.active = choices.get(next).map(|choice| choice.id.clone());
        if self.scroll.bounds_for_item(next).is_some() {
            self.scroll.scroll_to_reveal_item(next);
        } else {
            self.scroll.scroll_to(ListOffset {
                item_ix: next,
                offset_in_item: px(0.),
            });
        }
        cx.notify();
    }

    fn pending(&self) -> Option<DropdownOption> {
        let choices = &self.options.visible;
        let index = self
            .active
            .as_ref()
            .and_then(|id| choices.iter().position(|choice| &choice.id == id))
            .unwrap_or(0);
        choices.get(index).cloned()
    }
}

/// A Material field and anchored list in editable and selection-only modes.
/// Typing filters the list, while only selecting an option calls the owner.
#[derive(IntoElement)]
pub struct ExposedDropdown {
    id: ElementId,
    label: SharedString,
    state: Entity<ExposedDropdownState>,
    options: Vec<DropdownOption>,
    primary: Option<DropdownAnchor>,
    primary_enabled: bool,
    secondary_enabled: bool,
    secondary: Option<(SharedString, AnchorContent)>,
    editor: Option<Editor>,
    option_content: Option<OptionContent>,
    visibility: Visibility,
    match_anchor_width: bool,
    popup_width: Pixels,
    appearance: MenuSurfaceStyle,
    container_color: Option<Hsla>,
    on_select: Option<Select>,
}

impl ExposedDropdown {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<ExposedDropdownState>,
        options: impl IntoIterator<Item = DropdownOption>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            state: state.clone(),
            options: options.into_iter().collect(),
            primary: None,
            primary_enabled: true,
            secondary_enabled: true,
            secondary: None,
            editor: None,
            option_content: None,
            visibility: Visibility::default(),
            match_anchor_width: true,
            popup_width: px(240.),
            appearance: MenuSurfaceStyle::default(),
            container_color: None,
            on_select: None,
        }
    }

    /// Controls primary activation and focus. Pair the anchor with the state's editing mode.
    pub fn primary_anchor(mut self, anchor: DropdownAnchor) -> Self {
        self.primary = Some(anchor);
        self
    }

    pub fn primary_anchor_enabled(mut self, enabled: bool) -> Self {
        self.primary_enabled = enabled;
        self
    }

    /// Supplies the name and decorative content of the secondary anchor.
    pub fn secondary_anchor<E: IntoElement>(
        mut self,
        label: impl Into<SharedString>,
        content: impl FnOnce(bool) -> E + 'static,
    ) -> Self {
        self.secondary = Some((
            label.into(),
            Box::new(move |open| content(open).into_any_element()),
        ));
        self
    }

    pub fn secondary_anchor_enabled(mut self, enabled: bool) -> Self {
        self.secondary_enabled = enabled;
        self
    }

    /// Styles or wraps the field carrying the dropdown's retained editor and semantics.
    pub fn editor<E: IntoElement>(mut self, render: impl FnOnce(TextField) -> E + 'static) -> Self {
        self.editor = Some(Box::new(move |field| render(field).into_any_element()));
        self
    }

    /// Supplies repeatable decorative content for each visible option.
    pub fn option_content<E: IntoElement>(
        mut self,
        render: impl Fn(&DropdownOption, bool, &mut Window, &mut App) -> E + 'static,
    ) -> Self {
        self.option_content = Some(Rc::new(move |option, highlighted, window, cx| {
            render(option, highlighted, window, cx).into_any_element()
        }));
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.visibility.controlled = Some(open);
        self
    }

    pub fn on_open_change(
        mut self,
        callback: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.visibility.change = Some(Rc::new(callback));
        self
    }

    pub fn on_dismiss(mut self, callback: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.visibility.dismiss = Some(Rc::new(callback));
        self
    }

    pub fn match_anchor_width(mut self, matches: bool) -> Self {
        self.match_anchor_width = matches;
        self
    }

    /// Sets the independent width used with match_anchor_width(false).
    pub fn popup_width(mut self, width: Pixels) -> Self {
        if f32::from(width).is_finite() && width > px(0.) {
            self.popup_width = width;
        }
        self
    }

    pub fn appearance(mut self, appearance: MenuSurfaceStyle) -> Self {
        self.appearance = appearance;
        self
    }

    pub fn container_color(mut self, color: Hsla) -> Self {
        self.container_color = Some(color);
        self
    }

    pub fn on_select(
        mut self,
        on_select: impl Fn(&DropdownOption, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(on_select));
        self
    }
}

fn choose(
    state: &Entity<ExposedDropdownState>,
    choice: &DropdownOption,
    callback: &Option<Select>,
    window: &mut Window,
    cx: &mut App,
) {
    state.update(cx, |state, cx| {
        state.committed = choice.label.clone();
        state.selected = Some(choice.id.clone());
        state.reset_draft(window, cx);
        state.request_open(false, None, window, cx);
    });
    if let Some(callback) = callback {
        callback(choice, window, cx);
    }
}

#[derive(Clone)]
struct Interaction {
    state: Entity<ExposedDropdownState>,
    // The mounted handlers own the policy. The editor subscription observes it weakly.
    _visibility: Rc<Visibility>,
    select: Option<Select>,
    primary: Opening,
    primary_enabled: bool,
    editable: bool,
    parent: gpui::EntityId,
}

impl Interaction {
    fn request(&self, open: Option<bool>, opening: Opening, window: &mut Window, cx: &mut App) {
        self.state.update(cx, |state, cx| {
            let open = open.unwrap_or(!state.open);
            if open && matches!(opening, Opening::Editor) {
                state.input.read(cx).focus_handle(cx).focus(window, cx);
            }
            state.request_open(open, open.then_some(opening), window, cx);
        });
        cx.notify(self.parent);
    }

    fn dismiss(&self, window: &mut Window, cx: &mut App) {
        self.request(Some(false), self.primary, window, cx);
    }

    fn keys(&self, event: &KeyDownEvent, window: &mut Window, cx: &mut App) {
        let key = event.keystroke.key.as_str();
        if event.keystroke.modifiers.modified()
            || !(matches!(key, "up" | "down" | "enter" | "escape")
                || key == "space" && !self.editable)
        {
            return;
        }
        let model = self.state.read(cx);
        let open = model.open;
        let input = model.input.clone();
        if !open && !self.primary_enabled
            || input.update(cx, |input, cx| {
                input.marked_text_range(window, cx).is_some()
            })
        {
            return;
        }
        match key {
            "up" | "down" => self.state.update(cx, |state, cx| {
                state.navigate(key == "up", cx);
                if state.open {
                    return;
                }
                state.request_open(true, Some(self.primary), window, cx);
            }),
            "escape" if open => self.dismiss(window, cx),
            "enter" if open => {
                if let Some(choice) = self.state.read(cx).pending() {
                    choose(&self.state, &choice, &self.select, window, cx);
                }
            }
            "enter" | "space" if !self.editable => self.request(None, self.primary, window, cx),
            _ => return,
        }
        cx.notify(self.parent);
        cx.stop_propagation();
    }
}

impl RenderOnce for ExposedDropdown {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let primary = if self.primary == Some(DropdownAnchor::PrimaryNotEditable) {
            Opening::PopupFromPrimary
        } else {
            Opening::Editor
        };
        let visibility = Rc::new(self.visibility);
        self.state.update(cx, |state, cx| {
            state.present(
                &visibility,
                self.primary_enabled,
                self.secondary_enabled,
                primary,
                window,
                cx,
            );
        });
        let model = self.state.read(cx);
        let input = model.input.clone();
        let open = model.open;
        let editable = model.editable;
        let popup_focus = model.popup_focus.clone();
        let transfer_focus = model.restore_focus.is_some();
        let secondary_focus = model.secondary_focus.clone();
        let scroll = model.scroll.clone();
        let query = input.read(cx).value();
        let committed = model.committed.clone();
        let composing = input.update(cx, |input, cx| {
            input.marked_text_range(window, cx).is_some()
        });
        let filter = editable && !composing && query != committed;
        let (options, active, selected_option) = self.state.update(cx, |state, _| {
            if state
                .options
                .reconcile(self.options, filter.then_some(query))
            {
                let index = state.active.as_ref().and_then(|id| {
                    state
                        .options
                        .visible
                        .iter()
                        .position(|option| &option.id == id)
                });
                if index.is_none() {
                    state.active = None;
                }
            }
            (
                state.options.visible.clone(),
                state.active.clone(),
                state.selected.clone(),
            )
        });
        let last_visible_options = window
            .use_keyed_state((self.id.clone(), "presented-options"), cx, |_, _| {
                Rc::new(RefCell::new((
                    Rc::<[DropdownOption]>::default(),
                    None::<ElementId>,
                    None::<ElementId>,
                )))
            })
            .read(cx)
            .clone();
        if open {
            let mut visible = last_visible_options.borrow_mut();
            visible.0 = options.clone();
            visible.1.clone_from(&active);
            visible.2.clone_from(&selected_option);
        }
        let theme = current_material_theme(window, cx).clone();
        let ink = color(theme.colors.color(ColorRole::OnSurface));
        let selected = color(theme.colors.color(ColorRole::SecondaryContainer));
        let selected_ink = color(theme.colors.color(ColorRole::OnSecondaryContainer));
        let typography = theme.typography.clone();
        let text_style = TypeRole::LabelLarge.style(TypographyEmphasis::Baseline);
        let arrow = animate(
            (self.id.clone(), "arrow"),
            f32::from(open),
            theme
                .motion
                .spec(MotionKind::Effects, MotionSpeed::Fast, cx.reduce_motion()),
            window,
            cx,
        )
        .clamp(0., 1.);
        let parent = window.current_view();
        let interaction = Interaction {
            state: self.state.clone(),
            _visibility: visibility,
            select: self.on_select.clone(),
            primary,
            primary_enabled: self.primary_enabled,
            editable,
            parent,
        };
        let popup_keys = interaction.clone();
        let outside = interaction.clone();
        let keys = interaction.clone();
        let primary_click = interaction.clone();
        let request: Option<OpenChange> = self.primary_enabled.then(|| {
            let interaction = interaction.clone();
            Rc::new(move |open, window: &mut Window, cx: &mut App| {
                interaction.request(Some(open), primary, window, cx);
            }) as OpenChange
        });
        let (secondary_label, secondary_content) = self.secondary.map_or_else(
            || {
                let label = if open {
                    "Close options"
                } else {
                    "Open options"
                };
                (
                    SharedString::from(label),
                    dropdown_arrow(arrow, ink).into_any_element(),
                )
            },
            |(label, content)| (label, content(open)),
        );
        let secondary = if self.secondary_enabled {
            let pointer = interaction.clone();
            let a11y = interaction.clone();
            let expansion = interaction;
            let accessible_opening = if editable {
                Opening::PopupFromSecondary
            } else {
                primary
            };
            gpui_base::Button::new("toggle-options")
                .track_focus(&secondary_focus)
                .accessibility_label(secondary_label)
                .aria_expanded(open)
                .min_w(px(36.))
                .min_h(px(36.))
                .focus_visible(move |style| style.border_1().border_color(ink))
                .on_click(move |_, window, cx| {
                    cx.stop_propagation();
                    pointer.request(None, primary, window, cx);
                })
                .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
                    a11y.request(None, accessible_opening, window, cx);
                })
                .on_a11y_action(
                    if open {
                        AccessibleAction::Collapse
                    } else {
                        AccessibleAction::Expand
                    },
                    move |_, window, cx| {
                        expansion.request(Some(!open), accessible_opening, window, cx);
                    },
                )
                .child(secondary_content)
                .into_any_element()
        } else {
            div()
                .id("toggle-options-disabled")
                .role(Role::Group)
                .a11y_synthetic_children(|tree| tree.parent_node().set_hidden())
                .min_w(px(36.))
                .min_h(px(36.))
                .flex()
                .items_center()
                .justify_center()
                .opacity(0.38)
                .child(secondary_content)
                .into_any_element()
        };
        let options_label = format!("{} options", self.label);
        let field = TextField::new("value", self.label, &input)
            .variant(TextFieldVariant::Outlined)
            .dropdown(open, request)
            .trailing(secondary);
        let editor = if let Some(editor) = self.editor {
            editor(field)
        } else {
            field.into_any_element()
        };

        let bounds = window
            .use_keyed_state((self.id.clone(), "bounds"), cx, |_, _| {
                Rc::new(Cell::new(Bounds::default()))
            })
            .read(cx)
            .clone();
        let measure = bounds.clone();
        let measured = bounds.get();
        let viewport = window.viewport_size();
        let default_height = (viewport.height * 0.55).min(px(280.));
        let gap = px(4.);
        let margin = px(8.);
        let (anchor, height) = if measured.size.height > px(0.) {
            let below = (viewport.height - measured.bottom() - gap - margin).max(px(0.));
            let above = (measured.top() - gap - margin).max(px(0.));
            if below < default_height && above > below {
                (Anchor::BottomLeft, default_height.min(above))
            } else {
                (Anchor::TopLeft, default_height.min(below))
            }
        } else {
            (Anchor::TopLeft, default_height)
        };
        let anchor = logical_anchor(anchor, theme.layout.direction);
        let width = if self.match_anchor_width {
            if measured.size.width > px(0.) {
                measured.size.width
            } else {
                px(120.)
            }
        } else {
            self.popup_width
        };
        let width = width.min((viewport.width - margin * 2.).max(px(0.)));
        let trigger = div()
            .id((self.id.clone(), "combobox"))
            .test_support()
            .when(
                self.primary_enabled && (!editable || self.primary.is_some()),
                |trigger| {
                    trigger.on_click(move |_, window, cx| {
                        primary_click.request(None, primary, window, cx);
                    })
                },
            )
            // TextField owns the single combo box node and delegates IME to its editor.
            .capture_key_down(move |event, window, cx| keys.keys(event, window, cx))
            .child(editor)
            .relative()
            .child(
                canvas(
                    move |bounds, window, _| {
                        if measure.replace(bounds) != bounds && open {
                            window.refresh();
                        }
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .inset_0(),
            )
            // Keep the default cursor over the read-only editor's I-beam.
            .when(!editable, |trigger| {
                trigger.child(div().absolute().inset_0().cursor_default())
            });
        let motion = OverlayMotion::sample(&self.id, open, window, cx);

        let placement = window
            .use_keyed_state((self.id.clone(), "last-popup-bounds"), cx, |_, _| {
                Rc::new(Cell::new((Bounds::default(), Bounds::default())))
            })
            .read(cx)
            .clone();
        let (last_bounds, last_trigger) = placement.get();
        let trigger_bounds = placement.clone();
        let popup = Popup::new(self.id.clone(), trigger)
            .anchor(anchor)
            .offset(gap)
            .on_position(move |resolved, trigger| trigger_bounds.set((resolved.bounds, trigger)));
        let content = motion
            .map(|motion| {
                let dp = move |value| motion.px(value);
                let spacing = dp(16.);
                let (options, active, selected_option) = if open {
                    (options, active, selected_option)
                } else {
                    last_visible_options.borrow().clone()
                };
                let width = if open { width } else { last_bounds.size.width };
                let scroll_height =
                    (height - self.appearance.popup_vertical_insets()).max(px(0.)) * motion.scale;
                let font_revision = gpui_material_foundation::font_cache_revision(cx);
                self.state.update(cx, |state, _| {
                    let metrics = (
                        width,
                        motion.scale,
                        window.rem_size(),
                        typography,
                        font_revision,
                    );
                    if !Rc::ptr_eq(&state.presented, &options) {
                        let old_top = state.scroll.logical_scroll_top();
                        let old_id = state
                            .presented
                            .get(old_top.item_ix)
                            .map(|option| &option.id);
                        let anchor = active
                            .as_ref()
                            .or(old_id)
                            .and_then(|id| options.iter().position(|option| &option.id == id));
                        state.scroll.reset(options.len());
                        if let Some(index) = anchor {
                            state.scroll.scroll_to(ListOffset {
                                item_ix: index,
                                offset_in_item: if active.is_some() {
                                    px(0.)
                                } else {
                                    old_top.offset_in_item
                                },
                            });
                        }
                        state.presented = options.clone();
                    } else if state.metrics.as_ref() != Some(&metrics) {
                        let anchor = state.scroll.logical_scroll_top();
                        state.scroll.remeasure();
                        state.scroll.scroll_to(anchor);
                    }
                    state.metrics = Some(metrics);
                });
                let content = div()
                    .id(if open { "options" } else { "options-exit" })
                    .test_support()
                    .opacity(motion.alpha)
                    .w(width * motion.scale)
                    .text_color(ink)
                    .max_h(height * motion.scale)
                    .when(open, |list| {
                        list.role(Role::ListBox)
                            .aria_label(options_label)
                            .when(transfer_focus, |list| {
                                list.track_focus(&popup_focus).tab_stop(false)
                            })
                            .capture_key_down(move |event, window, cx| {
                                popup_keys.keys(event, window, cx)
                            })
                            .on_mouse_down_out(move |event, window, cx| {
                                if !bounds.get().contains(&event.position) {
                                    outside.dismiss(window, cx);
                                }
                            })
                    })
                    .child(
                        self.appearance
                            .dropdown_surface(&theme, motion)
                            .when_some(self.container_color, |surface, color| surface.bg(color))
                            .w_full()
                            .child(if options.is_empty() {
                                div()
                                    .id("options-scroll-region")
                                    .test_support()
                                    .max_h(scroll_height)
                                    .overflow_hidden()
                                    .child(
                                        div()
                                            .px(spacing * 0.75)
                                            .py(spacing * 0.5)
                                            .child("No matching options"),
                                    )
                                    .into_any_element()
                            } else {
                                let row_state = self.state.clone();
                                let row_callback = self.on_select.clone();
                                let option_content = self.option_content.clone();
                                div()
                                    .id("options-scroll-region")
                                    .test_support()
                                    .max_h(scroll_height)
                                    .overflow_hidden()
                                    .child(
                                        list(scroll.clone(), move |index, window, cx| {
                                            let choice = options[index].clone();
                                            let current =
                                                active.as_ref().or(selected_option.as_ref())
                                                    == Some(&choice.id);
                                            let content = option_content.as_ref().map_or_else(
                                                || choice.label.clone().into_any_element(),
                                                |content| content(&choice, current, window, cx),
                                            );
                                            let state = row_state.clone();
                                            let callback = row_callback.clone();
                                            let accessible = choice.clone();
                                            let accessible_state = state.clone();
                                            let accessible_callback = callback.clone();
                                            div()
                                                .id(choice.id.clone())
                                                .test_support()
                                                .when(open, |row| {
                                                    row.role(Role::ListBoxOption)
                                                        .aria_label(choice.label.clone())
                                                        .aria_selected(current)
                                                        .aria_position_in_set(index + 1)
                                                        .aria_size_of_set(options.len())
                                                })
                                                .when(current, |row| {
                                                    row.bg(selected).text_color(selected_ink)
                                                })
                                                .when(open && current, |row| {
                                                    row.aria_active_descendant()
                                                })
                                                .when(open, |row| {
                                                    row.hover(move |style| {
                                                        style.bg(if current {
                                                            selected
                                                                .blend(selected_ink.opacity(0.08))
                                                        } else {
                                                            ink.opacity(0.08)
                                                        })
                                                    })
                                                })
                                                .min_h(dp(48.))
                                                .w_full()
                                                .px(spacing * 0.75)
                                                .flex()
                                                .items_center()
                                                .child(content)
                                                .when(open, |row| {
                                                    row.on_a11y_action(
                                                        gpui::AccessibleAction::Click,
                                                        move |_, window, cx| {
                                                            choose(
                                                                &accessible_state,
                                                                &accessible,
                                                                &accessible_callback,
                                                                window,
                                                                cx,
                                                            );
                                                            cx.notify(parent);
                                                        },
                                                    )
                                                    .on_click(move |_, window, cx| {
                                                        choose(
                                                            &state, &choice, &callback, window, cx,
                                                        );
                                                        cx.notify(parent);
                                                    })
                                                })
                                                .into_any_element()
                                        })
                                        .with_sizing_behavior(ListSizingBehavior::Infer)
                                        .w_full()
                                        .max_h(scroll_height),
                                    )
                                    .into_any_element()
                            }),
                    )
                    .when(!open, |list| {
                        list.child(div().absolute().inset_0().occlude())
                    });
                let content = gpui_material_foundation::text_style(
                    content,
                    text_style,
                    motion.scale,
                    window,
                    cx,
                );
                ScaledOverlay::new(content, motion, move |menu| {
                    menu_pivot(
                        menu,
                        if open {
                            placement.get().1
                        } else {
                            last_trigger
                        },
                    )
                })
                .width(width)
            })
            .map(|content| ThemeScope::inherit(content, window, cx));
        if open {
            return div().child(popup.content(content.expect("open dropdown")));
        }
        let exit = content
            .filter(|_| last_bounds.size.width > px(0.))
            .map(|content| {
                gpui::deferred(
                    gpui_base::Positioner::corner(Anchor::TopLeft, last_bounds.origin)
                        .child(content),
                )
                .with_priority(100)
            });
        div().child(popup).children(exit)
    }
}

// The official dropdown symbol rotates with the existing expansion animation.
// Keep this decorative element separate from the Base trigger's semantics.
fn dropdown_arrow(progress: f32, ink: gpui::Hsla) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .text_color(ink)
        .child(
            crate::Icon::new(crate::IconName::ArrowDropDown)
                .size(px(24.))
                .rotate(gpui::radians(std::f32::consts::PI * progress)),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_snapshot_reuses_results_and_invalidates_labels_and_query() {
        let mut snapshot = OptionSnapshot::default();
        let options = || {
            vec![
                DropdownOption::new("a", "Alpha"),
                DropdownOption::new("b", "Beta"),
            ]
        };
        assert!(snapshot.reconcile(options(), None));
        let unfiltered = snapshot.visible.clone();
        assert!(Rc::ptr_eq(&unfiltered, &snapshot.source));
        assert!(!snapshot.reconcile(options(), None));
        assert!(Rc::ptr_eq(&unfiltered, &snapshot.visible));
        assert!(snapshot.reconcile(options(), Some("ALP".into())));
        assert_eq!(snapshot.visible.len(), 1);
        assert_eq!(snapshot.visible[0].id, ElementId::from("a"));
        assert!(snapshot.reconcile(
            vec![
                DropdownOption::new("a", "Renamed"),
                DropdownOption::new("b", "Beta")
            ],
            Some("ALP".into())
        ));
        assert!(snapshot.visible.is_empty());
    }
}
