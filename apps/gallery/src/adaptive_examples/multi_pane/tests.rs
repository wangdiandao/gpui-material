use std::time::Duration;

use gpui::{
    AppContext as _, Bounds, Context, ElementId, Entity, FocusHandle, InteractiveElement as _,
    IntoElement, Modifiers, MouseButton, ParentElement as _, Pixels, Render, Role, Size,
    Styled as _, TestAppContext, VisualTestContext, Window, div, px, size,
};
use gpui_base::test_support::find;
use gpui_material::{
    AdaptivePane, AdaptivePaneState, Button, ListDetail, MaterialTheme, PaneAdaptStrategy,
    PaneBackBehavior, PaneDestination, PaneExpansionAnchor, PaneLayoutDirective, PaneLayoutValue,
    PaneMargins, PaneOverlayAlignment, PanePlacement, PaneRole, PaneSeparator, Scrim,
    SupportingPane, foundation::set_window_material_theme, theme::Direction,
};

use PaneRole::{Primary, Secondary, Tertiary};

struct Preview {
    viewport: Size<Pixels>,
    dimensions: (usize, usize),
    state: Entity<AdaptivePaneState>,
    focus: [FocusHandle; 3],
    supporting_layout: bool,
    scrim: bool,
    hit_count: usize,
    background_presses: usize,
}

impl Render for Preview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.entity();
        let list = AdaptivePane::new(
            "master",
            Button::new("master-control", "Open report")
                .track_focus(&self.focus[1])
                .on_activate(move |_, cx| owner.update(cx, |view, _| view.hit_count += 1)),
        )
        .preferred_width(px(140.))
        .minimum_width(px(90.))
        .entry_focus(&self.focus[1]);
        let owner = cx.entity();
        let background = cx.entity();
        let detail = AdaptivePane::new(
            "detail",
            div()
                .size_full()
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    background.update(cx, |preview, _| preview.background_presses += 1);
                })
                .child(
                    Button::new("detail-control", "Approve report")
                        .track_focus(&self.focus[0])
                        .on_activate(move |_, cx| owner.update(cx, |view, _| view.hit_count += 1)),
                ),
        )
        .minimum_width(px(120.))
        .entry_focus(&self.focus[0]);
        let state = self.state.clone();
        let mut tertiary = AdaptivePane::new(
            "extra",
            div().size_full().child(
                Button::new("extra-control", "Return to report")
                    .track_focus(&self.focus[2])
                    .on_activate(move |_, cx| {
                        state.update(cx, |state, cx| {
                            state.navigate_back(PaneBackBehavior::PopLatest, cx)
                        });
                    }),
            ),
        )
        .preferred_width(px(130.))
        .preferred_height(px(108.))
        .minimum_width(px(90.))
        .margins(PaneMargins {
            start: px(6.),
            top: px(4.),
            ..Default::default()
        })
        .entry_focus(&self.focus[2]);
        if self.dimensions == (1, 2) {
            tertiary = tertiary.adapt(PaneAdaptStrategy::Reflow(Primary));
        } else if !self.supporting_layout && self.dimensions.0 < 3 {
            tertiary = tertiary.adapt(PaneAdaptStrategy::Levitate(PaneOverlayAlignment::BottomEnd));
        }
        if self.scrim {
            tertiary = tertiary.levitate_scrim(Scrim::new("audit-scrim"));
        }
        let layout = if self.supporting_layout {
            SupportingPane::adaptive("layout", self.viewport, &self.state, detail, list)
        } else {
            ListDetail::adaptive("layout", self.viewport, &self.state, list, detail)
        };
        layout
            .pane(Tertiary, tertiary)
            .directive(PaneLayoutDirective {
                horizontal_partitions: self.dimensions.0,
                vertical_partitions: self.dimensions.1,
                horizontal_gap: px(24.),
                vertical_gap: px(20.),
                preferred_height: px(140.),
                ..Default::default()
            })
            .separator(
                Secondary,
                Primary,
                PaneSeparator::new("Resize main divider").anchors([
                    PaneExpansionAnchor::FromStart(px(160.)),
                    PaneExpansionAnchor::Proportion(0.5),
                ]),
            )
            .separator(
                Primary,
                Tertiary,
                PaneSeparator::new("Resize comments divider"),
            )
    }
}

