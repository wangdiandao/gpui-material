//! Client-drawn Windows title bars backed by GPUI's native hit testing.

use gpui::{
    AccessibleAction, AnyElement, App, ClickEvent, Div, ElementId, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, Role, Stateful, StatefulInteractiveElement, Styled, Window,
    WindowControlArea, WindowOptions, div, prelude::FluentBuilder, px, transparent_black,
};
use gpui_material_foundation::{color, window_material_theme};
use gpui_material_theme::{ColorRole, Direction};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::{
        GetWindowTextLengthW, GetWindowTextW, PostMessageW, SC_RESTORE, WM_CLOSE, WM_SYSCOMMAND,
    },
};

/// Configures a window to draw its own title bar. Mount a [`TitleBar`] or
/// compose a custom drag region and [`CaptionButtons`] in the window view.
pub fn title_bar_window_options(mut options: WindowOptions) -> WindowOptions {
    options.titlebar.get_or_insert_default().appears_transparent = true;
    options
}

/// Displays the current window title, a drag region, and Windows caption buttons.
#[derive(IntoElement)]
pub struct TitleBar {
    id: ElementId,
    content: Option<AnyElement>,
}

impl TitleBar {
    /// Creates a title bar with a stable identity in the window view.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            content: None,
        }
    }

    /// Places client-area content between the drag region and the caption buttons.
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }
}

impl RenderOnce for TitleBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let foreground = if window.is_window_active() {
            ColorRole::OnSurface
        } else {
            ColorRole::OnSurfaceVariant
        };
        let drag_region = div()
            .flex()
            .flex_1()
            .min_w(px(32.))
            .items_center()
            .when(rtl, |title| title.flex_row_reverse().pr(px(16.)))
            .when(!rtl, |title| title.pl(px(16.)))
            .when(!window.is_fullscreen(), |title| {
                title.window_control_area(WindowControlArea::Drag)
            })
            .child(
                div()
                    .min_w_0()
                    .truncate()
                    .font_family("Segoe UI")
                    .text_sm()
                    .child(window_title(window)),
            );

        div()
            .id(self.id)
            .flex()
            .w_full()
            .min_h(px(48.))
            .flex_shrink_0()
            .items_stretch()
            .when(rtl, |bar| bar.flex_row_reverse())
            .bg(color(theme.colors.color(ColorRole::Surface)))
            .text_color(color(theme.colors.color(foreground)))
            .child(drag_region)
            .when_some(self.content, |bar, content| {
                bar.child(div().flex().min_w_0().items_center().child(content))
            })
            .child(CaptionButtons::new("caption-buttons"))
    }
}

/// The minimize, maximize or restore, and close buttons for a custom Windows
/// title bar. Use this in a window opened with [`title_bar_window_options`].
#[derive(IntoElement)]
pub struct CaptionButtons {
    id: ElementId,
}

impl CaptionButtons {
    /// Creates caption buttons with a stable identity in their parent layout.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into() }
    }
}

impl RenderOnce for CaptionButtons {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let buttons = div().id(self.id);
        if window.is_fullscreen() {
            return buttons;
        }

        buttons
            .flex()
            .items_stretch()
            .flex_shrink_0()
            .when(
                window_material_theme(window, cx).layout.direction == Direction::Rtl,
                |buttons| buttons.flex_row_reverse(),
            )
            .when(window.is_minimizable(), |buttons| {
                buttons.child(title_bar_control(CaptionButton::Minimize, window, cx))
            })
            .when(window.is_resizable(), |buttons| {
                let control = if window.is_maximized() {
                    CaptionButton::Restore
                } else {
                    CaptionButton::Maximize
                };
                buttons.child(title_bar_control(control, window, cx))
            })
            .child(title_bar_control(CaptionButton::Close, window, cx))
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
    fn area(self) -> WindowControlArea {
        match self {
            Self::Minimize => WindowControlArea::Min,
            Self::Maximize | Self::Restore => WindowControlArea::Max,
            Self::Close => WindowControlArea::Close,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Minimize => "Minimize",
            Self::Maximize => "Maximize",
            Self::Restore => "Restore",
            Self::Close => "Close",
        }
    }

    fn glyph(self) -> &'static str {
        match self {
            Self::Minimize => "\u{e921}",
            Self::Maximize => "\u{e922}",
            Self::Restore => "\u{e923}",
            Self::Close => "\u{e8bb}",
        }
    }

    fn activate(self, window: &Window) {
        match self {
            Self::Minimize => window.minimize_window(),
            Self::Maximize | Self::Restore if window.is_maximized() => {
                post_window_message(window, WM_SYSCOMMAND, WPARAM(SC_RESTORE as usize));
            }
            Self::Maximize | Self::Restore => window.zoom_window(),
            Self::Close => post_window_message(window, WM_CLOSE, WPARAM(0)),
        }
    }
}

