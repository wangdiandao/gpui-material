use gpui::{
    AccessibleAction, App, Bounds, ClickEvent, ElementId, Hsla, InteractiveElement, IntoElement,
    ParentElement, PathBuilder, Pixels, RenderOnce, Role, SharedString, StatefulInteractiveElement,
    Styled, Window, WindowControlArea, canvas, div, point, prelude::FluentBuilder, px,
    transparent_black,
};
use gpui_base::TestSupportExt as _;
use gpui_material_foundation::{color, current_material_theme};
use gpui_material_theme::{ColorRole, Direction};

use super::platform::{
    ChromePlatform, CloseHandler, activate_close, activate_maximize_or_restore, activate_minimize,
    sanitize_positive, should_show_caption_buttons,
};

/// Optional color overrides for [`CaptionButtons`]. Omitted fields resolve from
/// the current window Material theme.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CaptionButtonColors {
    pub active_foreground: Option<Hsla>,
    pub inactive_foreground: Option<Hsla>,
    pub hover_background: Option<Hsla>,
    pub hover_foreground: Option<Hsla>,
    pub pressed_background: Option<Hsla>,
    pub pressed_foreground: Option<Hsla>,
    pub close_hover_background: Option<Hsla>,
    pub close_hover_foreground: Option<Hsla>,
    pub close_pressed_background: Option<Hsla>,
    pub close_pressed_foreground: Option<Hsla>,
}

/// Localized accessible names for [`CaptionButtons`] and [`super::TitleBar`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptionButtonLabels {
    pub minimize: SharedString,
    pub maximize: SharedString,
    pub restore: SharedString,
    pub close: SharedString,
}

impl Default for CaptionButtonLabels {
    fn default() -> Self {
        Self {
            minimize: SharedString::new_static("Minimize"),
            maximize: SharedString::new_static("Maximize"),
            restore: SharedString::new_static("Restore"),
            close: SharedString::new_static("Close"),
        }
    }
}

impl CaptionButtonLabels {
    fn for_control(&self, control: CaptionButton) -> SharedString {
        match control {
            CaptionButton::Minimize => self.minimize.clone(),
            CaptionButton::Maximize => self.maximize.clone(),
            CaptionButton::Restore => self.restore.clone(),
            CaptionButton::Close => self.close.clone(),
        }
    }
}

/// Minimize, maximize or restore, and close buttons for a client-drawn title bar.
///
/// Use this directly in a custom title bar layout for a window opened with
/// [`super::title_bar_window_options`]. Buttons render crisp vector glyphs that
/// do not depend on platform-specific icon fonts.
#[derive(IntoElement)]
pub struct CaptionButtons {
    id: ElementId,
    colors: CaptionButtonColors,
    labels: CaptionButtonLabels,
    button_width: Option<Pixels>,
    platform: ChromePlatform,
    show: Option<bool>,
    on_close: Option<CloseHandler>,
}

impl CaptionButtons {
    /// Creates caption buttons with a stable identity in their parent layout.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            colors: CaptionButtonColors::default(),
            labels: CaptionButtonLabels::default(),
            button_width: None,
            platform: ChromePlatform::current(),
            show: None,
            on_close: None,
        }
    }

    /// Overrides the button palette. Omitted fields keep the Material theme defaults.
    pub fn colors(mut self, colors: CaptionButtonColors) -> Self {
        self.colors = colors;
        self
    }

    /// Overrides the localized accessible button names (Minimize, Maximize, Restore, Close).
    pub fn labels(mut self, labels: CaptionButtonLabels) -> Self {
        self.labels = labels;
        self
    }

    /// Sets the logical width of each caption button. Defaults to 46 px on
    /// Windows and 40 px on Linux.
    pub fn button_width(mut self, width: Pixels) -> Self {
        self.button_width = Some(width);
        self
    }

    /// Overrides the target chrome platform rules for rendering and activation.
    pub fn platform(mut self, platform: ChromePlatform) -> Self {
        self.platform = platform;
        self
    }

    /// Explicitly forces caption buttons to be shown or hidden (still hidden in
    /// fullscreen). When omitted, buttons follow platform defaults: shown on
    /// Windows, hidden on macOS, and shown on Linux under client-side decorations.
    pub fn show(mut self, show: bool) -> Self {
        self.show = Some(show);
        self
    }

    /// Installs a custom close callback invoked when the Close button activates.
    pub fn on_close(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(std::rc::Rc::new(handler));
        self
    }

    pub(super) fn with_close_handler(mut self, handler: Option<CloseHandler>) -> Self {
        self.on_close = handler;
        self
    }

    pub(super) fn with_show_override(mut self, show: Option<bool>) -> Self {
        self.show = show;
        self
    }

    pub(super) fn with_button_width_override(mut self, width: Option<Pixels>) -> Self {
        self.button_width = width;
        self
    }
}

