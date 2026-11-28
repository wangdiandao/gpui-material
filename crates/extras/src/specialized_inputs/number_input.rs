use std::rc::Rc;

use gpui::{
    AnyElement, App, AppContext, Context, ElementId, Empty, Entity, EventEmitter, FocusHandle,
    Focusable, InteractiveElement, IntoElement, MouseButton, ParentElement, Render, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Subscription, Window, div,
    prelude::FluentBuilder, px,
};
use gpui_base::{
    Button as BaseButton, NumberInput as BaseNumberInput, StepAction as BaseStepAction,
    TestSupportExt, step_value,
};
use gpui_material_components::{
    Icon, IconName, Text, TextField, TextFieldColors, TextFieldEvent, TextFieldState,
    TextFieldVariant,
};
use gpui_material_foundation::{color, current_material_theme};
use gpui_material_theme::{ColorRole, Density, Direction};

/// Direction of a discrete step on [`NumberInput`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumberStepAction {
    /// Decrease the numeric value by one step.
    Decrement,
    /// Increase the numeric value by one step.
    Increment,
}

impl From<BaseStepAction> for NumberStepAction {
    fn from(action: BaseStepAction) -> Self {
        match action {
            BaseStepAction::Decrement => Self::Decrement,
            BaseStepAction::Increment => Self::Increment,
        }
    }
}

impl From<NumberStepAction> for BaseStepAction {
    fn from(action: NumberStepAction) -> Self {
        match action {
            NumberStepAction::Decrement => Self::Decrement,
            NumberStepAction::Increment => Self::Increment,
        }
    }
}

/// Events emitted by [`NumberInputState`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NumberInputEvent {
    /// The parsed numeric value changed (either by typing or stepping).
    Change(Option<f64>),
    /// A stepper button or `Up`/`Down` key stepped the value.
    Step(NumberStepAction),
    /// The user pressed `Enter` in the numeric editor.
    Submit(Option<f64>),
}

/// Layout of the decrement and increment stepper buttons on [`NumberInput`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NumberInputStepper {
    /// Places decrement at the logical start and increment at the logical end (default).
    #[default]
    Split,
    /// Groups decrement and increment together at the logical end of the field.
    Trailing,
    /// Omits visible stepper buttons while keeping keyboard `Up`/`Down` stepping.
    None,
}

/// Retained state for a Material 3 [`NumberInput`].
///
/// Owns the inner [`TextFieldState`], numeric bounds, step size, and optional fixed
/// decimal precision. Subscribe to [`NumberInputEvent`] to react to edits and steps.
pub struct NumberInputState {
    editor: Entity<TextFieldState>,
    min: Option<f64>,
    max: Option<f64>,
    step: f64,
    precision: Option<usize>,
    _subscriptions: Vec<Subscription>,
}

