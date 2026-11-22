use std::{cell::Cell, rc::Rc};

use gpui::{
    App, AppContext, Bounds, Context, ElementId, Entity, Hsla, InteractiveElement, IntoElement,
    KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, MouseButton, ParentElement, Pixels, Render,
    RenderOnce, Role, Styled, TestAppContext, VisualTestContext, Window, canvas, div, px, rgb,
    size,
};
use gpui_base::test_support::find;
use gpui_material::{
    ButtonBorder, ButtonPadding, ButtonShapes, Chip, ChipColors, ChipElevations, ChipStateColors,
    ChipVariant, InteractionSource, InteractionState, MaterialRoot, MaterialTheme,
    foundation::{color, set_window_material_theme, window_material_theme},
    theme::{ColorRole, CornerRole, CornerShape, Direction, Elevation},
};

type Capture = Rc<Cell<Option<(Bounds<Pixels>, Hsla)>>>;

#[derive(IntoElement)]
struct Slot(Capture, f32, f32);

impl RenderOnce for Slot {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        canvas(
            move |bounds, window, _| self.0.set(Some((bounds, window.text_style().color))),
            |_, (), _, _| {},
        )
        .w(px(self.1))
        .h(px(self.2))
        .flex_none()
    }
}

struct Chips {
    source: Entity<InteractionSource>,
    assist_label: Capture,
    assist_trailing: Capture,
    filter_leading: Capture,
    elevated_leading: Capture,
    input_leading: Capture,
    input_avatar: Capture,
    input_trailing: Capture,
    suggestion_trailing: Capture,
    selected: bool,
    disabled: bool,
    dragged: bool,
    requested: Vec<bool>,
    removed: usize,
}