impl RenderOnce for CaptionButtons {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let buttons = div().id(self.id);
        if !should_show_caption_buttons(window, self.platform, self.show) {
            return buttons;
        }

        let controls = window.window_controls();
        let show_minimize = window.is_minimizable() && controls.minimize;
        let show_maximize = window.is_resizable() && controls.maximize;
        let button_width = self.button_width.map_or_else(
            || self.platform.default_caption_button_width(),
            |width| sanitize_positive(width, self.platform.default_caption_button_width()),
        );
        let rtl = current_material_theme(window, cx).layout.direction == Direction::Rtl;

        buttons
            .flex()
            .items_stretch()
            .flex_shrink_0()
            .when(rtl, |buttons| buttons.flex_row_reverse())
            .when(show_minimize, |buttons| {
                buttons.child(title_bar_control(
                    CaptionButton::Minimize,
                    self.platform,
                    button_width,
                    self.colors,
                    self.labels.for_control(CaptionButton::Minimize),
                    None,
                    window,
                    cx,
                ))
            })
            .when(show_maximize, |buttons| {
                let control = if window.is_maximized() {
                    CaptionButton::Restore
                } else {
                    CaptionButton::Maximize
                };
                buttons.child(title_bar_control(
                    control,
                    self.platform,
                    button_width,
                    self.colors,
                    self.labels.for_control(control),
                    None,
                    window,
                    cx,
                ))
            })
            .child(title_bar_control(
                CaptionButton::Close,
                self.platform,
                button_width,
                self.colors,
                self.labels.for_control(CaptionButton::Close),
                self.on_close,
                window,
                cx,
            ))
    }
}

#[derive(Clone, Copy)]
enum CaptionButton {
    Minimize,
    Maximize,
    Restore,
    Close,
}

