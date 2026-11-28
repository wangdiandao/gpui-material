use std::rc::Rc;

use gpui::{
    Anchor, App, AppContext, Context, ElementId, Empty, Entity, EventEmitter, FocusHandle,
    Focusable, Hsla, InteractiveElement, IntoElement, ParentElement, Render, RenderOnce, Rgba,
    Role, SharedString, StatefulInteractiveElement, Styled, Subscription, Window, div,
    prelude::FluentBuilder, px,
};
use gpui_base::{
    Button as BaseButton, ColorPicker as BaseColorPicker, ColorPickerEvent as BaseColorPickerEvent,
    ColorPickerState as BaseColorPickerState, ColorSwatch as BaseColorSwatch, Popover,
    TestSupportExt,
};
use gpui_material_components::{Icon, IconName, Slider, Text, TextField};
use gpui_material_foundation::{
    Surface, SurfaceBorder, color, current_material_theme, logical_anchor,
};
use gpui_material_theme::{Argb, ColorRole, CornerRole, Density, Direction, Elevation, TypeRole};

/// Converts a GPUI [`Hsla`] color into a packed Material [`Argb`] value.
pub fn hsla_to_argb(value: Hsla) -> Argb {
    let rgba = Rgba::from(value);
    let channel = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    Argb::from_rgba(
        channel(rgba.r),
        channel(rgba.g),
        channel(rgba.b),
        channel(rgba.a),
    )
}

/// Converts a packed Material [`Argb`] value into a GPUI [`Hsla`] color.
pub fn argb_to_hsla(argb: Argb) -> Hsla {
    let [a, r, g, b] = argb.channels();
    Hsla::from(Rgba {
        r: f32::from(r) / 255.0,
        g: f32::from(g) / 255.0,
        b: f32::from(b) / 255.0,
        a: f32::from(a) / 255.0,
    })
}

/// Formats an [`Argb`] as `#RRGGBB`, or `#RRGGBBAA` when `include_alpha` is `true`
/// and the color is translucent.
pub fn format_hex_argb(argb: Argb, include_alpha: bool) -> SharedString {
    let [a, r, g, b] = argb.channels();
    if include_alpha && a < 255 {
        format!("#{r:02X}{g:02X}{b:02X}{a:02X}").into()
    } else {
        format!("#{r:02X}{g:02X}{b:02X}").into()
    }
}

/// Parses `#RGB`, `#RGBA`, `#RRGGBB`, or `#RRGGBBAA` (with or without leading `#`)
/// into a packed [`Argb`].
pub fn parse_hex_argb(input: &str) -> Option<Argb> {
    let raw = input.trim().strip_prefix('#').unwrap_or(input.trim());
    if !raw.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let (width, has_alpha) = match raw.len() {
        3 => (1, false),
        4 => (1, true),
        6 => (2, false),
        8 => (2, true),
        _ => return None,
    };
    let read = |ix: usize| -> Option<u8> {
        let slice = &raw[ix * width..(ix + 1) * width];
        let val = u8::from_str_radix(slice, 16).ok()?;
        Some(if width == 1 { val * 0x11 } else { val })
    };
    let r = read(0)?;
    let g = read(1)?;
    let b = read(2)?;
    let a = if has_alpha { read(3)? } else { 255 };
    Some(Argb::from_rgba(r, g, b, a))
}

/// Events emitted by [`ColorPickerState`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColorPickerEvent {
    /// The committed color changed.
    Change(Option<Hsla>),
}

impl ColorPickerEvent {
    /// Returns the committed color as [`Hsla`].
    pub fn hsla(&self) -> Option<Hsla> {
        match *self {
            Self::Change(color) => color,
        }
    }

    /// Returns the committed color as a Material [`Argb`], ready for `MaterialTheme::from_seed`.
    pub fn argb(&self) -> Option<Argb> {
        self.hsla().map(hsla_to_argb)
    }
}

