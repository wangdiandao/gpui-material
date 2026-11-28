use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled, Window,
    div, px,
};
use gpui_material::{
    Button, ButtonVariant, ColorPicker, ColorPickerEvent, ColorPickerState, MaterialTheme,
    NumberInput, NumberInputState, NumberInputStepper, OtpInput, OtpInputEvent, OtpInputState,
    OtpInputVariant, Surface, Text, TextFieldVariant, ThemeScope, current_material_theme,
    theme::{Argb, ColorRole, CornerRole},
};

use crate::story::{Story, section, stack};

#[cfg(all(test, feature = "test-support"))]
mod tests;

pub(crate) struct SpecializedInputExamples {
    story: Story,
    quantity: Entity<NumberInputState>,
    price: Entity<NumberInputState>,
    disabled_qty: Entity<NumberInputState>,
    otp_code: Entity<OtpInputState>,
    otp_masked: Entity<OtpInputState>,
    otp_error: Entity<OtpInputState>,
    otp_disabled: Entity<OtpInputState>,
    last_completed_otp: SharedString,
    seed_picker: Entity<ColorPickerState>,
    accent_picker: Entity<ColorPickerState>,
    disabled_picker: Entity<ColorPickerState>,
}

impl SpecializedInputExamples {
    pub(crate) fn new(story: Story, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let quantity = cx.new(|cx| {
            NumberInputState::new(window, cx)
                .bounds(0.0, 20.0)
                .step(1.0)
                .default_value(4.0, window, cx)
        });
        let price = cx.new(|cx| {
            NumberInputState::new(window, cx)
                .bounds(0.0, 999.95)
                .step(0.25)
                .precision(2)
                .default_value(12.50, window, cx)
        });
        let disabled_qty = cx.new(|cx| {
            let mut state = NumberInputState::new(window, cx)
                .bounds(0.0, 10.0)
                .default_value(3.0, window, cx);
            state.set_disabled(true, cx);
            state
        });

        let otp_code = cx.new(|cx| OtpInputState::new(6, window, cx));
        let otp_masked = cx.new(|cx| {
            OtpInputState::new(4, window, cx)
                .default_value("42", window, cx)
                .masked(true, window, cx)
        });
        let otp_error =
            cx.new(|cx| OtpInputState::new(4, window, cx).default_value("12", window, cx));
        let otp_disabled =
            cx.new(|cx| OtpInputState::new(4, window, cx).default_value("9081", window, cx));

        let seed_picker = cx.new(|cx| {
            ColorPickerState::new(window, cx).default_argb(
                Argb::from_rgb(0x67, 0x50, 0xA4),
                window,
                cx,
            )
        });
        let accent_picker = cx.new(|cx| {
            ColorPickerState::new(window, cx).default_argb(
                Argb::from_rgb(0x00, 0x6A, 0x6A),
                window,
                cx,
            )
        });
        let disabled_picker = cx.new(|cx| {
            ColorPickerState::new(window, cx).default_argb(
                Argb::from_rgb(0x38, 0x6A, 0x20),
                window,
                cx,
            )
        });

        cx.observe(&quantity, |_, _, cx| cx.notify()).detach();
        cx.observe(&price, |_, _, cx| cx.notify()).detach();
        cx.observe(&otp_code, |_, _, cx| cx.notify()).detach();
        cx.observe(&otp_masked, |_, _, cx| cx.notify()).detach();
        cx.observe(&otp_error, |_, _, cx| cx.notify()).detach();
        cx.observe(&seed_picker, |_, _, cx| cx.notify()).detach();
        cx.observe(&accent_picker, |_, _, cx| cx.notify()).detach();
        cx.subscribe(&otp_code, |this, _, event: &OtpInputEvent, cx| {
            if let OtpInputEvent::Complete(code) = event {
                this.last_completed_otp = code.clone();
                cx.notify();
            }
        })
        .detach();
        cx.subscribe(&seed_picker, |_, _, _: &ColorPickerEvent, cx| cx.notify())
            .detach();

        Self {
            story,
            quantity,
            price,
            disabled_qty,
            otp_code,
            otp_masked,
            otp_error,
            otp_disabled,
            last_completed_otp: SharedString::default(),
            seed_picker,
            accent_picker,
            disabled_picker,
        }
    }

