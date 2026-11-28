use std::rc::Rc;

use gpui::{
    App, AppContext, Context, ElementId, Empty, Entity, EventEmitter, FocusHandle, Focusable, Hsla,
    InteractiveElement, IntoElement, KeyDownEvent, MouseButton, ParentElement, Pixels, Render,
    RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled, Subscription, Window, div,
    prelude::FluentBuilder, px,
};
use gpui_base::{
    OtpEvent as BaseOtpEvent, OtpInput as BaseOtpInput, OtpState as BaseOtpState, TestSupportExt,
};
use gpui_material_components::Text;
use gpui_material_foundation::{color, current_material_theme};
use gpui_material_theme::{ColorRole, CornerRole, Density, Direction, TypeRole};

/// Events emitted by [`OtpInputState`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OtpInputEvent {
    /// The verification code changed.
    Change(SharedString),
    /// All verification code slots are filled.
    Complete(SharedString),
    /// The OTP input gained keyboard focus.
    Focus,
    /// The OTP input lost keyboard focus.
    Blur,
}

/// Visual container treatment for [`OtpInput`] slots.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OtpInputVariant {
    /// Transparent/surface slots framed by an outline stroke (default).
    #[default]
    Outlined,
    /// Filled `SurfaceContainerHighest` slots with an active bottom/perimeter accent.
    Filled,
}

/// Per-instance color overrides for [`OtpInput`]. Omitted fields resolve from the window theme.
#[derive(Clone, Copy, Debug, Default)]
pub struct OtpInputColors {
    /// Resting slot container color.
    pub container: Option<Hsla>,
    /// Container color for slots that contain a digit.
    pub filled_container: Option<Hsla>,
    /// Resting slot border color.
    pub border: Option<Hsla>,
    /// Border / focus ring color for the active slot while focused.
    pub active_border: Option<Hsla>,
    /// Border and message color when invalid and enabled.
    pub error_border: Option<Hsla>,
    /// Digit or mask-dot foreground color.
    pub text: Option<Hsla>,
    /// Caret bar color in an active empty slot.
    pub caret: Option<Hsla>,
    /// Slot container color when disabled.
    pub disabled_container: Option<Hsla>,
    /// Slot border color when disabled.
    pub disabled_border: Option<Hsla>,
    /// Digit foreground color when disabled.
    pub disabled_text: Option<Hsla>,
}

/// Retained state for a Material 3 segmented [`OtpInput`].
///
/// Wraps the underlying base OTP focus/blinking state while adding slot-cursor
/// navigation (`Left`, `Right`, `Home`, `End`), in-place slot replacement, `Delete`,
/// and clipboard paste (`Ctrl/Cmd+V`).
pub struct OtpInputState {
    base: Entity<BaseOtpState>,
    length: usize,
    active_slot: usize,
    _subscriptions: Vec<Subscription>,
}

impl OtpInputState {
    /// Creates a new OTP state with `length` digit slots (minimum `1`).
    pub fn new(length: usize, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let length = length.max(1);
        let base = cx.new(|cx| BaseOtpState::new(length, window, cx));
        let subscriptions = vec![
            cx.observe(&base, |_, _, cx| cx.notify()),
            cx.subscribe_in(
                &base,
                window,
                |this, base, event: &BaseOtpEvent, _, cx| match event {
                    BaseOtpEvent::Change => {
                        let value = base.read(cx).value().clone();
                        this.active_slot = value.chars().count().min(this.length - 1);
                        cx.emit(OtpInputEvent::Change(value));
                        cx.notify();
                    }
                    BaseOtpEvent::Complete => {
                        let value = base.read(cx).value().clone();
                        this.active_slot = this.length - 1;
                        cx.emit(OtpInputEvent::Complete(value));
                        cx.notify();
                    }
                    BaseOtpEvent::Focus => {
                        let count = base.read(cx).value().chars().count();
                        this.active_slot = count.min(this.length - 1);
                        cx.emit(OtpInputEvent::Focus);
                        cx.notify();
                    }
                    BaseOtpEvent::Blur => {
                        cx.emit(OtpInputEvent::Blur);
                        cx.notify();
                    }
                },
            ),
        ];
        Self {
            base,
            length,
            active_slot: 0,
            _subscriptions: subscriptions,
        }
    }