fn open(
    cx: &mut TestAppContext,
    dimensions: (usize, usize),
    reduced: bool,
) -> (Entity<Preview>, &mut VisualTestContext) {
    cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(reduced);
    });
    let mut preview = None;
    let (_, cx) = cx.add_window_view(|window, cx| {
        window.activate_window();
        let view = cx.new(|cx| Preview {
            viewport: size(px(640.), px(340.)),
            dimensions,
            state: cx.new(|_| {
                AdaptivePaneState::new(PaneDestination::new(Secondary).content_key("inbox"))
            }),
            focus: std::array::from_fn(|_| cx.focus_handle()),
            supporting_layout: false,
            scrim: true,
            hit_count: 0,
            background_presses: 0,
        });
        preview = Some(view.clone());
        gpui_base::Root::new(view, window, cx)
    });
    draw(cx);
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

fn found(cx: &mut VisualTestContext, id: impl Into<ElementId>) -> Option<Bounds<Pixels>> {
    let id = id.into();
    cx.update(|window, _| find(window, &[], &id).map(|element| element.bounds()))
}

fn change_layout(
    cx: &mut VisualTestContext,
    view: &Entity<Preview>,
    change: impl FnOnce(&mut Preview),
) {
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            change(view);
            cx.notify();
        })
    });
    draw(cx);
}

fn navigate(cx: &mut VisualTestContext, view: &Entity<Preview>, role: PaneRole, key: &'static str) {
    cx.update(|_, cx| {
        let state = view.read(cx).state.clone();
        state.update(cx, |state, cx| {
            state.navigate_to(PaneDestination::new(role).content_key(key), cx)
        });
    });
    draw(cx);
}

#[gpui::test]
fn three_pane_boundaries_follow_roles_and_snap_only_on_completed_drag(cx: &mut TestAppContext) {
    let (preview, cx) = open(cx, (3, 1), true);
    let divider = ("pane-separator", 3_usize);
    let other_divider = ("pane-separator", 2_usize);
    let list = found(cx, "master").unwrap();
    let detail = found(cx, "detail").unwrap();
    let extra = found(cx, "extra").unwrap();
    assert!(list.right() < detail.left() && detail.right() < extra.left());
    assert!(found(cx, other_divider).is_some());
    cx.update(|window, _| {
        let split = find(window, &[], &divider.into()).unwrap();
        assert_eq!(split.role(), Some(Role::Slider));
        assert_eq!(split.label(), Some("Resize main divider"));
    });
    let point = found(cx, divider).unwrap().center();
    let moved = point + gpui::point(px(128.), px(0.));
    cx.simulate_mouse_down(point, MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_move(moved, MouseButton::Left, Modifiers::default());
    draw(cx);
    let intermediate = cx.update(|_, cx| {
        preview
            .read(cx)
            .state
            .read(cx)
            .expansion(Secondary, Primary)
            .unwrap()
    });
    assert!(matches!(intermediate, PaneExpansionAnchor::FromStart(_)));
    cx.simulate_mouse_up(moved, MouseButton::Left, Modifiers::default());
    draw(cx);
    assert_eq!(
        cx.update(|_, cx| preview
            .read(cx)
            .state
            .read(cx)
            .expansion(Secondary, Primary)),
        Some(PaneExpansionAnchor::Proportion(0.5))
    );

    let start = found(cx, divider).unwrap().center();
    let end = start - gpui::point(px(50.), px(0.));
    cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_move(end, MouseButton::Left, Modifiers::default());
    draw(cx);
    let accepted = cx.update(|_, cx| {
        preview
            .read(cx)
            .state
            .read(cx)
            .expansion(Secondary, Primary)
    });
    change_layout(cx, &preview, |view| view.viewport.width -= px(28.));
    cx.simulate_mouse_up(end, MouseButton::Left, Modifiers::default());
    draw(cx);
    cx.update(|window, cx| {
        assert_eq!(
            preview
                .read(cx)
                .state
                .read(cx)
                .expansion(Secondary, Primary),
            accepted
        );
        assert!(window.captured_hitbox().is_none());
        assert_eq!(
            find(window, &[], &divider.into()).unwrap().focused(),
            Some(true)
        );
        let mut theme = MaterialTheme::default();
        theme.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(theme), window, cx);
    });
    draw(cx);
    let list = found(cx, "master").unwrap();
    let detail = found(cx, "detail").unwrap();
    assert!(list.left() > detail.right());
    cx.simulate_keystrokes("left");
    draw(cx);
    assert!(found(cx, "master").unwrap().size.width > list.size.width);
    for (key, anchor) in [
        ("pagedown", PaneExpansionAnchor::FromStart(px(160.))),
        ("pageup", PaneExpansionAnchor::Proportion(0.5)),
    ] {
        cx.simulate_keystrokes(key);
        draw(cx);
        assert_eq!(
            cx.update(|_, cx| preview
                .read(cx)
                .state
                .read(cx)
                .expansion(Secondary, Primary)),
            Some(anchor),
            "{key} moves to the configured anchor"
        );
    }
    change_layout(cx, &preview, |view| view.dimensions = (2, 1));
    assert!(found(cx, divider).is_some());
    assert!(found(cx, other_divider).is_none());
    change_layout(cx, &preview, |view| view.dimensions = (1, 1));
    assert!(found(cx, divider).is_none());
    cx.update(|window, cx| assert!(preview.read(cx).focus[1].is_focused(window)));
}

