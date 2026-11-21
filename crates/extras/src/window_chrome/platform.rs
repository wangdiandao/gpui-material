use std::rc::Rc;

use gpui::{App, Decorations, Pixels, Window, WindowDecorations, WindowOptions, point, px};

/// Default logical height for a client-drawn [`super::TitleBar`].
pub const DEFAULT_TITLE_BAR_HEIGHT: Pixels = px(48.);

/// Default leading inset reserved for native macOS traffic light buttons.
pub const DEFAULT_MACOS_TRAFFIC_LIGHT_PADDING: Pixels = px(78.);

/// Default logical width for each caption button on Windows.
pub const DEFAULT_WINDOWS_CAPTION_BUTTON_WIDTH: Pixels = px(46.);

/// Default logical width for each caption button on Linux CSD title bars.
pub const DEFAULT_LINUX_CAPTION_BUTTON_WIDTH: Pixels = px(40.);

pub(super) type CloseHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// Target desktop chrome presentation model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChromePlatform {
    /// Windows client-drawn title bar with native `WindowControlArea` hit testing
    /// and Snap Layouts.
    Windows,
    /// macOS transparent title bar reserving space for native traffic light buttons.
    MacOs,
    /// Linux Wayland/X11 title bar adapting to `Decorations::Client` and
    /// `Decorations::Server`.
    Linux,
}

impl ChromePlatform {
    /// Returns the compile-time target chrome platform.
    pub const fn current() -> Self {
        #[cfg(target_os = "macos")]
        {
            Self::MacOs
        }
        #[cfg(windows)]
        {
            Self::Windows
        }
        #[cfg(not(any(windows, target_os = "macos")))]
        {
            Self::Linux
        }
    }

    /// Returns whether this platform uses native macOS traffic light buttons.
    pub const fn uses_traffic_lights(self) -> bool {
        matches!(self, Self::MacOs)
    }

    /// Returns whether this platform renders client-drawn caption buttons by default.
    pub const fn uses_caption_buttons(self) -> bool {
        matches!(self, Self::Windows | Self::Linux)
    }

    pub(super) const fn default_caption_button_width(self) -> Pixels {
        match self {
            Self::Windows => DEFAULT_WINDOWS_CAPTION_BUTTON_WIDTH,
            Self::MacOs | Self::Linux => DEFAULT_LINUX_CAPTION_BUTTON_WIDTH,
        }
    }
}

/// Configures a window to draw its own cross-platform title bar with the default
/// [`DEFAULT_TITLE_BAR_HEIGHT`].
///
/// Mount a [`super::TitleBar`] or compose a custom drag region and
/// [`super::CaptionButtons`] in the window view. Caller-supplied title, bounds,
/// minimum size, traffic light position, decoration preference, and capability
/// flags are preserved.
pub fn title_bar_window_options(options: WindowOptions) -> WindowOptions {
    title_bar_window_options_with_height(options, DEFAULT_TITLE_BAR_HEIGHT)
}

/// Configures a window to draw its own cross-platform title bar targeting a
/// specific logical title bar height.
///
/// On Windows and macOS this enables `TitlebarOptions::appears_transparent`. On
/// macOS it also computes a vertically centered default `traffic_light_position`
/// from `height` when the caller did not supply one. On Linux it requests
/// `WindowDecorations::Client` when `window_decorations` is unset.
pub fn title_bar_window_options_with_height(
    options: WindowOptions,
    height: Pixels,
) -> WindowOptions {
    configure_window_options(options, height, ChromePlatform::current())
}

pub(super) fn configure_window_options(
    mut options: WindowOptions,
    height: Pixels,
    platform: ChromePlatform,
) -> WindowOptions {
    let height = sanitize_positive(height, DEFAULT_TITLE_BAR_HEIGHT);
    let titlebar = options.titlebar.get_or_insert_default();
    titlebar.appears_transparent = true;

    match platform {
        ChromePlatform::MacOs => {
            if titlebar.traffic_light_position.is_none() {
                // Native macOS traffic light glyphs are roughly 14 px tall.
                let y = ((height - px(14.)) * 0.5).max(px(6.));
                titlebar.traffic_light_position = Some(point(px(12.), y));
            }
        }
        ChromePlatform::Linux => {
            if options.window_decorations.is_none() {
                options.window_decorations = Some(WindowDecorations::Client);
            }
        }
        ChromePlatform::Windows => {}
    }

    options
}

