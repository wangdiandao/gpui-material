//! Cross-platform client-drawn window title bars and controls backed by GPUI.

mod caption;
mod platform;
#[cfg(test)]
mod tests;

use gpui::{
    AnyElement, App, ElementId, Font, Hsla, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, ParentElement, Pixels, RenderOnce, SharedString, Styled, Window,
    WindowControlArea, div, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt as _;
use gpui_material_foundation::{color, current_material_theme, font_for_style};
use gpui_material_theme::{ColorRole, Direction, TypeRole, TypographyEmphasis};

pub use caption::{CaptionButtonColors, CaptionButtons};
pub use platform::{
    ChromePlatform, DEFAULT_LINUX_CAPTION_BUTTON_WIDTH, DEFAULT_MACOS_TRAFFIC_LIGHT_PADDING,
    DEFAULT_TITLE_BAR_HEIGHT, DEFAULT_WINDOWS_CAPTION_BUTTON_WIDTH, title_bar_window_options,
    title_bar_window_options_with_height,
};

use platform::{CloseHandler, read_window_title, sanitize_nonnegative, sanitize_positive};

/// Optional color overrides for [`TitleBar`]. Omitted fields resolve from the
/// current window Material theme.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TitleBarColors {
    pub background: Option<Hsla>,
    pub active_foreground: Option<Hsla>,
    pub inactive_foreground: Option<Hsla>,
    pub border: Option<Hsla>,
}

/// Cross-platform client-drawn title bar with a drag region, customizable title
/// typography, optional leading/content/trailing slots, macOS traffic-light
/// spacing, and platform-appropriate window controls.
#[derive(IntoElement)]
pub struct TitleBar {
    id: ElementId,
    title: Option<SharedString>,
    height: Pixels,
    title_role: TypeRole,
    title_font: Option<Font>,
    title_font_family: Option<SharedString>,
    title_font_size: Option<Pixels>,
    leading: Option<AnyElement>,
    content: Option<AnyElement>,
    trailing: Option<AnyElement>,
    traffic_light_padding: Pixels,
    colors: TitleBarColors,
    caption_colors: CaptionButtonColors,
    caption_button_width: Option<Pixels>,
    show_caption_buttons: Option<bool>,
    on_close: Option<CloseHandler>,
    platform: ChromePlatform,
}

impl TitleBar {
    /// Creates a title bar with a stable identity in the window view.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            title: None,
            height: DEFAULT_TITLE_BAR_HEIGHT,
            title_role: TypeRole::TitleSmall,
            title_font: None,
            title_font_family: None,
            title_font_size: None,
            leading: None,
            content: None,
            trailing: None,
            traffic_light_padding: DEFAULT_MACOS_TRAFFIC_LIGHT_PADDING,
            colors: TitleBarColors::default(),
            caption_colors: CaptionButtonColors::default(),
            caption_button_width: None,
            show_caption_buttons: None,
            on_close: None,
            platform: ChromePlatform::current(),
        }
    }

    /// Sets an explicit title string. When omitted, reads the live window title.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the logical height of the title bar. Defaults to [`DEFAULT_TITLE_BAR_HEIGHT`] (48 px).
    pub fn height(mut self, height: Pixels) -> Self {
        self.height = sanitize_positive(height, DEFAULT_TITLE_BAR_HEIGHT);
        self
    }

    /// Selects the Material typography role for the title text. Defaults to `TypeRole::TitleSmall`.
    pub fn title_role(mut self, role: TypeRole) -> Self {
        self.title_role = role;
        self
    }

    /// Overrides the full GPUI [`Font`] used for the title text.
    pub fn title_font(mut self, font: Font) -> Self {
        self.title_font = Some(font);
        self
    }

    /// Overrides only the title font family while preserving weight and theme metrics.
    pub fn title_font_family(mut self, family: impl Into<SharedString>) -> Self {
        self.title_font_family = Some(family.into());
        self
    }

    /// Overrides the title font size in logical pixels.
    pub fn title_font_size(mut self, size: Pixels) -> Self {
        self.title_font_size = Some(size);
        self
    }

    /// Places leading client-area content after any macOS traffic-light inset and
    /// before the title/drag region.
    pub fn leading(mut self, content: impl IntoElement) -> Self {
        self.leading = Some(content.into_any_element());
        self
    }

    /// Places client-area content between the drag region and trailing/caption controls.
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    /// Places trailing client-area content before the caption buttons.
    pub fn trailing(mut self, content: impl IntoElement) -> Self {
        self.trailing = Some(content.into_any_element());
        self
    }

    /// Sets the logical leading inset reserved for macOS traffic light buttons
    /// when not fullscreen. Defaults to [`DEFAULT_MACOS_TRAFFIC_LIGHT_PADDING`] (78 px).
    pub fn traffic_light_padding(mut self, padding: Pixels) -> Self {
        self.traffic_light_padding =
            sanitize_nonnegative(padding, DEFAULT_MACOS_TRAFFIC_LIGHT_PADDING);
        self
    }

    /// Overrides the title bar surface, foreground, and optional bottom divider colors.
    pub fn colors(mut self, colors: TitleBarColors) -> Self {
        self.colors = colors;
        self
    }

    /// Overrides the caption button palette.
    pub fn caption_colors(mut self, colors: CaptionButtonColors) -> Self {
        self.caption_colors = colors;
        self
    }

    /// Overrides the logical width of each caption button.
    pub fn caption_button_width(mut self, width: Pixels) -> Self {
        self.caption_button_width = Some(width);
        self
    }

    /// Explicitly shows or hides caption buttons (still hidden in fullscreen).
    pub fn show_caption_buttons(mut self, show: bool) -> Self {
        self.show_caption_buttons = Some(show);
        self
    }

    /// Installs a custom close callback invoked when the Close button activates.
    pub fn on_close(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(std::rc::Rc::new(handler));
        self
    }

    /// Overrides the target chrome platform rules. Defaults to [`ChromePlatform::current`].
    pub fn platform(mut self, platform: ChromePlatform) -> Self {
        self.platform = platform;
        self
    }
}

