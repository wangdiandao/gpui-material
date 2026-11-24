use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gpui::{
    AnyElement, App, Div, ElementId, Entity, FocusHandle, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Window,
};
use gpui_material_foundation::{
    InteractionSource, ThemeScope,
    control_feedback::{FeedbackState, feedback_target, ripple_state},
};
use gpui_material_theme::Direction;

use super::{row, selection_label};

/// Shared configuration owned by `Checkbox`, `Radio`, and `Switch`.
pub(super) struct SelectionControlBase {
    pub(super) id: ElementId,
    pub(super) label: SharedString,
    pub(super) show_label: bool,
    pub(super) label_content: Option<AnyElement>,
    pub(super) disabled: bool,
    pub(super) focus: Option<FocusHandle>,
    pub(super) interaction_source: Option<Entity<InteractionSource>>,
    pub(super) decorative: bool,
}

/// Prepared focus, ripple, and interaction tracking state for one render pass.
pub(super) struct SelectionInteraction {
    pub(super) ripple: Rc<RefCell<FeedbackState>>,
    pub(super) focus: FocusHandle,
    observes_input: bool,
    disabled: bool,
}

impl SelectionControlBase {
    pub(super) fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            show_label: true,
            label_content: None,
            disabled: false,
            focus: None,
            interaction_source: None,
            decorative: false,
        }
    }

    pub(super) fn show_label(&mut self, show: bool) {
        self.show_label = show;
    }

    pub(super) fn label_content(&mut self, content: impl IntoElement) {
        self.label_content = Some(content.into_any_element());
    }

    pub(super) fn disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    pub(super) fn track_focus(&mut self, focus: &FocusHandle) {
        self.focus = Some(focus.clone());
    }

    pub(super) fn interaction_source(&mut self, source: &Entity<InteractionSource>) {
        self.interaction_source = Some(source.clone());
    }

    #[cfg(feature = "surfaces")]
    pub(super) fn decorative(&mut self, decorative: bool) {
        self.decorative = decorative;
    }

    pub(super) fn group_name(&self, prefix: &str) -> SharedString {
        format!("{prefix}/{}", self.id).into()
    }

    pub(super) fn prepare_interaction(
        &mut self,
        override_focus: Option<FocusHandle>,
        window: &mut Window,
        cx: &mut App,
    ) -> SelectionInteraction {
        let ripple = ripple_state(&self.id, !self.disabled, window, cx);
        let focus = override_focus
            .or_else(|| self.focus.take())
            .unwrap_or_else(|| {
                window
                    .use_keyed_state(self.id.clone(), cx, |_, cx| cx.focus_handle())
                    .read(cx)
                    .clone()
            });
        let tracker = self
            .interaction_source
            .as_ref()
            .map(|source| InteractionSource::track(source, &self.id, window, cx));
        let observes_input = tracker.is_some();
        ripple
            .borrow_mut()
            .track(!self.disabled, &focus, tracker, window, cx);
        SelectionInteraction {
            ripple,
            focus,
            observes_input,
            disabled: self.disabled,
        }
    }

    pub(super) fn render_row(
        self,
        direction: Direction,
        target: impl IntoElement,
        window: &Window,
        cx: &App,
    ) -> Div {
        row(direction, self.show_label).child(target).children(
            self.show_label.then(|| {
                selection_label(self.label, self.label_content, self.disabled, window, cx)
            }),
        )
    }
}

impl SelectionInteraction {
    pub(super) fn attach_feedback<E>(&self, control: E) -> E
    where
        E: StatefulInteractiveElement,
    {
        if self.disabled {
            control
        } else {
            feedback_target(
                control,
                self.ripple.clone(),
                self.observes_input,
                self.observes_input.then(|| self.focus.clone()),
            )
        }
    }
}

/// Theme-scoped value-change callback that can be invoked with a value or its default next value.
pub(super) type ValueHandler<T> = Rc<dyn Fn(T, &mut Window, &mut App)>;

pub(super) struct BoundValueAction<T: Copy + 'static> {
    default: T,
    pending: Rc<Cell<T>>,
    bound: crate::action::Activate,
}

impl<T: Copy + 'static> Clone for BoundValueAction<T> {
    fn clone(&self) -> Self {
        Self {
            default: self.default,
            pending: self.pending.clone(),
            bound: self.bound.clone(),
        }
    }
}

impl<T: Copy + 'static> BoundValueAction<T> {
    pub(super) fn new(default: T, handler: ValueHandler<T>, window: &Window, cx: &App) -> Self {
        let pending = Rc::new(Cell::new(default));
        let pending_arg = pending.clone();
        let bound = Rc::new(ThemeScope::bind_action(
            move |window, cx| handler(pending_arg.get(), window, cx),
            window,
            cx,
        ));
        Self {
            default,
            pending,
            bound,
        }
    }

    pub(super) fn invoke(&self, value: T, window: &mut Window, cx: &mut App) {
        self.pending.set(value);
        (self.bound)(window, cx);
    }

    pub(super) fn invoke_default(&self, window: &mut Window, cx: &mut App) {
        self.invoke(self.default, window, cx);
    }
}

pub(super) fn bind_activate(
    handler: crate::action::Activate,
    window: &Window,
    cx: &App,
) -> crate::action::Activate {
    Rc::new(ThemeScope::bind_action(
        move |window, cx| handler(window, cx),
        window,
        cx,
    ))
}
