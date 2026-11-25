//! Headless UI integration testing helpers for GPUI Material components and applications.
//!
//! Enable `test-support` under `[dev-dependencies]` so production builds carry
//! zero observation overhead. Render views inside a headless window opened via
//! [`crate::MaterialRoot::open_window`], drive interactions through
//! [`TestWindowExt`] / [`TestEventExt`] / [`TestInput`], and assert both native
//! accessibility [`ElementSnapshot`] properties and application state.
//!
//! [`ElementSnapshot`] values are immutable. Call [`TestWindowExt::render_frame`]
//! after external state changes, or use [`TestAppContextExt::wait_for`] outside
//! `update_window` for asynchronous and deferred updates.

use std::future::Future;
use std::time::Duration;

use gpui::{
    AnyWindowHandle, App, AppContext, Corners, Edges, ElementId, InputEvent, KeyDownEvent,
    KeyUpEvent, Keystroke, Modifiers, ModifiersChangedEvent, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Pixels, Point, Quad, ScrollDelta, ScrollEvent, TestAppContext,
    Window, point, px,
};
use gpui_base::test_support as observation;

mod events;
mod input;
mod query;

pub use events::TestEventExt;
pub use gpui_base::TestSupportExt;
pub use gpui_base::test_support::{ElementSnapshot, TextSnapshot};
pub use input::TestInput;
pub use query::TestQueryExt;

/// Returns painted quads from the last frame with split border/background slices
/// sharing identical bounds and paint properties merged back together.
pub fn painted_quads(window: &Window) -> Vec<Quad> {
    let mut merged: Vec<Quad> = Vec::new();
    for quad in window.painted_quads() {
        if let Some(last) = merged.last_mut()
            && last.bounds == quad.bounds
            && last.background == quad.background
            && last.border_color == quad.border_color
            && last.border_style == quad.border_style
        {
            if last.corner_radii == Corners::default() {
                last.corner_radii = quad.corner_radii;
            }
            if last.border_widths == Edges::default() {
                last.border_widths = quad.border_widths;
            }
            last.content_mask.bounds = last.content_mask.bounds.union(&quad.content_mask.bounds);
            continue;
        }
        merged.push(quad);
    }
    merged
}

/// High-level query and native event dispatch operations on a GPUI [`Window`].
pub trait TestWindowExt {
    /// Returns the uniquely matching observed element from the last completed frame.
    ///
    /// Panics with all registered element paths if the target is missing or ambiguous.
    fn find(&self, id: impl Into<ElementId>) -> ElementSnapshot;

    /// Returns the uniquely matching observed element if present, or `None` when absent.
    ///
    /// Still panics if multiple registered paths match `id` in this scope.
    fn try_find(&self, id: impl Into<ElementId>) -> Option<ElementSnapshot>;

    /// Returns all observed matches for `id` from the last completed frame, including
    /// invisible elements, ordered by bounds origin `(y, x)`.
    fn find_all(&self, id: impl Into<ElementId>) -> Vec<ElementSnapshot>;

    /// Narrows subsequent queries and interactions to a GPUI [`ElementId`] scope
    /// without requiring an extra layout container.
    fn within(&mut self, id: impl Into<ElementId>) -> ScopedWindow<'_>;

    /// Refreshes and draws one frame so snapshots reflect external state changes.
    fn render_frame(&mut self, cx: &mut App);

    /// Moves the pointer to the target center and dispatches a left click.
    fn click(&mut self, id: impl Into<ElementId>, cx: &mut App);

    /// Clicks at a pixel offset relative to the target's top-left corner.
    fn click_at(&mut self, id: impl Into<ElementId>, offset: Point<Pixels>, cx: &mut App);

    /// Dispatches a configurable click sequence, temporarily applying and then
    /// restoring window modifiers while preserving Caps Lock.
    fn click_with_options(&mut self, id: impl Into<ElementId>, options: ClickOptions, cx: &mut App);

