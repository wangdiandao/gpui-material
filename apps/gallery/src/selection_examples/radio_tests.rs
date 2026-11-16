use gpui::{
    AppContext as _, Axis, Context, ElementId, Entity, Hsla, IntoElement, Modifiers, MouseButton,
    ParentElement as _, Render, Rgba, Role, Styled as _, TestAppContext, VisualTestContext, Window,
    div, px, rgb,
};
use gpui_base::test_support::{ElementSnapshot, find};
use gpui_material::foundation::{color, set_window_material_theme, window_material_theme};
use gpui_material::theme::{ColorRole, Direction};
use gpui_material::{
    InteractionSource, InteractionState, MaterialRoot, MaterialTheme, Radio, RadioColors,
    RadioGroup,
};

const GROUPS: [&str; 2] = ["group-a", "group-b"];

struct Preview {
    source: Entity<InteractionSource>,
    replacement: Entity<InteractionSource>,
    replace_source: bool,
    include_first: bool,
    selected: [ElementId; 2],
    standalone_selected: bool,
}

impl Render for Preview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let groups = GROUPS.into_iter().enumerate().map(|(index, group)| {
            let palette = palette(index);
            let options = [
                ("first", "First", false),
                ("unavailable", "Unavailable choice", true),
                ("middle", "Middle", false),
                ("last", "Last", false),
            ]
            .into_iter()
            .filter(|(id, _, _)| index != 0 || *id != "first" || self.include_first)
            .map(|(id, label, disabled)| {
                let source = if index == 0 && id == "first" && self.replace_source {
                    &self.replacement
                } else {
                    &self.source
                };
                Radio::new(id, label)
                    .disabled(disabled)
                    .colors(palette)
                    .interaction_source(source)
            });
            let owner = cx.weak_entity();
            RadioGroup::new(group, ["Alignment", "Density"][index])
                .axis(Axis::Horizontal)
                .selected(self.selected[index].clone())
                .options(options)
                .on_select(move |id, _, cx| {
                    let _ = owner.update(cx, |this, cx| {
                        this.selected[index] = id.clone();
                        cx.notify();
                    });
                })
        });
        let owner = cx.weak_entity();
        div()
            .w(px(600.))
            .flex()
            .flex_col()
            .gap_2()
            .children(groups)
            .child(
                Radio::new("standalone", "Standalone choice")
                    .selected(self.standalone_selected)
                    .interaction_source(&self.source)
                    .on_select(move |_, cx| {
                        let _ = owner.update(cx, |this, cx| {
                            this.standalone_selected = true;
                            cx.notify();
                        });
                    }),
            )
            .child(
                Radio::new("locked", "Locked selection")
                    .disabled(true)
                    .selected(true)
                    .colors(palette(1)),
            )
    }
}

fn palette(index: usize) -> RadioColors {
    let [selected, unselected, disabled_selected, disabled_unselected] = [
        [0x466800, 0x624000, 0x454545, 0x727272],
        [0x8740bb, 0x006169, 0x832b21, 0x757576],
    ][index]
        .map(|value| Some(Hsla::from(rgb(value))));
    RadioColors {
        selected,
        unselected,
        disabled_selected,
        disabled_unselected,
    }
}

fn option_id(group: &str, option: &'static str) -> ElementId {
    (ElementId::from(group.to_string()), option).into()
}

fn open(cx: &mut TestAppContext) -> (Entity<Preview>, &mut VisualTestContext) {
    cx.update(|cx| {
        MaterialRoot::init(MaterialTheme::default(), cx);
        cx.set_reduce_motion(true);
    });
    let (preview, cx) = cx.add_window_view(|window, cx| {
        window.activate_window();
        Preview {
            source: cx.new(|_| InteractionSource::new()),
            replacement: cx.new(|_| InteractionSource::new()),
            replace_source: false,
            include_first: true,
            selected: ["first".into(), "first".into()],
            standalone_selected: false,
        }
    });
    draw(cx);
    (preview, cx)
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
    cx.run_until_parked();
}

fn snapshot(window: &Window, id: &ElementId) -> ElementSnapshot {
    find(window, &[], id).expect("present radio")
}

fn indicator(window: &Window, view: &ElementSnapshot, tint: Hsla) {
    let scale = window.scale_factor();
    let bounds = view.bounds().scale(scale);
    let expected = Rgba::from(tint);
    assert!(
        window.painted_quads().iter().any(|quad| {
            let actual = Rgba::from(quad.border_color);
            (px(19.).scale(scale)..=px(21.).scale(scale)).contains(&quad.bounds.size.width)
                && quad.border_widths.top == px(2.).scale(scale)
                && bounds.contains(&quad.bounds.center())
                && [
                    actual.r - expected.r,
                    actual.g - expected.g,
                    actual.b - expected.b,
                    actual.a - expected.a,
                ]
                .into_iter()
                .all(|difference| difference.abs() < 0.0001)
        }),
        "expected indicator color {tint:?} for {:?}",
        view.label()
    );
}