/// A named preset swatch displayed in [`ColorPicker`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColorPickerPreset {
    /// Accessible name for this swatch.
    pub label: SharedString,
    /// Packed sRGB swatch color.
    pub color: Argb,
}

impl ColorPickerPreset {
    /// Creates a named preset swatch.
    pub fn new(label: impl Into<SharedString>, color: Argb) -> Self {
        Self {
            label: label.into(),
            color,
        }
    }

    /// Returns a curated Material 3 palette of seed-friendly color presets.
    pub fn default_presets() -> Vec<Self> {
        vec![
            Self::new("Baseline purple", Argb::from_rgb(0x67, 0x50, 0xA4)),
            Self::new("Indigo", Argb::from_rgb(0x3F, 0x51, 0xB5)),
            Self::new("Ocean blue", Argb::from_rgb(0x00, 0x61, 0xA4)),
            Self::new("Teal", Argb::from_rgb(0x00, 0x6A, 0x6A)),
            Self::new("Forest green", Argb::from_rgb(0x38, 0x6A, 0x20)),
            Self::new("Lime", Argb::from_rgb(0x61, 0x62, 0x00)),
            Self::new("Amber", Argb::from_rgb(0x82, 0x55, 0x00)),
            Self::new("Tangerine", Argb::from_rgb(0x9C, 0x41, 0x16)),
            Self::new("Crimson", Argb::from_rgb(0xB3, 0x26, 0x1E)),
            Self::new("Rose", Argb::from_rgb(0x98, 0x40, 0x61)),
            Self::new("Plum", Argb::from_rgb(0x7D, 0x52, 0x60)),
            Self::new("Slate", Argb::from_rgb(0x52, 0x60, 0x70)),
        ]
    }
}

/// Per-instance color overrides for [`ColorPicker`]. Omitted fields resolve from the window theme.
#[derive(Clone, Copy, Debug, Default)]
pub struct ColorPickerColors {
    /// Panel or trigger container background color.
    pub container: Option<Hsla>,
    /// Container border color.
    pub border: Option<Hsla>,
    /// Primary label color.
    pub label: Option<Hsla>,
    /// Supporting caption color.
    pub supporting: Option<Hsla>,
    /// Selection ring color around the active preset swatch.
    pub selected_ring: Option<Hsla>,
}

/// Retained state for a Material 3 [`ColorPicker`].
///
/// Keeps swatches, HSLA sliders, the hex text field, transient hover previews, and the
/// popover open state synchronized. Use [`Self::argb`] to feed the committed color directly
/// into `MaterialTheme::from_seed(...)` when `dynamic-color` is enabled.
pub struct ColorPickerState {
    base: Entity<BaseColorPickerState>,
    _subscriptions: Vec<Subscription>,
}