    /// Dispatches a centered left click with temporary modifiers, restoring previous
    /// modifiers afterward.
    fn click_with_modifiers(
        &mut self,
        id: impl Into<ElementId>,
        modifiers: Modifiers,
        cx: &mut App,
    ) {
        self.click_with_options(id, ClickOptions::new().with_modifiers(modifiers), cx);
    }

    /// Dispatches a centered right click on `id`.
    fn right_click(&mut self, id: impl Into<ElementId>, cx: &mut App);

    /// Dispatches a centered left double-click on `id`.
    fn double_click(&mut self, id: impl Into<ElementId>, cx: &mut App);

    /// Moves the pointer to the center of `id` without pressing a mouse button.
    fn hover(&mut self, id: impl Into<ElementId>, cx: &mut App);

    /// Moves the pointer to `id` and dispatches a native wheel scroll event.
    fn scroll(&mut self, id: impl Into<ElementId>, delta: ScrollDelta, cx: &mut App);

    /// Drags with the left mouse button between two window-local positions using
    /// interpolated pointer moves and native hit testing.
    fn drag(&mut self, from: Point<Pixels>, to: Point<Pixels>, cx: &mut App);

    /// Drags from the center of `from` to the center of `to` using native hit testing.
    fn drag_to(&mut self, from: impl Into<ElementId>, to: impl Into<ElementId>, cx: &mut App);

    /// Dispatches native key-down and key-up events for a GPUI keystroke string
    /// (such as `"backspace"`, `"enter"`, `"tab"`, or `"secondary-c"`).
    ///
    /// `"enter"` and `"tab"` invoke key bindings without synthesizing IME text.
    fn press(&mut self, key: &str, cx: &mut App);

    /// Types Unicode text character-by-character into the current keyboard focus.
    fn input(&mut self, text: &str, cx: &mut App);
}

/// Configuration for [`TestWindowExt::click_with_options`].
///
/// Defaults to a single centered left click with no modifiers.
#[derive(Clone, Copy, Debug)]
pub struct ClickOptions {
    offset: Option<Point<Pixels>>,
    button: MouseButton,
    count: usize,
    modifiers: Modifiers,
}

impl Default for ClickOptions {
    fn default() -> Self {
        Self {
            offset: None,
            button: MouseButton::Left,
            count: 1,
            modifiers: Modifiers::default(),
        }
    }
}

impl ClickOptions {
    /// Creates options for a single centered left click without modifiers.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets an offset relative to the target's top-left corner.
    pub fn with_offset(mut self, offset: Point<Pixels>) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Returns the configured local offset, or `None` for the element center.
    pub fn offset(&self) -> Option<Point<Pixels>> {
        self.offset
    }

    /// Sets the mouse button used for the click sequence.
    pub fn with_button(mut self, button: MouseButton) -> Self {
        self.button = button;
        self
    }

    /// Returns the configured mouse button.
    pub fn button(&self) -> MouseButton {
        self.button
    }

    /// Sets the number of clicks. Panics if `count == 0`.
    pub fn with_count(mut self, count: usize) -> Self {
        assert!(count > 0, "click count must be positive");
        self.count = count;
        self
    }

    /// Returns the positive click count.
    pub fn count(&self) -> usize {
        self.count
    }

    /// Sets temporary keyboard modifiers applied for the duration of the click.
    pub fn with_modifiers(mut self, modifiers: Modifiers) -> Self {
        self.modifiers = modifiers;
        self
    }

    /// Returns the configured modifier flags.
    pub fn modifiers(&self) -> Modifiers {
        self.modifiers
    }
}

fn require(window: &Window, scope: &[ElementId], id: &ElementId) -> ElementSnapshot {
    observation::find(window, scope, id).unwrap_or_else(|| {
        panic!(
            "missing ElementId {id:?} in scope {scope:?}. Registered paths: {}. \
             Check the ID, .test_support() registration, and completed frame.",
            observation::registered_paths(window)
        )
    })
}