fn assert_group(window: &Window, group: usize, selected: &str) {
    let colors = palette(group);
    for (option, label) in [
        ("first", "First"),
        ("unavailable", "Unavailable choice"),
        ("middle", "Middle"),
        ("last", "Last"),
    ] {
        let row = snapshot(window, &option_id(GROUPS[group], option));
        assert_eq!(row.label(), Some(label));
        assert_eq!(row.role(), Some(Role::RadioButton));
        assert_eq!(row.checked(), Some(option == selected));
        assert_eq!(row.selected(), Some(option == selected));
        let tint = if option == "unavailable" {
            assert_eq!(row.disabled(), Some(true));
            colors.disabled_unselected
        } else if option == selected {
            colors.selected
        } else {
            colors.unselected
        }
        .unwrap();
        indicator(window, &row, tint);
    }
}

#[gpui::test]
fn radio_groups_keep_independent_colors_selection_and_rtl_focus(cx: &mut TestAppContext) {
    let (_, cx) = open(cx);
    cx.update(|window, cx| {
        for group in 0..2 {
            assert_group(window, group, "first");
        }
        let standalone = snapshot(window, &"standalone".into());
        assert_eq!(standalone.label(), Some("Standalone choice"));
        let theme = window_material_theme(window, cx);
        indicator(
            window,
            &standalone,
            color(theme.colors.color(ColorRole::OnSurfaceVariant)),
        );
        let locked = snapshot(window, &"locked".into());
        assert_eq!(locked.checked(), Some(true));
        assert_eq!(locked.disabled(), Some(true));
        indicator(window, &locked, palette(1).disabled_selected.unwrap());
        window.focus_next(cx);
    });
    draw(cx);
    cx.simulate_keystrokes("right");
    draw(cx);
    cx.update(|window, cx| {
        assert_group(window, 0, "middle");
        assert_group(window, 1, "first");
        assert_eq!(
            snapshot(window, &option_id(GROUPS[0], "middle")).focused(),
            Some(true)
        );
        window.focus_next(cx);
    });
    draw(cx);
    cx.update(|window, cx| {
        assert_eq!(
            snapshot(window, &option_id(GROUPS[1], "first")).focused(),
            Some(true)
        );
        window.focus_prev(cx);
        let mut theme = MaterialTheme::default();
        theme.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(theme), window, cx);
    });
    draw(cx);
    cx.simulate_keystrokes("right");
    draw(cx);
    cx.update(|window, _| {
        assert_group(window, 0, "first");
        assert_group(window, 1, "first");
        let first = snapshot(window, &option_id(GROUPS[0], "first"));
        let last = snapshot(window, &option_id(GROUPS[0], "last"));
        assert_eq!(first.focused(), Some(true));
        assert!(first.bounds().left() > last.bounds().right());
    });
    cx.simulate_keystrokes("left");
    draw(cx);
    cx.update(|window, _| {
        assert_group(window, 0, "middle");
        assert_group(window, 1, "first");
    });
}

fn observed(
    cx: &mut VisualTestContext,
    preview: &Entity<Preview>,
    id: &ElementId,
    replacement: bool,
) -> Option<InteractionState> {
    cx.update(|window, cx| {
        let preview = preview.read(cx);
        let source = if replacement {
            &preview.replacement
        } else {
            &preview.source
        };
        source
            .read(cx)
            .state(window.window_handle().window_id(), id)
    })
}

#[gpui::test]
fn radio_observation_tracks_standalone_input_and_detaches_group_options(cx: &mut TestAppContext) {
    let (preview, cx) = open(cx);
    let observed_option = option_id(GROUPS[0], "first");
    let other_group = option_id(GROUPS[1], "first");
    assert!(observed(cx, &preview, &other_group, false).is_some());
    assert!(observed(cx, &preview, &observed_option, false).is_some());

    let standalone = ElementId::from("standalone");
    let position = cx.update(|window, _| snapshot(window, &standalone).bounds().center());
    cx.simulate_mouse_move(position, None, Modifiers::default());
    cx.simulate_mouse_down(position, MouseButton::Left, Modifiers::default());
    draw(cx);
    let interaction = observed(cx, &preview, &standalone, false).unwrap();
    assert!(interaction.hovered && interaction.pressed);
    cx.simulate_mouse_up(position, MouseButton::Left, Modifiers::default());
    draw(cx);
    cx.update(|window, _| {
        assert_eq!(snapshot(window, &standalone).checked(), Some(true));
    });
    assert!(!observed(cx, &preview, &standalone, false).unwrap().pressed);

    cx.update(|_, cx| {
        preview.update(cx, |this, cx| {
            this.replace_source = true;
            cx.notify();
        });
    });
    draw(cx);
    assert!(observed(cx, &preview, &observed_option, false).is_none());
    assert!(observed(cx, &preview, &observed_option, true).is_some());
    assert!(observed(cx, &preview, &other_group, false).is_some());

    cx.update(|_, cx| {
        preview.update(cx, |this, cx| {
            this.include_first = false;
            this.selected[0] = "middle".into();
            cx.notify();
        });
    });
    draw(cx);
    assert!(observed(cx, &preview, &observed_option, true).is_none());
    assert!(observed(cx, &preview, &other_group, false).is_some());
}