impl ColorPickerState {
    /// Creates an empty, closed color picker state.
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let base = cx.new(|cx| BaseColorPickerState::new(window, cx));
        let subscriptions = vec![
            cx.observe(&base, |_, _, cx| cx.notify()),
            cx.subscribe_in(
                &base,
                window,
                |_, _, event: &BaseColorPickerEvent, _, cx| match *event {
                    BaseColorPickerEvent::Change(color) => {
                        cx.emit(ColorPickerEvent::Change(color));
                        cx.notify();
                    }
                },
            ),
        ];
        Self {
            base,
            _subscriptions: subscriptions,
        }
    }

    /// Sets the initial color from an [`Argb`] during state construction.
    pub fn default_argb(self, argb: Argb, window: &mut Window, cx: &mut Context<Self>) -> Self {
        self.default_color(argb_to_hsla(argb), window, cx)
    }

    /// Sets the initial color from an [`Hsla`] during state construction.
    pub fn default_color(self, value: Hsla, window: &mut Window, cx: &mut Context<Self>) -> Self {
        self.base
            .update(cx, |base, cx| base.set_value(value, window, cx));
        self
    }

    /// Returns the committed color as [`Hsla`].
    pub fn value(&self, cx: &App) -> Option<Hsla> {
        self.base.read(cx).value()
    }

    /// Returns the committed color as a Material [`Argb`].
    pub fn argb(&self, cx: &App) -> Option<Argb> {
        self.value(cx).map(hsla_to_argb)
    }

    /// Returns the transient hovered or typed preview color as [`Hsla`].
    pub fn preview(&self, cx: &App) -> Option<Hsla> {
        self.base.read(cx).preview()
    }

    /// Returns the transient preview color as [`Argb`].
    pub fn preview_argb(&self, cx: &App) -> Option<Argb> {
        self.preview(cx).map(hsla_to_argb)
    }

    /// Returns the preview color, falling back to the committed color.
    pub fn displayed_color(&self, cx: &App) -> Option<Hsla> {
        self.base.read(cx).displayed_color()
    }

    /// Returns the displayed color as [`Argb`].
    pub fn displayed_argb(&self, cx: &App) -> Option<Argb> {
        self.displayed_color(cx).map(hsla_to_argb)
    }

    /// Returns the formatted hex string (`#RRGGBB` or `#RRGGBBAA`) of the displayed color,
    /// or an empty string if no color is set.
    pub fn hex(&self, cx: &App) -> SharedString {
        self.displayed_argb(cx)
            .map(|argb| format_hex_argb(argb, true))
            .unwrap_or_default()
    }

    /// Replaces the committed color without emitting [`ColorPickerEvent::Change`].
    pub fn set_color(&mut self, value: Hsla, window: &mut Window, cx: &mut Context<Self>) {
        self.base
            .update(cx, |base, cx| base.set_value(value, window, cx));
        cx.notify();
    }

    /// Replaces the committed color from an [`Argb`] without emitting [`ColorPickerEvent::Change`].
    pub fn set_argb(&mut self, argb: Argb, window: &mut Window, cx: &mut Context<Self>) {
        self.set_color(argb_to_hsla(argb), window, cx);
    }

    /// Commits a color, closes the popover if open, and emits [`ColorPickerEvent::Change`].
    pub fn select_color(&mut self, value: Hsla, window: &mut Window, cx: &mut Context<Self>) {
        self.base
            .update(cx, |base, cx| base.select_color(value, window, cx));
        cx.notify();
    }

    /// Commits an [`Argb`] color, closes the popover if open, and emits [`ColorPickerEvent::Change`].
    pub fn select_argb(&mut self, argb: Argb, window: &mut Window, cx: &mut Context<Self>) {
        self.select_color(argb_to_hsla(argb), window, cx);
    }

    /// Clears the committed color without emitting [`ColorPickerEvent::Change`].
    pub fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.base
            .update(cx, |base, cx| base.clear_value(window, cx));
        cx.notify();
    }

    /// Returns whether the popover panel is currently open.
    pub fn is_open(&self, cx: &App) -> bool {
        self.base.read(cx).is_open()
    }

    /// Sets whether the popover panel is open.
    pub fn set_open(&mut self, open: bool, cx: &mut Context<Self>) {
        self.base.update(cx, |base, cx| base.set_open(open, cx));
        cx.notify();
    }

    /// Toggles the popover panel open state.
    pub fn toggle_open(&mut self, cx: &mut Context<Self>) {
        self.base.update(cx, |base, cx| base.toggle_open(cx));
        cx.notify();
    }

    /// Focuses the picker trigger.
    pub fn focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_handle(cx).focus(window, cx);
    }
}

impl Focusable for ColorPickerState {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.base.read(cx).focus_handle(cx)
    }
}

impl EventEmitter<ColorPickerEvent> for ColorPickerState {}

