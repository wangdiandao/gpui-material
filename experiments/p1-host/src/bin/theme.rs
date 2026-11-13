use gpui::{
    App, AppContext, Context, IntoElement, Render, Role, Styled, TitlebarOptions, Window,
    WindowOptions, div, prelude::*,
};
use gpui_base::{Button, ColorTokens, TestSupportExt, Theme, ThemeAppearance};
use p1_host::open_probe;

struct ThemeProbe {
    local_dark: bool,
}

fn toggle_app_theme(cx: &mut App) {
    let theme = Theme::global_mut(cx);
    match theme.appearance {
        ThemeAppearance::Light => {
            theme.appearance = ThemeAppearance::Dark;
            theme.tokens.colors = ColorTokens::dark();
        }
        ThemeAppearance::Dark => {
            theme.appearance = ThemeAppearance::Light;
            theme.tokens.colors = ColorTokens::light();
        }
    }
    cx.refresh_windows();
}

impl Render for ThemeProbe {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::global(cx);
        let local = if self.local_dark {
            ColorTokens::dark()
        } else {
            theme.tokens.colors.clone()
        };
        let colors = theme.tokens.colors;
        let status = format!(
            "App {:?} · 局部 {}",
            theme.appearance,
            if self.local_dark {
                "深色"
            } else {
                "跟随 App"
            }
        );
        div()
            .size_full()
            .flex()
            .flex_col()
            .items_start()
            .gap_2()
            .p_4()
            .bg(colors.background)
            .text_color(colors.foreground)
            .child(
                Button::new("switch-app")
                    .border_1()
                    .border_color(colors.border)
                    .px_3()
                    .py_2()
                    .focus_visible(|style| style.border_color(colors.ring))
                    .accessibility_label("切换应用主题")
                    .child("切换应用主题")
                    .on_click(|_, _, cx| toggle_app_theme(cx)),
            )
            .child(
                div()
                    .id("projection")
                    .role(Role::Status)
                    .test_support()
                    .aria_label(status.clone())
                    .p_2()
                    .bg(local.background)
                    .text_color(local.foreground)
                    .child(status),
            )
    }
}

fn main() {
    gpui_platform::application().run(|cx| {
        gpui_base::init(cx);
        for (title, local_dark) in [("P1-08 · 跟随 App", false), ("P1-08 · 局部深色", true)]
        {
            let options = WindowOptions {
                titlebar: Some(TitlebarOptions {
                    title: Some(title.into()),
                    ..Default::default()
                }),
                ..Default::default()
            };
            open_probe(cx, options, |_, cx| cx.new(|_| ThemeProbe { local_dark })).unwrap();
        }
    });
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn two_windows_read_one_app_projection(cx: &mut gpui::TestAppContext) {
    use gpui_base::test_support;

    let (first, second) = cx.update(|cx| {
        gpui_base::init(cx);
        let first = open_probe(cx, WindowOptions::default(), |_, cx| {
            cx.new(|_| ThemeProbe { local_dark: false })
        })
        .unwrap();
        let second = open_probe(cx, WindowOptions::default(), |_, cx| {
            cx.new(|_| ThemeProbe { local_dark: true })
        })
        .unwrap();
        (first, second)
    });

    let label = |cx: &mut gpui::TestAppContext, window: gpui::WindowHandle<gpui_base::Root>| {
        cx.update_window(window.into(), |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
            test_support::find(window, &[], &"projection".into())
                .unwrap()
                .label()
                .unwrap()
                .to_owned()
        })
        .unwrap()
    };
    assert_eq!(label(cx, first), "App Light · 局部 跟随 App");
    assert_eq!(label(cx, second), "App Light · 局部 深色");
    assert_eq!(
        cx.update(|cx| Theme::global(cx).appearance),
        ThemeAppearance::Light
    );
    cx.update(toggle_app_theme);
    assert_eq!(label(cx, first), "App Dark · 局部 跟随 App");
    assert_eq!(label(cx, second), "App Dark · 局部 深色");
    assert_eq!(
        cx.update(|cx| Theme::global(cx).appearance),
        ThemeAppearance::Dark
    );
}

#[cfg(all(test, feature = "test-support"))]
#[gpui::test]
fn calendar_respects_a_disabled_leap_day(cx: &mut gpui::TestAppContext) {
    use gpui_base::{CalendarState, Date, Matcher};

    cx.open_window(gpui::size(gpui::px(480.), gpui::px(280.)), |window, cx| {
        let jan31 = Date::Single(Some("2024-01-31".parse().unwrap()));
        let leap_day = "2024-02-29".parse().unwrap();
        let blocked = Matcher::range(Some(leap_day), Some(leap_day));
        assert!(blocked.matched(&leap_day));
        let mut calendar = CalendarState::new(window, cx).disabled_matcher(blocked);
        calendar.set_date(jan31, window, cx);
        assert!(!calendar.select_date(leap_day));
        assert_eq!(calendar.date(), jan31);
        calendar.set_disabled_matcher_shared(None);
        assert!(calendar.select_date(leap_day));
        assert_eq!(calendar.date(), Date::Single(Some(leap_day)));
        calendar
    });
}
