use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, Entity, FocusHandle, InteractiveElement as _, IntoElement,
    ParentElement as _, RenderOnce, Role, SharedString, StatefulInteractiveElement as _, Window,
    div, prelude::FluentBuilder as _, px,
};
use gpui_material_components::{DialogAction, DialogCloseReason, DialogProperties, TextField};
use gpui_material_foundation::Surface;
use gpui_material_theme::ColorRole;

use super::Confirm;

mod calendar;
mod dialog;
mod docked;
mod state;
mod types;

pub use state::DatePickerState;
pub use types::{
    DatePickerColors, DatePickerDisplayMode, DatePickerDraft, DatePickerLocale, DatePickerVariant,
    DateSelection,
};

use types::{
    CustomAction, CustomAnchor, DatePickerOptions, DismissRequest, DockedActions, OnClose,
    Presentation,
};

/// A single or range date picker with a trigger, a standalone panel, or a modal dialog.
#[derive(IntoElement)]
pub struct DatePicker {
    id: ElementId,
    label: SharedString,
    state: Entity<DatePickerState>,
    options: DatePickerOptions,
    presentation: Presentation,
    anchor: Option<CustomAnchor>,
    show_mode_toggle: bool,
    title: Option<AnyElement>,
    headline: Option<AnyElement>,
    confirm_content: Option<CustomAction>,
    dismiss_content: Option<CustomAction>,
    confirm: Option<Confirm<DateSelection>>,
    properties: DialogProperties,
    request: Option<DismissRequest>,
    closed: Option<OnClose>,
    colors: DatePickerColors,
    disabled: bool,
}

