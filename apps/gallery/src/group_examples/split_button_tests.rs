use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyWindowHandle, App, AppContext, Bounds, Context, ElementId, Entity, FocusHandle, Hsla,
    InputEvent, IntoElement, KeyDownEvent, KeyUpEvent, Keystroke, MouseButton, MouseDownEvent,
    MouseUpEvent, ParentElement, Pixels, Point, Render, Role, Styled, TestAppContext, Window,
    canvas, div, px, rgb,
};
use gpui_base::test_support::find;
use gpui_material::{
    ButtonBorder, ButtonColors, ButtonElevations, ButtonPadding, ButtonShapes, ButtonVariant, Icon,
    IconName, InteractionSource, MaterialRoot, MenuItem, SplitButton, SplitButtonSegmentStyle,
    foundation::{color, set_window_material_theme, window_material_theme},
    theme::{ColorRole, CornerShape, Direction, Elevation},
};

fn id(split: &'static str, part: &'static str) -> ElementId {
    ElementId::from((ElementId::from(split), part))
}

fn ink(hex: u32) -> Hsla {
    rgb(hex).into()
}

const LEADING: u32 = 0xcedeff;
const TRAILING: u32 = 0xc8ead8;
const CHECKED: u32 = 0xead5ff;

type Capture = Rc<Cell<Option<(Bounds<Pixels>, Hsla)>>>;

fn swatch(width: f32, capture: Capture) -> impl IntoElement {
    canvas(
        move |bounds, window, _| capture.set(Some((bounds, window.text_style().color))),
        |_, (), _, _| {},
    )
    .w(px(width))
    .h(px(22.))
}

#[derive(Clone, Copy)]
enum Mode {
    Menu,
    ToggleLast,
    CommandLast,
}

struct Example {
    selected: bool,
    mode: Mode,
    disable_leading: bool,
    disable_trailing: bool,
    leading_source: Entity<InteractionSource>,
    trailing_source: Entity<InteractionSource>,
    focus: (FocusHandle, FocusHandle),
    leading_calls: usize,
    trailing_calls: usize,
    alternative_calls: usize,
    requested: Vec<bool>,
    leading: Capture,
    trailing: Capture,
}

impl Render for Example {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let primary = cx.entity();
        let secondary = cx.entity();
        let menu = cx.entity();
        let toggle = cx.entity();
        let button = SplitButton::new(
            "custom-split",
            "Publish items",
            "Publishing options",
            [
                MenuItem::new("other-publish", "Publish as draft").on_activate(move |_, cx| {
                    menu.update(cx, |this, _| this.alternative_calls += 1);
                }),
            ],
        )
        .spacing(px(7.))
        .disabled(self.disable_leading)
        .trailing_disabled(self.disable_trailing)
        .leading_content(swatch(56., self.leading.clone()))
        .trailing_content(swatch(52., self.trailing.clone()))
        .leading_style(SplitButtonSegmentStyle {
            colors: ButtonColors {
                container: Some(ink(LEADING)),
                content: Some(ink(0x182940)),
                disabled_container: Some(ink(0xd7dbe7)),
                ..Default::default()
            },
            shapes: ButtonShapes {
                resting: Some(CornerShape::Radii([3., 7., 9., 11.])),
                pressed: Some(CornerShape::Radii([2.; 4])),
                ..Default::default()
            },
            border: Some(ButtonBorder {
                width: px(2.),
                color: ink(0x3b4d6e),
            }),
            padding: Some(ButtonPadding {
                start: px(11.),
                end: px(3.),
                top: px(4.),
                bottom: px(2.),
            }),
            elevations: ButtonElevations {
                resting: Some(Elevation::Level2),
                ..Default::default()
            },
        })
        .trailing_style(SplitButtonSegmentStyle {
            colors: ButtonColors {
                container: Some(ink(TRAILING)),
                content: Some(ink(0x254434)),
                checked_container: Some(ink(CHECKED)),
                checked_content: Some(ink(0x45305f)),
                disabled_container: Some(ink(0xc9d2ce)),
                ..Default::default()
            },
            shapes: ButtonShapes {
                resting: Some(CornerShape::Radii([4., 8., 10., 12.])),
                checked: Some(CornerShape::Radii([9.; 4])),
                ..Default::default()
            },
            padding: Some(ButtonPadding {
                start: px(4.),
                end: px(8.),
                ..Default::default()
            }),
            border: Some(ButtonBorder {
                width: px(1.),
                color: ink(0x31533d),
            }),
            elevations: ButtonElevations {
                focused: Some(Elevation::Level3),
                ..Default::default()
            },
        })
        .track_focus(&self.focus.0, &self.focus.1)
        .leading_interaction_source(&self.leading_source)
        .trailing_interaction_source(&self.trailing_source)
        .on_activate(move |_, cx| {
            primary.update(cx, |this, _| this.leading_calls += 1);
        });
        let command = move |_: &mut Window, cx: &mut App| {
            secondary.update(cx, |this, _| this.trailing_calls += 1);
        };
        let change = move |requested, _: &mut Window, cx: &mut App| {
            toggle.update(cx, |this, _| this.requested.push(requested));
        };
        let button = match self.mode {
            Mode::Menu => button,
            Mode::ToggleLast => button
                .on_trailing_activate(command)
                .trailing_toggle(self.selected, change),
            Mode::CommandLast => button
                .trailing_toggle(self.selected, change)
                .on_trailing_activate(command),
        };
        div().flex().items_start().child(button)
    }
}