#[gpui::test]
fn levitated_pane_blocks_background_and_restores_focus_through_history(cx: &mut TestAppContext) {
    let (preview, cx) = open(cx, (2, 1), true);
    let detail_button = found(cx, "detail-control").unwrap().center();
    cx.simulate_click(detail_button, Modifiers::default());
    draw(cx);
    navigate(cx, &preview, Primary, "march");
    navigate(cx, &preview, Primary, "april");
    navigate(cx, &preview, Tertiary, "comments");
    cx.update(|window, cx| {
        let preview = preview.read(cx);
        let state = preview.state.read(cx);
        assert_eq!(
            state.current_value()[Tertiary],
            PanePlacement::Levitate(PaneOverlayAlignment::BottomEnd)
        );
        assert!(state.can_navigate_back(PaneBackBehavior::PopUntilScaffoldValueChange));
        assert!(preview.focus[2].is_focused(window));
    });
    let extra = found(cx, "extra").unwrap();
    assert!(extra.top() > px(200.) && extra.size.width < px(140.));
    cx.simulate_click(detail_button, Modifiers::default());
    draw(cx);
    cx.update(|_, cx| assert_eq!(preview.read(cx).hit_count, 1));
    assert!(found(cx, ("pane-separator", 3_usize)).is_none());

    let close = found(cx, "extra-control").unwrap().center();
    cx.simulate_click(close, Modifiers::default());
    draw(cx);
    cx.update(|window, cx| {
        let preview = preview.read(cx);
        let state = preview.state.read(cx);
        assert_eq!(
            state.current_destination().content_key,
            Some("april".into())
        );
        assert!(state.can_navigate_back(PaneBackBehavior::PopUntilContentChange));
        assert!(preview.focus[0].is_focused(window));
    });
    assert!(found(cx, "extra").is_none());
    change_layout(cx, &preview, |view| view.dimensions = (1, 1));
    assert!(found(cx, "master").is_none());
    assert!(found(cx, "detail").is_some());
    let back = cx.update(|_, cx| {
        let state = preview.read(cx).state.clone();
        state.update(cx, |state, cx| {
            assert!(state.seek_back(PaneBackBehavior::PopUntilCurrentDestinationChange, 0.5, cx));
            state.history().len()
        })
    });
    draw(cx);
    cx.update(|_, cx| {
        let state = preview.read(cx).state.read(cx);
        assert_eq!(state.history().len(), back);
        assert_eq!(state.target_value()[Secondary], PanePlacement::Expanded);
        assert_eq!(state.progress(), 0.5);
    });
    cx.update(|_, cx| {
        let state = preview.read(cx).state.clone();
        state.update(cx, |state, cx| {
            assert!(state.navigate_back(PaneBackBehavior::PopUntilCurrentDestinationChange, cx));
        });
    });
    draw(cx);
    assert!(found(cx, "master").is_some());
    assert!(found(cx, "detail").is_none());
}