    /// Sets an initial digit value during construction (non-digits are stripped and truncated to `length`).
    pub fn default_value(
        mut self,
        value: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let sanitized = sanitize_digits(value.into().as_str(), self.length);
        self.active_slot = sanitized.chars().count().min(self.length - 1);
        self.base
            .update(cx, |base, cx| base.set_value(sanitized, window, cx));
        self
    }

    /// Enables or disables masked bullet display during construction.
    pub fn masked(self, masked: bool, window: &mut Window, cx: &mut Context<Self>) -> Self {
        self.base
            .update(cx, |base, cx| base.set_masked(masked, window, cx));
        self
    }

    /// Returns the current entered digits.
    pub fn value(&self, cx: &App) -> SharedString {
        self.base.read(cx).value().clone()
    }

    /// Returns the total number of code slots.
    pub fn length(&self) -> usize {
        self.length
    }

    /// Returns whether no digits have been entered.
    pub fn is_empty(&self, cx: &App) -> bool {
        self.base.read(cx).is_empty()
    }

    /// Returns whether all slots contain a digit.
    pub fn is_complete(&self, cx: &App) -> bool {
        self.base.read(cx).value().chars().count() >= self.length
    }

    /// Returns whether digits are masked with bullets.
    pub fn is_masked(&self, cx: &App) -> bool {
        self.base.read(cx).is_masked()
    }

    /// Returns the currently targeted slot index (`0..length()`).
    pub fn active_slot(&self) -> usize {
        self.active_slot
    }

    /// Sets whether digits are masked with bullets.
    pub fn set_masked(&mut self, masked: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.base
            .update(cx, |base, cx| base.set_masked(masked, window, cx));
        cx.notify();
    }

    /// Replaces the entered digits programmatically without emitting [`OtpInputEvent::Change`].
    pub fn set_value(
        &mut self,
        value: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let sanitized = sanitize_digits(value.into().as_str(), self.length);
        self.active_slot = sanitized.chars().count().min(self.length - 1);
        self.base
            .update(cx, |base, cx| base.set_value(sanitized, window, cx));
        cx.notify();
    }

