//! Headless integration tests for system appearance and reduced-motion sync helpers.

use std::{cell::Cell, rc::Rc};

use gpui::{
    AppContext, Context, IntoElement, Render, Subscription, TestAppContext, Window,
    WindowAppearance, WindowOptions, div,
};
use gpui_material::{
    AppearanceMode, MaterialRoot, MaterialTheme, ReduceMotionPreference, appearance_from_window,
    material_theme, observe_system_appearance, observe_window_system_appearance,
    set_reduce_motion_preference, set_window_material_theme, sync_system_reduce_motion,
    system_appearance,
    theme::{Appearance, Density, MotionScheme},
    window_material_theme,
};

struct SyncHost {
    _subscriptions: Vec<Subscription>,
}

impl Render for SyncHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

#[gpui::test]
fn appearance_from_window_and_system_appearance_map_gpui_variants(cx: &mut TestAppContext) {
    assert_eq!(
        appearance_from_window(WindowAppearance::Light),
        Appearance::Light
    );
    assert_eq!(
        appearance_from_window(WindowAppearance::VibrantLight),
        Appearance::Light
    );
    assert_eq!(
        appearance_from_window(WindowAppearance::Dark),
        Appearance::Dark
    );
    assert_eq!(
        appearance_from_window(WindowAppearance::VibrantDark),
        Appearance::Dark
    );

    cx.read(|cx| {
        assert_eq!(
            system_appearance(cx),
            appearance_from_window(cx.window_appearance())
        );
        assert_eq!(system_appearance(cx), Appearance::Light);
    });
}

#[gpui::test]
fn observe_system_appearance_applies_initial_mode_skips_redundant_callbacks_and_preserves_window_overrides(
    cx: &mut TestAppContext,
) {
    let build_calls = Rc::new(Cell::new(0usize));

    let (override_window, synced_window) = cx.update(|cx| {
        gpui_material::init(MaterialTheme::baseline(Appearance::Dark), cx);
        cx.set_reduce_motion(true);

        let override_window =
            MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
                set_window_material_theme(
                    Some(MaterialTheme::baseline(Appearance::Dark).density(Density::Comfortable)),
                    window,
                    cx,
                );
                cx.new(|_| SyncHost {
                    _subscriptions: Vec::new(),
                })
            })
            .unwrap();

        let calls = build_calls.clone();
        let synced_window =
            MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
                let sub = observe_system_appearance(
                    window,
                    cx,
                    AppearanceMode::System,
                    move |appearance| {
                        calls.set(calls.get() + 1);
                        MaterialTheme::baseline(appearance)
                            .motion(MotionScheme::Expressive)
                            .density(Density::Compact)
                    },
                );
                cx.new(|_| SyncHost {
                    _subscriptions: vec![sub],
                })
            })
            .unwrap();

        (override_window, synced_window)
    });

    // Immediate sync matched the window's Light appearance and preserved builder settings.
    cx.read(|cx| {
        let app_theme = material_theme(cx);
        assert_eq!(app_theme.colors.appearance(), Appearance::Light);
        assert_eq!(app_theme.motion, MotionScheme::Expressive);
        assert_eq!(app_theme.layout.density, Density::Compact);
    });
    assert_eq!(build_calls.get(), 1);

    // Let GPUI flush its deferred `window.appearance_changed(cx)` notification.
    // Because the resolved Appearance is still Light, `build_theme` is not called again.
    cx.run_until_parked();
    assert_eq!(build_calls.get(), 1);

    // Window A retains its explicit dark window override while Window B inherits the synced app theme.
    cx.update_window(override_window.into(), |_, window, cx| {
        let local = window_material_theme(window, cx);
        assert_eq!(local.colors.appearance(), Appearance::Dark);
        assert_eq!(local.layout.density, Density::Comfortable);
    })
    .unwrap();
    cx.update_window(synced_window.into(), |_, window, cx| {
        let local = window_material_theme(window, cx);
        assert_eq!(local.colors.appearance(), Appearance::Light);
        assert_eq!(local.layout.density, Density::Compact);

        // Switching to an explicit mode via `MaterialRoot::sync_system_appearance` updates immediately.
        let _dark_sub = MaterialRoot::sync_system_appearance(
            window,
            cx,
            AppearanceMode::Dark,
            MaterialTheme::baseline,
        );
        assert_eq!(material_theme(cx).colors.appearance(), Appearance::Dark);

        let _light_sub = MaterialRoot::sync_system_appearance(
            window,
            cx,
            AppearanceMode::Light,
            MaterialTheme::baseline,
        );
        assert_eq!(material_theme(cx).colors.appearance(), Appearance::Light);
    })
    .unwrap();
}

