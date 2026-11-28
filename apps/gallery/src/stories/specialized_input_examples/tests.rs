use std::{cell::RefCell, rc::Rc};

use gpui::{
    AppContext, ClipboardItem, ElementId, Entity, Focusable, Role, TestAppContext, WindowOptions,
    px, size,
};
use gpui_material::{
    ColorPickerEvent, MaterialRoot, MaterialTheme, NumberInputEvent, NumberStepAction,
    OtpInputEvent, material_theme, set_material_theme,
    test::{TestQueryExt, TestWindowExt},
    theme::{Appearance, Argb, ColorRole, Direction},
};

use super::SpecializedInputExamples;
use crate::story::Story;

fn part(parent: &'static str, suffix: &'static str) -> ElementId {
    (ElementId::from(parent), suffix).into()
}

fn open_story(
    story: Story,
    cx: &mut TestAppContext,
) -> (
    gpui::WindowHandle<gpui_base::Root>,
    Entity<SpecializedInputExamples>,
) {
    let (window, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let mut view = None;
        let window = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let entity = cx.new(|cx| SpecializedInputExamples::new(story, window, cx));
            view = Some(entity.clone());
            entity
        })
        .unwrap();
        (window, view.unwrap())
    });
    cx.simulate_window_resize(window.into(), size(px(800.), px(1000.)));
    cx.update_window(window.into(), |_, window, cx| window.render_frame(cx))
        .unwrap();
    (window, view)
}

#[gpui::test]
fn number_input_steps_via_pointer_and_keyboard_clamps_bounds_and_mirrors_rtl(
    cx: &mut TestAppContext,
) {
    let (window, view) = open_story(Story::NumberInput, cx);
    let events: Rc<RefCell<Vec<NumberInputEvent>>> = Rc::new(RefCell::new(Vec::new()));
    let qty = cx.update(|cx| {
        let qty = view.read(cx).quantity.clone();
        let events = events.clone();
        cx.subscribe(&qty, move |_, event: &NumberInputEvent, _| {
            events.borrow_mut().push(*event);
        })
        .detach();
        qty
    });

    cx.update_window(window.into(), |_, window, cx| {
        let spinbuttons = window.find_all_by_role(Role::SpinButton);
        assert!(!spinbuttons.is_empty());
        let dec = window.find(part("qty-input", "decrement"));
        let inc = window.find(part("qty-input", "increment"));
        assert!(dec.bounds().right() < inc.bounds().left());
        assert_eq!(qty.read(cx).value(cx), Some(4.0));

        // Pointer click increment keeps focus and increments by 1.
        window.click(part("qty-input", "increment"), cx);
        assert_eq!(qty.read(cx).value(cx), Some(5.0));
    })
    .unwrap();

    cx.update(|_| {
        assert!(
            events
                .borrow()
                .contains(&NumberInputEvent::Step(NumberStepAction::Increment))
        );
    });

    cx.update_window(window.into(), |_, window, cx| {
        // Keyboard Up / Down steps through gpui_base::NumberInput key context.
        window.press("up", cx);
        assert_eq!(qty.read(cx).value(cx), Some(6.0));
        window.press("down", cx);
        assert_eq!(qty.read(cx).value(cx), Some(5.0));

        // Precision stepping on outlined trailing price field (+0.25 -> 12.75).
        window.click(part("price-input", "increment"), cx);
        let price = view.read(cx).price.clone();
        assert_eq!(price.read(cx).text(cx).as_ref(), "12.75");

        // Clamping at bounds disables further stepping.
        qty.update(cx, |state, cx| state.set_value(20.0, window, cx));
        window.render_frame(cx);
        window.click(part("qty-input", "increment"), cx);
        assert_eq!(qty.read(cx).value(cx), Some(20.0));
        assert!(!qty.read(cx).can_increment(cx));
        assert!(qty.read(cx).can_decrement(cx));

        // Disabled number input ignores clicks.
        window.click(part("disabled-qty", "increment"), cx);
        let disabled = view.read(cx).disabled_qty.clone();
        assert_eq!(disabled.read(cx).value(cx), Some(3.0));

        // RTL flips split decrement/increment horizontally.
        let mut theme = material_theme(cx).clone();
        theme.layout.direction = Direction::Rtl;
        set_material_theme(theme, cx);
        window.render_frame(cx);
        let dec = window.find(part("qty-input", "decrement"));
        let inc = window.find(part("qty-input", "increment"));
        assert!(inc.bounds().right() < dec.bounds().left());
    })
    .unwrap();
}