impl RenderOnce for TitleBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = current_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let tone = |role| color(theme.colors.color(role));

        let background = self
            .colors
            .background
            .unwrap_or_else(|| tone(ColorRole::Surface));
        let foreground = if window.is_window_active() {
            self.colors
                .active_foreground
                .unwrap_or_else(|| tone(ColorRole::OnSurface))
        } else {
            self.colors
                .inactive_foreground
                .unwrap_or_else(|| tone(ColorRole::OnSurfaceVariant))
        };

        let type_style = theme
            .typography
            .style(self.title_role, TypographyEmphasis::Baseline);
        let mut resolved_font = self
            .title_font
            .unwrap_or_else(|| font_for_style(&theme.typography, type_style, cx));
        if let Some(family) = self.title_font_family {
            resolved_font.family = family;
        }
        let font_size = self.title_font_size.map_or(px(type_style.size), |size| {
            sanitize_positive(size, px(type_style.size))
        });

        let title_text = self
            .title
            .unwrap_or_else(|| SharedString::from(read_window_title(window)));
        let fullscreen = window.is_fullscreen();
        let platform = self.platform;
        let show_traffic_light_inset = platform.uses_traffic_lights() && !fullscreen;
        let has_leading = self.leading.is_some();
        let title_start_pad = if show_traffic_light_inset || has_leading {
            px(8.)
        } else {
            px(16.)
        };

        let drag_region = div()
            .flex()
            .flex_1()
            .min_w(px(32.))
            .items_center()
            .when(rtl, |title| title.flex_row_reverse().pr(title_start_pad))
            .when(!rtl, |title| title.pl(title_start_pad))
            .when(!fullscreen, |title| {
                let title = title.window_control_area(WindowControlArea::Drag);
                match platform {
                    ChromePlatform::MacOs => title.on_mouse_down(
                        MouseButton::Left,
                        |event: &MouseDownEvent, window, _| {
                            if event.click_count == 2 {
                                window.titlebar_double_click();
                            }
                        },
                    ),
                    ChromePlatform::Linux => title
                        .on_mouse_down(MouseButton::Left, |event: &MouseDownEvent, window, _| {
                            if event.click_count == 2 {
                                if window.is_resizable() && window.window_controls().maximize {
                                    window.zoom_window();
                                }
                            } else {
                                window.start_window_move();
                            }
                        })
                        .on_mouse_down(MouseButton::Right, |event: &MouseDownEvent, window, _| {
                            if window.window_controls().window_menu {
                                window.show_window_menu(event.position);
                            }
                        }),
                    ChromePlatform::Windows => title,
                }
            })
            .child(
                div()
                    .min_w_0()
                    .truncate()
                    .font(resolved_font)
                    .text_size(font_size)
                    .child(title_text),
            );

        let caption_buttons = CaptionButtons::new("caption-buttons")
            .colors(self.caption_colors)
            .with_button_width_override(self.caption_button_width)
            .platform(platform)
            .with_show_override(self.show_caption_buttons)
            .with_close_handler(self.on_close);

        div()
            .id(self.id.clone())
            .flex()
            .w_full()
            .h(self.height)
            .min_h(self.height)
            .flex_shrink_0()
            .items_stretch()
            .when(rtl, |bar| bar.flex_row_reverse())
            .bg(background)
            .text_color(foreground)
            .when_some(self.colors.border, |bar, border| {
                bar.border_b_1().border_color(border)
            })
            .when(show_traffic_light_inset, |bar| {
                bar.child(
                    div()
                        .id((self.id.clone(), "traffic-light-inset"))
                        .w(self.traffic_light_padding)
                        .h_full()
                        .flex_shrink_0(),
                )
            })
            .when_some(self.leading, |bar, leading| {
                bar.child(
                    div()
                        .flex()
                        .flex_shrink_0()
                        .items_center()
                        .when(!show_traffic_light_inset && !rtl, |slot| slot.pl(px(8.)))
                        .when(!show_traffic_light_inset && rtl, |slot| slot.pr(px(8.)))
                        .child(leading),
                )
            })
            .child(drag_region)
            .when_some(self.content, |bar, content| {
                bar.child(div().flex().min_w_0().items_center().child(content))
            })
            .when_some(self.trailing, |bar, trailing| {
                bar.child(div().flex().min_w_0().items_center().child(trailing))
            })
            .child(caption_buttons)
            .test_support()
    }
}