    /// Clears all entered digits without emitting [`OtpInputEvent::Change`].
    pub fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_value("", window, cx);
    }

    /// Moves the active slot index, clamped to the reachable slot range.
    pub fn set_active_slot(&mut self, index: usize, cx: &mut Context<Self>) {
        let filled = self.base.read(cx).value().chars().count();
        let max_slot = filled.min(self.length - 1);
        self.active_slot = index.min(max_slot);
        cx.notify();
    }

    /// Pastes digits extracted from `text` starting at the active slot (or replacing
    /// the whole code when `text` contains at least `length()` digits).
    ///
    /// Returns `true` if the value changed.
    pub fn paste(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let current = self.base.read(cx).value().clone();
        let Some((next, next_slot)) =
            apply_paste(current.as_str(), self.active_slot, text, self.length)
        else {
            return false;
        };
        if next == current.as_str() {
            return false;
        }
        let shared: SharedString = next.into();
        self.active_slot = next_slot;
        self.base
            .update(cx, |base, cx| base.set_value(shared.clone(), window, cx));
        cx.emit(OtpInputEvent::Change(shared.clone()));
        if shared.chars().count() == self.length {
            cx.emit(OtpInputEvent::Complete(shared));
        }
        cx.notify();
        true
    }

    /// Focuses the OTP input.
    pub fn focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.base.update(cx, |base, cx| base.focus(window, cx));
    }

    fn handle_capture_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<(SharedString, bool)> {
        let key = event.keystroke.key.as_str();
        let modifiers = event.keystroke.modifiers;

        if (modifiers.control || modifiers.platform) && key.eq_ignore_ascii_case("v") {
            if let Some(item) = cx.read_from_clipboard()
                && let Some(text) = item.text()
            {
                let changed = self.paste(&text, window, cx);
                window.prevent_default();
                cx.stop_propagation();
                if changed {
                    let val = self.value(cx);
                    let complete = val.chars().count() == self.length;
                    return Some((val, complete));
                }
            }
            return None;
        }

        if modifiers.control || modifiers.platform || modifiers.alt {
            return None;
        }

        let current = self.base.read(cx).value().clone();
        let filled = current.chars().count();
        let max_slot = filled.min(self.length - 1);

        match key {
            "left" => {
                self.active_slot = self.active_slot.saturating_sub(1);
                window.prevent_default();
                cx.stop_propagation();
                cx.notify();
                return None;
            }
            "right" => {
                self.active_slot = (self.active_slot + 1).min(max_slot);
                window.prevent_default();
                cx.stop_propagation();
                cx.notify();
                return None;
            }
            "home" => {
                self.active_slot = 0;
                window.prevent_default();
                cx.stop_propagation();
                cx.notify();
                return None;
            }
            "end" => {
                self.active_slot = max_slot;
                window.prevent_default();
                cx.stop_propagation();
                cx.notify();
                return None;
            }
            "delete" => {
                if let Some((next, next_slot)) =
                    delete_at(current.as_str(), self.active_slot, self.length)
                {
                    let shared: SharedString = next.into();
                    self.active_slot = next_slot;
                    self.base
                        .update(cx, |base, cx| base.set_value(shared.clone(), window, cx));
                    cx.emit(OtpInputEvent::Change(shared.clone()));
                    window.prevent_default();
                    cx.stop_propagation();
                    cx.notify();
                    return Some((shared, false));
                }
                window.prevent_default();
                cx.stop_propagation();
                return None;
            }
            "backspace" => {
                if filled == 0 {
                    window.prevent_default();
                    cx.stop_propagation();
                    return None;
                }
                let target = if self.active_slot < filled {
                    self.active_slot
                } else {
                    filled - 1
                };
                if let Some((next, next_slot)) = delete_at(current.as_str(), target, self.length) {
                    let shared: SharedString = next.into();
                    self.active_slot = next_slot;
                    self.base
                        .update(cx, |base, cx| base.set_value(shared.clone(), window, cx));
                    cx.emit(OtpInputEvent::Change(shared.clone()));
                    window.prevent_default();
                    cx.stop_propagation();
                    cx.notify();
                    return Some((shared, false));
                }
                return None;
            }
            _ => {}
        }

        // Handle digit entry synchronously so `active_slot` stays exact even across
        // rapid multi-key input within a single frame.
        if let Some(digit) = extract_single_digit(key, event.keystroke.key_char.as_deref()) {
            if self.active_slot >= self.length {
                return None;
            }
            let (next, next_slot) =
                replace_at(current.as_str(), self.active_slot, digit, self.length);
            let shared: SharedString = next.into();
            let complete = shared.chars().count() == self.length;
            self.active_slot = next_slot;
            self.base
                .update(cx, |base, cx| base.set_value(shared.clone(), window, cx));
            cx.emit(OtpInputEvent::Change(shared.clone()));
            if complete {
                cx.emit(OtpInputEvent::Complete(shared.clone()));
            }
            window.prevent_default();
            cx.stop_propagation();
            cx.notify();
            return Some((shared, complete));
        }

        None
    }
}

impl Focusable for OtpInputState {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.base.read(cx).focus_handle(cx)
    }
}

impl EventEmitter<OtpInputEvent> for OtpInputState {}