impl DatePicker {
    /// Creates a triggered date picker bound to `state` with the caller-owned committed `value`.
    ///
    /// Defaults to [`DatePickerVariant::Modal`], [`DatePickerLocale::default()`], and the mode
    /// toggle visible.
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<DatePickerState>,
        value: DateSelection,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            state: state.clone(),
            options: DatePickerOptions {
                value,
                variant: DatePickerVariant::default(),
                locale: DatePickerLocale::default(),
                focus_target: None,
            },
            presentation: Presentation::Triggered,
            anchor: None,
            show_mode_toggle: true,
            title: None,
            headline: None,
            confirm_content: None,
            dismiss_content: None,
            confirm: None,
            properties: DialogProperties::default(),
            request: None,
            closed: None,
            colors: DatePickerColors::default(),
            disabled: false,
        }
    }

    /// Embeds the retained draft and display mode directly in another layout.
    pub fn panel(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<DatePickerState>,
    ) -> Self {
        Self {
            presentation: Presentation::Panel,
            ..Self::new(id, label, state, DateSelection::Single(None))
        }
    }

    /// Renders a modal opened by a caller-owned control with `DatePickerState::open`.
    pub fn dialog(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<DatePickerState>,
        value: DateSelection,
    ) -> Self {
        Self {
            presentation: Presentation::Dialog,
            ..Self::new(id, label, state, value)
        }
    }

    /// Sets the picker presentation variant.
    pub fn variant(mut self, variant: DatePickerVariant) -> Self {
        self.options.variant = variant;
        self
    }

    /// Sets the locale formatting, parsing, and first-day-of-week rules.
    pub fn locale(mut self, locale: DatePickerLocale) -> Self {
        self.options.locale = locale;
        self
    }

    /// Chooses the initial focus when the built-in control opens the picker.
    pub fn focus_target(mut self, focus: &FocusHandle) -> Self {
        self.options.focus_target = Some(focus.clone());
        self
    }

    /// Wraps/customizes the retained docked editor and supplied trailing trigger.
    pub fn docked_anchor<E: IntoElement>(
        mut self,
        render: impl FnOnce(TextField) -> E + 'static,
    ) -> Self {
        self.anchor = Some(Box::new(move |field| render(field).into_any_element()));
        self
    }

    /// Shows or hides the button that toggles between calendar and text-input display modes.
    pub fn show_mode_toggle(mut self, show: bool) -> Self {
        self.show_mode_toggle = show;
        self
    }

    /// Replaces the visible title while preserving the accessible picker label.
    pub fn title_content(mut self, element: impl IntoElement) -> Self {
        self.title = Some(element.into_any_element());
        self
    }

    /// Replaces the selection headline while preserving its date announcement.
    pub fn headline_content(mut self, element: impl IntoElement) -> Self {
        self.headline = Some(element.into_any_element());
        self
    }

    /// Customizes the confirm action control rendered in modal or docked dialogs.
    pub fn confirm_content<E: IntoElement>(
        mut self,
        build: impl FnOnce(DialogAction) -> E + 'static,
    ) -> Self {
        self.confirm_content = Some(Box::new(move |confirm| build(confirm).into_any_element()));
        self
    }

    /// Customizes the dismiss action control rendered in modal or docked dialogs.
    pub fn dismiss_content<E: IntoElement>(
        mut self,
        build: impl FnOnce(DialogAction) -> E + 'static,
    ) -> Self {
        self.dismiss_content = Some(Box::new(move |dismiss| build(dismiss).into_any_element()));
        self
    }

    /// Registers a callback invoked with the committed [`DateSelection`] when confirmed.
    pub fn on_confirm(
        mut self,
        callback: impl Fn(DateSelection, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.confirm = Some(Rc::new(callback));
        self
    }

    /// Configures escape and outside-click dismissal behavior for modal and docked popups.
    pub fn properties(mut self, properties: DialogProperties) -> Self {
        self.properties = properties;
        self
    }

    /// Registers a gate callback called before dismissing; return `false` to keep the picker open.
    pub fn on_dismiss_request(
        mut self,
        callback: impl Fn(DialogCloseReason, &mut Window, &mut App) -> bool + 'static,
    ) -> Self {
        self.request = Some(Rc::new(callback));
        self
    }

    /// Registers a callback invoked after the picker closes, with the [`DialogCloseReason`].
    pub fn on_closed(
        mut self,
        callback: impl Fn(DialogCloseReason, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.closed = Some(Rc::new(callback));
        self
    }

    /// Overrides the picker's local color palette.
    pub fn colors(mut self, colors: DatePickerColors) -> Self {
        self.colors = colors;
        self
    }

    /// Disables the built-in trigger control.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl RenderOnce for DatePicker {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let docked = self.options.variant == DatePickerVariant::Docked
            && self.presentation == Presentation::Triggered;
        let (needs_locale_init, open) = {
            let model = self.state.read(cx);
            let open = if self.presentation == Presentation::Panel {
                true
            } else if docked {
                model.docked_open
            } else {
                model.dialog.read(cx).is_open()
            };
            (!model.locale_initialized, open)
        };
        if needs_locale_init {
            self.state.update(cx, |state, cx| {
                state.locale = self.options.locale.clone();
                state.locale_initialized = true;
                if self.presentation == Presentation::Panel {
                    let draft = state.calendar.read(cx).date();
                    state.reset_input(draft, &self.options.locale, window, cx);
                }
            });
        }
        let width = (window.viewport_size().width - px(96.)).clamp(px(70.), px(336.));
        if self.presentation == Presentation::Panel {
            let body = self.body(width, None, true, window, cx);
            return Surface::new(ColorRole::SurfaceContainerHigh)
                .when_some(self.colors.container, Surface::container_color)
                .child(
                    div()
                        .id(self.id)
                        .role(Role::Group)
                        .aria_label(self.label)
                        .child(body),
                )
                .into_any_element();
        }

        let model = self.state.read(cx);
        let scope = model.docked.clone();
        let overlay = model.dialog.clone();
        let docked_actions = DockedActions {
            state: self.state.clone(),
            generation: model.docked_generation,
            properties: self.properties,
            confirm: self.confirm.clone(),
            request: self.request.clone(),
            closed: self.closed.clone(),
        };

        let trigger = (self.presentation == Presentation::Triggered)
            .then(|| self.trigger(docked, open, docked_actions.clone(), window, cx));
        if docked {
            let (trigger, bounds) = trigger.expect("docked pickers have an anchor");
            return self.render_docked(trigger, bounds, open, scope, docked_actions, window, cx);
        }

        self.render_dialog(trigger, overlay, open, width, window, cx)
    }
}
