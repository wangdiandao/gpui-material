use std::{cell::Cell, rc::Rc};

use gpui::{
    AppContext as _, Bounds, Context, Element as _, FocusHandle, Hsla, InteractiveElement as _,
    IntoElement, Modifiers, ParentElement as _, Render, RenderOnce as _, Role, Styled as _,
    TestAppContext, Window, WindowOptions, accesskit, div, point, px, size,
};
use gpui_base::Button;
use gpui_material::foundation::{color, set_window_material_theme};
use gpui_material::theme::{Argb, ColorRole};
use gpui_material::{MaterialRoot, MaterialTheme, Scrim};

#[gpui::test]
fn scrim_uses_the_window_role_and_fills_its_parent(cx: &mut TestAppContext) {
    let mut app_theme = MaterialTheme::default();
    let app_scrim = Argb::from_rgb(21, 42, 84);
    app_theme.colors.set_color(ColorRole::Scrim, app_scrim);
    let mut window_theme = app_theme.clone();
    let window_scrim = Argb::from_rgb(84, 42, 21);
    window_theme
        .colors
        .set_color(ColorRole::Scrim, window_scrim);
    cx.update(|cx| MaterialRoot::init(app_theme, cx));

    let cases = [(None, app_scrim), (Some(window_theme), window_scrim)];
    let windows = cases.map(|(theme, role)| {
        let handle = cx.update(|cx| {
            cx.open_window(WindowOptions::default(), move |window, cx| {
                if theme.is_some() {
                    set_window_material_theme(theme, window, cx);
                }
                cx.new(|_| ScrimPaint {
                    tint: None,
                    opacity: None,
                })
            })
            .unwrap()
        });
        (handle, role)
    });

    let tint = color(Argb::from_u32(0x807c3ee3));
    for (handle, role) in windows.into_iter().rev() {
        for (custom, opacity, expected) in [
            (None, None, color(role).opacity(0.32)),
            (Some(tint), Some(0.4), tint.opacity(0.4)),
            (Some(tint), Some(2.), tint),
            (Some(tint), Some(f32::INFINITY), tint),
            (Some(tint), Some(f32::NAN), tint.opacity(0.32)),
            (Some(tint), Some(-0.1), tint.opacity(0.)),
            (Some(tint), Some(f32::NEG_INFINITY), tint.opacity(0.)),
        ] {
            // Let the root update finish before drawing it.
            handle
                .update(cx, |view, _, cx| {
                    view.tint = custom;
                    view.opacity = opacity;
                    cx.notify();
                })
                .unwrap();
            cx.update_window(handle.into(), |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
                let bounds = Bounds::new(point(px(23.), px(17.)), size(px(84.), px(65.)))
                    .scale(window.scale_factor());
                let quads = window.painted_quads();
                let mut backgrounds = quads
                    .iter()
                    .filter(|quad| quad.bounds == bounds)
                    .map(|quad| quad.background);
                if expected.a == 0. {
                    assert!(backgrounds.all(|fill| fill == expected.into()));
                } else {
                    assert_eq!(backgrounds.next(), Some(expected.into()));
                    assert!(backgrounds.next().is_none());
                }
            })
            .unwrap();
        }
    }
}

struct ScrimPaint {
    tint: Option<Hsla>,
    opacity: Option<f32>,
}

impl Render for ScrimPaint {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let mut scrim = Scrim::new("scrim");
        if let Some(tint) = self.tint {
            scrim = scrim.color(tint);
        }
        if let Some(opacity) = self.opacity {
            scrim = scrim.opacity(opacity);
        }
        div().size_full().child(
            div()
                .absolute()
                .left(px(23.))
                .top(px(17.))
                .w(px(84.))
                .h(px(65.))
                .child(scrim),
        )
    }
}

struct ScrimInteraction {
    foreground_focus: FocusHandle,
    background_clicks: Rc<Cell<usize>>,
    scrim_clicks: Rc<Cell<usize>>,
    interactive: bool,
}

impl Render for ScrimInteraction {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let background_clicks = self.background_clicks.clone();
        let mut scrim = Scrim::new("scrim").accessibility_label("Dismiss dialog");
        if self.interactive {
            let scrim_clicks = self.scrim_clicks.clone();
            scrim = scrim.on_click(move |_, _| scrim_clicks.set(scrim_clicks.get() + 1));
        }
        div()
            .size(px(100.))
            .child(
                Button::new("background")
                    .absolute()
                    .inset_0()
                    .on_click(move |_, _, _| background_clicks.set(background_clicks.get() + 1)),
            )
            .child(scrim)
            .child(
                Button::new("foreground")
                    .track_focus(&self.foreground_focus)
                    .absolute()
                    .top(px(25.))
                    .left(px(25.))
                    .size(px(50.))
                    .occlude(),
            )
    }
}

#[gpui::test]
fn scrim_occludes_pointer_input_only_when_interactive(cx: &mut TestAppContext) {
    cx.update(|cx| MaterialRoot::init(MaterialTheme::default(), cx));
    let background_clicks = Rc::new(Cell::new(0));
    let scrim_clicks = Rc::new(Cell::new(0));
    let (view, cx) = cx.add_window_view(|_, cx| ScrimInteraction {
        foreground_focus: cx.focus_handle(),
        background_clicks: background_clicks.clone(),
        scrim_clicks: scrim_clicks.clone(),
        interactive: false,
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.simulate_click(point(px(10.), px(10.)), Modifiers::default());
    assert_eq!(background_clicks.get(), 1);
    assert_eq!(scrim_clicks.get(), 0);

    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.interactive = true;
            view.foreground_focus.focus(window, cx);
            cx.notify();
        });
        window.refresh();
        window.draw(cx).clear(cx);
    });
    cx.simulate_click(point(px(10.), px(10.)), Modifiers::default());
    assert_eq!(background_clicks.get(), 1);
    assert_eq!(scrim_clicks.get(), 1);
    cx.update(|window, cx| assert!(view.read(cx).foreground_focus.is_focused(window)));
    cx.simulate_click(point(px(50.), px(50.)), Modifiers::default());
    assert_eq!(scrim_clicks.get(), 1);
    assert_eq!(background_clicks.get(), 1);
}

#[gpui::test]
fn scrim_exposes_an_accessibility_click_only_when_interactive(cx: &mut TestAppContext) {
    cx.update(|cx| MaterialRoot::init(MaterialTheme::default(), cx));
    let (_, cx) = cx.add_window_view(|_, _| ScrimPaint {
        tint: None,
        opacity: None,
    });
    cx.update(|window, cx| {
        let decorative = Scrim::new("decorative")
            .accessibility_label("Dismiss dialog")
            .render(window, cx)
            .into_element();
        let mut node = accesskit::Node::new(Role::Unknown);
        decorative.write_a11y_info(&mut node);
        assert_eq!(decorative.a11y_role(), None);
        assert_eq!(node.label(), None);
        assert!(!node.supports_action(accesskit::Action::Click));

        for label in [None, Some("Dismiss dialog")] {
            let mut scrim = Scrim::new("interactive").on_click(|_, _| {});
            if let Some(label) = label {
                scrim = scrim.accessibility_label(label);
            }
            let interactive = scrim.render(window, cx).into_element();
            assert_eq!(interactive.a11y_role(), Some(Role::Button));
            let mut node = accesskit::Node::new(Role::Button);
            interactive.write_a11y_info(&mut node);
            assert_eq!(node.label(), Some(label.unwrap_or("Dismiss overlay")));
            assert!(node.supports_action(accesskit::Action::Click));
            assert!(!node.supports_action(accesskit::Action::Focus));
        }
    });
}