impl Render for OtpInputState {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

type OtpCallback = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

/// Material 3 segmented one-time-password / verification code input.
///
/// Numeric slots always remain in left-to-right order per Material 3 and Unicode BiDi
/// guidance, even inside right-to-left windows, while the label and supporting/error
/// caption follow the window's logical direction.
#[derive(IntoElement)]
pub struct OtpInput {
    id: ElementId,
    label: SharedString,
    state: Entity<OtpInputState>,
    variant: OtpInputVariant,
    masked_override: Option<bool>,
    disabled: bool,
    invalid: bool,
    error: Option<SharedString>,
    supporting: Option<SharedString>,
    show_label: bool,
    separator_after: Option<usize>,
    slot_size: Option<(Pixels, Pixels)>,
    gap: Pixels,
    colors: OtpInputColors,
    on_change: Option<OtpCallback>,
    on_complete: Option<OtpCallback>,
}

impl OtpInput {
    /// Creates an Outlined segmented OTP input bound to `state`.
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<OtpInputState>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            state: state.clone(),
            variant: OtpInputVariant::Outlined,
            masked_override: None,
            disabled: false,
            invalid: false,
            error: None,
            supporting: None,
            show_label: true,
            separator_after: None,
            slot_size: None,
            gap: px(8.),
            colors: OtpInputColors::default(),
            on_change: None,
            on_complete: None,
        }
    }

    /// Selects Outlined or Filled slot styling.
    pub fn variant(mut self, variant: OtpInputVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Overrides whether entered digits render as bullets (`•`).
    pub fn masked(mut self, masked: bool) -> Self {
        self.masked_override = Some(masked);
        self
    }

    /// Disables keyboard and pointer interaction.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the code invalid and displays an error message below the slots.
    pub fn error(mut self, message: impl Into<SharedString>) -> Self {
        self.error = Some(message.into());
        self.invalid = true;
        self
    }

    /// Marks the code invalid without replacing the supporting message.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Displays supporting helper text below the slots.
    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting = Some(text.into());
        self
    }

    /// Controls whether the visible label above the slots is shown. The constructor
    /// label always remains the accessible group name.
    pub fn show_label(mut self, show: bool) -> Self {
        self.show_label = show;
        self
    }

    /// Inserts a decorative dash separator after the `index`-th slot (1-based count,
    /// e.g. `separator_after(3)` renders `1 2 3 - 4 5 6` for a 6-digit code).
    pub fn separator_after(mut self, count: usize) -> Self {
        self.separator_after = (count > 0).then_some(count);
        self
    }

    /// Sets explicit slot width and height in logical pixels.
    pub fn slot_size(mut self, width: Pixels, height: Pixels) -> Self {
        if f32::from(width).is_finite()
            && f32::from(height).is_finite()
            && width > px(0.)
            && height > px(0.)
        {
            self.slot_size = Some((width, height));
        }
        self
    }

    /// Sets horizontal spacing between slots in logical pixels (defaults to `8 px`).
    pub fn gap(mut self, gap: Pixels) -> Self {
        if f32::from(gap).is_finite() && gap >= px(0.) {
            self.gap = gap;
        }
        self
    }

    /// Supplies per-instance color overrides.
    pub fn colors(mut self, colors: OtpInputColors) -> Self {
        self.colors = colors;
        self
    }

    /// Invoked when user interaction changes the verification code.
    pub fn on_change(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// Invoked when user interaction fills the final verification code slot.
    pub fn on_complete(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_complete = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for OtpInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = current_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let reduce_motion = cx.reduce_motion();
        let disabled = self.disabled;
        let error_active = self.invalid && !disabled;

        let (slot_w, slot_h) = self.slot_size.unwrap_or(match theme.layout.density {
            Density::Compact => (px(44.), px(48.)),
            Density::Standard => (px(48.), px(56.)),
            Density::Comfortable => (px(52.), px(60.)),
        });
        let radius = px(CornerRole::Medium.shape().radii(
            f32::from(slot_w),
            f32::from(slot_h),
            Direction::Ltr,
        )[0]);

        let base_entity = self.state.read(cx).base.clone();
        let focus_handle = self.state.read(cx).focus_handle(cx);
        let focused = !disabled && focus_handle.is_focused(window);
        let length = self.state.read(cx).length();
        let value = self.state.read(cx).value(cx);
        let chars: Vec<char> = value.chars().collect();
        let active_slot = self.state.read(cx).active_slot().min(length - 1);
        let masked = self
            .masked_override
            .unwrap_or_else(|| self.state.read(cx).is_masked(cx));
        let cursor_visible = focused && (reduce_motion || base_entity.read(cx).cursor_visible(cx));

        let surface_highest = color(theme.colors.color(ColorRole::SurfaceContainerHighest));
        let surface_container = color(theme.colors.color(ColorRole::SurfaceContainerLow));
        let outline = color(theme.colors.color(ColorRole::Outline));
        let primary = color(theme.colors.color(ColorRole::Primary));
        let error_color = color(theme.colors.color(ColorRole::Error));
        let on_surface = color(theme.colors.color(ColorRole::OnSurface));
        let on_surface_variant = color(theme.colors.color(ColorRole::OnSurfaceVariant));

        let resting_bg = self.colors.container.unwrap_or(match self.variant {
            OtpInputVariant::Outlined => gpui::transparent_black(),
            OtpInputVariant::Filled => surface_highest,
        });
        let filled_bg = self.colors.filled_container.unwrap_or(match self.variant {
            OtpInputVariant::Outlined => surface_container,
            OtpInputVariant::Filled => surface_highest,
        });
        let resting_border = self.colors.border.unwrap_or(outline);
        let active_border = self.colors.active_border.unwrap_or(primary);
        let err_border = self.colors.error_border.unwrap_or(error_color);
        let digit_color = self.colors.text.unwrap_or(on_surface);
        let caret_color = self.colors.caret.unwrap_or(primary);
        let disabled_bg = self
            .colors
            .disabled_container
            .unwrap_or(on_surface.opacity(0.04));
        let disabled_border = self
            .colors
            .disabled_border
            .unwrap_or(on_surface.opacity(0.12));
        let disabled_text = self
            .colors
            .disabled_text
            .unwrap_or(on_surface.opacity(0.38));

        let on_change_cb = self.on_change.clone();
        let on_complete_cb = self.on_complete.clone();
        let state_for_capture = self.state.clone();
        let state_for_bubble = self.state.clone();
        let value_before_key = value.clone();

        let mut slot_elements =
            Vec::with_capacity(length + usize::from(self.separator_after.is_some()));
        for index in 0..length {
            if self.separator_after == Some(index) && index < length {
                slot_elements.push(
                    div()
                        .w(px(12.))
                        .h(px(2.))
                        .rounded_full()
                        .bg(if disabled {
                            disabled_border
                        } else {
                            on_surface_variant
                        })
                        .into_any_element(),
                );
            }

            let digit = chars.get(index).copied();
            let is_active = focused && index == active_slot;
            let bg = if disabled {
                disabled_bg
            } else if digit.is_some() {
                filled_bg
            } else {
                resting_bg
            };
            let border = if disabled {
                disabled_border
            } else if error_active {
                err_border
            } else if is_active {
                active_border
            } else {
                resting_border
            };
            let border_width = if is_active || error_active {
                px(2.)
            } else {
                px(1.)
            };
            let show_caret = is_active && digit.is_none() && cursor_visible;
            let slot_state = self.state.clone();

            let slot = div()
                .id((self.id.clone(), format!("slot-{index}")))
                .test_support()
                .w(slot_w)
                .h(slot_h)
                .rounded(radius)
                .bg(bg)
                .border(border_width)
                .border_color(border)
                .flex()
                .items_center()
                .justify_center()
                .when(!disabled, |this| {
                    this.cursor_text()
                        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            slot_state.update(cx, |state, cx| {
                                state.set_active_slot(index, cx);
                                state.focus(window, cx);
                            });
                            window.prevent_default();
                            cx.stop_propagation();
                        })
                })
                .when_some(digit, |this, ch| {
                    let display = if masked {
                        SharedString::new_static("•")
                    } else {
                        ch.to_string().into()
                    };
                    this.child(
                        Text::new(display)
                            .role(TypeRole::HeadlineSmall)
                            .color_override(if disabled { disabled_text } else { digit_color }),
                    )
                })
                .when(show_caret, |this| {
                    this.child(div().w(px(2.)).h(px(22.)).rounded_full().bg(caret_color))
                });

            slot_elements.push(slot.into_any_element());
        }

        let base_otp = BaseOtpInput::new(&base_entity)
            .disabled(disabled)
            .flex()
            .flex_row()
            .items_center()
            .gap(self.gap)
            .children(slot_elements);

        let caption = if error_active {
            self.error.or(self.supporting)
        } else {
            self.supporting
        };

        let on_change_after = on_change_cb.clone();
        let on_complete_after = on_complete_cb.clone();

        div()
            .id(self.id.clone())
            .test_support()
            .role(Role::Group)
            .aria_label(self.label.clone())
            .flex()
            .flex_col()
            .gap_1p5()
            .when(rtl, Styled::items_end)
            .when(!disabled, |this| {
                this.capture_key_down(move |event, window, cx| {
                    let handled = state_for_capture.update(cx, |state, cx| {
                        state.handle_capture_key_down(event, window, cx)
                    });
                    if let Some((next_value, complete)) = handled {
                        if let Some(cb) = on_change_cb.as_ref() {
                            cb(next_value.clone(), window, cx);
                        }
                        if complete && let Some(cb) = on_complete_cb.as_ref() {
                            cb(next_value, window, cx);
                        }
                    }
                })
                .on_key_down(move |_, window, cx| {
                    let after = state_for_bubble.read(cx).value(cx);
                    if after != value_before_key {
                        if let Some(cb) = on_change_after.as_ref() {
                            cb(after.clone(), window, cx);
                        }
                        if after.chars().count() == length
                            && let Some(cb) = on_complete_after.as_ref()
                        {
                            cb(after, window, cx);
                        }
                    }
                })
            })
            .when(self.show_label, |this| {
                this.child(
                    Text::new(self.label.clone())
                        .role(TypeRole::LabelLarge)
                        .color_override(if disabled {
                            disabled_text
                        } else if error_active {
                            err_border
                        } else {
                            on_surface_variant
                        }),
                )
            })
            .child(base_otp)
            .when_some(caption, |this, text| {
                this.child(
                    div()
                        .id((self.id.clone(), "supporting"))
                        .test_support()
                        .role(if error_active {
                            Role::Alert
                        } else {
                            Role::Status
                        })
                        .aria_label(text.clone())
                        .child(Text::new(text).role(TypeRole::BodySmall).color_override(
                            if disabled {
                                disabled_text
                            } else if error_active {
                                err_border
                            } else {
                                on_surface_variant
                            },
                        )),
                )
            })
    }
}