#[gpui::test]
fn observe_window_system_appearance_scopes_updates_without_mutating_app_theme(
    cx: &mut TestAppContext,
) {
    let build_calls = Rc::new(Cell::new(0usize));

    let (window_a, window_b) = cx.update(|cx| {
        gpui_material::init(MaterialTheme::baseline(Appearance::Light), cx);
        cx.set_reduce_motion(true);

        let window_a = MaterialRoot::open_window(WindowOptions::default(), cx, |window, cx| {
            let sub =
                observe_window_system_appearance(window, cx, AppearanceMode::Dark, |appearance| {
                    MaterialTheme::baseline(appearance).density(Density::Compact)
                });
            cx.new(|_| SyncHost {
                _subscriptions: vec![sub],
            })
        })
        .unwrap();

        let window_b = MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|_| SyncHost {
                _subscriptions: Vec::new(),
            })
        })
        .unwrap();

        (window_a, window_b)
    });

    cx.read(|cx| {
        assert_eq!(material_theme(cx).colors.appearance(), Appearance::Light);
        assert_eq!(material_theme(cx).layout.density, Density::Standard);
    });

    cx.update_window(window_a.into(), |_, window, cx| {
        let local = window_material_theme(window, cx);
        assert_eq!(local.colors.appearance(), Appearance::Dark);
        assert_eq!(local.layout.density, Density::Compact);
    })
    .unwrap();

    cx.update_window(window_b.into(), |_, window, cx| {
        let local = window_material_theme(window, cx);
        assert_eq!(local.colors.appearance(), Appearance::Light);
        assert_eq!(local.layout.density, Density::Standard);
    })
    .unwrap();

    // Window-scoped `AppearanceMode::System` follows window appearance and skips duplicate notifications.
    let calls = build_calls.clone();
    let _system_sub = cx
        .update_window(window_a.into(), |_, window, cx| {
            let sub = MaterialRoot::sync_window_system_appearance(
                window,
                cx,
                AppearanceMode::System,
                move |appearance| {
                    calls.set(calls.get() + 1);
                    MaterialTheme::baseline(appearance).density(Density::Comfortable)
                },
            );
            let local = window_material_theme(window, cx);
            assert_eq!(local.colors.appearance(), Appearance::Light);
            assert_eq!(local.layout.density, Density::Comfortable);
            assert_eq!(material_theme(cx).layout.density, Density::Standard);
            sub
        })
        .unwrap();

    cx.run_until_parked();
    assert_eq!(build_calls.get(), 1);

    // Clearing the window override restores inheritance from the app theme.
    cx.update_window(window_a.into(), |_, window, cx| {
        set_window_material_theme(None, window, cx);
        assert_eq!(
            window_material_theme(window, cx).layout.density,
            Density::Standard
        );
    })
    .unwrap();
}

#[gpui::test]
fn observe_system_appearance_honors_reduced_motion_and_smooth_transitions(cx: &mut TestAppContext) {
    let window = cx.update(|cx| {
        gpui_material::init(MaterialTheme::baseline(Appearance::Light), cx);
        set_reduce_motion_preference(ReduceMotionPreference::Allow, cx);
        MaterialRoot::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|_| SyncHost {
                _subscriptions: Vec::new(),
            })
        })
        .unwrap()
    });

    // With motion allowed and a window open, switching to Dark starts a smooth palette transition
    // while immediately updating layout/density.
    cx.update_window(window.into(), |_, window, cx| {
        let _sub = observe_system_appearance(window, cx, AppearanceMode::Dark, |appearance| {
            MaterialTheme::baseline(appearance).density(Density::Compact)
        });
        assert_eq!(material_theme(cx).layout.density, Density::Compact);
        assert_eq!(material_theme(cx).colors.appearance(), Appearance::Light);
    })
    .unwrap();

    // Enabling reduced motion makes the next appearance sync apply colors immediately.
    cx.update_window(window.into(), |_, window, cx| {
        set_reduce_motion_preference(ReduceMotionPreference::Reduce, cx);
        let _sub = observe_system_appearance(window, cx, AppearanceMode::Dark, |appearance| {
            MaterialTheme::baseline(appearance).density(Density::Compact)
        });
        assert_eq!(material_theme(cx).colors.appearance(), Appearance::Dark);
    })
    .unwrap();
}

#[gpui::test]
fn sync_system_reduce_motion_preserves_explicit_overrides_and_test_determinism(
    cx: &mut TestAppContext,
) {
    cx.update(|cx| {
        gpui_material::init(MaterialTheme::default(), cx);

        // Under GPUI's test scheduler, syncing never queries the OS or flips the flag.
        sync_system_reduce_motion(cx);
        assert!(!cx.reduce_motion());

        // Direct overrides (common in tests) survive `sync_system_reduce_motion`.
        cx.set_reduce_motion(true);
        sync_system_reduce_motion(cx);
        assert!(cx.reduce_motion());

        // Explicit `ReduceMotionPreference` switches update `cx.reduce_motion()` deterministically.
        set_reduce_motion_preference(ReduceMotionPreference::Allow, cx);
        assert!(!cx.reduce_motion());
        sync_system_reduce_motion(cx);
        assert!(!cx.reduce_motion());

        set_reduce_motion_preference(ReduceMotionPreference::Reduce, cx);
        assert!(cx.reduce_motion());
        sync_system_reduce_motion(cx);
        assert!(cx.reduce_motion());

        // Resuming `System` without a platform reading keeps the current flag under test scheduler.
        set_reduce_motion_preference(ReduceMotionPreference::System, cx);
        sync_system_reduce_motion(cx);
        assert!(cx.reduce_motion());
    });
}
