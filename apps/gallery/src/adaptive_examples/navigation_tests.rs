use std::{cell::Cell, rc::Rc, time::Duration};

use gpui::{
    AppContext as _, Bounds, Context, ElementId, Entity, FocusHandle, Modifiers,
    ParentElement as _, Pixels, Render, Role, SharedString, Styled as _, TestAppContext,
    VisualTestContext, Window, div, px, rgb, size,
};
use gpui_base::test_support::find;
use gpui_material::{
    AdaptiveNavigation, AdaptiveNavigationInfo, AdaptiveNavigationState, AdaptiveNavigationType,
    AdaptiveNavigationVisibility, Button, Icon, IconName, MaterialTheme, NavigationBarArrangement,
    NavigationItem, NavigationItemColors, NavigationItems, NavigationRailArrangement,
    NavigationRailState, RailMode, Scaffold, Text,
    foundation::set_window_material_theme,
    theme::{Direction, WidthClass},
};

const SLOT: &str = "adaptive-navigation-slot";
const MAIL: &str = "adaptive-test-mail";
const ARCHIVE: &str = "adaptive-test-archive";
const BODY: &str = "scaffold-content";
const FIRST: &str = "adaptive-test-first";
const OUTSIDE: &str = "adaptive-test-outside";

struct Preview {
    navigation: Entity<AdaptiveNavigationState>,
    rail: Entity<NavigationRailState>,
    kind: Option<AdaptiveNavigationType>,
    policy: Option<Rc<Cell<Option<AdaptiveNavigationInfo>>>>,
    custom: bool,
    selected: SharedString,
    activations: usize,
    first_focus: FocusHandle,
    outside_focus: FocusHandle,
}

impl Render for Preview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let scaffold = Scaffold::new(
            "adaptive-test-scaffold",
            Button::new(OUTSIDE, "Outside navigation").track_focus(&self.outside_focus),
        )
        .app_bar(Button::new(FIRST, "Before navigation").track_focus(&self.first_focus));
        let items = NavigationItems::new(
            "adaptive-test-destinations",
            "Audit destinations",
            [
                NavigationItem::new(MAIL, "Mail")
                    .icon(Icon::new(IconName::Inbox))
                    .colors(NavigationItemColors {
                        selected_indicator: Some(rgb(0x9d52c7).into()),
                        ..Default::default()
                    }),
                NavigationItem::new("adaptive-test-disabled", "Unavailable").disabled(true),
                NavigationItem::new(ARCHIVE, "Archive").icon(Icon::new(IconName::Description)),
            ],
        )
        .selected(self.selected.clone())
        .on_select(cx.listener(|view, id: &SharedString, _, cx| {
            view.selected = id.clone();
            view.activations += 1;
            cx.notify();
        }));
        let area = window.viewport_size();
        let mut adaptive = AdaptiveNavigation::new(scaffold, items, area.width)
            .state(&self.navigation)
            .available_size(area);
        if let Some(kind) = self.kind {
            adaptive = adaptive.navigation_type(kind);
        } else if let Some(probe) = self.policy.clone() {
            adaptive = adaptive.navigation_policy(move |info| {
                probe.set(Some(info));
                if info.available_size.height < px(490.) {
                    AdaptiveNavigationType::ShortBarMedium
                } else {
                    AdaptiveNavigationType::RailExpanded
                }
            });
        }
        if self.custom {
            let state = self.rail.clone();
            adaptive = adaptive
                .navigation_bar(|_, bar| bar.arrangement(NavigationBarArrangement::Centered))
                .navigation_rail(move |_, rail| {
                    rail.state(&state)
                        .expanded_width(px(296.))
                        .arrangement(NavigationRailArrangement::Center)
                })
                .navigation_content(|_, standard| {
                    div()
                        .flex()
                        .flex_col()
                        .child(Text::new("Audit workspace").id("adaptive-test-header"))
                        .child(standard)
                });
        }
        adaptive
    }
}