impl CaptionButton {
    fn id(self) -> &'static str {
        match self {
            Self::Minimize => "minimize",
            Self::Maximize | Self::Restore => "maximize-restore",
            Self::Close => "close",
        }
    }

    fn area(self) -> WindowControlArea {
        match self {
            Self::Minimize => WindowControlArea::Min,
            Self::Maximize | Self::Restore => WindowControlArea::Max,
            Self::Close => WindowControlArea::Close,
        }
    }

    fn activate(self, window: &mut Window, cx: &mut App, on_close: Option<&CloseHandler>) {
        match self {
            Self::Minimize => activate_minimize(window),
            Self::Maximize | Self::Restore => activate_maximize_or_restore(window),
            Self::Close => activate_close(window, cx, on_close),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn title_bar_control(
    control: CaptionButton,
    platform: ChromePlatform,
    button_width: Pixels,
    colors: CaptionButtonColors,
    label: SharedString,
    on_close: Option<CloseHandler>,
    window: &Window,
    cx: &App,
) -> impl IntoElement {
    let theme = current_material_theme(window, cx);
    let tone = |role| color(theme.colors.color(role));
    let foreground = if window.is_window_active() {
        colors
            .active_foreground
            .unwrap_or_else(|| tone(ColorRole::OnSurface))
    } else {
        colors
            .inactive_foreground
            .unwrap_or_else(|| tone(ColorRole::OnSurfaceVariant))
    };
    let (hover_fill, hover_text, pressed_fill, pressed_text) = match control {
        CaptionButton::Close => (
            colors
                .close_hover_background
                .unwrap_or_else(|| tone(ColorRole::Error)),
            colors
                .close_hover_foreground
                .unwrap_or_else(|| tone(ColorRole::OnError)),
            colors
                .close_pressed_background
                .unwrap_or_else(|| tone(ColorRole::ErrorContainer)),
            colors
                .close_pressed_foreground
                .unwrap_or_else(|| tone(ColorRole::OnErrorContainer)),
        ),
        _ => (
            colors
                .hover_background
                .unwrap_or_else(|| tone(ColorRole::SurfaceContainerHigh)),
            colors
                .hover_foreground
                .unwrap_or_else(|| tone(ColorRole::OnSurface)),
            colors
                .pressed_background
                .unwrap_or_else(|| tone(ColorRole::SurfaceContainerHighest)),
            colors
                .pressed_foreground
                .unwrap_or_else(|| tone(ColorRole::OnSurface)),
        ),
    };
    let ring = tone(ColorRole::Secondary);

    // Windows routes native pointer clicks (and Snap Layouts on Maximize) through
    // non-client hit areas unless a custom `on_close` callback overrides Close.
    let use_native_win32_area = platform == ChromePlatform::Windows
        && !(matches!(control, CaptionButton::Close) && on_close.is_some());

    let click_close = on_close.clone();
    let a11y_close = on_close;

    div()
        .id(control.id())
        .flex()
        .items_center()
        .justify_center()
        .w(button_width)
        .min_h(px(32.))
        .flex_shrink_0()
        .border_1()
        .border_color(transparent_black())
        .text_color(foreground)
        .when(use_native_win32_area, |button| {
            button.window_control_area(control.area())
        })
        .role(Role::Button)
        .aria_label(label)
        .tab_index(0)
        .focus_visible(move |style| style.border_color(ring))
        .hover(move |style| style.bg(hover_fill).text_color(hover_text))
        .active(move |style| style.bg(pressed_fill).text_color(pressed_text))
        .on_click(move |event, window, cx| {
            if !use_native_win32_area || matches!(event, ClickEvent::Keyboard(_)) {
                control.activate(window, cx, click_close.as_ref());
            }
        })
        .on_a11y_action(AccessibleAction::Click, move |_, window, cx| {
            control.activate(window, cx, a11y_close.as_ref());
        })
        .child(caption_glyph(control))
        .test_support()
}

fn caption_glyph(control: CaptionButton) -> impl IntoElement {
    canvas(
        |_, _, _| {},
        move |bounds: Bounds<Pixels>, _, window: &mut Window, _| {
            let stroke = window.text_style().color;
            let x = bounds.origin.x;
            let y = bounds.origin.y;
            let w = bounds.size.width;
            let h = bounds.size.height;
            let mut builder = PathBuilder::stroke(px(1.));
            match control {
                CaptionButton::Minimize => {
                    let mid_y = y + h * 0.5;
                    builder.move_to(point(x, mid_y));
                    builder.line_to(point(x + w, mid_y));
                }
                CaptionButton::Maximize => {
                    let inset = px(0.5);
                    let x0 = x + inset;
                    let y0 = y + inset;
                    let x1 = x + w - inset;
                    let y1 = y + h - inset;
                    builder.add_polygon(
                        &[point(x0, y0), point(x1, y0), point(x1, y1), point(x0, y1)],
                        true,
                    );
                }
                CaptionButton::Restore => {
                    let inset = px(0.5);
                    let offset = px(2.);
                    // Front lower-left square.
                    let fx0 = x + inset;
                    let fy0 = y + offset + inset;
                    let fx1 = x + w - offset - inset;
                    let fy1 = y + h - inset;
                    builder.add_polygon(
                        &[
                            point(fx0, fy0),
                            point(fx1, fy0),
                            point(fx1, fy1),
                            point(fx0, fy1),
                        ],
                        true,
                    );
                    // Back upper-right L-frame.
                    let bx0 = x + offset + inset;
                    let by0 = y + inset;
                    let bx1 = x + w - inset;
                    let by1 = y + h - offset - inset;
                    builder.move_to(point(bx0, fy0));
                    builder.line_to(point(bx0, by0));
                    builder.line_to(point(bx1, by0));
                    builder.line_to(point(bx1, by1));
                    builder.line_to(point(fx1, by1));
                }
                CaptionButton::Close => {
                    let inset = px(0.5);
                    let x0 = x + inset;
                    let y0 = y + inset;
                    let x1 = x + w - inset;
                    let y1 = y + h - inset;
                    builder.move_to(point(x0, y0));
                    builder.line_to(point(x1, y1));
                    builder.move_to(point(x1, y0));
                    builder.line_to(point(x0, y1));
                }
            }
            if let Ok(path) = builder.build() {
                window.paint_path(path, stroke);
            }
        },
    )
    .size(px(10.))
    .flex_shrink_0()
}