fn target_position(
    window: &Window,
    scope: &[ElementId],
    id: &ElementId,
    offset: Option<Point<Pixels>>,
) -> Point<Pixels> {
    let target = require(window, scope, id);
    assert!(
        target.visible(),
        "ElementId {id:?} is not visible (path {:?})",
        target.path()
    );
    if let Some(offset) = offset {
        let size = target.bounds().size;
        assert!(
            offset.x >= px(0.)
                && offset.y >= px(0.)
                && offset.x < size.width
                && offset.y < size.height,
            "click offset {offset:?} is outside ElementId {id:?} bounds {:?}",
            target.bounds()
        );
        target.bounds().origin + offset
    } else {
        target.bounds().center()
    }
}

pub(crate) fn move_pointer(
    window: &mut Window,
    position: Point<Pixels>,
    pressed_button: Option<MouseButton>,
    modifiers: Modifiers,
    cx: &mut App,
) {
    window.dispatch_event(
        MouseMoveEvent {
            position,
            pressed_button,
            modifiers,
        }
        .to_platform_input(),
        cx,
    );
    window.render_frame(cx);
}

pub(crate) fn mouse_down(
    window: &mut Window,
    position: Point<Pixels>,
    button: MouseButton,
    click_count: usize,
    modifiers: Modifiers,
    cx: &mut App,
) {
    window.dispatch_event(
        MouseDownEvent {
            button,
            position,
            modifiers,
            click_count,
            first_mouse: false,
        }
        .to_platform_input(),
        cx,
    );
    window.render_frame(cx);
}

pub(crate) fn mouse_up(
    window: &mut Window,
    position: Point<Pixels>,
    button: MouseButton,
    click_count: usize,
    modifiers: Modifiers,
    cx: &mut App,
) {
    window.dispatch_event(
        MouseUpEvent {
            button,
            position,
            modifiers,
            click_count,
        }
        .to_platform_input(),
        cx,
    );
    window.render_frame(cx);
}

pub(crate) fn change_modifiers(window: &mut Window, modifiers: Modifiers, cx: &mut App) {
    if window.modifiers() != modifiers {
        window.dispatch_event(
            ModifiersChangedEvent {
                modifiers,
                capslock: window.capslock(),
            }
            .to_platform_input(),
            cx,
        );
        window.render_frame(cx);
    }
}

fn click_target(
    window: &mut Window,
    scope: &[ElementId],
    id: ElementId,
    options: ClickOptions,
    cx: &mut App,
) {
    let ClickOptions {
        offset,
        button,
        count,
        modifiers: _,
    } = options;
    window.render_frame(cx);
    let modifiers = window.modifiers();
    let position = target_position(window, scope, &id, offset);
    move_pointer(window, position, None, modifiers, cx);
    for click_count in 1..=count {
        mouse_down(window, position, button, click_count, modifiers, cx);
        mouse_up(window, position, button, click_count, modifiers, cx);
    }
}

fn click_with_options_target(
    window: &mut Window,
    scope: &[ElementId],
    id: ElementId,
    options: ClickOptions,
    cx: &mut App,
) {
    window.render_frame(cx);
    target_position(window, scope, &id, options.offset);
    let previous = window.modifiers();
    change_modifiers(window, options.modifiers, cx);
    click_target(window, scope, id, options, cx);
    change_modifiers(window, previous, cx);
}

fn hover_target(window: &mut Window, scope: &[ElementId], id: ElementId, cx: &mut App) {
    window.render_frame(cx);
    let position = target_position(window, scope, &id, None);
    move_pointer(window, position, None, window.modifiers(), cx);
}

