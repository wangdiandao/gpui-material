use gpui::{App, Axis, FocusHandle, Window};
use gpui_material_theme::Direction;

/// Moves within an ordered focus group, skipping disabled items and wrapping.
/// The caller owns selection and sets the active item's tab stop.
pub fn move_roving_focus(
    items: &[(&FocusHandle, bool)],
    key: &str,
    axis: Axis,
    direction: Direction,
    window: &mut Window,
    cx: &mut App,
) -> Option<usize> {
    let backward = match key {
        "left" if axis == Axis::Horizontal => direction == Direction::Ltr,
        "right" if axis == Axis::Horizontal => direction == Direction::Rtl,
        "up" if axis == Axis::Vertical => true,
        "down" if axis == Axis::Vertical => false,
        _ => return None,
    };
    let len = items.len();
    let current = items.iter().position(|(focus, _)| focus.is_focused(window));
    let start = current.unwrap_or(if backward { 0 } else { len.saturating_sub(1) });
    (1..=len).find_map(|offset| {
        let ix = if backward {
            (start + len - offset) % len
        } else {
            (start + offset) % len
        };
        let (focus, enabled) = items[ix];
        enabled.then(|| {
            focus.focus(window, cx);
            ix
        })
    })
}

/// Watches the active GPUI drag for loss of native Win32 capture.
/// Keep the guard for the drag lifetime and drop it on completion or cancellation.
pub struct DragCapture {
    #[cfg(windows)]
    _watch: Option<gpui::Task<()>>,
}

impl DragCapture {
    /// Call from GPUI's drag start handler. The callback runs once on the window thread
    /// after GPUI's drag is cancelled. Handle Esc, disable and window blur at the owner.
    pub fn watch(
        window: &Window,
        cx: &App,
        on_lost: impl FnOnce(&mut Window, &mut App) + 'static,
    ) -> Self {
        #[cfg(windows)]
        {
            use std::time::Duration;
            use windows::Win32::UI::Input::KeyboardAndMouse::GetCapture;

            // GPUI takes capture before starting a native mouse drag. GetCapture
            // is thread-local, and this task resumes on the window's UI thread.
            let capture = unsafe { GetCapture() };
            let watch = (!capture.is_invalid()).then(|| {
                let executor = cx.background_executor().clone();
                let mut on_lost = Some(on_lost);
                window.spawn(cx, async move |cx| {
                    loop {
                        executor.timer(Duration::from_millis(16)).await;
                        let keep_watching = cx.update(|window, cx| {
                            if !cx.has_active_drag() {
                                return false;
                            }
                            if unsafe { GetCapture() } == capture {
                                return true;
                            }
                            window.release_pointer();
                            cx.stop_active_drag(window);
                            if let Some(on_lost) = on_lost.take() {
                                on_lost(window, cx);
                            }
                            false
                        });
                        if !keep_watching.unwrap_or(false) {
                            break;
                        }
                    }
                })
            });
            Self { _watch: watch }
        }
        #[cfg(not(windows))]
        {
            let _ = (window, cx, on_lost);
            Self {}
        }
    }

    /// Cancels the current GPUI drag and releases its captured pointer.
    pub fn cancel(self, window: &mut Window, cx: &mut App) {
        window.release_pointer();
        cx.stop_active_drag(window);
    }
}