fn to_digit_char(ch: char) -> Option<char> {
    if ch.is_ascii_digit() {
        Some(ch)
    } else {
        let digit = (ch as u32).checked_sub('０' as u32)?;
        char::from_digit(digit, 10)
    }
}

fn extract_single_digit(key: &str, key_char: Option<&str>) -> Option<char> {
    let from_key = if key.chars().count() == 1 {
        key.chars().next().and_then(to_digit_char)
    } else {
        None
    };
    from_key.or_else(|| {
        let kc = key_char?;
        if kc.chars().count() == 1 {
            kc.chars().next().and_then(to_digit_char)
        } else {
            None
        }
    })
}

fn sanitize_digits(input: &str, max_len: usize) -> String {
    input
        .chars()
        .filter_map(to_digit_char)
        .take(max_len)
        .collect()
}

fn replace_at(current: &str, slot: usize, digit: char, length: usize) -> (String, usize) {
    let mut chars: Vec<char> = current.chars().take(length).collect();
    if slot < chars.len() {
        chars[slot] = digit;
    } else if chars.len() < length {
        chars.push(digit);
    }
    let next_slot = (slot + 1).min(length - 1);
    (chars.into_iter().collect(), next_slot)
}

fn delete_at(current: &str, slot: usize, length: usize) -> Option<(String, usize)> {
    let mut chars: Vec<char> = current.chars().take(length).collect();
    if slot >= chars.len() {
        return None;
    }
    chars.remove(slot);
    let next_slot = slot.min(chars.len()).min(length - 1);
    Some((chars.into_iter().collect(), next_slot))
}