fn scroll_target(
    window: &mut Window,
    scope: &[ElementId],
    id: ElementId,
    delta: ScrollDelta,
    cx: &mut App,
) {
    window.render_frame(cx);
    let position = target_position(window, scope, &id, None);
    move_pointer(window, position, None, window.modifiers(), cx);
    window.dispatch_event(
        ScrollEvent {
            position,
            delta,
            modifiers: window.modifiers(),
            ..Default::default()
        }
        .to_platform_input(),
        cx,
    );
    window.render_frame(cx);
}

fn drag_targets(
    window: &mut Window,
    scope: &[ElementId],
    from: ElementId,
    to: ElementId,
    cx: &mut App,
) {
    window.render_frame(cx);
    let from = target_position(window, scope, &from, None);
    let to = target_position(window, scope, &to, None);
    window.drag(from, to, cx);
}

impl TestWindowExt for Window {
    fn find(&self, id: impl Into<ElementId>) -> ElementSnapshot {
        require(self, &[], &id.into())
    }

    fn try_find(&self, id: impl Into<ElementId>) -> Option<ElementSnapshot> {
        observation::find(self, &[], &id.into())
    }

    fn find_all(&self, id: impl Into<ElementId>) -> Vec<ElementSnapshot> {
        observation::find_all(self, &[], &id.into())
    }

    fn within(&mut self, id: impl Into<ElementId>) -> ScopedWindow<'_> {
        let scope = observation::scope(self, &[], &id.into());
        ScopedWindow {
            window: self,
            scope,
        }
    }

    fn render_frame(&mut self, cx: &mut App) {
        self.refresh();
        self.draw(cx).clear(cx);
    }

    fn click(&mut self, id: impl Into<ElementId>, cx: &mut App) {
        click_target(self, &[], id.into(), ClickOptions::new(), cx);
    }

    fn click_at(&mut self, id: impl Into<ElementId>, offset: Point<Pixels>, cx: &mut App) {
        click_target(
            self,
            &[],
            id.into(),
            ClickOptions::new().with_offset(offset),
            cx,
        );
    }

    fn click_with_options(
        &mut self,
        id: impl Into<ElementId>,
        options: ClickOptions,
        cx: &mut App,
    ) {
        click_with_options_target(self, &[], id.into(), options, cx);
    }

    fn right_click(&mut self, id: impl Into<ElementId>, cx: &mut App) {
        click_target(
            self,
            &[],
            id.into(),
            ClickOptions::new().with_button(MouseButton::Right),
            cx,
        );
    }

    fn double_click(&mut self, id: impl Into<ElementId>, cx: &mut App) {
        click_target(self, &[], id.into(), ClickOptions::new().with_count(2), cx);
    }

    fn hover(&mut self, id: impl Into<ElementId>, cx: &mut App) {
        hover_target(self, &[], id.into(), cx);
    }

    fn scroll(&mut self, id: impl Into<ElementId>, delta: ScrollDelta, cx: &mut App) {
        scroll_target(self, &[], id.into(), delta, cx);
    }

    fn drag_to(&mut self, from: impl Into<ElementId>, to: impl Into<ElementId>, cx: &mut App) {
        drag_targets(self, &[], from.into(), to.into(), cx);
    }

    fn drag(&mut self, from: Point<Pixels>, to: Point<Pixels>, cx: &mut App) {
        self.render_frame(cx);
        let modifiers = self.modifiers();
        move_pointer(self, from, None, modifiers, cx);
        mouse_down(self, from, MouseButton::Left, 1, modifiers, cx);
        for step in 1..=8 {
            let fraction = step as f32 / 8.;
            move_pointer(
                self,
                point(
                    from.x + (to.x - from.x) * fraction,
                    from.y + (to.y - from.y) * fraction,
                ),
                Some(MouseButton::Left),
                modifiers,
                cx,
            );
        }
        mouse_up(self, to, MouseButton::Left, 1, modifiers, cx);
    }

    fn press(&mut self, key: &str, cx: &mut App) {
        let key = Keystroke::parse_with_os(key, cx.platform_info().os)
            .unwrap_or_else(|error| panic!("invalid test keystroke: {error}"));
        self.render_frame(cx);
        press_key(self, key, cx);
        self.render_frame(cx);
    }

    fn input(&mut self, text: &str, cx: &mut App) {
        input_text(self, text, None, cx);
    }
}

