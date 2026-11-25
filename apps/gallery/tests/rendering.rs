//! Offscreen pixel rendering verification for GPUI Material.
//!
//! Uses GPUI's real offscreen renderer: Metal on macOS or WGPU (including Mesa
//! lavapipe) on Linux. Runs with `harness = false` so AppKit initializes on the
//! process main thread on macOS. On Windows, where the pinned `gpui-pre`
//! platform crate supplies no headless renderer, the binary reports an explicit
//! skip rather than faking pixels.

fn main() {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    rendering::run();
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    println!("rendering: skipped; GPUI does not supply a headless renderer on this platform");
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
mod rendering {
    use std::{borrow::Cow, sync::Arc};

    use gpui::{
        App, AppContext, AssetSource, Bounds, Context, Entity, HeadlessAppContext, IntoElement,
        ParentElement, Pixels, Point, Render, Result, SharedString, Size, Styled, Window,
        WindowBounds, WindowHandle, WindowOptions, div, px, size,
    };
    use gpui_base::Root;
    use gpui_material::{
        Button, Checkbox, CheckboxState, CircularProgress, Icon, IconName, LoadingIndicator,
        MaterialRoot, MaterialTheme, TextField, TextFieldState, TextFieldVariant, ThemeScope,
        foundation::color,
        test::TestWindowExt as _,
        theme::{Appearance, ColorRole, Direction},
    };

    struct EmptyAssets;

    impl AssetSource for EmptyAssets {
        fn load(&self, _path: &str) -> Result<Option<Cow<'static, [u8]>>> {
            Ok(None)
        }

        fn list(&self, _path: &str) -> Result<Vec<SharedString>> {
            Ok(Vec::new())
        }
    }

    fn headless_context(appearance: Appearance) -> HeadlessAppContext {
        let mut cx = HeadlessAppContext::with_platform(
            gpui_platform::current_platform(true).text_system(),
            Arc::new(EmptyAssets),
            gpui_platform::current_headless_renderer,
        );
        cx.update(|cx| {
            gpui_material::init(MaterialTheme::baseline(appearance), cx);
            cx.set_reduce_motion(true);
        });
        cx
    }

    fn open_headless<V: Render + 'static>(
        cx: &mut HeadlessAppContext,
        viewport: Size<Pixels>,
        build: impl FnOnce(&mut Window, &mut App) -> Entity<V>,
    ) -> WindowHandle<Root> {
        let handle = cx.update(|cx| {
            let bounds = Bounds {
                origin: Point::default(),
                size: viewport,
            };
            MaterialRoot::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    focus: false,
                    show: false,
                    ..Default::default()
                },
                cx,
                build,
            )
            .expect("open headless rendering window")
        });
        cx.simulate_window_scale_factor_change(handle.into(), 1.);
        cx.update_window(handle.into(), |_, window, cx| {
            window.activate_window();
            window.render_frame(cx);
        })
        .expect("initial headless frame");
        handle
    }

    fn capture_rgba(cx: &mut HeadlessAppContext, handle: WindowHandle<Root>) -> Vec<u8> {
        cx.update_window(handle.into(), |_, window, cx| {
            window.render_frame(cx);
        })
        .expect("render before screenshot");
        let image = cx
            .capture_screenshot(handle.into())
            .expect("offscreen renderer must produce a screenshot on macOS/Linux");
        assert!(
            image.width() > 0 && image.height() > 0,
            "captured screenshot must have non-zero dimensions"
        );
        image.into_raw()
    }

    fn differing_pixels(a: &[u8], b: &[u8]) -> usize {
        assert_eq!(
            a.len(),
            b.len(),
            "screenshots must have identical byte size"
        );
        a.chunks_exact(4)
            .zip(b.chunks_exact(4))
            .filter(|(pa, pb)| pa != pb)
            .count()
    }

    fn test_embedded_icons_filled_and_rtl() {
        struct IconScene {
            icon: IconName,
            filled: bool,
            rtl: bool,
            show_icon: bool,
        }

        impl Render for IconScene {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let surface = color(
                    MaterialTheme::baseline(Appearance::Light)
                        .colors
                        .color(ColorRole::Surface),
                );
                let content = div()
                    .size_full()
                    .bg(surface)
                    .flex()
                    .items_center()
                    .justify_center()
                    .children(self.show_icon.then(|| {
                        Icon::new(self.icon)
                            .filled(self.filled)
                            .color(ColorRole::OnSurface)
                            .mirror_in_rtl(true)
                            .size(px(36.))
                    }));
                if self.rtl {
                    ThemeScope::new(content).direction(Direction::Rtl)
                } else {
                    ThemeScope::new(content).direction(Direction::Ltr)
                }
            }
        }

        let mut cx = headless_context(Appearance::Light);
        let mut view_entity = None;
        let handle = open_headless(&mut cx, size(px(120.), px(120.)), |_, cx| {
            let v = cx.new(|_| IconScene {
                icon: IconName::Favorite,
                filled: false,
                rtl: false,
                show_icon: false,
            });
            view_entity = Some(v.clone());
            v
        });
        let view = view_entity.unwrap();

        let blank = capture_rgba(&mut cx, handle);
        cx.update(|cx| {
            view.update(cx, |s, cx| {
                s.show_icon = true;
                cx.notify();
            });
        });
        let outlined = capture_rgba(&mut cx, handle);
        let outlined_repeat = capture_rgba(&mut cx, handle);
        assert_eq!(
            outlined, outlined_repeat,
            "repeated icon renders must be deterministic"
        );
        assert!(
            differing_pixels(&blank, &outlined) > 20,
            "embedded Material Symbol SVG must rasterize visible pixels"
        );

        cx.update(|cx| {
            view.update(cx, |s, cx| {
                s.filled = true;
                cx.notify();
            });
        });
        let filled = capture_rgba(&mut cx, handle);
        assert!(
            differing_pixels(&outlined, &filled) > 20,
            "filled icon variant must rasterize different pixels from outlined variant"
        );

        cx.update(|cx| {
            view.update(cx, |s, cx| {
                s.icon = IconName::ArrowForward;
                s.filled = false;
                s.rtl = false;
                cx.notify();
            });
        });
        let arrow_ltr = capture_rgba(&mut cx, handle);
        cx.update(|cx| {
            view.update(cx, |s, cx| {
                s.rtl = true;
                cx.notify();
            });
        });
        let arrow_rtl = capture_rgba(&mut cx, handle);
        assert!(
            differing_pixels(&arrow_ltr, &arrow_rtl) > 20,
            "mirror_in_rtl(true) directional icon must change rendered pixels under RTL"
        );
    }

    fn test_checkbox_renders_in_light_and_dark_themes() {
        struct CheckboxScene {
            state: CheckboxState,
            appearance: Appearance,
        }

        impl Render for CheckboxScene {
            fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                let bg = color(
                    MaterialTheme::baseline(self.appearance)
                        .colors
                        .color(ColorRole::Surface),
                );
                let entity = cx.entity();
                div().size_full().bg(bg).p_4().flex().items_center().child(
                    Checkbox::new("agree", "Accept", self.state).on_change(move |next, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.state = next;
                            cx.notify();
                        });
                    }),
                )
            }
        }

        for appearance in [Appearance::Light, Appearance::Dark] {
            let mut cx = headless_context(appearance);
            let handle = open_headless(&mut cx, size(px(220.), px(96.)), |_, cx| {
                cx.new(|_| CheckboxScene {
                    state: CheckboxState::Unchecked,
                    appearance,
                })
            });
            let unchecked = capture_rgba(&mut cx, handle);
            let unchecked_repeat = capture_rgba(&mut cx, handle);
            assert_eq!(unchecked, unchecked_repeat);

            cx.update_window(handle.into(), |_, window, cx| {
                window.click("agree", cx);
                assert_eq!(window.find("agree").checked(), Some(true));
            })
            .unwrap();
            let checked = capture_rgba(&mut cx, handle);
            assert!(
                differing_pixels(&unchecked, &checked) > 30,
                "checked Checkbox must visibly change pixels in {appearance:?} theme"
            );
        }
    }

    fn test_text_field_renders_visible_cjk_and_latin_text() {
        struct FieldScene {
            state: Entity<TextFieldState>,
        }

        impl Render for FieldScene {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let bg = color(
                    MaterialTheme::baseline(Appearance::Light)
                        .colors
                        .color(ColorRole::Surface),
                );
                div().size_full().bg(bg).p_4().child(
                    TextField::new("name", "Project", &self.state)
                        .variant(TextFieldVariant::Outlined),
                )
            }
        }

        let mut cx = headless_context(Appearance::Light);
        let mut field_state = None;
        let handle = open_headless(&mut cx, size(px(360.), px(120.)), |window, cx| {
            let state = cx.new(|cx| TextFieldState::new(window, cx));
            field_state = Some(state.clone());
            cx.new(|_| FieldScene { state })
        });
        let state = field_state.unwrap();

        let empty_img = capture_rgba(&mut cx, handle);
        cx.update_window(handle.into(), |_, window, cx| {
            state.update(cx, |s, cx| s.replace_all("Material 测试", window, cx));
        })
        .unwrap();
        let filled_img = capture_rgba(&mut cx, handle);
        assert!(
            differing_pixels(&empty_img, &filled_img) > 40,
            "TextField value must visibly rasterize glyphs on screen"
        );
    }

    fn test_path_based_progress_and_loading_indicators_rasterize() {
        struct PathScene {
            show_controls: bool,
        }

        impl Render for PathScene {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let bg = color(
                    MaterialTheme::baseline(Appearance::Light)
                        .colors
                        .color(ColorRole::Surface),
                );
                div()
                    .size_full()
                    .bg(bg)
                    .p_4()
                    .flex()
                    .items_center()
                    .gap_4()
                    .children(self.show_controls.then(|| {
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(LoadingIndicator::new("loading", "Loading").contained(true))
                            .child(CircularProgress::new("upload", "Upload").value(0.65))
                    }))
            }
        }

        let mut cx = headless_context(Appearance::Light);
        let mut view_entity = None;
        let handle = open_headless(&mut cx, size(px(220.), px(120.)), |_, cx| {
            let v = cx.new(|_| PathScene {
                show_controls: false,
            });
            view_entity = Some(v.clone());
            v
        });
        let view = view_entity.unwrap();

        let blank = capture_rgba(&mut cx, handle);
        cx.update(|cx| {
            view.update(cx, |s, cx| {
                s.show_controls = true;
                cx.notify();
            });
        });
        let painted = capture_rgba(&mut cx, handle);
        assert!(
            differing_pixels(&blank, &painted) > 100,
            "Path-painted LoadingIndicator and CircularProgress must rasterize visible pixels"
        );
    }

    fn test_button_keyboard_focus_ring_visible_in_both_themes() {
        struct ButtonScene {
            appearance: Appearance,
        }

        impl Render for ButtonScene {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let bg = color(
                    MaterialTheme::baseline(self.appearance)
                        .colors
                        .color(ColorRole::Surface),
                );
                div()
                    .size_full()
                    .bg(bg)
                    .p_6()
                    .flex()
                    .items_center()
                    .child(Button::new("save", "Save").on_activate(|_, _| {}))
            }
        }

        for appearance in [Appearance::Light, Appearance::Dark] {
            let mut cx = headless_context(appearance);
            let handle = open_headless(&mut cx, size(px(220.), px(110.)), |_, cx| {
                cx.new(|_| ButtonScene { appearance })
            });
            let resting = capture_rgba(&mut cx, handle);

            cx.update_window(handle.into(), |_, window, cx| {
                window.press("tab", cx);
                assert_eq!(window.find("save").focused(), Some(true));
            })
            .unwrap();
            let focused = capture_rgba(&mut cx, handle);
            assert!(
                differing_pixels(&resting, &focused) > 40,
                "keyboard focus ring on Button must visibly render in {appearance:?} theme"
            );
        }
    }

    pub fn run() {
        test_embedded_icons_filled_and_rtl();
        test_checkbox_renders_in_light_and_dark_themes();
        test_text_field_renders_visible_cjk_and_latin_text();
        test_path_based_progress_and_loading_indicators_rasterize();
        test_button_keyboard_focus_ring_visible_in_both_themes();
        println!("rendering: 5 offscreen pixel checks passed");
    }
}