fn open(cx: &mut TestAppContext) -> (AnyWindowHandle, Entity<Example>, Entity<InteractionSource>) {
    let (handle, view, source) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let source = cx.new(|_| InteractionSource::new());
        let view = cx.new(|cx| Example {
            selected: false,
            mode: Mode::Menu,
            disable_leading: false,
            disable_trailing: false,
            leading_source: source.clone(),
            trailing_source: source.clone(),
            focus: (cx.focus_handle(), cx.focus_handle()),
            leading_calls: 0,
            trailing_calls: 0,
            alternative_calls: 0,
            requested: Vec::new(),
            leading: Capture::default(),
            trailing: Capture::default(),
        });
        let handle =
            MaterialRoot::open_window(Default::default(), cx, |_, _| view.clone()).unwrap();
        (handle.into(), view, source)
    });
    cx.update_window(handle, |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();
    (handle, view, source)
}

fn draw(cx: &mut TestAppContext, handle: AnyWindowHandle) {
    cx.update_window(handle, |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .unwrap();
    cx.run_until_parked();
}

fn key(window: &mut Window, cx: &mut App, name: &str) {
    let keystroke = Keystroke::parse_with_os(name, cx.platform_info().os).unwrap();
    window.dispatch_event(
        KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        }
        .to_platform_input(),
        cx,
    );
    window.dispatch_event(KeyUpEvent { keystroke }.to_platform_input(), cx);
}

fn pointer_down(window: &mut Window, cx: &mut App, position: Point<Pixels>) {
    window.dispatch_event(
        MouseDownEvent {
            button: MouseButton::Left,
            position,
            click_count: 1,
            ..Default::default()
        }
        .to_platform_input(),
        cx,
    );
}

fn pointer_up(window: &mut Window, cx: &mut App, position: Point<Pixels>) {
    window.dispatch_event(
        MouseUpEvent {
            button: MouseButton::Left,
            position,
            click_count: 1,
            ..Default::default()
        }
        .to_platform_input(),
        cx,
    );
}

fn paint(window: &Window, rgb: u32) -> gpui::Quad {
    window
        .painted_quads()
        .into_iter()
        .find(|quad| quad.background.as_solid() == Some(ink(rgb)))
        .expect("segment paint")
}

