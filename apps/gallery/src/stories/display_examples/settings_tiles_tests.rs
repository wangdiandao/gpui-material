mod choice_and_slider;
mod section_and_slots;
mod toggle_tiles;

use gpui::{
    App, InputEvent, KeyDownEvent, KeyUpEvent, Keystroke, MouseButton, MouseDownEvent,
    MouseUpEvent, Pixels, Point, Window,
};

fn click_at(position: Point<Pixels>, window: &mut Window, cx: &mut App) {
    window.dispatch_event(
        MouseDownEvent {
            button: MouseButton::Left,
            position,
            click_count: 1,
            first_mouse: false,
            modifiers: Default::default(),
        }
        .to_platform_input(),
        cx,
    );
    window.dispatch_event(
        MouseUpEvent {
            button: MouseButton::Left,
            position,
            click_count: 1,
            modifiers: Default::default(),
        }
        .to_platform_input(),
        cx,
    );
}

fn press_key(name: &'static str, window: &mut Window, cx: &mut App) {
    let keystroke = Keystroke::parse(name).unwrap();
    window.dispatch_event(
        KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        }
        .to_platform_input(),
        cx,
    );
    window.dispatch_event(KeyUpEvent { keystroke }.to_platform_input(), cx);
}