impl NumberInputState {
    /// Creates an empty numeric input state with a default step of `1.0`.
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let editor = cx.new(|cx| {
            let mut state = TextFieldState::new(window, cx);
            state.ensure_number_mask();
            state
        });
        let subscriptions = vec![
            cx.observe(&editor, |_, _, cx| cx.notify()),
            cx.subscribe_in(
                &editor,
                window,
                |_, editor, event: &TextFieldEvent, _, cx| match event {
                    TextFieldEvent::Change => {
                        let parsed = parse_number(editor.read(cx).unmask_value().as_str());
                        cx.emit(NumberInputEvent::Change(parsed));
                        cx.notify();
                    }
                    TextFieldEvent::PressEnter { .. } => {
                        let parsed = parse_number(editor.read(cx).unmask_value().as_str());
                        cx.emit(NumberInputEvent::Submit(parsed));
                    }
                    _ => {}
                },
            ),
        ];
        Self {
            editor,
            min: None,
            max: None,
            step: 1.0,
            precision: None,
            _subscriptions: subscriptions,
        }
    }

    /// Sets the initial numeric value during state construction.
    pub fn default_value(self, value: f64, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let clamped = clamp_value(value, self.min, self.max);
        let formatted = format_number(clamped, self.step, self.precision, "");
        self.editor
            .update(cx, |editor, cx| editor.set_value(formatted, window, cx));
        self
    }

    /// Sets the inclusive lower bound during construction. Non-finite values are ignored.
    pub fn min(mut self, min: f64) -> Self {
        if min.is_finite() {
            self.min = Some(min);
        }
        self
    }

    /// Sets the inclusive upper bound during construction. Non-finite values are ignored.
    pub fn max(mut self, max: f64) -> Self {
        if max.is_finite() {
            self.max = Some(max);
        }
        self
    }

    /// Sets inclusive lower and upper bounds during construction.
    ///
    /// If `min > max`, the bounds are swapped so `min <= max`.
    pub fn bounds(mut self, min: f64, max: f64) -> Self {
        if min.is_finite() && max.is_finite() {
            let (lo, hi) = if min <= max { (min, max) } else { (max, min) };
            self.min = Some(lo);
            self.max = Some(hi);
        }
        self
    }

    /// Sets the step increment used by buttons and `Up`/`Down` keys.
    /// Non-finite or non-positive steps are ignored.
    pub fn step(mut self, step: f64) -> Self {
        if step.is_finite() && step > 0.0 {
            self.step = step;
        }
        self
    }

    /// Fixes the number of decimal digits formatted after stepping or calling [`Self::set_value`].
    pub fn precision(mut self, digits: usize) -> Self {
        self.precision = Some(digits.min(12));
        self
    }

    /// Updates the inclusive lower bound.
    pub fn set_min(&mut self, min: Option<f64>, cx: &mut Context<Self>) {
        self.min = min.filter(|v| v.is_finite());
        cx.notify();
    }

    /// Updates the inclusive upper bound.
    pub fn set_max(&mut self, max: Option<f64>, cx: &mut Context<Self>) {
        self.max = max.filter(|v| v.is_finite());
        cx.notify();
    }

    /// Updates the step size. Non-finite or non-positive values are ignored.
    pub fn set_step(&mut self, step: f64, cx: &mut Context<Self>) {
        if step.is_finite() && step > 0.0 {
            self.step = step;
            cx.notify();
        }
    }

    /// Returns the configured lower bound, if any.
    pub fn min_value(&self) -> Option<f64> {
        self.min
    }

    /// Returns the configured upper bound, if any.
    pub fn max_value(&self) -> Option<f64> {
        self.max
    }

    /// Returns the configured step size.
    pub fn step_value(&self) -> f64 {
        self.step
    }

    /// Returns the current parsed numeric value, or `None` when empty or incomplete.
    pub fn value(&self, cx: &App) -> Option<f64> {
        parse_number(self.editor.read(cx).unmask_value().as_str())
    }

    /// Returns the current formatted text in the editor.
    pub fn text(&self, cx: &App) -> SharedString {
        self.editor.read(cx).value()
    }

    /// Replaces the numeric value, clamping to configured bounds without emitting [`NumberInputEvent::Change`].
    pub fn set_value(&mut self, value: f64, window: &mut Window, cx: &mut Context<Self>) {
        if !value.is_finite() {
            return;
        }
        let clamped = clamp_value(value, self.min, self.max);
        let current = self.editor.read(cx).unmask_value();
        let formatted = format_number(clamped, self.step, self.precision, current.as_str());
        self.editor
            .update(cx, |editor, cx| editor.set_value(formatted, window, cx));
        cx.notify();
    }

    /// Replaces the raw editor text without emitting [`NumberInputEvent::Change`].
    pub fn set_text(
        &mut self,
        text: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let text: SharedString = text.into();
        self.editor
            .update(cx, |editor, cx| editor.set_value(text, window, cx));
        cx.notify();
    }

    /// Returns whether the editor is disabled.
    pub fn is_disabled(&self, cx: &App) -> bool {
        self.editor.read(cx).presentation().is_disabled()
    }

    /// Sets whether the editor is disabled.
    pub fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>) {
        self.editor
            .update(cx, |editor, cx| editor.set_disabled(disabled, cx));
        cx.notify();
    }

    /// Returns whether the editor is read-only.
    pub fn is_readonly(&self, cx: &App) -> bool {
        self.editor.read(cx).presentation().is_readonly()
    }

    /// Sets whether the editor is read-only.
    pub fn set_readonly(&mut self, readonly: bool, cx: &mut Context<Self>) {
        self.editor
            .update(cx, |editor, cx| editor.set_readonly(readonly, cx));
        cx.notify();
    }

    /// Returns whether stepping upward is currently permitted.
    pub fn can_increment(&self, cx: &App) -> bool {
        if self.is_disabled(cx) || self.is_readonly(cx) {
            return false;
        }
        match (self.value(cx), self.max) {
            (Some(value), Some(max)) => value < max,
            _ => true,
        }
    }

    /// Returns whether stepping downward is currently permitted.
    pub fn can_decrement(&self, cx: &App) -> bool {
        if self.is_disabled(cx) || self.is_readonly(cx) {
            return false;
        }
        match (self.value(cx), self.min) {
            (Some(value), Some(min)) => value > min,
            _ => true,
        }
    }

    /// Steps the current value in `action` direction, clamping to configured bounds.
    ///
    /// Returns `true` when the value changed.
    pub fn step_by(
        &mut self,
        action: NumberStepAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.is_disabled(cx) || self.is_readonly(cx) {
            return false;
        }
        let raw = self.editor.read(cx).unmask_value();
        let next = compute_step(
            raw.as_str(),
            action,
            self.step,
            self.min,
            self.max,
            self.precision,
        );
        let Some(next) = next else {
            return false;
        };
        let parsed = parse_number(&next);
        self.editor
            .update(cx, |editor, cx| editor.set_value(next, window, cx));
        cx.emit(NumberInputEvent::Step(action));
        cx.emit(NumberInputEvent::Change(parsed));
        cx.notify();
        true
    }

    /// Increments the numeric value by one step.
    pub fn increment(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        self.step_by(NumberStepAction::Increment, window, cx)
    }

    /// Decrements the numeric value by one step.
    pub fn decrement(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        self.step_by(NumberStepAction::Decrement, window, cx)
    }

    /// Moves keyboard focus to the inner numeric text editor.
    pub fn focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor
            .update(cx, |editor, cx| editor.focus(window, cx));
    }

    /// Returns the underlying retained [`TextFieldState`] editor entity.
    pub fn editor(&self) -> &Entity<TextFieldState> {
        &self.editor
    }
}