#[gpui::test]
fn supporting_pane_reflows_and_explicit_motion_retargets_from_the_current_frame(
    cx: &mut TestAppContext,
) {
    let (preview, cx) = open(cx, (1, 1), false);
    change_layout(cx, &preview, |view| view.supporting_layout = true);
    change_layout(cx, &preview, |view| view.dimensions = (1, 2));
    cx.executor().advance_clock(Duration::from_millis(250));
    draw(cx);
    let stack_top = found(cx, "detail").unwrap();
    let stack_bottom = found(cx, "master").unwrap();
    assert!(stack_top.bottom() < stack_bottom.top());
    assert_eq!(stack_top.size.width, stack_bottom.size.width);
    assert_eq!(
        cx.update(|_, cx| preview.read(cx).state.read(cx).current_value()[Secondary]),
        PanePlacement::Reflow(Primary)
    );

    let expanded = PaneLayoutValue::new(
        PanePlacement::Expanded,
        PanePlacement::Expanded,
        PanePlacement::Hidden,
    );
    cx.update(|_, cx| {
        let state = preview.read(cx).state.clone();
        state.update(cx, |state, cx| state.seek_to(expanded, 0.4, cx));
    });
    draw(cx);
    let sampled = found(cx, "detail").unwrap();
    assert!(
        sampled.size.width < stack_top.size.width && sampled.size.height > stack_top.size.height
    );
    cx.update(|_, cx| {
        let state = preview.read(cx).state.clone();
        state.update(cx, |state, cx| state.animate_to(expanded, cx));
    });
    draw(cx);
    assert_eq!(found(cx, "detail"), Some(sampled));
    cx.executor().advance_clock(Duration::from_millis(260));
    draw(cx);
    cx.update(|_, cx| {
        let state = preview.read(cx).state.read(cx);
        assert_eq!(state.current_value(), expanded);
        assert_eq!(state.target_value(), expanded);
        assert_eq!(state.progress(), 0.);
    });
}

#[gpui::test]
fn a_nonmodal_levitated_pane_blocks_only_its_bounds(cx: &mut TestAppContext) {
    let (preview, cx) = open(cx, (2, 1), true);
    change_layout(cx, &preview, |view| view.scrim = false);
    let underlying = found(cx, "detail").unwrap();
    let outside = gpui::point(underlying.left() + px(45.), underlying.top() + px(65.));
    cx.simulate_click(outside, Modifiers::default());
    navigate(cx, &preview, Tertiary, "floating-comments");
    let overlay = found(cx, "extra").unwrap();
    let overlap = gpui::point(overlay.left() + px(12.), overlay.bottom() - px(12.));
    assert!(underlying.contains(&overlap));
    cx.simulate_click(overlap, Modifiers::default());
    cx.update(|_, cx| assert_eq!(preview.read(cx).background_presses, 1));
    cx.simulate_click(outside, Modifiers::default());
    cx.update(|_, cx| assert_eq!(preview.read(cx).background_presses, 2));
}

#[gpui::test]
fn levitate_to_reflow_retains_the_child_and_releases_modal_tab_scope(cx: &mut TestAppContext) {
    let (preview, cx) = open(cx, (2, 1), true);
    let detail = found(cx, "detail-control").unwrap().center();
    cx.simulate_click(detail, Modifiers::default());
    navigate(cx, &preview, Tertiary, "comments");
    for key in ["tab", "shift-tab"] {
        cx.simulate_keystrokes(key);
        draw(cx);
        cx.update(|window, cx| {
            let preview = preview.read(cx);
            assert!(preview.focus[2].is_focused(window));
            assert!(gpui_base::active_focus_trap(window, cx).is_some());
        });
    }
    change_layout(cx, &preview, |view| view.dimensions = (1, 2));
    cx.update(|window, cx| {
        let preview = preview.read(cx);
        assert_eq!(
            preview.state.read(cx).current_value()[Tertiary],
            PanePlacement::Reflow(Primary)
        );
        assert!(preview.focus[2].is_focused(window));
        assert!(gpui_base::active_focus_trap(window, cx).is_none());
    });
    cx.simulate_keystrokes("tab");
    draw(cx);
    cx.update(|window, cx| assert!(preview.read(cx).focus[0].is_focused(window)));
}