impl Render for ColorPickerState {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

type ColorChangeCallback = Rc<dyn Fn(Option<Hsla>, &mut Window, &mut App)>;
type ArgbChangeCallback = Rc<dyn Fn(Option<Argb>, &mut Window, &mut App)>;

/// Material 3 color picker with swatch presets, HSLA sliders, live hex preview/editor,
/// and seamless [`Argb`] / [`Hsla`] conversion.
#[derive(IntoElement)]
pub struct ColorPicker {
    id: ElementId,
    label: SharedString,
    state: Entity<ColorPickerState>,
    inline: bool,
    presets: Vec<ColorPickerPreset>,
    show_presets: bool,
    show_sliders: bool,
    show_alpha: bool,
    show_hex_input: bool,
    disabled: bool,
    supporting: Option<SharedString>,
    colors: ColorPickerColors,
    on_change: Option<ColorChangeCallback>,
    on_argb_change: Option<ArgbChangeCallback>,
}

impl ColorPicker {
    /// Creates a Material 3 color picker trigger that opens a dismissible popover panel.
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<ColorPickerState>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            state: state.clone(),
            inline: false,
            presets: ColorPickerPreset::default_presets(),
            show_presets: true,
            show_sliders: true,
            show_alpha: false,
            show_hex_input: true,
            disabled: false,
            supporting: None,
            colors: ColorPickerColors::default(),
            on_change: None,
            on_argb_change: None,
        }
    }

    /// Creates an always-visible inline Material 3 color picker panel.
    pub fn inline(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<ColorPickerState>,
    ) -> Self {
        let mut picker = Self::new(id, label, state);
        picker.inline = true;
        picker
    }

    /// Replaces the preset swatch palette with custom named presets.
    pub fn presets(mut self, presets: impl IntoIterator<Item = ColorPickerPreset>) -> Self {
        self.presets = presets.into_iter().collect();
        self
    }

    /// Replaces the preset swatch palette with [`Argb`] colors labeled by their hex code.
    pub fn swatches(mut self, swatches: impl IntoIterator<Item = Argb>) -> Self {
        self.presets = swatches
            .into_iter()
            .map(|argb| ColorPickerPreset::new(format_hex_argb(argb, true), argb))
            .collect();
        self
    }

    /// Controls whether preset swatches are shown (defaults to `true`).
    pub fn show_presets(mut self, show: bool) -> Self {
        self.show_presets = show;
        self
    }

    /// Controls whether Hue, Saturation, and Lightness sliders are shown (defaults to `true`).
    pub fn show_sliders(mut self, show: bool) -> Self {
        self.show_sliders = show;
        self
    }

    /// Controls whether the Alpha channel slider is shown (defaults to `false`).
    pub fn show_alpha(mut self, show: bool) -> Self {
        self.show_alpha = show;
        self
    }

    /// Controls whether the editable `#RRGGBB` hex input field is shown (defaults to `true`).
    pub fn show_hex_input(mut self, show: bool) -> Self {
        self.show_hex_input = show;
        self
    }

    /// Disables all picker interaction.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Displays supporting text beneath the trigger or panel.
    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting = Some(text.into());
        self
    }

    /// Supplies per-instance color overrides.
    pub fn colors(mut self, colors: ColorPickerColors) -> Self {
        self.colors = colors;
        self
    }

    /// Invoked when a preset swatch or color commitment updates the color as [`Hsla`].
    pub fn on_change(
        mut self,
        handler: impl Fn(Option<Hsla>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// Invoked when a preset swatch or color commitment updates the color as [`Argb`].
    pub fn on_argb_change(
        mut self,
        handler: impl Fn(Option<Argb>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_argb_change = Some(Rc::new(handler));
        self
    }
}

#[derive(Clone)]
struct PanelConfig {
    id: ElementId,
    label: SharedString,
    state: Entity<ColorPickerState>,
    base: Entity<BaseColorPickerState>,
    presets: Vec<ColorPickerPreset>,
    show_presets: bool,
    show_sliders: bool,
    show_alpha: bool,
    show_hex_input: bool,
    disabled: bool,
    colors: ColorPickerColors,
    inline: bool,
    on_change: Option<ColorChangeCallback>,
    on_argb_change: Option<ArgbChangeCallback>,
}

fn render_panel(config: &PanelConfig, window: &mut Window, cx: &mut App) -> gpui::AnyElement {
    let theme = current_material_theme(window, cx);
    let rtl = theme.layout.direction == Direction::Rtl;
    let swatch_side = match theme.layout.density {
        Density::Compact => px(26.),
        Density::Standard => px(30.),
        Density::Comfortable => px(34.),
    };
    let surface_role = if config.inline {
        ColorRole::SurfaceContainerLow
    } else {
        ColorRole::SurfaceContainerHigh
    };
    let outline_variant = config
        .colors
        .border
        .unwrap_or_else(|| color(theme.colors.color(ColorRole::OutlineVariant)));
    let selected_ring = config
        .colors
        .selected_ring
        .unwrap_or_else(|| color(theme.colors.color(ColorRole::Primary)));
    let label_color = config
        .colors
        .label
        .unwrap_or_else(|| color(theme.colors.color(ColorRole::OnSurface)));
    let muted_color = config
        .colors
        .supporting
        .unwrap_or_else(|| color(theme.colors.color(ColorRole::OnSurfaceVariant)));
    let disabled_color = color(theme.colors.color(ColorRole::OnSurface)).opacity(0.38);

    let displayed = config.state.read(cx).displayed_color(cx);
    let displayed_argb = displayed.map(hsla_to_argb);
    let committed_argb = config.state.read(cx).argb(cx);
    let hex_label = displayed_argb
        .map(|argb| format_hex_argb(argb, config.show_alpha))
        .unwrap_or_else(|| SharedString::new_static("No color"));
    let preview_swatch_color =
        displayed.unwrap_or_else(|| color(theme.colors.color(ColorRole::SurfaceContainerHighest)));

    let sliders = config.base.read(cx).sliders().clone();
    let hex_input = config.base.read(cx).hex_input().clone();

    let mut panel = Surface::new(surface_role)
        .shape(CornerRole::Large.shape())
        .border(SurfaceBorder {
            width: px(1.),
            color: outline_variant,
        })
        .when(!config.inline, |s| s.shadow_elevation(Elevation::Level3))
        .when_some(config.colors.container, |s, bg| s.container_color(bg))
        .w(px(304.))
        .max_w_full()
        .p_4()
        .flex()
        .flex_col()
        .gap_3();

    // Header row: live color preview chip + label + live hex readout
    let header = div()
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .when(rtl, Styled::flex_row_reverse)
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                .min_w_0()
                .when(rtl, Styled::flex_row_reverse)
                .child(
                    div()
                        .id((config.id.clone(), "preview"))
                        .test_support()
                        .w(px(36.))
                        .h(px(36.))
                        .flex_none()
                        .rounded_full()
                        .bg(preview_swatch_color)
                        .border_1()
                        .border_color(outline_variant),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .min_w_0()
                        .when(rtl, Styled::items_end)
                        .child(
                            Text::new(config.label.clone())
                                .role(TypeRole::TitleSmall)
                                .color_override(if config.disabled {
                                    disabled_color
                                } else {
                                    label_color
                                }),
                        )
                        .child(
                            Text::new(hex_label)
                                .id((config.id.clone(), "hex-preview"))
                                .role(TypeRole::BodySmall)
                                .color_override(if config.disabled {
                                    disabled_color
                                } else {
                                    muted_color
                                }),
                        ),
                ),
        );
    panel = panel.child(header);

    // Swatch presets grid
    if config.show_presets && !config.presets.is_empty() {
        let mut grid = div()
            .id((config.id.clone(), "presets"))
            .test_support()
            .role(Role::RadioGroup)
            .aria_label("Color presets")
            .flex()
            .flex_wrap()
            .gap_2();

        for (index, preset) in config.presets.iter().enumerate() {
            let swatch_hsla = argb_to_hsla(preset.color);
            let is_selected = committed_argb == Some(preset.color);
            let check_tint = contrasting_ink(preset.color);
            let base_hover = config.base.clone();
            let base_click = config.base.clone();
            let on_change = config.on_change.clone();
            let on_argb_change = config.on_argb_change.clone();
            let inline = config.inline;

            let swatch = div()
                .id((config.id.clone(), format!("swatch-{index}")))
                .test_support()
                .child(
                    BaseColorSwatch::new(
                        (config.id.clone(), format!("swatch-control-{index}")),
                        swatch_hsla,
                    )
                    .selected(is_selected)
                    .disabled(config.disabled)
                    .accessibility_label(preset.label.clone())
                    .w(swatch_side)
                    .h(swatch_side)
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(swatch_hsla)
                    .border(if is_selected { px(2.) } else { px(1.) })
                    .border_color(if is_selected {
                        selected_ring
                    } else {
                        outline_variant
                    })
                    .when(!config.disabled, |this| this.cursor_pointer())
                    .on_hover(move |color, entered, window, cx| {
                        base_hover.update(cx, |state, cx| {
                            if entered {
                                state.preview_color(color, window, cx);
                            } else {
                                state.clear_preview(window, cx);
                            }
                        });
                    })
                    .on_click(move |color, _, window, cx| {
                        base_click.update(cx, |state, cx| {
                            if inline {
                                state.update_color(color, window, cx);
                            } else {
                                state.select_color(color, window, cx);
                            }
                        });
                        if let Some(cb) = on_change.as_ref() {
                            cb(Some(color), window, cx);
                        }
                        if let Some(cb) = on_argb_change.as_ref() {
                            cb(Some(hsla_to_argb(color)), window, cx);
                        }
                    })
                    .when(is_selected, |this| {
                        this.child(
                            Icon::new(IconName::Check)
                                .size(px(16.))
                                .color_override(check_tint),
                        )
                    }),
                );
            grid = grid.child(swatch);
        }
        panel = panel.child(grid);
    }

    // HSLA Sliders
    if config.show_sliders {
        let mut slider_rows = div().w_full().flex().flex_col().gap_2();
        let channels: [(
            &'static str,
            &'static str,
            &Entity<gpui_material_components::SliderState>,
            bool,
        ); 4] = [
            ("hue", "Hue", sliders.hue(), true),
            ("saturation", "Saturation", sliders.saturation(), true),
            ("lightness", "Lightness", sliders.lightness(), true),
            ("alpha", "Alpha", sliders.alpha(), config.show_alpha),
        ];

        for (key, name, slider_state, visible) in channels {
            if !visible {
                continue;
            }
            let pct =
                (slider_state.read(cx).value().start().clamp(0.0, 1.0) * 100.0).round() as u32;
            let value_text: SharedString = if key == "hue" {
                let deg =
                    (slider_state.read(cx).value().start().clamp(0.0, 1.0) * 360.0).round() as u32;
                format!("{deg}°").into()
            } else {
                format!("{pct}%").into()
            };
            let row = div()
                .w_full()
                .flex()
                .flex_col()
                .gap_0p5()
                .child(
                    div()
                        .w_full()
                        .flex()
                        .items_center()
                        .justify_between()
                        .when(rtl, Styled::flex_row_reverse)
                        .child(Text::new(name).role(TypeRole::LabelMedium).color_override(
                            if config.disabled {
                                disabled_color
                            } else {
                                muted_color
                            },
                        ))
                        .child(
                            Text::new(value_text)
                                .role(TypeRole::LabelSmall)
                                .color_override(if config.disabled {
                                    disabled_color
                                } else {
                                    muted_color
                                }),
                        ),
                )
                .child(
                    Slider::new((config.id.clone(), key), name, slider_state)
                        .disabled(config.disabled)
                        .value_indicator(false),
                );
            slider_rows = slider_rows.child(row);
        }
        panel = panel.child(slider_rows);
    }

    // Hex TextField
    if config.show_hex_input {
        panel = panel.child(
            TextField::outlined((config.id.clone(), "hex"), "Hex", &hex_input)
                .placeholder_content("#6750A4", Text::new("#6750A4"))
                .disabled(config.disabled),
        );
    }

    panel.into_any_element()
}

impl RenderOnce for ColorPicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let base = self.state.read(cx).base.clone();
        base.update(cx, |state, cx| state.sync_pending_value(window, cx));

        let theme = current_material_theme(window, cx);
        let direction = theme.layout.direction;
        let rtl = direction == Direction::Rtl;
        let outline_variant = self
            .colors
            .border
            .unwrap_or_else(|| color(theme.colors.color(ColorRole::OutlineVariant)));
        let label_color = self
            .colors
            .label
            .unwrap_or_else(|| color(theme.colors.color(ColorRole::OnSurface)));
        let muted_color = self
            .colors
            .supporting
            .unwrap_or_else(|| color(theme.colors.color(ColorRole::OnSurfaceVariant)));
        let disabled_color = color(theme.colors.color(ColorRole::OnSurface)).opacity(0.38);

        let panel_config = PanelConfig {
            id: self.id.clone(),
            label: self.label.clone(),
            state: self.state.clone(),
            base: base.clone(),
            presets: self.presets,
            show_presets: self.show_presets,
            show_sliders: self.show_sliders,
            show_alpha: self.show_alpha,
            show_hex_input: self.show_hex_input,
            disabled: self.disabled,
            colors: self.colors,
            inline: self.inline,
            on_change: self.on_change,
            on_argb_change: self.on_argb_change,
        };

        let content = if self.inline {
            div()
                .id(self.id.clone())
                .test_support()
                .role(Role::Group)
                .aria_label(self.label.clone())
                .child(render_panel(&panel_config, window, cx))
                .into_any_element()
        } else {
            let open = self.state.read(cx).is_open(cx);
            let focus_handle = self.state.read(cx).focus_handle(cx);
            let displayed = self.state.read(cx).displayed_color(cx);
            let hex_text = displayed
                .map(|c| format_hex_argb(hsla_to_argb(c), self.show_alpha))
                .unwrap_or_else(|| SharedString::new_static("Select color"));
            let swatch_fill = displayed
                .unwrap_or_else(|| color(theme.colors.color(ColorRole::SurfaceContainerHighest)));
            let trigger_bg = self.colors.container.unwrap_or_else(|| {
                if self.disabled {
                    color(theme.colors.color(ColorRole::OnSurface)).opacity(0.04)
                } else {
                    color(theme.colors.color(ColorRole::SurfaceContainerLow))
                }
            });
            let trigger_h = match theme.layout.density {
                Density::Compact => px(40.),
                Density::Standard => px(48.),
                Density::Comfortable => px(52.),
            };
            let trigger_radius =
                px(CornerRole::Medium
                    .shape()
                    .radii(200., f32::from(trigger_h), direction)[0]);

            let state_popover = self.state.clone();
            let state_root = self.state.clone();
            let trigger_id = (self.id.clone(), "trigger");
            let label_for_trigger = self.label.clone();
            let disabled = self.disabled;

            let trigger = BaseButton::new(trigger_id)
                .disabled(disabled)
                .accessibility_label(label_for_trigger.clone())
                .min_h(trigger_h)
                .px_3()
                .py_1p5()
                .rounded(trigger_radius)
                .bg(trigger_bg)
                .border_1()
                .border_color(if open && !disabled {
                    color(theme.colors.color(ColorRole::Primary))
                } else {
                    outline_variant
                })
                .flex()
                .items_center()
                .gap_2p5()
                .when(rtl, Styled::flex_row_reverse)
                .when(!disabled, Styled::cursor_pointer)
                .child(
                    div()
                        .w(px(24.))
                        .h(px(24.))
                        .flex_none()
                        .rounded_full()
                        .bg(swatch_fill)
                        .border_1()
                        .border_color(outline_variant),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .when(rtl, Styled::items_end)
                        .child(
                            Text::new(label_for_trigger)
                                .role(TypeRole::LabelMedium)
                                .color_override(if disabled {
                                    disabled_color
                                } else {
                                    label_color
                                }),
                        )
                        .child(
                            Text::new(hex_text)
                                .role(TypeRole::BodySmall)
                                .color_override(if disabled {
                                    disabled_color
                                } else {
                                    muted_color
                                }),
                        ),
                )
                .child(
                    Icon::new(IconName::ArrowDropDown)
                        .size(px(20.))
                        .color_override(if disabled {
                            disabled_color
                        } else {
                            muted_color
                        }),
                );

            let popover = Popover::new((self.id.clone(), "popover"))
                .anchor(logical_anchor(Anchor::BottomLeft, direction))
                .offset(px(6.))
                .open(open && !disabled)
                .on_open_change(move |next_open, _, cx| {
                    if !disabled {
                        state_popover.update(cx, |state, cx| state.set_open(*next_open, cx));
                    }
                })
                .trigger_with(move |_, _, _| trigger.into_any_element())
                .content(move |_, window, cx| render_panel(&panel_config, window, cx));

            div()
                .id(self.id.clone())
                .test_support()
                .child(
                    BaseColorPicker::new((self.id.clone(), "base"))
                        .open(open && !disabled)
                        .disabled(disabled)
                        .track_focus(&focus_handle)
                        .accessibility_label(self.label.clone())
                        .on_open_change(move |next_open, _, cx| {
                            state_root.update(cx, |state, cx| state.set_open(next_open, cx));
                        })
                        .child(popover),
                )
                .into_any_element()
        };

        div()
            .flex()
            .flex_col()
            .gap_1()
            .when(rtl, Styled::items_end)
            .child(content)
            .when_some(self.supporting, |this, text| {
                this.child(Text::new(text).role(TypeRole::BodySmall).color_override(
                    if self.disabled {
                        disabled_color
                    } else {
                        muted_color
                    },
                ))
            })
    }
}