/// A borrowed GPUI [`ElementId`] scope returned by [`TestWindowExt::within`].
///
/// Restricts descendant queries and scoped keyboard checks without inserting any
/// extra layout wrapper into the element tree.
pub struct ScopedWindow<'a> {
    pub(crate) window: &'a mut Window,
    pub(crate) scope: Vec<ElementId>,
}

impl ScopedWindow<'_> {
    /// Returns the uniquely matching observed descendant in this scope.
    pub fn find(&self, id: impl Into<ElementId>) -> ElementSnapshot {
        require(self.window, &self.scope, &id.into())
    }

    /// Returns the uniquely matching observed descendant in this scope, if present.
    pub fn try_find(&self, id: impl Into<ElementId>) -> Option<ElementSnapshot> {
        observation::find(self.window, &self.scope, &id.into())
    }

    /// Returns all observed descendants matching `id` in this scope, ordered by `(y, x)`.
    pub fn find_all(&self, id: impl Into<ElementId>) -> Vec<ElementSnapshot> {
        observation::find_all(self.window, &self.scope, &id.into())
    }

    /// Narrows further to a nested [`ElementId`] scope.
    pub fn within(&mut self, id: impl Into<ElementId>) -> ScopedWindow<'_> {
        let scope = observation::scope(self.window, &self.scope, &id.into());
        ScopedWindow {
            window: self.window,
            scope,
        }
    }

    /// Dispatches a centered left click on a descendant in this scope.
    pub fn click(&mut self, id: impl Into<ElementId>, cx: &mut App) {
        click_target(self.window, &self.scope, id.into(), ClickOptions::new(), cx);
    }

    /// Clicks a descendant at a local pixel offset from its top-left corner.
    pub fn click_at(&mut self, id: impl Into<ElementId>, offset: Point<Pixels>, cx: &mut App) {
        click_target(
            self.window,
            &self.scope,
            id.into(),
            ClickOptions::new().with_offset(offset),
            cx,
        );
    }

    /// Dispatches a configurable click inside this scope, restoring prior modifiers.
    pub fn click_with_options(
        &mut self,
        id: impl Into<ElementId>,
        options: ClickOptions,
        cx: &mut App,
    ) {
        click_with_options_target(self.window, &self.scope, id.into(), options, cx);
    }

    /// Dispatches a centered left click with temporary modifiers inside this scope.
    pub fn click_with_modifiers(
        &mut self,
        id: impl Into<ElementId>,
        modifiers: Modifiers,
        cx: &mut App,
    ) {
        self.click_with_options(id, ClickOptions::new().with_modifiers(modifiers), cx);
    }

    /// Dispatches a centered right click on a descendant in this scope.
    pub fn right_click(&mut self, id: impl Into<ElementId>, cx: &mut App) {
        click_target(
            self.window,
            &self.scope,
            id.into(),
            ClickOptions::new().with_button(MouseButton::Right),
            cx,
        );
    }

    /// Dispatches a centered left double-click on a descendant in this scope.
    pub fn double_click(&mut self, id: impl Into<ElementId>, cx: &mut App) {
        click_target(
            self.window,
            &self.scope,
            id.into(),
            ClickOptions::new().with_count(2),
            cx,
        );
    }

    /// Moves the pointer to the center of a descendant in this scope.
    pub fn hover(&mut self, id: impl Into<ElementId>, cx: &mut App) {
        hover_target(self.window, &self.scope, id.into(), cx);
    }

    /// Dispatches a wheel scroll event over a descendant in this scope.
    pub fn scroll(&mut self, id: impl Into<ElementId>, delta: ScrollDelta, cx: &mut App) {
        scroll_target(self.window, &self.scope, id.into(), delta, cx);
    }

    /// Drags between two descendants resolved within this scope.
    pub fn drag_to(&mut self, from: impl Into<ElementId>, to: impl Into<ElementId>, cx: &mut App) {
        drag_targets(self.window, &self.scope, from.into(), to.into(), cx);
    }

    /// Dispatches a keystroke to current focus, requiring an observed focus binding
    /// inside this scope.
    pub fn press(&mut self, key: &str, cx: &mut App) {
        let key = Keystroke::parse_with_os(key, cx.platform_info().os)
            .unwrap_or_else(|error| panic!("invalid test keystroke: {error}"));
        self.window.render_frame(cx);
        require_scope_focus(self.window, &self.scope);
        press_key(self.window, key, cx);
        self.window.render_frame(cx);
    }

    /// Types text into current focus, checking observed focus membership inside this
    /// scope before every character.
    pub fn input(&mut self, text: &str, cx: &mut App) {
        input_text(self.window, text, Some(&self.scope), cx);
    }
}