impl Render for Chips {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selection_owner = cx.entity();
        let removal_owner = cx.entity();
        div()
            .flex()
            .flex_col()
            .items_start()
            .child(
                Chip::new("assist", "Assist accessible label")
                    .content(Slot(self.assist_label.clone(), 29., 12.))
                    .trailing(Slot(self.assist_trailing.clone(), 11., 13.))
                    .spacing(px(17.))
                    .content_padding(ButtonPadding {
                        start: px(9.),
                        end: px(5.),
                        top: px(12.),
                        bottom: px(14.),
                    })
                    .shapes(ButtonShapes {
                        resting: Some(CornerShape::Radii([3., 6., 9., 12.])),
                        ..Default::default()
                    })
                    .colors(ChipColors {
                        enabled: ChipStateColors {
                            container: Some(rgb(0xffebc2).into()),
                            label: Some(rgb(0x392816).into()),
                            trailing: Some(rgb(0x234125).into()),
                            border: Some(rgb(0x673425).into()),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .border(ButtonBorder {
                        width: px(2.),
                        color: rgb(0x234225).into(),
                    }),
            )
            .child(
                Chip::new("tonal", "Customize filters")
                    .variant(ChipVariant::Filter)
                    .tonal(true)
                    .leading_icon(Slot(self.filter_leading.clone(), 10., 10.))
                    .colors(ChipColors {
                        enabled: ChipStateColors {
                            container: Some(rgb(0xb7c9e7).into()),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
            )
            .child(
                Chip::new("elevated", "Customize elevated filters")
                    .variant(ChipVariant::Filter)
                    .elevated(true)
                    .tonal(true)
                    .leading_icon(Slot(self.elevated_leading.clone(), 10., 10.)),
            )
            .child(
                Chip::new("input", "Selected contact")
                    .variant(ChipVariant::Input)
                    .tonal(true)
                    .selected(self.selected)
                    .disabled(self.disabled)
                    .dragged(self.dragged)
                    .leading_icon(Slot(self.input_leading.clone(), 10., 10.))
                    .avatar(Slot(self.input_avatar.clone(), 24., 24.))
                    .trailing(Slot(self.input_trailing.clone(), 26., 34.))
                    .content_padding(ButtonPadding {
                        start: px(7.),
                        end: px(9.),
                        top: px(4.),
                        bottom: px(3.),
                    })
                    .shapes(ButtonShapes {
                        pressed: Some(CornerShape::Radii([2.; 4])),
                        checked: Some(CornerShape::Radii([11.; 4])),
                        ..Default::default()
                    })
                    .colors(ChipColors {
                        enabled: ChipStateColors {
                            container: Some(rgb(0xbadced).into()),
                            ..Default::default()
                        },
                        selected: ChipStateColors {
                            container: Some(rgb(0xfedbd6).into()),
                            trailing: Some(rgb(0x254432).into()),
                            ..Default::default()
                        },
                        disabled: ChipStateColors {
                            container: Some(rgb(0xdfd4e3).into()),
                            trailing: Some(rgb(0x635966).into()),
                            ..Default::default()
                        },
                        disabled_selected: ChipStateColors {
                            container: Some(rgb(0xccc5ce).into()),
                            ..Default::default()
                        },
                    })
                    .elevations(ChipElevations {
                        resting: Some(Elevation::Level1),
                        pressed: Some(Elevation::Level2),
                        focused: Some(Elevation::Level3),
                        hovered: Some(Elevation::Level4),
                        dragged: Some(Elevation::Level5),
                        disabled: Some(Elevation::Level0),
                    })
                    .interaction_source(&self.source)
                    .on_change(move |requested, _, cx| {
                        selection_owner.update(cx, |this, _| this.requested.push(requested));
                    })
                    .on_remove("Remove selected contact", None, move |_, cx| {
                        removal_owner.update(cx, |this, _| this.removed += 1);
                    }),
            )
            .child(
                Chip::new("suggestion", "Default suggestion")
                    .trailing(Slot(self.suggestion_trailing.clone(), 30., 12.))
                    .variant(ChipVariant::Suggestion)
                    .tonal(true),
            )
    }
}

fn open(cx: &mut TestAppContext) -> (Entity<Chips>, &mut VisualTestContext) {
    cx.update(|cx| {
        MaterialRoot::init(MaterialTheme::default(), cx);
        cx.set_reduce_motion(true);
    });
    let (view, cx) = cx.add_window_view(|window, cx| {
        window.activate_window();
        Chips {
            source: cx.new(|_| InteractionSource::new()),
            assist_label: Capture::default(),
            assist_trailing: Capture::default(),
            filter_leading: Capture::default(),
            elevated_leading: Capture::default(),
            input_leading: Capture::default(),
            input_avatar: Capture::default(),
            input_trailing: Capture::default(),
            suggestion_trailing: Capture::default(),
            selected: true,
            disabled: false,
            dragged: false,
            requested: Vec::new(),
            removed: 0,
        }
    });
    draw(cx);
    (view, cx)
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
    cx.run_until_parked();
}

fn action(id: &'static str, target: &'static str) -> ElementId {
    ElementId::from((ElementId::from(id), target))
}

fn press_space(cx: &mut VisualTestContext) {
    let keystroke =
        cx.update(|_, cx| Keystroke::parse_with_os("space", cx.platform_info().os).unwrap());
    cx.simulate_event(KeyDownEvent {
        keystroke: keystroke.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.simulate_event(KeyUpEvent { keystroke });
}

fn state(cx: &mut VisualTestContext, view: &Entity<Chips>) -> InteractionState {
    cx.update(|window, cx| {
        view.read(cx)
            .source
            .read(cx)
            .state(window.window_handle().window_id(), &"input".into())
            .unwrap()
    })
}

fn container(window: &Window, pigment: u32) -> gpui::Quad {
    window
        .painted_quads()
        .into_iter()
        .find(|quad| quad.background.as_solid() == Some(rgb(pigment).into()))
        .expect("chip container pigment")
}

#[gpui::test]
fn chips_paint_custom_slots_tonal_presets_and_logical_edges(cx: &mut TestAppContext) {
    let (view, cx) = open(cx);
    cx.update(|window, cx| {
        let preview = view.read(cx);
        let assist = find(window, &[], &action("assist", "activate")).unwrap();
        assert_eq!(assist.label(), Some("Assist accessible label"));
        assert_eq!(assist.role(), Some(Role::Button));
        let (label, label_color) = preview.assist_label.get().unwrap();
        let (trailing, trailing_color) = preview.assist_trailing.get().unwrap();
        assert_eq!(trailing.left() - label.right(), px(17.));
        assert_eq!(label_color, rgb(0x392816).into());
        assert_eq!(trailing_color, rgb(0x234125).into());

        let scale = window.scale_factor();
        let assist_quad = container(window, 0xffebc2);
        assert_eq!(assist_quad.border_widths.top, px(2.).scale(scale));
        assert_eq!(assist_quad.border_color, rgb(0x673425).into());
        assert_eq!(assist_quad.corner_radii.top_left, px(3.).scale(scale));
        assert_eq!(assist_quad.corner_radii.top_right, px(6.).scale(scale));
        assert_eq!(assist_quad.bounds.size.height, px(39.).scale(scale));
        assert_eq!(assist_quad.bounds.size.width, px(71.).scale(scale));

        let theme = window_material_theme(window, cx);
        let preset_leading = color(theme.colors.color(ColorRole::OnSurfaceVariant));
        assert_eq!(preview.filter_leading.get().unwrap().1, preset_leading);
        assert_eq!(preview.elevated_leading.get().unwrap().1, preset_leading);
        assert_eq!(
            container(window, 0xb7c9e7).corner_radii.top_left,
            px(CornerRole::Medium.shape().radii(90., 32., Direction::Ltr)[0]).scale(scale)
        );
        assert!(preview.input_leading.get().is_none());
        assert!(preview.suggestion_trailing.get().is_none());
        assert_eq!(
            preview.input_avatar.get().unwrap().0.size,
            size(px(24.), px(24.))
        );
        assert_eq!(
            preview.input_trailing.get().unwrap().1,
            rgb(0x254432).into()
        );
        let input_quad = container(window, 0xfedbd6);
        assert_eq!(input_quad.corner_radii.top_left, px(11.).scale(scale));
        assert_eq!(input_quad.bounds.size.height, px(41.).scale(scale));
        let suggestion = find(window, &[], &action("suggestion", "activate")).unwrap();
        assert_eq!(suggestion.label(), Some("Default suggestion"));
        assert_eq!(suggestion.bounds().size.height, px(48.));

        let mut theme = MaterialTheme::default();
        theme.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(theme), window, cx);
    });
    draw(cx);
    cx.update(|window, cx| {
        let preview = view.read(cx);
        let label = preview.assist_label.get().unwrap().0;
        let trailing = preview.assist_trailing.get().unwrap().0;
        assert_eq!(label.left() - trailing.right(), px(17.));
        let select = find(window, &[], &action("input", "activate")).unwrap();
        let remove = find(window, &[], &action("input", "remove")).unwrap();
        assert!(remove.bounds().right() <= select.bounds().left());
        assert!(preview.input_avatar.get().unwrap().0.center().x > select.bounds().center().x);
        let quad = container(window, 0xffebc2);
        assert_eq!(
            quad.corner_radii.top_left,
            px(6.).scale(window.scale_factor())
        );
    });
}

#[gpui::test]
fn chips_keep_independent_actions_controlled_values_and_drag_cancellation(cx: &mut TestAppContext) {
    let (view, cx) = open(cx);
    let (activate, remove) = cx.update(|window, cx| {
        let activate = find(window, &[], &action("input", "activate")).unwrap();
        let remove = find(window, &[], &action("input", "remove")).unwrap();
        assert_eq!(activate.label(), Some("Selected contact"));
        assert_eq!(activate.checked(), Some(true));
        assert_eq!(remove.label(), Some("Remove selected contact"));
        assert_eq!(remove.role(), Some(Role::Button));
        assert!(activate.bounds().right() <= remove.bounds().left());
        for _ in 0..4 {
            window.focus_next(cx);
        }
        (activate.bounds().center(), remove.bounds().center())
    });
    draw(cx);
    cx.update(|window, _| {
        assert_eq!(
            find(window, &[], &action("input", "activate"))
                .unwrap()
                .focused(),
            Some(true)
        );
    });
    press_space(cx);
    draw(cx);
    cx.update(|window, cx| {
        assert_eq!(view.read(cx).requested, [false]);
        assert_eq!(
            find(window, &[], &action("input", "activate"))
                .unwrap()
                .checked(),
            Some(true)
        );
        window.focus_next(cx);
    });
    draw(cx);
    cx.update(|window, _| {
        assert_eq!(
            find(window, &[], &action("input", "remove"))
                .unwrap()
                .focused(),
            Some(true)
        );
    });
    press_space(cx);
    draw(cx);
    cx.update(|_, cx| {
        assert_eq!(view.read(cx).requested, [false]);
        assert_eq!(view.read(cx).removed, 1);
    });

    cx.simulate_mouse_move(activate, None, Modifiers::default());
    draw(cx);
    assert!(state(cx, &view).hovered);
    cx.simulate_mouse_down(activate, MouseButton::Left, Modifiers::default());
    draw(cx);
    assert!(state(cx, &view).pressed);
    cx.update(|window, _| {
        assert_eq!(
            container(window, 0xfedbd6).corner_radii.top_left,
            px(2.).scale(window.scale_factor())
        );
    });
    for dragged in [true, false] {
        cx.update(|_, cx| {
            view.update(cx, |this, cx| {
                this.dragged = dragged;
                cx.notify();
            });
        });
        draw(cx);
        let state = state(cx, &view);
        assert_eq!(state.dragged, dragged);
        assert!(!state.pressed);
    }
    cx.simulate_mouse_up(activate, MouseButton::Left, Modifiers::default());
    draw(cx);
    cx.update(|_, cx| assert_eq!(view.read(cx).requested, [false]));
    cx.simulate_click(activate, Modifiers::default());
    cx.simulate_click(remove, Modifiers::default());
    draw(cx);
    cx.update(|_, cx| {
        assert_eq!(view.read(cx).requested, [false, false]);
        assert_eq!(view.read(cx).removed, 2);
        view.update(cx, |this, cx| {
            this.dragged = true;
            this.disabled = true;
            cx.notify();
        });
    });
    draw(cx);
    let disabled = state(cx, &view);
    assert!(disabled.disabled && !disabled.dragged && !disabled.pressed);
    cx.simulate_click(activate, Modifiers::default());
    cx.simulate_click(remove, Modifiers::default());
    draw(cx);
    cx.update(|window, cx| {
        assert_eq!(view.read(cx).requested, [false, false]);
        assert_eq!(view.read(cx).removed, 2);
        for target in ["activate", "remove"] {
            assert_eq!(
                find(window, &[], &action("input", target))
                    .unwrap()
                    .disabled(),
                Some(true)
            );
        }
        assert_eq!(
            view.read(cx).input_trailing.get().unwrap().1,
            rgb(0x635966).into()
        );
        container(window, 0xccc5ce);
    });
    cx.update(|_, cx| {
        view.update(cx, |this, cx| {
            this.selected = false;
            this.dragged = false;
            cx.notify();
        });
    });
    draw(cx);
    cx.update(|window, _| {
        assert_eq!(
            find(window, &[], &action("input", "activate"))
                .unwrap()
                .checked(),
            Some(false)
        );
        container(window, 0xdfd4e3);
    });
}

#[gpui::test]
fn filter_removal_has_its_own_keyboard_action_and_restores_focus(cx: &mut TestAppContext) {
    use super::SelectionExamples;
    use crate::story::Story;
    use gpui::InputEvent;

    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let handle = MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
            let selection = cx.new(|cx| SelectionExamples::for_story(Story::Chip, cx));
            view = Some(selection.clone());
            selection
        })
        .unwrap();
        (handle, view.unwrap())
    });
    cx.update_window(handle.into(), |_, window, cx| {
        let draw = |window: &mut Window, cx: &mut gpui::App| {
            window.refresh();
            window.draw(cx).clear(cx);
        };
        draw(window, cx);
        let remove = ElementId::from((ElementId::from("removable-filter"), "remove"));
        let target = find(window, &[], &remove).expect("Filter chip exposes removal separately");
        assert_eq!(target.label(), Some("Remove downloads filter"));
        assert!(target.bounds().size.width >= px(48.));
        for _ in 0..12 {
            if find(window, &[], &remove).unwrap().focused() == Some(true) {
                break;
            }
            window.focus_next(cx);
            draw(window, cx);
        }
        assert_eq!(find(window, &[], &remove).unwrap().focused(), Some(true));
        let key = Keystroke::parse_with_os("space", cx.platform_info().os).unwrap();
        window.dispatch_event(
            KeyDownEvent {
                keystroke: key.clone(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
        window.dispatch_event(KeyUpEvent { keystroke: key }.to_platform_input(), cx);
        draw(window, cx);
        assert!(view.read(cx).filter_removed);
        assert!(
            !view.read(cx).filter,
            "remove must not invoke the selection callback"
        );
        assert!(view.read(cx).restore.is_focused(window));
        assert!(find(window, &[], &remove).is_none());
    })
    .unwrap();
}

#[gpui::test]
fn controlled_selection_and_chip_removal(cx: &mut TestAppContext) {
    use super::SelectionExamples;
    use gpui::{InputEvent, MouseDownEvent, MouseUpEvent};
    use gpui_material::CheckboxState;

    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let mut view = None;
        let handle = MaterialRoot::open_window(gpui::WindowOptions::default(), cx, |_, cx| {
            let selection = cx.new(SelectionExamples::new);
            view = Some(selection.clone());
            selection
        })
        .unwrap();
        (handle, view.unwrap())
    });
    cx.simulate_window_resize(handle.into(), size(px(850.), px(960.)));
    cx.update_window(handle.into(), |_, window, cx| {
        window.activate_window();
        let draw = |window: &mut Window, cx: &mut gpui::App| {
            window.refresh();
            window.draw(cx).clear(cx);
        };
        let assert_switch_centered = |window: &Window| {
            let id = ElementId::from("notifications");
            let track = find(window, &[], &(id.clone(), "switch-track").into()).unwrap();
            let thumb = find(window, &[], &(id, "switch-thumb").into()).unwrap();
            assert_eq!(track.bounds().center().y, thumb.bounds().center().y);
            assert!(thumb.bounds().left() >= track.bounds().left() + px(2.));
            assert!(thumb.bounds().right() <= track.bounds().right() - px(2.));
        };
        draw(window, cx);
        assert_switch_centered(window);
        for (scope, id) in [
            (vec![], "palette-all".into()),
            (vec![], "notifications".into()),
            (vec![], ElementId::from((ElementId::from("tag"), "remove"))),
        ] {
            let target = find(window, &scope, &id).unwrap_or_else(|| {
                panic!(
                    "missing {id:?} in {scope:?}, observed {:?}",
                    gpui_base::test_support::snapshots(window)
                        .iter()
                        .map(|snapshot| snapshot.label())
                        .collect::<Vec<_>>()
                )
            });
            assert!(target.visible(), "{id}: {:?}", target.bounds());
            let position = target.bounds().center();
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
            draw(window, cx);
            assert_switch_centered(window);
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
            draw(window, cx);
            assert_switch_centered(window);
        }
        let state = view.read(cx);
        assert_eq!(state.colors, [CheckboxState::Checked; 2]);
        assert!(state.notifications);
        assert!(!state.chip_visible);
        assert!(!state.chip_selected);
        assert!(state.restore.is_focused(window));
    })
    .unwrap();
}

#[gpui::test]
fn track_focus_attaches_caller_handles_to_chip_checkbox_radio_switch_and_card(
    cx: &mut TestAppContext,
) {
    use gpui::FocusHandle;
    use gpui_material::{
        Card, Checkbox, CheckboxState, Radio, Switch, ThemeScope, theme::Appearance,
    };

    struct FocusHarness {
        chip_focus: FocusHandle,
        checkbox_focus: FocusHandle,
        radio_focus: FocusHandle,
        switch_focus: FocusHandle,
        card_focus: FocusHandle,
        scoped_dark: Cell<Option<bool>>,
    }

    impl Render for FocusHarness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let owner = cx.entity();
            ThemeScope::new(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        Chip::new("tracked-chip", "Tracked chip")
                            .variant(ChipVariant::Filter)
                            .track_focus(&self.chip_focus)
                            .on_change(move |_, window, cx| {
                                let is_dark =
                                    gpui_material::foundation::current_material_theme(window, cx)
                                        .colors
                                        .appearance()
                                        == Appearance::Dark;
                                owner.update(cx, |this, _| this.scoped_dark.set(Some(is_dark)));
                            }),
                    )
                    .child(
                        Checkbox::new(
                            "tracked-checkbox",
                            "Tracked checkbox",
                            CheckboxState::Unchecked,
                        )
                        .track_focus(&self.checkbox_focus),
                    )
                    .child(
                        Radio::new("tracked-radio", "Tracked radio").track_focus(&self.radio_focus),
                    )
                    .child(
                        Switch::new("tracked-switch", "Tracked switch", false)
                            .track_focus(&self.switch_focus),
                    )
                    .child(
                        Card::new("tracked-card")
                            .track_focus(&self.card_focus)
                            .on_activate("Tracked card", |_, _| {})
                            .child("Card body"),
                    ),
            )
            .theme(MaterialTheme::baseline(Appearance::Dark))
        }
    }