pub(super) fn sanitize_positive(value: Pixels, fallback: Pixels) -> Pixels {
    let raw = f32::from(value);
    if raw.is_finite() && raw > 0. {
        value
    } else {
        fallback
    }
}

pub(super) fn sanitize_nonnegative(value: Pixels, fallback: Pixels) -> Pixels {
    let raw = f32::from(value);
    if raw.is_finite() && raw >= 0. {
        value
    } else {
        fallback
    }
}

pub(super) fn should_show_caption_buttons(
    window: &Window,
    platform: ChromePlatform,
    explicit: Option<bool>,
) -> bool {
    if window.is_fullscreen() {
        return false;
    }
    if let Some(show) = explicit {
        return show;
    }
    match platform {
        ChromePlatform::Windows => true,
        ChromePlatform::MacOs => false,
        ChromePlatform::Linux => matches!(window.window_decorations(), Decorations::Client { .. }),
    }
}

pub(super) fn read_window_title(window: &Window) -> String {
    #[cfg(windows)]
    {
        if let Some(title) = win32::window_title(window) {
            return title;
        }
    }
    window.window_title()
}

pub(super) fn activate_minimize(window: &mut Window) {
    window.minimize_window();
}

pub(super) fn activate_maximize_or_restore(window: &mut Window) {
    #[cfg(windows)]
    {
        if window.is_maximized() && win32::restore_window(window) {
            return;
        }
    }
    window.zoom_window();
}

pub(super) fn activate_close(window: &mut Window, cx: &mut App, on_close: Option<&CloseHandler>) {
    if let Some(handler) = on_close {
        handler(window, cx);
        return;
    }
    #[cfg(windows)]
    {
        if win32::close_window(window) {
            return;
        }
    }
    window.remove_window();
}

#[cfg(windows)]
mod win32 {
    use gpui::Window;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows::Win32::{
        Foundation::{HWND, LPARAM, WPARAM},
        UI::WindowsAndMessaging::{
            GetWindowTextLengthW, GetWindowTextW, PostMessageW, SC_RESTORE, WM_CLOSE, WM_SYSCOMMAND,
        },
    };

    pub(super) fn window_title(window: &Window) -> Option<String> {
        let hwnd = window_hwnd(window)?;
        // GPUI's window_title getter currently only reads the platform title on macOS.
        // Windows reads the title from the live HWND into a caller-owned buffer.
        let length = unsafe { GetWindowTextLengthW(hwnd) };
        if length == 0 {
            return Some(String::new());
        }
        let mut title = vec![0; length as usize + 1];
        let copied = unsafe { GetWindowTextW(hwnd, &mut title) };
        Some(String::from_utf16_lossy(&title[..copied as usize]))
    }

    pub(super) fn restore_window(window: &Window) -> bool {
        post_window_message(window, WM_SYSCOMMAND, WPARAM(SC_RESTORE as usize))
    }

    pub(super) fn close_window(window: &Window) -> bool {
        post_window_message(window, WM_CLOSE, WPARAM(0))
    }

    fn post_window_message(window: &Window, message: u32, wparam: WPARAM) -> bool {
        let Some(hwnd) = window_hwnd(window) else {
            return false;
        };
        // Queue the command so GPUI handles it after the current input dispatch.
        // The HWND comes from the live window, and neither message carries pointers.
        let _ = unsafe { PostMessageW(Some(hwnd), message, wparam, LPARAM(0)) };
        true
    }

    fn window_hwnd(window: &Window) -> Option<HWND> {
        let handle = HasWindowHandle::window_handle(window).ok()?;
        let RawWindowHandle::Win32(handle) = handle.as_raw() else {
            return None;
        };
        Some(HWND(handle.hwnd.get() as *mut _))
    }
}