#[gpui::test]
fn split_button_paints_each_slot_and_state_independently_in_rtl(cx: &mut TestAppContext) {
    let (handle, view, _) = open(cx);
    draw(cx, handle);
    cx.update_window(handle, |_, window, cx| {
        let left = find(window, &[], &id("custom-split", "primary")).unwrap();
        let right = find(window, &[], &id("custom-split", "secondary")).unwrap();
        assert_eq!(left.role(), Some(Role::Button));
        assert_eq!(left.label(), Some("Publish items"));
        assert_eq!(right.role(), Some(Role::Button));
        assert_eq!(right.label(), Some("Publishing options"));
        assert_eq!(right.bounds().left() - left.bounds().right(), px(7.));
        let scale = window.scale_factor();
        let leading = paint(window, LEADING);
        let trailing = paint(window, TRAILING);
        assert_eq!(leading.border_widths.top, px(2.).scale(scale));
        assert_eq!(trailing.border_widths.top, px(1.).scale(scale));
        assert_eq!(leading.corner_radii.top_left, px(3.).scale(scale));
        assert_eq!(trailing.corner_radii.top_left, px(4.).scale(scale));
        assert_eq!(view.read(cx).leading.get().unwrap().1, ink(0x182940));
        assert_eq!(view.read(cx).trailing.get().unwrap().1, ink(0x254434));
        assert_eq!(
            view.read(cx).leading.get().unwrap().0.left() - left.bounds().left(),
            px(11.)
        );
        pointer_down(window, cx, left.bounds().center());
    })
    .unwrap();
    draw(cx, handle);
    cx.update_window(handle, |_, window, cx| {
        assert_eq!(
            paint(window, LEADING).corner_radii.top_left,
            px(2.).scale(window.scale_factor())
        );
        assert_eq!(
            paint(window, TRAILING).corner_radii.top_left,
            px(4.).scale(window.scale_factor())
        );
        let left = find(window, &[], &id("custom-split", "primary")).unwrap();
        pointer_up(window, cx, left.bounds().center());
        assert_eq!(view.read(cx).leading_calls, 1);
        assert_eq!(view.read(cx).trailing_calls, 0);
        let mut theme = window_material_theme(window, cx).clone();
        theme.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(theme), window, cx);
        view.update(cx, |this, cx| {
            this.mode = Mode::ToggleLast;
            this.selected = true;
            cx.notify();
        });
    })
    .unwrap();
    draw(cx, handle);
    cx.update_window(handle, |_, window, cx| {
        let lead = find(window, &[], &id("custom-split", "primary")).unwrap();
        let trail = find(window, &[], &id("custom-split", "secondary")).unwrap();
        assert_eq!(lead.bounds().left() - trail.bounds().right(), px(7.));
        assert_eq!(trail.checked(), Some(true));
        assert_eq!(
            paint(window, LEADING).corner_radii.top_left,
            px(7.).scale(window.scale_factor())
        );
        assert_eq!(
            paint(window, CHECKED).corner_radii.top_left,
            px(9.).scale(window.scale_factor())
        );
        assert_eq!(view.read(cx).leading.get().unwrap().1, ink(0x182940));
        assert_eq!(view.read(cx).trailing.get().unwrap().1, ink(0x45305f));
        assert_eq!(
            lead.bounds().right() - view.read(cx).leading.get().unwrap().0.right(),
            px(11.)
        );
    })
    .unwrap();
}

#[gpui::test]
fn split_button_routes_menu_commands_and_controlled_toggle_and_releases_sources(
    cx: &mut TestAppContext,
) {
    let (handle, view, original) = open(cx);
    let replacement = cx.update(|cx| cx.new(|_| InteractionSource::new()));
    draw(cx, handle);
    let primary_id = id("custom-split", "primary");
    let trailing_id = id("custom-split", "secondary");
    cx.update_window(handle, |_, window, cx| {
        let window_id = window.window_handle().window_id();
        assert!(original.read(cx).state(window_id, &primary_id).is_some());
        assert!(original.read(cx).state(window_id, &trailing_id).is_some());
        window.focus_next(cx);
        assert!(view.read(cx).focus.0.is_focused(window));
        window.focus_next(cx);
        assert!(view.read(cx).focus.1.is_focused(window));
        key(window, cx, "enter");
    })
    .unwrap();
    draw(cx, handle);
    draw(cx, handle);
    cx.update_window(handle, |_, window, cx| {
        assert!(find(window, &[], &"other-publish".into()).is_some());
        assert_eq!(
            find(window, &[], &trailing_id).unwrap().role(),
            Some(Role::Button)
        );
        assert_eq!(
            paint(window, CHECKED).corner_radii.top_left,
            px(9.).scale(window.scale_factor())
        );
        key(window, cx, "escape");
    })
    .unwrap();
    draw(cx, handle);
    cx.update_window(handle, |_, window, cx| {
        assert!(view.read(cx).focus.1.is_focused(window));
        assert!(find(window, &[], &"other-publish".into()).is_none());
        view.update(cx, |this, cx| {
            this.mode = Mode::CommandLast;
            this.leading_source = replacement.clone();
            this.disable_leading = true;
            cx.notify();
        });
    })
    .unwrap();
    draw(cx, handle);
    cx.update_window(handle, |_, window, cx| {
        let window_id = window.window_handle().window_id();
        let primary = find(window, &[], &primary_id).unwrap();
        let trailing = find(window, &[], &trailing_id).unwrap();
        assert_eq!(primary.disabled(), Some(true));
        assert_eq!(trailing.checked(), None);
        assert!(original.read(cx).state(window_id, &primary_id).is_none());
        assert!(original.read(cx).state(window_id, &trailing_id).is_some());
        assert!(
            replacement
                .read(cx)
                .state(window_id, &primary_id)
                .unwrap()
                .disabled
        );
        pointer_down(window, cx, primary.bounds().center());
        pointer_up(window, cx, primary.bounds().center());
        pointer_down(window, cx, trailing.bounds().center());
        pointer_up(window, cx, trailing.bounds().center());
        assert_eq!(view.read(cx).leading_calls, 0);
        assert_eq!(view.read(cx).trailing_calls, 1);
        assert!(view.read(cx).requested.is_empty());
        assert!(find(window, &[], &"other-publish".into()).is_none());
        view.update(cx, |this, cx| {
            this.mode = Mode::ToggleLast;
            this.trailing_source = replacement.clone();
            this.disable_leading = false;
            cx.notify();
        });
    })
    .unwrap();
    draw(cx, handle);
    cx.update_window(handle, |_, window, cx| {
        let window_id = window.window_handle().window_id();
        assert!(original.read(cx).state(window_id, &trailing_id).is_none());
        assert!(
            replacement
                .read(cx)
                .state(window_id, &trailing_id)
                .is_some()
        );
        assert!(
            !replacement
                .read(cx)
                .state(window_id, &primary_id)
                .unwrap()
                .disabled
        );
        assert_eq!(
            find(window, &[], &trailing_id).unwrap().checked(),
            Some(false)
        );
        view.read(cx).focus.1.clone().focus(window, cx);
        key(window, cx, "space");
    })
    .unwrap();
    draw(cx, handle);
    cx.update_window(handle, |_, window, cx| {
        assert_eq!(view.read(cx).requested, [true]);
        assert_eq!(view.read(cx).trailing_calls, 1);
        assert_eq!(view.read(cx).alternative_calls, 0);
        assert_eq!(
            find(window, &[], &trailing_id).unwrap().checked(),
            Some(false)
        );
        view.update(cx, |this, cx| {
            this.selected = true;
            this.disable_trailing = true;
            cx.notify();
        });
    })
    .unwrap();
    draw(cx, handle);
    cx.update_window(handle, |_, window, cx| {
        let window_id = window.window_handle().window_id();
        let trailing = find(window, &[], &trailing_id).unwrap();
        assert_eq!(trailing.checked(), Some(true));
        assert_eq!(trailing.disabled(), Some(true));
        assert!(
            replacement
                .read(cx)
                .state(window_id, &trailing_id)
                .unwrap()
                .disabled
        );
        pointer_down(window, cx, trailing.bounds().center());
        pointer_up(window, cx, trailing.bounds().center());
        assert_eq!(view.read(cx).requested, [true]);
    })
    .unwrap();
}