impl Focusable for NumberInputState {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.editor.read(cx).focus_handle(cx)
    }
}

impl EventEmitter<NumberInputEvent> for NumberInputState {}

impl Render for NumberInputState {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

type StepCallback = Rc<dyn Fn(NumberStepAction, &mut Window, &mut App)>;
type ChangeCallback = Rc<dyn Fn(Option<f64>, &mut Window, &mut App)>;

/// Material 3 numeric input with Filled/Outlined presentation, decrement/increment
/// stepper buttons, bounds clamping, keyboard `Up`/`Down` stepping, and spinbutton
/// accessibility semantics.
#[derive(IntoElement)]
pub struct NumberInput {
    id: ElementId,
    label: SharedString,
    state: Entity<NumberInputState>,
    variant: TextFieldVariant,
    stepper: NumberInputStepper,
    placeholder: Option<SharedString>,
    prefix: Option<AnyElement>,
    suffix: Option<AnyElement>,
    supporting: Option<SharedString>,
    error: Option<SharedString>,
    invalid: bool,
    disabled: bool,
    required: bool,
    show_label: bool,
    decrement_label: SharedString,
    increment_label: SharedString,
    colors: TextFieldColors,
    on_step: Option<StepCallback>,
    on_change: Option<ChangeCallback>,
}

impl NumberInput {
    /// Creates a Filled Material 3 numeric input bound to `state`.
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<NumberInputState>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            state: state.clone(),
            variant: TextFieldVariant::Filled,
            stepper: NumberInputStepper::Split,
            placeholder: None,
            prefix: None,
            suffix: None,
            supporting: None,
            error: None,
            invalid: false,
            disabled: false,
            required: false,
            show_label: true,
            decrement_label: SharedString::new_static("Decrease"),
            increment_label: SharedString::new_static("Increase"),
            colors: TextFieldColors::default(),
            on_step: None,
            on_change: None,
        }
    }

    /// Creates a Filled Material 3 numeric input bound to `state`.
    pub fn filled(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<NumberInputState>,
    ) -> Self {
        Self::new(id, label, state).variant(TextFieldVariant::Filled)
    }

    /// Creates an Outlined Material 3 numeric input bound to `state`.
    pub fn outlined(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<NumberInputState>,
    ) -> Self {
        Self::new(id, label, state).variant(TextFieldVariant::Outlined)
    }

    /// Chooses Filled or Outlined container styling.
    pub fn variant(mut self, variant: TextFieldVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Configures the placement of the decrement and increment stepper buttons.
    pub fn stepper(mut self, stepper: NumberInputStepper) -> Self {
        self.stepper = stepper;
        self
    }

    /// Sets placeholder hint text shown when the field is empty.
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// Adds a decorative prefix element before the numeric text (such as a currency sign).
    pub fn prefix(mut self, element: impl IntoElement) -> Self {
        self.prefix = Some(element.into_any_element());
        self
    }

    /// Adds a decorative suffix element after the numeric text (such as a unit label).
    pub fn suffix(mut self, element: impl IntoElement) -> Self {
        self.suffix = Some(element.into_any_element());
        self
    }

    /// Sets supporting text below the field.
    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting = Some(text.into());
        self
    }

    /// Marks the input invalid and displays an error message below the field.
    pub fn error(mut self, message: impl Into<SharedString>) -> Self {
        self.error = Some(message.into());
        self.invalid = true;
        self
    }

    /// Marks the input invalid without replacing the supporting message.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Disables editing and stepper interaction for this render.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Exposes the field as required to assistive technology.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Controls whether the visible floating label is rendered. The constructor label
    /// continues to provide the accessible name.
    pub fn show_label(mut self, show: bool) -> Self {
        self.show_label = show;
        self
    }

    /// Overrides the accessible label for the decrement button (defaults to `"Decrease"`).
    pub fn decrement_label(mut self, label: impl Into<SharedString>) -> Self {
        self.decrement_label = label.into();
        self
    }

    /// Overrides the accessible label for the increment button (defaults to `"Increase"`).
    pub fn increment_label(mut self, label: impl Into<SharedString>) -> Self {
        self.increment_label = label.into();
        self
    }

    /// Supplies per-state color overrides for the underlying Material [`TextField`].
    pub fn colors(mut self, colors: TextFieldColors) -> Self {
        self.colors = colors;
        self
    }

    /// Invoked whenever a stepper button or `Up`/`Down` key requests a step.
    pub fn on_step(
        mut self,
        handler: impl Fn(NumberStepAction, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_step = Some(Rc::new(handler));
        self
    }

    /// Invoked whenever stepping changes the parsed numeric value.
    pub fn on_change(
        mut self,
        handler: impl Fn(Option<f64>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for NumberInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = current_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let button_side = match theme.layout.density {
            Density::Compact => px(32.),
            Density::Standard => px(36.),
            Density::Comfortable => px(40.),
        };
        let editor = self.state.read(cx).editor().clone();
        let state_disabled = self.state.read(cx).is_disabled(cx);
        let state_readonly = self.state.read(cx).is_readonly(cx);
        let disabled = self.disabled || state_disabled;
        let can_dec = !disabled && !state_readonly && self.state.read(cx).can_decrement(cx);
        let can_inc = !disabled && !state_readonly && self.state.read(cx).can_increment(cx);

        let icon_enabled = color(theme.colors.color(ColorRole::OnSurfaceVariant));
        let icon_disabled = color(theme.colors.color(ColorRole::OnSurface)).opacity(0.38);
        let hover_bg = color(theme.colors.color(ColorRole::OnSurface)).opacity(0.08);
        let active_bg = color(theme.colors.color(ColorRole::OnSurface)).opacity(0.12);

        let state_for_step = self.state.clone();
        let on_step_cb = self.on_step.clone();
        let on_change_cb = self.on_change.clone();
        let perform_step: StepCallback = Rc::new(move |action, window, cx| {
            let changed = state_for_step.update(cx, |state, cx| {
                state.focus(window, cx);
                state.step_by(action, window, cx)
            });
            if changed {
                if let Some(cb) = on_step_cb.as_ref() {
                    cb(action, window, cx);
                }
                if let Some(cb) = on_change_cb.as_ref() {
                    let value = state_for_step.read(cx).value(cx);
                    cb(value, window, cx);
                }
            }
        });

        let build_step_button = |suffix: &'static str,
                                 label: SharedString,
                                 icon: IconName,
                                 enabled: bool,
                                 action: NumberStepAction,
                                 perform: StepCallback| {
            let tint = if enabled { icon_enabled } else { icon_disabled };
            BaseButton::new((self.id.clone(), suffix))
                .focusable(false)
                .disabled(!enabled)
                .accessibility_label(label)
                .w(button_side)
                .h(button_side)
                .rounded_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(tint)
                .when(enabled, |this| {
                    this.cursor_pointer()
                        .hover(move |style| style.bg(hover_bg))
                        .active(move |style| style.bg(active_bg))
                        .on_mouse_down(MouseButton::Left, |_, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                        })
                        .on_click(move |_, window, cx| {
                            cx.stop_propagation();
                            perform(action, window, cx);
                        })
                })
                .child(Icon::new(icon).size(px(18.)).inherit_color())
        };

        let mut field = TextField::new((self.id.clone(), "field"), self.label.clone(), &editor)
            .variant(self.variant)
            .disabled(disabled)
            .invalid(self.invalid)
            .required(self.required)
            .show_label(self.show_label)
            .colors(self.colors);

        if let Some(placeholder) = self.placeholder {
            field = field.placeholder_content(placeholder.clone(), Text::new(placeholder));
        }
        if let Some(prefix) = self.prefix {
            field = field.prefix(prefix);
        }
        if let Some(suffix) = self.suffix {
            field = field.suffix(suffix);
        }
        if let Some(error) = self.error {
            field = field.error(error);
        } else if let Some(supporting) = self.supporting {
            field = field.supporting_text(supporting);
        }

        match self.stepper {
            NumberInputStepper::Split => {
                let dec = build_step_button(
                    "decrement",
                    self.decrement_label,
                    IconName::Remove,
                    can_dec,
                    NumberStepAction::Decrement,
                    perform_step.clone(),
                );
                let inc = build_step_button(
                    "increment",
                    self.increment_label,
                    IconName::Add,
                    can_inc,
                    NumberStepAction::Increment,
                    perform_step.clone(),
                );
                field = field.leading(dec).trailing(inc);
            }
            NumberInputStepper::Trailing => {
                let dec = build_step_button(
                    "decrement",
                    self.decrement_label,
                    IconName::Remove,
                    can_dec,
                    NumberStepAction::Decrement,
                    perform_step.clone(),
                );
                let inc = build_step_button(
                    "increment",
                    self.increment_label,
                    IconName::Add,
                    can_inc,
                    NumberStepAction::Increment,
                    perform_step.clone(),
                );
                let group = div()
                    .flex()
                    .items_center()
                    .gap_0p5()
                    .when(rtl, Styled::flex_row_reverse)
                    .child(dec)
                    .child(inc);
                field = field.trailing(group);
            }
            NumberInputStepper::None => {}
        }

        let base_step = perform_step.clone();
        div().id(self.id).test_support().w_full().child(
            BaseNumberInput::new(&editor)
                .w_full()
                .stepper(false)
                .disabled(disabled)
                .accessibility_label(self.label)
                .on_step(move |action, window, cx| {
                    base_step(action.into(), window, cx);
                })
                .input(field),
        )
    }
}

