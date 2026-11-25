use std::ops::Range;

use gpui::{AnyWindowHandle, AppContext, InputHandler, TestAppContext, UTF16Selection};

use super::TestWindowExt as _;

/// Bridges an explicit GPUI [`InputHandler`] to a test window so tests can
/// exercise UTF-16 composition (`replace_and_mark_text_in_range`), preedit
/// cancellation, and text commits without relying on an OS IME.
pub struct TestInput {
    handler: Box<dyn InputHandler>,
    window: AnyWindowHandle,
}

impl TestInput {
    /// Creates a composition bridge bound to `handler` and `window`.
    pub fn new(handler: impl InputHandler + 'static, window: impl Into<AnyWindowHandle>) -> Self {
        Self {
            handler: Box::new(handler),
            window: window.into(),
        }
    }

    /// Commits `text` through [`InputHandler::replace_text_in_range`] and completes a frame.
    pub fn commit_text(&mut self, text: &str, cx: &mut TestAppContext) {
        cx.update_window(self.window, |_, window, cx| {
            self.handler.replace_text_in_range(None, text, window, cx);
            window.render_frame(cx);
        })
        .expect("test window closed while committing text");
    }

    /// Updates marked/preedit text through [`InputHandler::replace_and_mark_text_in_range`]
    /// using UTF-16 code-unit ranges and completes a frame.
    pub fn compose_text(
        &mut self,
        replacement: Option<Range<usize>>,
        text: &str,
        selection: Option<Range<usize>>,
        cx: &mut TestAppContext,
    ) {
        cx.update_window(self.window, |_, window, cx| {
            self.handler
                .replace_and_mark_text_in_range(replacement, text, selection, window, cx);
            window.render_frame(cx);
        })
        .expect("test window closed while composing text");
    }

    /// Returns the document-relative UTF-16 marked range, if any.
    pub fn marked_text_range(&mut self, cx: &mut TestAppContext) -> Option<Range<usize>> {
        cx.update_window(self.window, |_, window, cx| {
            self.handler.marked_text_range(window, cx)
        })
        .expect("test window closed while querying marked range")
    }

    /// Returns the current UTF-16 selection range and direction, if any.
    pub fn selected_text_range(&mut self, cx: &mut TestAppContext) -> Option<UTF16Selection> {
        cx.update_window(self.window, |_, window, cx| {
            self.handler.selected_text_range(false, window, cx)
        })
        .expect("test window closed while querying selected range")
    }

    /// Clears active preedit marking while keeping its text and completes a frame.
    pub fn unmark_text(&mut self, cx: &mut TestAppContext) {
        cx.update_window(self.window, |_, window, cx| {
            self.handler.unmark_text(window, cx);
            window.render_frame(cx);
        })
        .expect("test window closed while unmarking text");
    }
}