#[gpui::test]
fn otp_input_handles_typing_navigation_paste_disabled_error_and_ltr_slots_in_rtl(
    cx: &mut TestAppContext,
) {
    let (window, view) = open_story(Story::OtpInput, cx);
    let completed: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let otp = cx.update(|cx| {
        let otp = view.read(cx).otp_code.clone();
        let completed = completed.clone();
        cx.subscribe(&otp, move |_, event: &OtpInputEvent, _| {
            if let OtpInputEvent::Complete(code) = event {
                completed.borrow_mut().push(code.to_string());
            }
        })
        .detach();
        otp
    });

    cx.update_window(window.into(), |_, window, cx| {
        // Click slot 0 to focus, then type 4 digits.
        window.click(part("otp-verify", "slot-0"), cx);
        for key in ["1", "2", "3", "4"] {
            window.press(key, cx);
        }
        assert_eq!(otp.read(cx).value(cx).as_ref(), "1234");
        assert_eq!(otp.read(cx).active_slot(), 4);

        // Arrow left moves active slot back, typing replaces in place.
        window.press("left", cx);
        window.press("left", cx);
        assert_eq!(otp.read(cx).active_slot(), 2);
        window.press("9", cx);
        assert_eq!(otp.read(cx).value(cx).as_ref(), "1294");

        // Backspace deletes at cursor / end.
        window.press("end", cx);
        window.press("backspace", cx);
        assert_eq!(otp.read(cx).value(cx).as_ref(), "129");

        // Clipboard paste via Ctrl+V fills all 6 digits and emits Complete.
        cx.write_to_clipboard(ClipboardItem::new_string("654-321".to_owned()));
        window.press("ctrl-v", cx);
        window.render_frame(cx);
        assert_eq!(otp.read(cx).value(cx).as_ref(), "654321");
        assert!(otp.read(cx).is_complete(cx));
    })
    .unwrap();

    cx.update(|_| {
        assert_eq!(completed.borrow().as_slice(), &["654321"]);
    });

    cx.update_window(window.into(), |_, window, cx| {
        // Disabled OTP ignores input; error OTP exposes Role::Alert caption.
        window.click(part("otp-disabled", "slot-0"), cx);
        window.press("0", cx);
        assert_eq!(
            view.read(cx).otp_disabled.read(cx).value(cx).as_ref(),
            "9081"
        );
        let alert = window.find(part("otp-error", "supporting"));
        assert_eq!(alert.role(), Some(Role::Alert));

        // Numeric OTP slots stay left-to-right even in RTL windows.
        let mut theme = material_theme(cx).clone();
        theme.layout.direction = Direction::Rtl;
        set_material_theme(theme, cx);
        window.render_frame(cx);
        let first = window.find(part("otp-verify", "slot-0"));
        let last = window.find(part("otp-verify", "slot-5"));
        assert!(first.bounds().right() < last.bounds().left());
    })
    .unwrap();
}

#[gpui::test]
fn color_picker_selects_swatches_converts_argb_for_dynamic_color_and_opens_popover(
    cx: &mut TestAppContext,
) {
    let (window, view) = open_story(Story::ColorPicker, cx);
    let changes: Rc<RefCell<Vec<Option<Argb>>>> = Rc::new(RefCell::new(Vec::new()));
    let seed = cx.update(|cx| {
        let seed = view.read(cx).seed_picker.clone();
        let changes = changes.clone();
        cx.subscribe(&seed, move |_, event: &ColorPickerEvent, _| {
            changes.borrow_mut().push(event.argb());
        })
        .detach();
        seed
    });

    cx.update_window(window.into(), |_, window, cx| {
        // Initial seed is Material baseline purple (#6750A4).
        assert_eq!(
            seed.read(cx).argb(cx),
            Some(Argb::from_rgb(0x67, 0x50, 0xA4))
        );
        assert_eq!(seed.read(cx).hex(cx).as_ref(), "#6750A4");

        // Clicking the 4th preset (index 3 = Teal #006A6A) commits Argb & works with from_seed.
        window.click(part("seed-picker", "swatch-3"), cx);
        let picked = seed.read(cx).argb(cx).expect("picked argb");
        assert_eq!(picked, Argb::from_rgb(0x00, 0x6A, 0x6A));
        assert_eq!(seed.read(cx).hex(cx).as_ref(), "#006A6A");
        let theme = MaterialTheme::from_seed(picked, Appearance::Light, 0.0);
        assert_ne!(
            theme.colors.color(ColorRole::Primary),
            MaterialTheme::default().colors.color(ColorRole::Primary)
        );
    })
    .unwrap();

    cx.update(|_| {
        assert_eq!(
            changes.borrow().last().copied().flatten(),
            Some(Argb::from_rgb(0x00, 0x6A, 0x6A))
        );
    });

    cx.update_window(window.into(), |_, window, cx| {
        // Popover trigger opens on click and closes on Escape.
        let accent = view.read(cx).accent_picker.clone();
        assert!(!accent.read(cx).is_open(cx));
        window.click(part("accent-picker", "trigger"), cx);
        assert!(accent.read(cx).is_open(cx));

        accent.read(cx).focus_handle(cx).focus(window, cx);
        window.render_frame(cx);
        window.press("escape", cx);
        assert!(!accent.read(cx).is_open(cx));

        // Disabled picker ignores clicks and never opens.
        let disabled = view.read(cx).disabled_picker.clone();
        window.click(part("disabled-picker", "trigger"), cx);
        assert!(!disabled.read(cx).is_open(cx));
    })
    .unwrap();
}