fn apply_paste(current: &str, slot: usize, pasted: &str, length: usize) -> Option<(String, usize)> {
    let digits: Vec<char> = pasted.chars().filter_map(to_digit_char).collect();
    if digits.is_empty() {
        return None;
    }
    if digits.len() >= length {
        let next: String = digits.into_iter().take(length).collect();
        return Some((next, length - 1));
    }
    let mut chars: Vec<char> = current.chars().take(length).collect();
    let mut cursor = slot.min(chars.len()).min(length - 1);
    for digit in digits {
        if cursor < chars.len() {
            chars[cursor] = digit;
        } else if chars.len() < length {
            chars.push(digit);
        }
        if cursor + 1 < length {
            cursor += 1;
        }
    }
    let next_slot = cursor.min(chars.len()).min(length - 1);
    Some((chars.into_iter().collect(), next_slot))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_and_paste_handle_ascii_fullwidth_and_noise() {
        assert_eq!(sanitize_digits("12-34 ５６78", 6), "123456");
        assert_eq!(apply_paste("12", 1, "98", 4), Some(("198".to_owned(), 3)));
        assert_eq!(
            apply_paste("1234", 2, "654321", 4),
            Some(("6543".to_owned(), 3))
        );
        assert_eq!(apply_paste("12", 0, "abc", 4), None);
    }

    #[test]
    fn slot_replace_and_delete_preserve_cursor_bounds() {
        assert_eq!(replace_at("1234", 1, '9', 4), ("1934".to_owned(), 2));
        assert_eq!(replace_at("1234", 3, '0', 4), ("1230".to_owned(), 3));
        assert_eq!(delete_at("1234", 1, 4), Some(("134".to_owned(), 1)));
        assert_eq!(delete_at("1", 0, 4), Some((String::new(), 0)));
        assert_eq!(delete_at("12", 2, 4), None);
    }
}