fn open(cx: &mut TestAppContext) -> (Entity<Preview>, &mut VisualTestContext) {
    cx.update(crate::init_gallery);
    let mut preview = None;
    let (_, cx) = cx.add_window_view(|window, cx| {
        window.activate_window();
        let view = cx.new(|cx| Preview {
            navigation: cx.new(|_| AdaptiveNavigationState::default()),
            rail: cx.new(|_| NavigationRailState::new(RailMode::Expanded)),
            kind: None,
            policy: None,
            custom: false,
            selected: MAIL.into(),
            activations: 0,
            first_focus: cx.focus_handle(),
            outside_focus: cx.focus_handle(),
        });
        preview = Some(view.clone());
        gpui_base::Root::new(view, window, cx)
    });
    resize(cx, 599., 580.);
    (preview.unwrap(), cx)
}

fn draw(cx: &mut VisualTestContext) {
    for _ in 0..2 {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
    }
}

fn resize(cx: &mut VisualTestContext, width: f32, height: f32) {
    let window = cx.update(|window, _| window.window_handle());
    cx.simulate_window_resize(window, size(px(width), px(height)));
    draw(cx);
}

fn bounds(cx: &mut VisualTestContext, id: &'static str) -> Bounds<Pixels> {
    cx.update(|window, _| {
        find(window, &[], &ElementId::from(id))
            .unwrap_or_else(|| panic!("missing {id}"))
            .bounds()
    })
}

fn check_focused(cx: &mut VisualTestContext, id: &'static str) {
    cx.update(|window, _| {
        let snapshot = find(window, &[], &id.into()).unwrap_or_else(|| panic!("missing {id}"));
        assert_eq!(snapshot.focused(), Some(true), "{id} holds focus");
    });
}

fn check_selection(cx: &mut VisualTestContext, selected: &'static str, other: &'static str) {
    cx.update(|window, _| {
        let selected = find(window, &[], &selected.into()).unwrap();
        let other = find(window, &[], &other.into()).unwrap();
        assert_eq!(selected.role(), Some(Role::ListBoxOption));
        assert_eq!(selected.selected(), Some(true));
        assert_eq!(other.selected(), Some(false));
    });
}

