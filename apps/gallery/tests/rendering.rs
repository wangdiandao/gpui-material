//! Offscreen pixel rendering verification for GPUI Material.
//!
//! Uses GPUI's real offscreen renderer: DirectX hidden-window readback (with
//! WARP software fallback) on Windows, Metal on macOS, or WGPU (including Mesa
//! lavapipe) on Linux. Runs with `harness = false` so platform event loops and
//! AppKit initialize on the process main thread.

fn main() {
    #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
    rendering::run();
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    println!("rendering: skipped; GPUI does not supply an offscreen renderer on this platform");
}

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
mod rendering {
    use std::{borrow::Cow, ops::Range};

    use gpui::{
        App, AppContext, AssetSource, Bounds, Context, Entity, IntoElement, ParentElement, Pixels,
        Point, Render, Result, SharedString, Size, Styled, Window, WindowBounds, WindowHandle,
        WindowOptions, div, px, size,
    };
    use gpui_base::Root;
    use gpui_material::{
        Button, Checkbox, CheckboxState, CircularProgress, Icon, IconName, ListItem,
        LoadingIndicator, MaterialRoot, MaterialShape, MaterialTheme, Surface, TextField,
        TextFieldState, TextFieldVariant, ThemeScope,
        foundation::color,
        test::TestWindowExt as _,
        theme::{Appearance, Argb, ColorRole, CornerShape, Direction, Elevation},
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

    #[derive(Clone, PartialEq, Eq)]
    struct RgbaFrame {
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    }

    impl RgbaFrame {
        fn new(width: u32, height: u32, rgba: Vec<u8>) -> Self {
            assert!(
                width > 0 && height > 0,
                "captured screenshot must have non-zero dimensions"
            );
            assert_eq!(
                rgba.len(),
                (width as usize) * (height as usize) * 4,
                "RGBA buffer length must match width * height * 4"
            );
            Self {
                width,
                height,
                rgba,
            }
        }

        fn differing_pixels(&self, other: &Self) -> usize {
            assert_eq!(
                (self.width, self.height),
                (other.width, other.height),
                "screenshots must have identical dimensions"
            );
            self.rgba
                .chunks_exact(4)
                .zip(other.rgba.chunks_exact(4))
                .filter(|(pa, pb)| pa != pb)
                .count()
        }

        fn mean_luma(&self) -> f64 {
            let total: f64 = self
                .rgba
                .chunks_exact(4)
                .map(|px| {
                    0.299 * f64::from(px[0]) + 0.587 * f64::from(px[1]) + 0.114 * f64::from(px[2])
                })
                .sum();
            total / (f64::from(self.width) * f64::from(self.height))
        }

        fn pixel_at(&self, x: u32, y: u32) -> [u8; 4] {
            let idx = ((y as usize) * (self.width as usize) + (x as usize)) * 4;
            [
                self.rgba[idx],
                self.rgba[idx + 1],
                self.rgba[idx + 2],
                self.rgba[idx + 3],
            ]
        }

        fn count_in_x_range(&self, xs: Range<u32>, predicate: impl Fn([u8; 4]) -> bool) -> usize {
            let mut count = 0;
            for y in 0..self.height {
                for x in xs.clone() {
                    if predicate(self.pixel_at(x, y)) {
                        count += 1;
                    }
                }
            }
            count
        }
    }

    fn channels_close(a: [u8; 4], b: [u8; 4], tolerance: u8) -> bool {
        a.iter()
            .zip(b.iter())
            .take(3)
            .all(|(&ca, &cb)| ca.abs_diff(cb) <= tolerance)
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    type SessionBackend = gpui::HeadlessAppContext;

    #[cfg(target_os = "windows")]
    type SessionBackend = gpui::AsyncApp;

    struct SceneSession<'a, V> {
        backend: &'a mut SessionBackend,
        handle: WindowHandle<Root>,
        view: Entity<V>,
    }

    impl<V: 'static> SceneSession<'_, V> {
        fn update_view<R>(&mut self, f: impl FnOnce(&mut V, &mut Context<V>) -> R) -> R {
            let view = self.view.clone();
            self.backend.update(|cx| view.update(cx, f))
        }

        fn update_window<R>(&mut self, f: impl FnOnce(&mut Window, &mut App) -> R) -> R {
            self.backend
                .update_window(self.handle.into(), |_, window, cx| f(window, cx))
                .expect("update offscreen window")
        }

        #[cfg(any(target_os = "macos", target_os = "linux"))]
        fn capture_rgba(&mut self) -> RgbaFrame {
            self.backend
                .update_window(self.handle.into(), |_, window, cx| {
                    window.render_frame(cx);
                })
                .expect("render before screenshot");
            let image = self
                .backend
                .capture_screenshot(self.handle.into())
                .expect("offscreen renderer must produce a screenshot on macOS/Linux");
            RgbaFrame::new(image.width(), image.height(), image.into_raw())
        }

        #[cfg(target_os = "windows")]
        fn capture_rgba(&mut self) -> RgbaFrame {
            let image = self
                .backend
                .update_window(self.handle.into(), |_, window, cx| {
                    window.render_frame(cx);
                    window.render_to_image()
                })
                .expect("render before DirectX screenshot")
                .expect("DirectX offscreen renderer must produce a screenshot on Windows");
            RgbaFrame::new(image.width(), image.height(), image.into_raw())
        }
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    struct Runner;

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    impl Runner {
        async fn with_scene<V: Render + 'static, R>(
            &mut self,
            theme: MaterialTheme,
            viewport: Size<Pixels>,
            build: impl FnOnce(&mut Window, &mut App) -> Entity<V>,
            run: impl FnOnce(&mut SceneSession<'_, V>) -> R,
        ) -> R {
            use std::sync::Arc;

            let mut cx = gpui::HeadlessAppContext::with_platform(
                gpui_platform::current_platform(true).text_system(),
                Arc::new(EmptyAssets),
                gpui_platform::current_headless_renderer,
            );
            cx.update(|cx| {
                gpui_material::init(theme, cx);
                cx.set_reduce_motion(true);
            });
            let mut view_entity = None;
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
                    |window, cx| {
                        let view = build(window, cx);
                        view_entity = Some(view.clone());
                        view
                    },
                )
                .expect("open headless rendering window")
            });
            cx.simulate_window_scale_factor_change(handle.into(), 1.);
            cx.update_window(handle.into(), |_, window, cx| {
                window.activate_window();
                window.render_frame(cx);
            })
            .expect("initial headless frame");

            let mut session = SceneSession {
                backend: &mut cx,
                handle,
                view: view_entity.expect("scene root view"),
            };
            run(&mut session)
        }
    }

    #[cfg(target_os = "windows")]
    struct Runner {
        cx: gpui::AsyncApp,
    }

    #[cfg(target_os = "windows")]
    impl Runner {
        async fn with_scene<V: Render + 'static, R>(
            &mut self,
            theme: MaterialTheme,
            viewport: Size<Pixels>,
            build: impl FnOnce(&mut Window, &mut App) -> Entity<V>,
            run: impl FnOnce(&mut SceneSession<'_, V>) -> R,
        ) -> R {
            let mut view_entity = None;
            let handle = self.cx.update(|cx| {
                gpui_material::set_material_theme(theme, cx);
                let bounds = Bounds {
                    origin: Point::default(),
                    size: viewport,
                };
                MaterialRoot::open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(bounds)),
                        kind: gpui::WindowKind::PopUp,
                        is_resizable: false,
                        focus: false,
                        show: false,
                        ..Default::default()
                    },
                    cx,
                    |window, cx| {
                        let view = build(window, cx);
                        view_entity = Some(view.clone());
                        view
                    },
                )
                .expect("open hidden DirectX rendering window")
            });

            // Resize the hidden HWND before capturing so DirectXRenderer matches `viewport`
            // without ever showing the window (`SetWindowPos` runs on the foreground queue).
            self.cx
                .update_window(handle.into(), |_, window, _| {
                    window.resize(viewport);
                })
                .expect("resize hidden DirectX window");
            self.cx.spawn(async move |_| {}).await;

            self.cx
                .update_window(handle.into(), |_, window, cx| {
                    assert!(
                        !window.is_visible(),
                        "Windows DirectX offscreen test window must remain hidden"
                    );
                    let actual = window.viewport_size();
                    assert!(
                        (f32::from(actual.width) - f32::from(viewport.width)).abs() <= 2.0
                            && (f32::from(actual.height) - f32::from(viewport.height)).abs() <= 2.0,
                        "hidden DirectX window viewport {actual:?} must match requested {viewport:?}"
                    );
                    window.render_frame(cx);
                })
                .expect("initial DirectX offscreen frame");

            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut session = SceneSession {
                    backend: &mut self.cx,
                    handle,
                    view: view_entity.expect("scene root view"),
                };
                run(&mut session)
            }));

            let _ = self.cx.update_window(handle.into(), |_, window, _| {
                window.remove_window();
            });
            self.cx.spawn(async move |_| {}).await;

            match outcome {
                Ok(result) => result,
                Err(payload) => std::panic::resume_unwind(payload),
            }
        }
    }

    async fn test_embedded_icons_filled_and_rtl(runner: &mut Runner) {
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

        runner
            .with_scene(
                MaterialTheme::baseline(Appearance::Light),
                size(px(120.), px(120.)),
                |_, cx| {
                    cx.new(|_| IconScene {
                        icon: IconName::Favorite,
                        filled: false,
                        rtl: false,
                        show_icon: false,
                    })
                },
                |session| {
                    let blank = session.capture_rgba();
                    session.update_view(|s, cx| {
                        s.show_icon = true;
                        cx.notify();
                    });
                    let outlined = session.capture_rgba();
                    let outlined_repeat = session.capture_rgba();
                    assert_eq!(
                        outlined.rgba, outlined_repeat.rgba,
                        "repeated icon renders must be deterministic"
                    );
                    assert!(
                        blank.differing_pixels(&outlined) > 20,
                        "embedded Material Symbol SVG must rasterize visible pixels"
                    );

                    session.update_view(|s, cx| {
                        s.filled = true;
                        cx.notify();
                    });
                    let filled = session.capture_rgba();
                    assert!(
                        outlined.differing_pixels(&filled) > 20,
                        "filled icon variant must rasterize different pixels from outlined variant"
                    );

                    session.update_view(|s, cx| {
                        s.icon = IconName::ArrowForward;
                        s.filled = false;
                        s.rtl = false;
                        cx.notify();
                    });
                    let arrow_ltr = session.capture_rgba();
                    session.update_view(|s, cx| {
                        s.rtl = true;
                        cx.notify();
                    });
                    let arrow_rtl = session.capture_rgba();
                    assert!(
                        arrow_ltr.differing_pixels(&arrow_rtl) > 20,
                        "mirror_in_rtl(true) directional icon must change rendered pixels under RTL"
                    );
                },
            )
            .await;
    }

    async fn test_checkbox_renders_in_light_and_dark_themes(runner: &mut Runner) {
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
            runner
                .with_scene(
                    MaterialTheme::baseline(appearance),
                    size(px(220.), px(96.)),
                    |_, cx| {
                        cx.new(|_| CheckboxScene {
                            state: CheckboxState::Unchecked,
                            appearance,
                        })
                    },
                    |session| {
                        let unchecked = session.capture_rgba();
                        let unchecked_repeat = session.capture_rgba();
                        assert_eq!(unchecked.rgba, unchecked_repeat.rgba);

                        session.update_window(|window, cx| {
                            window.click("agree", cx);
                            assert_eq!(window.find("agree").checked(), Some(true));
                        });
                        let checked = session.capture_rgba();
                        assert!(
                            unchecked.differing_pixels(&checked) > 30,
                            "checked Checkbox must visibly change pixels in {appearance:?} theme"
                        );
                    },
                )
                .await;
        }
    }

    async fn test_text_field_renders_visible_cjk_and_latin_text(runner: &mut Runner) {
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

        runner
            .with_scene(
                MaterialTheme::baseline(Appearance::Light),
                size(px(360.), px(120.)),
                |window, cx| {
                    let state = cx.new(|cx| TextFieldState::new(window, cx));
                    cx.new(|_| FieldScene { state })
                },
                |session| {
                    let state = session.update_view(|s, _| s.state.clone());
                    let empty_img = session.capture_rgba();
                    session.update_window(|window, cx| {
                        state.update(cx, |s, cx| s.replace_all("Material 测试", window, cx));
                    });
                    let filled_img = session.capture_rgba();
                    assert!(
                        empty_img.differing_pixels(&filled_img) > 40,
                        "TextField value must visibly rasterize glyphs on screen"
                    );
                },
            )
            .await;
    }

    async fn test_path_based_progress_and_loading_indicators_rasterize(runner: &mut Runner) {
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

        runner
            .with_scene(
                MaterialTheme::baseline(Appearance::Light),
                size(px(220.), px(120.)),
                |_, cx| {
                    cx.new(|_| PathScene {
                        show_controls: false,
                    })
                },
                |session| {
                    let blank = session.capture_rgba();
                    session.update_view(|s, cx| {
                        s.show_controls = true;
                        cx.notify();
                    });
                    let painted = session.capture_rgba();
                    assert!(
                        blank.differing_pixels(&painted) > 100,
                        "Path-painted LoadingIndicator and CircularProgress must rasterize visible pixels"
                    );
                },
            )
            .await;
    }

    async fn test_button_keyboard_focus_ring_visible_in_both_themes(runner: &mut Runner) {
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
            runner
                .with_scene(
                    MaterialTheme::baseline(appearance),
                    size(px(220.), px(110.)),
                    |_, cx| cx.new(|_| ButtonScene { appearance }),
                    |session| {
                        let resting = session.capture_rgba();
                        session.update_window(|window, cx| {
                            window.press("tab", cx);
                            assert_eq!(window.find("save").focused(), Some(true));
                        });
                        let focused = session.capture_rgba();
                        assert!(
                            resting.differing_pixels(&focused) > 40,
                            "keyboard focus ring on Button must visibly render in {appearance:?} theme"
                        );
                    },
                )
                .await;
        }
    }

    async fn test_expressive_polygon_surface_and_shape_clip_backdrop(runner: &mut Runner) {
        #[derive(Clone, Copy)]
        enum Mode {
            Rectangle,
            Cookie,
            CookieWithOverflowUnclipped,
            CookieWithOverflowClipped,
        }

        struct PolygonScene {
            mode: Mode,
        }

        impl Render for PolygonScene {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let theme = MaterialTheme::baseline(Appearance::Light);
                let backdrop = color(theme.colors.color(ColorRole::Surface));
                let overflow_fill = color(theme.colors.color(ColorRole::Error));

                let surface = match self.mode {
                    Mode::Rectangle => Surface::new(ColorRole::PrimaryContainer)
                        .w(px(120.))
                        .h(px(120.)),
                    Mode::Cookie => Surface::new(ColorRole::PrimaryContainer)
                        .polygon_shape(MaterialShape::Cookie9Sided)
                        .w(px(120.))
                        .h(px(120.)),
                    Mode::CookieWithOverflowUnclipped => Surface::new(ColorRole::PrimaryContainer)
                        .polygon_shape(MaterialShape::Cookie9Sided)
                        .w(px(120.))
                        .h(px(120.))
                        .child(div().size_full().bg(overflow_fill)),
                    Mode::CookieWithOverflowClipped => Surface::new(ColorRole::PrimaryContainer)
                        .polygon_shape(MaterialShape::Cookie9Sided)
                        .shape_clip_backdrop(backdrop)
                        .w(px(120.))
                        .h(px(120.))
                        .child(div().size_full().bg(overflow_fill)),
                };

                div()
                    .size_full()
                    .bg(backdrop)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(surface)
            }
        }

        runner
            .with_scene(
                MaterialTheme::baseline(Appearance::Light),
                size(px(160.), px(160.)),
                |_, cx| {
                    cx.new(|_| PolygonScene {
                        mode: Mode::Rectangle,
                    })
                },
                |session| {
                    let rect = session.capture_rgba();

                    session.update_view(|s, cx| {
                        s.mode = Mode::Cookie;
                        cx.notify();
                    });
                    let cookie = session.capture_rgba();
                    let cookie_repeat = session.capture_rgba();
                    assert_eq!(
                        cookie.rgba, cookie_repeat.rgba,
                        "Expressive polygon Surface renders must be deterministic"
                    );
                    assert!(
                        rect.differing_pixels(&cookie) > 200,
                        "MaterialShape::Cookie9Sided Surface must rasterize a distinct contour from a rectangle"
                    );

                    session.update_view(|s, cx| {
                        s.mode = Mode::CookieWithOverflowUnclipped;
                        cx.notify();
                    });
                    let unclipped = session.capture_rgba();

                    session.update_view(|s, cx| {
                        s.mode = Mode::CookieWithOverflowClipped;
                        cx.notify();
                    });
                    let clipped = session.capture_rgba();
                    assert!(
                        unclipped.differing_pixels(&clipped) > 200,
                        "shape_clip_backdrop must clip overflowing child pixels outside the polygon contour"
                    );

                    // Top-left corner of the 120x120 box inside the 160x160 viewport lies outside
                    // the inscribed 9-sided cookie and must be masked back to the uniform backdrop.
                    let backdrop_sample = clipped.pixel_at(2, 2);
                    let inset_corner = clipped.pixel_at(
                        (clipped.width * 24) / 160,
                        (clipped.height * 24) / 160,
                    );
                    assert!(
                        channels_close(backdrop_sample, inset_corner, 4),
                        "corner outside Cookie9Sided contour must match backdrop ({backdrop_sample:?} vs {inset_corner:?})"
                    );
                },
            )
            .await;
    }

    async fn test_high_contrast_and_tonal_elevation_differentiation(runner: &mut Runner) {
        struct SurfaceScene {
            elevation: Elevation,
        }

        impl Render for SurfaceScene {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                Surface::new(ColorRole::Surface)
                    .tonal_elevation(self.elevation)
                    .size_full()
                    .p_4()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        Surface::new(ColorRole::PrimaryContainer)
                            .shape(CornerShape::Full)
                            .w(px(140.))
                            .h(px(48.)),
                    )
            }
        }

        let seed = Argb::from_u32(0xFF6750A4);
        let light_theme = MaterialTheme::from_seed(seed, Appearance::Light, 0.0);
        let dark_theme = MaterialTheme::from_seed(seed, Appearance::Dark, 0.0);
        let high_contrast_light = MaterialTheme::from_seed(seed, Appearance::Light, 1.0);

        let (light_level0, light_level5) = runner
            .with_scene(
                light_theme,
                size(px(200.), px(120.)),
                |_, cx| {
                    cx.new(|_| SurfaceScene {
                        elevation: Elevation::Level0,
                    })
                },
                |session| {
                    let level0 = session.capture_rgba();
                    session.update_view(|s, cx| {
                        s.elevation = Elevation::Level5;
                        cx.notify();
                    });
                    let level5 = session.capture_rgba();
                    (level0, level5)
                },
            )
            .await;

        let (dark_level0, dark_level5) = runner
            .with_scene(
                dark_theme,
                size(px(200.), px(120.)),
                |_, cx| {
                    cx.new(|_| SurfaceScene {
                        elevation: Elevation::Level0,
                    })
                },
                |session| {
                    let level0 = session.capture_rgba();
                    session.update_view(|s, cx| {
                        s.elevation = Elevation::Level5;
                        cx.notify();
                    });
                    let level5 = session.capture_rgba();
                    (level0, level5)
                },
            )
            .await;

        let high_contrast_frame = runner
            .with_scene(
                high_contrast_light,
                size(px(200.), px(120.)),
                |_, cx| {
                    cx.new(|_| SurfaceScene {
                        elevation: Elevation::Level0,
                    })
                },
                |session| session.capture_rgba(),
            )
            .await;

        assert!(
            light_level0.mean_luma() > dark_level0.mean_luma() + 80.0,
            "Light Surface must rasterize significantly brighter than Dark Surface ({:.1} vs {:.1})",
            light_level0.mean_luma(),
            dark_level0.mean_luma()
        );
        assert!(
            light_level0.differing_pixels(&light_level5) > 500,
            "Elevation::Level5 tonal tint must visibly differentiate pixels in Light theme"
        );
        assert!(
            dark_level0.differing_pixels(&dark_level5) > 500,
            "Elevation::Level5 tonal tint must visibly differentiate pixels in Dark theme"
        );
        assert!(
            light_level0.differing_pixels(&high_contrast_frame) > 500,
            "High-contrast dynamic scheme must visibly differentiate pixels from standard contrast"
        );
    }

    async fn test_rtl_layout_mirroring(runner: &mut Runner) {
        struct RtlListScene {
            rtl: bool,
        }

        impl Render for RtlListScene {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let theme = MaterialTheme::baseline(Appearance::Light);
                let surface = color(theme.colors.color(ColorRole::Surface));
                let marker = color(theme.colors.color(ColorRole::Error));

                let content = div().size_full().bg(surface).flex().items_center().child(
                    div().w_full().child(
                        ListItem::new("inbox", "Inbox")
                            .leading(div().w(px(36.)).h(px(36.)).bg(marker)),
                    ),
                );
                if self.rtl {
                    ThemeScope::new(content).direction(Direction::Rtl)
                } else {
                    ThemeScope::new(content).direction(Direction::Ltr)
                }
            }
        }

        runner
            .with_scene(
                MaterialTheme::baseline(Appearance::Light),
                size(px(240.), px(96.)),
                |_, cx| cx.new(|_| RtlListScene { rtl: false }),
                |session| {
                    let ltr = session.capture_rgba();
                    session.update_view(|s, cx| {
                        s.rtl = true;
                        cx.notify();
                    });
                    let rtl = session.capture_rgba();

                    assert!(
                        ltr.differing_pixels(&rtl) > 400,
                        "ThemeScope::direction(Direction::Rtl) must visibly mirror ListItem layout"
                    );

                    // The error-colored leading marker is strongly red-dominant compared to the
                    // neutral surface; verify it moves from the left third (LTR) to the right third (RTL).
                    let is_marker = |px: [u8; 4]| {
                        i16::from(px[0]) - i16::from(px[1]) > 50
                            && i16::from(px[0]) - i16::from(px[2]) > 50
                    };
                    let third = ltr.width / 3;
                    let ltr_left = ltr.count_in_x_range(0..third, is_marker);
                    let ltr_right = ltr.count_in_x_range((ltr.width - third)..ltr.width, is_marker);
                    let rtl_left = rtl.count_in_x_range(0..third, is_marker);
                    let rtl_right = rtl.count_in_x_range((rtl.width - third)..rtl.width, is_marker);

                    assert!(
                        ltr_left > ltr_right + 100,
                        "LTR leading marker must render on the left side (left={ltr_left}, right={ltr_right})"
                    );
                    assert!(
                        rtl_right > rtl_left + 100,
                        "RTL leading marker must mirror to the right side (left={rtl_left}, right={rtl_right})"
                    );
                },
            )
            .await;
    }

    async fn run_all_checks(runner: &mut Runner) -> usize {
        test_embedded_icons_filled_and_rtl(runner).await;
        test_checkbox_renders_in_light_and_dark_themes(runner).await;
        test_text_field_renders_visible_cjk_and_latin_text(runner).await;
        test_path_based_progress_and_loading_indicators_rasterize(runner).await;
        test_button_keyboard_focus_ring_visible_in_both_themes(runner).await;
        test_expressive_polygon_surface_and_shape_clip_backdrop(runner).await;
        test_high_contrast_and_tonal_elevation_differentiation(runner).await;
        test_rtl_layout_mirroring(runner).await;
        8
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn block_on_ready<F: std::future::Future>(future: F) -> F::Output {
        let waker = std::task::Waker::noop();
        let mut cx = std::task::Context::from_waker(waker);
        let mut future = std::pin::pin!(future);
        match future.as_mut().poll(&mut cx) {
            std::task::Poll::Ready(output) => output,
            std::task::Poll::Pending => {
                unreachable!("macOS/Linux HeadlessAppContext runner never suspends")
            }
        }
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    pub fn run() {
        let mut runner = Runner;
        let passed = block_on_ready(run_all_checks(&mut runner));
        println!("rendering: {passed} offscreen pixel checks passed");
    }

    #[cfg(target_os = "windows")]
    struct CatchUnwind<F>(std::pin::Pin<Box<F>>);

    #[cfg(target_os = "windows")]
    impl<F: std::future::Future> std::future::Future for CatchUnwind<F> {
        type Output = std::thread::Result<F::Output>;

        fn poll(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.0.as_mut().poll(cx)
            })) {
                Ok(std::task::Poll::Pending) => std::task::Poll::Pending,
                Ok(std::task::Poll::Ready(value)) => std::task::Poll::Ready(Ok(value)),
                Err(payload) => std::task::Poll::Ready(Err(payload)),
            }
        }
    }

    #[cfg(target_os = "windows")]
    pub fn run() {
        use std::{cell::RefCell, rc::Rc};

        enum RunOutcome {
            Passed(usize),
            Skipped(String),
            Panicked(Box<dyn std::any::Any + Send>),
        }

        let outcome: Rc<RefCell<Option<RunOutcome>>> = Rc::new(RefCell::new(None));
        let outcome_slot = outcome.clone();

        gpui_platform::application()
            .with_assets(EmptyAssets)
            .with_windowing(gpui::WindowingRequest::Headless)
            .run(move |cx| {
                cx.spawn(async move |cx| {
                    let attach = cx
                        .update(|cx| {
                            cx.request_windowing(gpui::WindowingRequest::Windowed(
                                gpui::GraphicalEnvironment::detect(),
                            ))
                        })
                        .await;
                    if let Err(err) = attach {
                        *outcome_slot.borrow_mut() = Some(RunOutcome::Skipped(format!("{err:#}")));
                        cx.update(|cx| cx.quit());
                        return;
                    }

                    cx.update(|cx| {
                        gpui_material::init(MaterialTheme::baseline(Appearance::Light), cx);
                        cx.set_reduce_motion(true);
                    });

                    let mut runner = Runner { cx: cx.clone() };
                    let result = CatchUnwind(Box::pin(run_all_checks(&mut runner))).await;
                    *outcome_slot.borrow_mut() = Some(match result {
                        Ok(passed) => RunOutcome::Passed(passed),
                        Err(payload) => RunOutcome::Panicked(payload),
                    });
                    cx.update(|cx| cx.quit());
                })
                .detach();
            });

        match outcome.borrow_mut().take() {
            Some(RunOutcome::Passed(passed)) => {
                println!("rendering: {passed} offscreen pixel checks passed (Windows DirectX)");
            }
            Some(RunOutcome::Skipped(reason)) => {
                if std::env::var_os("CI").is_some() {
                    panic!(
                        "rendering: Windows DirectX offscreen renderer unavailable in CI: {reason}"
                    );
                }
                println!("rendering: skipped on Windows; DirectX offscreen unavailable: {reason}");
            }
            Some(RunOutcome::Panicked(payload)) => std::panic::resume_unwind(payload),
            None => {
                panic!("rendering: Windows offscreen runner exited without recording an outcome")
            }
        }
    }
}