    cx.update(|cx| {
        MaterialRoot::init(MaterialTheme::baseline(Appearance::Light), cx);
        cx.set_reduce_motion(true);
    });
    let (view, cx) = cx.add_window_view(|window, cx| {
        window.activate_window();
        FocusHarness {
            chip_focus: cx.focus_handle(),
            checkbox_focus: cx.focus_handle(),
            radio_focus: cx.focus_handle(),
            switch_focus: cx.focus_handle(),
            card_focus: cx.focus_handle(),
            scoped_dark: Cell::new(None),
        }
    });
    draw(cx);

    type FocusTarget = (&'static str, ElementId, fn(&FocusHarness) -> FocusHandle);
    let targets: [FocusTarget; 5] = [
        ("chip", action("tracked-chip", "activate"), |h| {
            h.chip_focus.clone()
        }),
        ("checkbox", "tracked-checkbox".into(), |h| {
            h.checkbox_focus.clone()
        }),
        ("radio", "tracked-radio".into(), |h| h.radio_focus.clone()),
        ("switch", "tracked-switch".into(), |h| {
            h.switch_focus.clone()
        }),
        ("card", "tracked-card".into(), |h| h.card_focus.clone()),
    ];
    for (name, id, get_handle) in targets {
        cx.update(|window, cx| {
            let handle = get_handle(view.read(cx));
            handle.focus(window, cx);
        });
        draw(cx);
        cx.update(|window, cx| {
            let handle = get_handle(view.read(cx));
            assert!(
                handle.is_focused(window),
                "{name} focus handle should be focused"
            );
            assert_eq!(
                find(window, &[], &id).unwrap().focused(),
                Some(true),
                "{name} node should report focused"
            );
        });
    }

    // Verify ThemeScope::bind_action preserves the Dark subtree theme on toggle activation.
    cx.update(|window, cx| {
        let focus = view.read(cx).chip_focus.clone();
        focus.focus(window, cx);
    });
    draw(cx);
    press_space(cx);
    draw(cx);
    cx.update(|_, cx| {
        assert_eq!(view.read(cx).scoped_dark.get(), Some(true));
    });
}

#[gpui::test]
fn chip_shorthands_and_semantic_tones_respect_selection_disabled_and_overrides(
    cx: &mut TestAppContext,
) {
    use gpui_material::SemanticTone;

    struct ToneChips {
        unselected_icon: Capture,
        selected_icon: Capture,
        disabled_icon: Capture,
        overridden_icon: Capture,
    }

    impl Render for ToneChips {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .flex()
                .flex_col()
                .items_start()
                .gap_2()
                .p_4()
                .child(
                    Chip::assist("shorthand-assist", "Assist")
                        .tone(SemanticTone::Danger)
                        .leading_icon(Slot(self.unselected_icon.clone(), 12., 12.)),
                )
                .child(
                    Chip::filter("shorthand-filter", "Success filter")
                        .selected(true)
                        .tone(SemanticTone::Success)
                        .leading_icon(Slot(self.selected_icon.clone(), 12., 12.)),
                )
                .child(
                    Chip::input("shorthand-input", "Disabled warning")
                        .disabled(true)
                        .tone(SemanticTone::Warning)
                        .leading_icon(Slot(self.disabled_icon.clone(), 12., 12.)),
                )
                .child(
                    Chip::suggestion("shorthand-suggestion", "Overridden info")
                        .tone(SemanticTone::Info)
                        .leading_icon(Slot(self.overridden_icon.clone(), 12., 12.))
                        .colors(ChipColors {
                            enabled: ChipStateColors {
                                leading: Some(rgb(0x225588).into()),
                                border: Some(rgb(0x884422).into()),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                )
        }
    }

    cx.update(|cx| {
        MaterialRoot::init(MaterialTheme::default(), cx);
        cx.set_reduce_motion(true);
    });
    let (view, cx) = cx.add_window_view(|window, _| {
        window.activate_window();
        ToneChips {
            unselected_icon: Capture::default(),
            selected_icon: Capture::default(),
            disabled_icon: Capture::default(),
            overridden_icon: Capture::default(),
        }
    });
    draw(cx);

    cx.update(|window, cx| {
        let theme = window_material_theme(window, cx);
        let error = color(theme.colors.color(ColorRole::Error));
        let success_container = color(theme.colors.color(ColorRole::SuccessContainer));
        let on_success_container = color(theme.colors.color(ColorRole::OnSuccessContainer));
        let disabled_fg = color(theme.colors.color(ColorRole::OnSurface)).opacity(0.38);
        let state = view.read(cx);

        assert_eq!(state.unselected_icon.get().unwrap().1, error);
        assert_eq!(state.selected_icon.get().unwrap().1, on_success_container);
        assert_eq!(state.disabled_icon.get().unwrap().1, disabled_fg);
        assert_eq!(state.overridden_icon.get().unwrap().1, rgb(0x225588).into());

        let quads = window.painted_quads();
        assert!(
            quads
                .iter()
                .any(|quad| quad.background == success_container.into()),
            "selected Success chip must paint SuccessContainer"
        );
        assert!(
            quads.iter().any(|quad| quad.border_color == error),
            "unselected Danger chip must paint Error outline"
        );
        assert!(
            quads
                .iter()
                .any(|quad| quad.border_color == rgb(0x884422).into()),
            "explicit ChipColors border must override SemanticTone"
        );
    });
}

#[gpui::test]
fn removable_chip_invokes_on_remove_on_backspace_and_delete_when_focused(cx: &mut TestAppContext) {
    struct RemovableChipView {
        chip_focus: gpui::FocusHandle,
        disabled_focus: gpui::FocusHandle,
        restore_focus: gpui::FocusHandle,
        removed: usize,
        disabled_removed: usize,
        parent_keys: Vec<String>,
    }

    impl Render for RemovableChipView {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let v1 = cx.entity();
            let v2 = cx.entity();
            let v3 = cx.entity();
            let restore = self.restore_focus.clone();
            div()
                .id("chip-key-root")
                .on_key_down(move |event, _, cx| {
                    v3.update(cx, |this, _| {
                        this.parent_keys.push(event.keystroke.key.clone());
                    });
                })
                .child(
                    div()
                        .id("restore-target")
                        .track_focus(&self.restore_focus)
                        .size(px(20.)),
                )
                .child(
                    Chip::input("removable", "Rust")
                        .track_focus(&self.chip_focus)
                        .on_remove("Remove Rust", Some(restore), move |_, cx| {
                            v1.update(cx, |this, _| this.removed += 1);
                        }),
                )
                .child(
                    Chip::input("disabled-removable", "Locked")
                        .disabled(true)
                        .track_focus(&self.disabled_focus)
                        .on_remove("Remove Locked", None, move |_, cx| {
                            v2.update(cx, |this, _| this.disabled_removed += 1);
                        }),
                )
        }
    }

    cx.update(|cx| {
        MaterialRoot::init(MaterialTheme::default(), cx);
        cx.set_reduce_motion(true);
    });
    let (view, cx) = cx.add_window_view(|window, cx| {
        window.activate_window();
        RemovableChipView {
            chip_focus: cx.focus_handle(),
            disabled_focus: cx.focus_handle(),
            restore_focus: cx.focus_handle(),
            removed: 0,
            disabled_removed: 0,
            parent_keys: Vec::new(),
        }
    });
    draw(cx);

    // Backspace on focused primary chip action invokes on_remove, restores focus, stops propagation.
    cx.update(|window, cx| view.read(cx).chip_focus.clone().focus(window, cx));
    draw(cx);
    cx.simulate_keystrokes("backspace");
    draw(cx);
    cx.update(|window, cx| {
        assert_eq!(view.read(cx).removed, 1);
        assert!(view.read(cx).restore_focus.is_focused(window));
        assert!(view.read(cx).parent_keys.is_empty());
    });

    // Delete on focused primary chip action invokes on_remove and stops propagation.
    cx.update(|window, cx| view.read(cx).chip_focus.clone().focus(window, cx));
    draw(cx);
    cx.simulate_keystrokes("delete");
    draw(cx);
    cx.update(|_, cx| {
        assert_eq!(view.read(cx).removed, 2);
        assert!(view.read(cx).parent_keys.is_empty());
    });

    // Modified Backspace/Delete do NOT invoke on_remove.
    cx.update(|window, cx| view.read(cx).chip_focus.clone().focus(window, cx));
    draw(cx);
    cx.simulate_keystrokes("ctrl-backspace shift-delete");
    draw(cx);
    cx.update(|_, cx| {
        assert_eq!(view.read(cx).removed, 2);
        assert_eq!(view.read(cx).parent_keys, vec!["backspace", "delete"]);
    });

    // Disabled removable chip does NOT invoke on_remove on Backspace/Delete.
    cx.update(|window, cx| view.read(cx).disabled_focus.clone().focus(window, cx));
    draw(cx);
    cx.simulate_keystrokes("backspace delete");
    draw(cx);
    cx.update(|_, cx| {
        assert_eq!(view.read(cx).disabled_removed, 0);
    });
}