#[gpui::test]
fn visibility_retargets_across_breakpoints_and_keeps_one_roving_stop(cx: &mut TestAppContext) {
    let (preview, cx) = open(cx);
    let state = cx.update(|_, cx| preview.read(cx).navigation.clone());
    let initial = bounds(cx, SLOT);
    assert_eq!(initial.size.width, px(599.));
    assert_eq!(initial.top(), bounds(cx, BODY).bottom());
    let pos = bounds(cx, MAIL).center();
    cx.simulate_click(pos, Modifiers::default());
    cx.simulate_keystrokes("right");
    draw(cx);
    check_focused(cx, ARCHIVE);
    check_selection(cx, MAIL, ARCHIVE);
    cx.update(|_, cx| state.update(cx, |state, cx| state.hide(cx)));
    cx.executor().advance_clock(Duration::from_millis(105));
    draw(cx);
    let partial = bounds(cx, SLOT).size.height;
    assert!(partial > px(0.) && partial < initial.size.height);
    assert_eq!(bounds(cx, SLOT).top(), bounds(cx, BODY).bottom());
    cx.update(|_, cx| {
        let state = state.read(cx);
        assert_eq!(state.current_value(), AdaptiveNavigationVisibility::Shown);
        assert_eq!(state.target_value(), AdaptiveNavigationVisibility::Hidden);
        assert!(state.is_animating());
    });

    resize(cx, 600., 580.);
    check_focused(cx, ARCHIVE);
    check_selection(cx, MAIL, ARCHIVE);
    let medium_height = bounds(cx, SLOT).size.height;
    cx.update(|_, cx| state.update(cx, |state, cx| state.toggle(cx)));
    draw(cx);
    assert!((bounds(cx, SLOT).size.height - medium_height).abs() <= px(1.));
    resize(cx, 840., 580.);
    assert_eq!(bounds(cx, SLOT).right(), bounds(cx, BODY).left());
    assert!(bounds(cx, SLOT).size.width < px(96.));
    check_focused(cx, ARCHIVE);
    cx.executor().advance_clock(Duration::from_millis(250));
    draw(cx);
    assert_eq!(bounds(cx, SLOT).size.width, px(96.));
    check_selection(cx, MAIL, ARCHIVE);

    cx.update(|_, cx| {
        state.update(cx, |state, cx| {
            state.snap_to(AdaptiveNavigationVisibility::Hidden, cx)
        });
    });
    draw(cx);
    cx.update(|window, cx| {
        assert!(find(window, &[], &ARCHIVE.into()).is_none());
        assert!(find(window, &[], &MAIL.into()).is_none());
        assert!(preview.read(cx).first_focus.is_focused(window));
        preview.update(cx, |view, cx| {
            view.kind = Some(AdaptiveNavigationType::ShortBarCompact);
            cx.notify();
        });
        state.update(cx, |state, cx| state.show(cx));
    });
    cx.executor().advance_clock(Duration::from_millis(250));
    draw(cx);
    assert_eq!(bounds(cx, SLOT).top(), bounds(cx, BODY).bottom());
    check_focused(cx, FIRST);
    check_selection(cx, MAIL, ARCHIVE);

    cx.simulate_keystrokes("tab");
    draw(cx);
    check_focused(cx, OUTSIDE);
    cx.simulate_keystrokes("tab");
    draw(cx);
    check_focused(cx, ARCHIVE);
    cx.simulate_keystrokes("tab");
    draw(cx);
    check_focused(cx, FIRST);
    cx.simulate_keystrokes("shift-tab");
    draw(cx);
    check_focused(cx, ARCHIVE);
    let keystroke = gpui::Keystroke::parse("enter").unwrap();
    cx.simulate_event(gpui::KeyDownEvent {
        keystroke: keystroke.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.simulate_event(gpui::KeyUpEvent { keystroke });
    draw(cx);
    check_selection(cx, ARCHIVE, MAIL);
    cx.update(|_, cx| assert_eq!(preview.read(cx).activations, 2));
}

#[gpui::test]
fn policy_custom_layout_and_reduced_motion_follow_rtl(cx: &mut TestAppContext) {
    let (preview, cx) = open(cx);
    let policy = Rc::new(Cell::new(None));
    let state = cx.update(|window, cx| {
        cx.set_reduce_motion(true);
        let mut theme = MaterialTheme::default();
        theme.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(theme), window, cx);
        preview.update(cx, |view, cx| {
            view.policy = Some(policy.clone());
            view.custom = true;
            cx.notify();
            view.navigation.clone()
        })
    });
    resize(cx, 720., 520.);
    let info = policy.get().unwrap();
    assert_eq!(info.width_class, WidthClass::Medium);
    assert_eq!(info.available_size, size(px(720.), px(520.)));
    assert_eq!(info.direction, Direction::Rtl);
    let slot = bounds(cx, SLOT);
    let body = bounds(cx, BODY);
    assert_eq!(slot.size.width, px(296.));
    assert_eq!(slot.left(), body.right());
    assert!(bounds(cx, "adaptive-test-header").bottom() <= bounds(cx, MAIL).top());
    assert!(cx.update(|window, _| {
        window
            .painted_quads()
            .iter()
            .any(|quad| quad.background == gpui::Hsla::from(rgb(0x9d52c7)).into())
    }));
    let pos = bounds(cx, MAIL).center();
    cx.simulate_click(pos, Modifiers::default());
    cx.simulate_keystrokes("down");
    draw(cx);
    check_focused(cx, ARCHIVE);
    resize(cx, 720., 440.);
    assert_eq!(bounds(cx, SLOT).top(), bounds(cx, BODY).bottom());
    assert!(bounds(cx, MAIL).left() > bounds(cx, ARCHIVE).left());
    check_focused(cx, ARCHIVE);
    cx.simulate_keystrokes("right");
    draw(cx);
    check_focused(cx, MAIL);
    cx.update(|_, cx| {
        state.update(cx, |state, cx| {
            state.hide(cx);
            assert_eq!(state.progress(), 0.);
            assert!(!state.is_animating());
        });
    });
    draw(cx);
    cx.update(|window, cx| {
        assert!(find(window, &[], &SLOT.into()).is_none());
        assert!(find(window, &[], &MAIL.into()).is_none());
        assert!(preview.read(cx).first_focus.is_focused(window));
        preview.update(cx, |view, cx| {
            view.kind = Some(AdaptiveNavigationType::None);
            cx.notify();
        });
        state.update(cx, |state, cx| state.show(cx));
    });
    draw(cx);
    cx.update(|window, cx| {
        assert!(find(window, &[], &SLOT.into()).is_none());
        assert_eq!(
            state.read(cx).current_value(),
            AdaptiveNavigationVisibility::Shown
        );
    });
}