fn require_scope_focus(window: &Window, scope: &[ElementId]) {
    assert!(
        observation::has_observed_focus(window, scope),
        "no observed keyboard focus inside scope {scope:?}; register the focused control \
         with .test_support().track_focus(&handle) inside this scope before press/input"
    );
}

fn press_key(window: &mut Window, key: Keystroke, cx: &mut App) {
    let key = if matches!(key.key.as_str(), "enter" | "tab") {
        window.dispatch_event(
            KeyDownEvent {
                keystroke: key.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        key
    } else {
        let key = key.with_simulated_ime();
        window.dispatch_keystroke(key.clone(), cx);
        key
    };
    window.dispatch_event(KeyUpEvent { keystroke: key }.to_platform_input(), cx);
}

fn input_text(window: &mut Window, text: &str, scope: Option<&[ElementId]>, cx: &mut App) {
    window.render_frame(cx);
    for character in text.chars() {
        if let Some(scope) = scope {
            require_scope_focus(window, scope);
        }
        let text = character.to_string();
        let mut key = Keystroke::parse_with_os(&text, cx.platform_info().os)
            .expect("a Unicode character is a valid GPUI keystroke");
        key.key_char = Some(text);
        window.dispatch_keystroke(key, cx);
        window.render_frame(cx);
    }
}

/// Executor-aware asynchronous helpers on [`TestAppContext`] that run outside
/// a borrowed window update.
pub trait TestAppContextExt {
    /// Repeatedly refreshes frames on the GPUI test clock (every 10 ms) until
    /// `predicate` returns `true`, or panics with all registered element paths
    /// once `timeout` elapses.
    fn wait_for(
        &mut self,
        window: AnyWindowHandle,
        timeout: Duration,
        predicate: impl FnMut(&mut Window, &mut App) -> bool,
    ) -> impl Future<Output = ()>;
}

impl TestAppContextExt for TestAppContext {
    async fn wait_for(
        &mut self,
        handle: AnyWindowHandle,
        timeout: Duration,
        mut predicate: impl FnMut(&mut Window, &mut App) -> bool,
    ) {
        let mut elapsed = Duration::ZERO;
        loop {
            let (ready, paths) = self
                .update_window(handle, |_, window, cx| {
                    window.render_frame(cx);
                    let ready = predicate(window, cx);
                    let paths = if !ready && elapsed >= timeout {
                        observation::registered_paths(window)
                    } else {
                        String::new()
                    };
                    (ready, paths)
                })
                .expect("test window closed while waiting");
            if ready {
                return;
            }
            assert!(
                elapsed < timeout,
                "UI condition timed out after {timeout:?}. Registered paths: {paths}"
            );
            let interval = Duration::from_millis(10).min(timeout - elapsed);
            self.executor().timer(interval).await;
            elapsed += interval;
        }
    }
}