#[gpui::test]
fn split_button_variants_keep_their_base_paint_and_name_icon_only_buttons(cx: &mut TestAppContext) {
    struct Variants;
    impl Render for Variants {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().flex().flex_col().items_start().children(
                [
                    ("filled", ButtonVariant::Filled),
                    ("tonal", ButtonVariant::FilledTonal),
                    ("elevated", ButtonVariant::Elevated),
                    ("outlined", ButtonVariant::Outlined),
                ]
                .into_iter()
                .map(|(id, variant)| {
                    let split = SplitButton::new(id, id, "Toggle option", [])
                        .variant(variant)
                        .trailing_toggle(true, |_, _, _| {});
                    if variant == ButtonVariant::Filled {
                        split.leading_icon_only(Icon::new(IconName::Add))
                    } else {
                        split.leading_icon(Icon::new(IconName::Add))
                    }
                }),
            )
        }
    }
    let handle = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        MaterialRoot::open_window(Default::default(), cx, |_, cx| cx.new(|_| Variants))
            .unwrap()
            .into()
    });
    draw(cx, handle);
    cx.update_window(handle, |_, window, cx| {
        let quads = window.painted_quads();
        let theme = window_material_theme(window, cx);
        for (name, role) in [
            ("filled", Some(ColorRole::Primary)),
            ("tonal", Some(ColorRole::SecondaryContainer)),
            ("elevated", Some(ColorRole::SurfaceContainerLow)),
            ("outlined", None),
        ] {
            let leading = find(window, &[], &id(name, "primary")).unwrap();
            let trailing = find(window, &[], &id(name, "secondary")).unwrap();
            assert_eq!(leading.label(), Some(name));
            assert_eq!(trailing.label(), Some("Toggle option"));
            assert_eq!(trailing.checked(), Some(true));
            let scale = window.scale_factor();
            let matched = quads.iter().any(|quad| {
                trailing
                    .bounds()
                    .scale(scale)
                    .contains(&quad.bounds.center())
                    && quad.bounds.size.height == px(40.).scale(scale)
                    && if let Some(role) = role {
                        quad.background.as_solid() == Some(color(theme.colors.color(role)))
                    } else {
                        quad.border_widths.top == px(1.).scale(scale)
                    }
            });
            assert!(matched, "default base paint for {name}");
            if name == "filled" {
                assert_eq!(leading.bounds().size.width, px(48.));
            }
        }
    })
    .unwrap();
}