fn title_bar_control(control: CaptionButton, window: &Window, cx: &App) -> Stateful<Div> {
    let theme = window_material_theme(window, cx);
    let tone = |role| color(theme.colors.color(role));
    let foreground = if window.is_window_active() {
        ColorRole::OnSurface
    } else {
        ColorRole::OnSurfaceVariant
    };
    let (hover_fill, hover_text, pressed_fill, pressed_text) = match control {
        CaptionButton::Close => (
            ColorRole::Error,
            ColorRole::OnError,
            ColorRole::ErrorContainer,
            ColorRole::OnErrorContainer,
        ),
        _ => (
            ColorRole::SurfaceContainerHigh,
            ColorRole::OnSurface,
            ColorRole::SurfaceContainerHighest,
            ColorRole::OnSurface,
        ),
    };
    let (hover_fill, hover_text, pressed_fill, pressed_text) = (
        tone(hover_fill),
        tone(hover_text),
        tone(pressed_fill),
        tone(pressed_text),
    );
    let ring = tone(ColorRole::Primary);
    let id = match control {
        CaptionButton::Minimize => "minimize",
        CaptionButton::Maximize | CaptionButton::Restore => "maximize-restore",
        CaptionButton::Close => "close",
    };

    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .w(px(46.))
        .min_h(px(32.))
        .flex_shrink_0()
        .border_1()
        .border_color(transparent_black())
        .text_color(tone(foreground))
        .font_family("Segoe Fluent Icons")
        .text_size(px(10.))
        .window_control_area(control.area())
        .role(Role::Button)
        .aria_label(control.label())
        .tab_index(0)
        .focus_visible(move |style| style.border_color(ring))
        .hover(move |style| style.bg(hover_fill).text_color(hover_text))
        .active(move |style| style.bg(pressed_fill).text_color(pressed_text))
        // Windows processes pointer clicks through the non-client hit area.
        .on_click(move |event, window, _| {
            if matches!(event, ClickEvent::Keyboard(_)) {
                control.activate(window);
            }
        })
        .on_a11y_action(AccessibleAction::Click, move |_, window, _| {
            control.activate(window);
        })
        .child(control.glyph())
}

fn window_title(window: &Window) -> String {
    let Some(hwnd) = window_hwnd(window) else {
        return String::new();
    };
    // GPUI's window_title getter currently only reads the platform title on macOS.
    // Windows reads the title from the live HWND into a caller-owned buffer.
    let length = unsafe { GetWindowTextLengthW(hwnd) };
    if length == 0 {
        return String::new();
    }
    let mut title = vec![0; length as usize + 1];
    let copied = unsafe { GetWindowTextW(hwnd, &mut title) };
    String::from_utf16_lossy(&title[..copied as usize])
}

fn post_window_message(window: &Window, message: u32, wparam: WPARAM) {
    let Some(hwnd) = window_hwnd(window) else {
        return;
    };
    // Queue the command so GPUI handles it after the current input dispatch.
    // The HWND comes from the live window, and neither message carries pointers.
    let _ = unsafe { PostMessageW(Some(hwnd), message, wparam, LPARAM(0)) };
}

fn window_hwnd(window: &Window) -> Option<HWND> {
    let handle = HasWindowHandle::window_handle(window).ok()?;
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return None;
    };
    Some(HWND(handle.hwnd.get() as *mut _))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{TitlebarOptions, size};

    #[test]
    fn chrome_options_preserve_the_callers_window_settings() {
        let min_size = size(px(360.), px(240.));
        let options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some("My window".into()),
                ..Default::default()
            }),
            window_min_size: Some(min_size),
            is_resizable: false,
            is_minimizable: false,
            ..Default::default()
        };

        let options = title_bar_window_options(title_bar_window_options(options));
        let titlebar = options.titlebar.unwrap();
        assert!(titlebar.appears_transparent);
        assert_eq!(titlebar.title.as_deref(), Some("My window"));
        assert_eq!(options.window_min_size, Some(min_size));
        assert!(!options.is_resizable);
        assert!(!options.is_minimizable);
    }

    #[test]
    fn chrome_options_create_a_titlebar_configuration_when_absent() {
        let options = title_bar_window_options(WindowOptions {
            titlebar: None,
            ..Default::default()
        });
        assert!(options.titlebar.unwrap().appears_transparent);
    }
}