    fn render_number_input(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let qty_value = self.quantity.read(cx).value(cx);
        let qty_status = match qty_value {
            Some(v) => format!("Current quantity: {v} (bounds 0..=20)"),
            None => "Enter a number between 0 and 20".to_owned(),
        };
        stack()
            .max_w(px(420.))
            .child(
                section("Filled split stepper")
                    .description("Use − / + buttons or Up / Down keys to step between 0 and 20.")
                    .child(
                        NumberInput::filled("qty-input", "Quantity", &self.quantity)
                            .supporting_text(qty_status),
                    ),
            )
            .child(
                section("Outlined trailing stepper with precision")
                    .description("Steps by 0.25 with two fixed decimal digits and a prefix/suffix.")
                    .child(
                        NumberInput::outlined("price-input", "Unit price", &self.price)
                            .stepper(NumberInputStepper::Trailing)
                            .prefix(Text::body_large("$").muted())
                            .suffix(Text::body_medium("USD").muted())
                            .supporting_text("Range: $0.00 – $999.95"),
                    ),
            )
            .child(
                section("Disabled and error")
                    .child(
                        NumberInput::outlined(
                            "disabled-qty",
                            "Locked allocation",
                            &self.disabled_qty,
                        )
                        .disabled(true)
                        .supporting_text("Stepping and editing are disabled."),
                    )
                    .child(
                        NumberInput::filled("error-qty", "Seats", &self.quantity)
                            .variant(TextFieldVariant::Filled)
                            .error("Must be confirmed before checkout"),
                    ),
            )
    }

    fn render_otp_input(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let code_status = if self.last_completed_otp.is_empty() {
            format!("Entered: {}", self.otp_code.read(cx).value(cx))
        } else {
            format!("Last completed code: {}", self.last_completed_otp)
        };
        stack()
            .max_w(px(480.))
            .child(
                section("Six-digit verification code")
                    .description("Supports typing, paste (Ctrl/Cmd+V), Backspace, Delete, and arrow navigation.")
                    .child(
                        OtpInput::new("otp-verify", "Verification code", &self.otp_code)
                            .separator_after(3)
                            .supporting_text(code_status),
                    ),
            )
            .child(
                section("Filled masked PIN")
                    .description("Masks entered digits as bullets while keeping navigation active.")
                    .child(
                        OtpInput::new("otp-pin", "Security PIN", &self.otp_masked)
                            .variant(OtpInputVariant::Filled)
                            .supporting_text("4-digit masked PIN"),
                    ),
            )
            .child(
                section("Error and disabled")
                    .child(
                        OtpInput::new("otp-error", "Expired code", &self.otp_error)
                            .error("Code expired. Request a new one."),
                    )
                    .child(
                        OtpInput::new("otp-disabled", "Verified code", &self.otp_disabled)
                            .disabled(true),
                    ),
            )
    }

    fn render_color_picker(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let appearance = current_material_theme(window, cx).colors.appearance();
        let seed_argb = self
            .seed_picker
            .read(cx)
            .displayed_argb(cx)
            .unwrap_or(Argb::from_rgb(0x67, 0x50, 0xA4));
        let generated_theme = MaterialTheme::from_seed(seed_argb, appearance, 0.0);

        stack()
            .max_w(px(640.))
            .child(
                section("Inline seed color picker + dynamic-color preview")
                    .description(
                        "Pick a swatch, drag HSLA sliders, or type a hex value to regenerate a Material 3 theme live.",
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_start()
                            .gap_4()
                            .child(
                                ColorPicker::inline("seed-picker", "Seed color", &self.seed_picker)
                                    .supporting_text("Feeds Argb straight into MaterialTheme::from_seed"),
                            )
                            .child(
                                ThemeScope::new(
                                    Surface::new(ColorRole::PrimaryContainer)
                                        .shape(CornerRole::Large.shape())
                                        .p_4()
                                        .w(px(260.))
                                        .flex()
                                        .flex_col()
                                        .gap_2()
                                        .child(
                                            Text::title_medium("Dynamic scheme preview")
                                                .emphasized(true),
                                        )
                                        .child(
                                            Text::body_small(format!(
                                                "Seed: {}",
                                                self.seed_picker.read(cx).hex(cx)
                                            ))
                                            .wrap(),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .gap_2()
                                                .child(Button::filled("preview-action", "Primary"))
                                                .child(
                                                    Button::new("preview-tonal", "Tonal")
                                                        .variant(ButtonVariant::FilledTonal),
                                                ),
                                        ),
                                )
                                .theme(generated_theme),
                            ),
                    ),
            )
            .child(
                section("Popover trigger with alpha channel and disabled state")
                    .description("Click the trigger to open a dismissible Material popover.")
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_start()
                            .gap_4()
                            .child(
                                ColorPicker::new("accent-picker", "Accent overlay", &self.accent_picker)
                                    .show_alpha(true)
                                    .supporting_text("Supports alpha slider and #RRGGBBAA"),
                            )
                            .child(
                                ColorPicker::new("disabled-picker", "Locked color", &self.disabled_picker)
                                    .disabled(true),
                            ),
                    ),
            )
    }
}

impl Render for SpecializedInputExamples {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        match self.story {
            Story::NumberInput => self.render_number_input(cx).into_any_element(),
            Story::OtpInput => self.render_otp_input(cx).into_any_element(),
            Story::ColorPicker => self.render_color_picker(window, cx).into_any_element(),
            _ => div().into_any_element(),
        }
    }
}
