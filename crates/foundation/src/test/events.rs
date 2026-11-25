use gpui::{
    App, InputEvent, KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, MouseButton, Pixels, Point,
    Window,
};

use super::{TestWindowExt as _, change_modifiers, mouse_down, mouse_up, move_pointer};

/// Step-level pointer, modifier, and keyboard dispatch helpers for tests that
/// inspect intermediate gesture state.
pub trait TestEventExt {
    /// Dispatches a single mouse-down event at `position` and completes a frame.
    fn pointer_down(&mut self, position: Point<Pixels>, button: MouseButton, cx: &mut App);

    /// Dispatches a single pointer-move event at `position` with an explicit held
    /// button state and completes a frame.
    fn pointer_move(
        &mut self,
        position: Point<Pixels>,
        pressed_button: Option<MouseButton>,
        cx: &mut App,
    );

    /// Dispatches a single mouse-up event at `position` and completes a frame.
    fn pointer_up(&mut self, position: Point<Pixels>, button: MouseButton, cx: &mut App);

    /// Updates the window's active modifiers (preserving Caps Lock) and completes a frame.
    fn change_modifiers(&mut self, modifiers: Modifiers, cx: &mut App);

    /// Dispatches a key-down event (optionally with auto-repeat `is_held = true`)
    /// combining any parsed modifiers with the window's held modifiers.
    fn key_down(&mut self, key: &str, is_held: bool, cx: &mut App);

    /// Dispatches a matching key-up event combining any parsed modifiers with the
    /// window's held modifiers.
    fn key_up(&mut self, key: &str, cx: &mut App);
}

fn parse_with_held_modifiers(window: &Window, key: &str, cx: &App) -> Keystroke {
    let mut parsed = Keystroke::parse_with_os(key, cx.platform_info().os)
        .unwrap_or_else(|error| panic!("invalid test keystroke: {error}"));
    let held = window.modifiers();
    parsed.modifiers.control |= held.control;
    parsed.modifiers.alt |= held.alt;
    parsed.modifiers.shift |= held.shift;
    parsed.modifiers.platform |= held.platform;
    parsed.modifiers.function |= held.function;
    parsed
}

impl TestEventExt for Window {
    fn pointer_down(&mut self, position: Point<Pixels>, button: MouseButton, cx: &mut App) {
        self.render_frame(cx);
        let modifiers = self.modifiers();
        mouse_down(self, position, button, 1, modifiers, cx);
    }

    fn pointer_move(
        &mut self,
        position: Point<Pixels>,
        pressed_button: Option<MouseButton>,
        cx: &mut App,
    ) {
        self.render_frame(cx);
        let modifiers = self.modifiers();
        move_pointer(self, position, pressed_button, modifiers, cx);
    }

    fn pointer_up(&mut self, position: Point<Pixels>, button: MouseButton, cx: &mut App) {
        self.render_frame(cx);
        let modifiers = self.modifiers();
        mouse_up(self, position, button, 1, modifiers, cx);
    }

    fn change_modifiers(&mut self, modifiers: Modifiers, cx: &mut App) {
        self.render_frame(cx);
        change_modifiers(self, modifiers, cx);
    }

    fn key_down(&mut self, key: &str, is_held: bool, cx: &mut App) {
        self.render_frame(cx);
        let keystroke = parse_with_held_modifiers(self, key, cx);
        self.dispatch_event(
            KeyDownEvent {
                keystroke,
                is_held,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        self.render_frame(cx);
    }

    fn key_up(&mut self, key: &str, cx: &mut App) {
        self.render_frame(cx);
        let keystroke = parse_with_held_modifiers(self, key, cx);
        self.dispatch_event(KeyUpEvent { keystroke }.to_platform_input(), cx);
        self.render_frame(cx);
    }
}