fn parse_number(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let parsed = trimmed.parse::<f64>().ok()?;
    parsed.is_finite().then_some(parsed)
}

fn clamp_value(value: f64, min: Option<f64>, max: Option<f64>) -> f64 {
    let mut clamped = value;
    if let Some(min) = min {
        clamped = clamped.max(min);
    }
    if let Some(max) = max {
        clamped = clamped.min(max);
    }
    if clamped == 0.0 { 0.0 } else { clamped }
}

fn fraction_digits(value: &str) -> usize {
    value.split('.').nth(1).map_or(0, |frac| frac.len())
}

fn format_number(value: f64, step: f64, precision: Option<usize>, current_text: &str) -> String {
    let clean = if value == 0.0 { 0.0 } else { value };
    if let Some(digits) = precision {
        return format!("{clean:.digits$}");
    }
    let digits = fraction_digits(current_text).max(fraction_digits(&step.to_string()));
    format!("{clean:.digits$}")
}

fn compute_step(
    current_text: &str,
    action: NumberStepAction,
    step: f64,
    min: Option<f64>,
    max: Option<f64>,
    precision: Option<usize>,
) -> Option<String> {
    let stepped = step_value(current_text, action.into(), step, min, max)?;
    if let Some(digits) = precision {
        let parsed = parse_number(&stepped)?;
        let clean = if parsed == 0.0 { 0.0 } else { parsed };
        Some(format!("{clean:.digits$}"))
    } else if stepped == "-0" || stepped.starts_with("-0.") && parse_number(&stepped) == Some(0.0) {
        Some(stepped.trim_start_matches('-').to_owned())
    } else {
        Some(stepped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_step_respects_bounds_precision_and_zero_sign() {
        assert_eq!(
            compute_step("0.1", NumberStepAction::Increment, 0.2, None, None, None).as_deref(),
            Some("0.3")
        );
        assert_eq!(
            compute_step(
                "10",
                NumberStepAction::Increment,
                1.0,
                Some(0.0),
                Some(10.0),
                None
            ),
            None
        );
        assert_eq!(
            compute_step(
                "0",
                NumberStepAction::Decrement,
                1.0,
                Some(0.0),
                Some(10.0),
                None
            ),
            None
        );
        assert_eq!(
            compute_step(
                "9.5",
                NumberStepAction::Increment,
                1.0,
                Some(0.0),
                Some(10.0),
                Some(2)
            )
            .as_deref(),
            Some("10.00")
        );
        assert_eq!(
            compute_step("-1", NumberStepAction::Increment, 1.0, None, None, None).as_deref(),
            Some("0")
        );
    }

    #[test]
    fn clamp_and_parse_filter_invalid_values() {
        assert_eq!(parse_number(""), None);
        assert_eq!(parse_number("  "), None);
        assert_eq!(parse_number("NaN"), None);
        assert_eq!(parse_number("42.5"), Some(42.5));
        assert_eq!(clamp_value(-5.0, Some(0.0), Some(10.0)), 0.0);
        assert_eq!(clamp_value(25.0, Some(0.0), Some(10.0)), 10.0);
        assert_eq!(clamp_value(-0.0, None, None).to_bits(), 0.0_f64.to_bits());
    }
}