fn contrasting_ink(argb: Argb) -> Hsla {
    let [_, r, g, b] = argb.channels();
    let linear = |c: u8| {
        let s = f32::from(c) / 255.0;
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    };
    let luminance = 0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b);
    if luminance > 0.42 {
        gpui::black()
    } else {
        gpui::white()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argb_hsla_and_hex_round_trip() {
        let samples = [
            Argb::from_rgb(0x67, 0x50, 0xA4),
            Argb::from_rgb(0x00, 0x00, 0x00),
            Argb::from_rgb(0xFF, 0xFF, 0xFF),
            Argb::from_rgb(0x00, 0x6A, 0x6A),
            Argb::from_rgba(0xB3, 0x26, 0x1E, 0x80),
        ];
        for sample in samples {
            let hsla = argb_to_hsla(sample);
            assert_eq!(hsla_to_argb(hsla), sample);
            let hex = format_hex_argb(sample, true);
            assert_eq!(parse_hex_argb(hex.as_ref()), Some(sample));
        }

        assert_eq!(
            parse_hex_argb("#fff"),
            Some(Argb::from_rgb(0xFF, 0xFF, 0xFF))
        );
        assert_eq!(
            parse_hex_argb("6750a4"),
            Some(Argb::from_rgb(0x67, 0x50, 0xA4))
        );
        assert_eq!(parse_hex_argb("#12345"), None);
        assert_eq!(parse_hex_argb("#GGGGGG"), None);
    }

    #[test]
    fn default_presets_are_non_empty_and_unique() {
        let presets = ColorPickerPreset::default_presets();
        assert!(presets.len() >= 8);
        assert_eq!(presets[0].color, Argb::from_rgb(0x67, 0x50, 0xA4));
    }
}
