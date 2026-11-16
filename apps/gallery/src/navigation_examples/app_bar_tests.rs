use std::{cell::Cell, rc::Rc, time::Duration};

use gpui::{
    App, AppContext, Bounds, Context, ElementId, Entity, Hsla, InteractiveElement, IntoElement,
    KeyDownEvent, KeyUpEvent, Keystroke, ParentElement, Pixels, Render, Role, ScrollHandle,
    StatefulInteractiveElement, Styled, TestAppContext, Window, canvas, div, point, px, rgb,
};
use gpui_base::{TestSupportExt as _, test_support::find};
use gpui_material::{
    AppBar, AppBarAlignment, AppBarColors, AppBarColumn, AppBarItem, AppBarMenuState,
    AppBarPadding, AppBarRow, AppBarScrollBehavior, AppBarScrollState, AppBarVariant, Button, Icon,
    IconName, MaterialRoot, MenuItem, Scaffold,
    foundation::{set_window_material_theme, window_material_theme},
    theme::{CubicBezier, Direction, MotionSpec},
};

type Capture = Rc<Cell<Option<(Bounds<Pixels>, Hsla)>>>;

fn probe(id: &'static str, width: f32, height: f32, state: Capture) -> impl IntoElement {
    div()
        .id(id)
        .test_support()
        .role(Role::Label)
        .aria_label(id)
        .w(px(width))
        .h(px(height))
        .child(
            canvas(
                move |bounds, window, _| state.set(Some((bounds, window.text_style().color))),
                |_, (), _, _| {},
            )
            .size_full(),
        )
}

fn draw(window: &mut Window, cx: &mut App) {
    window.refresh();
    window.draw(cx).clear(cx);
}

fn key(name: &str, window: &mut Window, cx: &mut App) {
    use gpui::InputEvent as _;
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

fn height(window: &Window, id: &'static str) -> f32 {
    f32::from(find(window, &[], &id.into()).unwrap().bounds().size.height)
}

const SURFACE: u32 = 0xd3ecff;
const SCROLLED: u32 = 0xfae6bd;
const TITLE: u32 = 0x5a3879;
const SUBTITLE: u32 = 0x2c513c;
const NAVIGATION: u32 = 0x0b3c75;
const ACTIONS: u32 = 0x6b453b;

fn pigment(hex: u32) -> Hsla {
    rgb(hex).into()
}

struct Headings {
    state: Entity<AppBarScrollState>,
    strategy: AppBarScrollBehavior,
    enabled: bool,
    title: Capture,
    subtitle: Capture,
    navigation: Capture,
    actions: Capture,
    occupied: Rc<Cell<Pixels>>,
}

impl Render for Headings {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let title = self.title.clone();
        let subtitle = self.subtitle.clone();
        let occupied = self.occupied.clone();
        div().w(px(350.)).h(px(290.)).child(
            Scaffold::with_layout("page", move |layout, _, _| {
                occupied.set(layout.occupied.top);
                div()
            })
            .app_bar(
                AppBar::new("rich", "Design workspace")
                    .variant(AppBarVariant::TwoRows)
                    .title_for_rows(move |expanded| {
                        probe(
                            if expanded {
                                "expanded-title"
                            } else {
                                "compact-title"
                            },
                            45.,
                            21.,
                            title,
                        )
                    })
                    .subtitle_for_rows(move |expanded| {
                        probe(
                            if expanded {
                                "expanded-subtitle"
                            } else {
                                "compact-subtitle"
                            },
                            40.,
                            16.,
                            subtitle,
                        )
                    })
                    .title_alignment(AppBarAlignment::End)
                    .subtitle_alignment(AppBarAlignment::Start)
                    .logo(probe("brand", 20., 12., Capture::default()))
                    .second_row(probe("supplement", 25., 9., Capture::default()))
                    .navigation(probe("leading", 30., 25., self.navigation.clone()))
                    .actions(probe("trailing", 36., 25., self.actions.clone()))
                    .colors(AppBarColors {
                        container: Some(pigment(SURFACE)),
                        scrolled_container: Some(pigment(SCROLLED)),
                        title: Some(pigment(TITLE)),
                        subtitle: Some(pigment(SUBTITLE)),
                        navigation: Some(pigment(NAVIGATION)),
                        actions: Some(pigment(ACTIONS)),
                    })
                    .content_padding(AppBarPadding {
                        start: px(11.),
                        end: px(23.),
                        top: px(2.),
                        bottom: px(4.),
                    })
                    .collapsed_height(px(60.))
                    .expanded_height(px(150.))
                    .scroll_state(&self.state)
                    .scroll_behavior(self.strategy)
                    .scroll_enabled(self.enabled)
                    .scroll_motion(MotionSpec::Tween {
                        duration_ms: 200,
                        easing: CubicBezier {
                            x1: 0.,
                            y1: 0.,
                            x2: 1.,
                            y2: 1.,
                        },
                    }),
            ),
        )
    }
}

#[gpui::test]
fn app_bar_rich_rows_move_independently_and_report_measured_scaffold_height(
    cx: &mut TestAppContext,
) {
    let (handle, view, state) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let state = cx.new(|_| AppBarScrollState::new());
        let view = cx.new(|_| Headings {
            state: state.clone(),
            strategy: AppBarScrollBehavior::EnterAlways,
            enabled: true,
            title: Capture::default(),
            subtitle: Capture::default(),
            navigation: Capture::default(),
            actions: Capture::default(),
            occupied: Rc::new(Cell::new(px(0.))),
        });
        let handle =
            MaterialRoot::open_window(Default::default(), cx, |_, _| view.clone()).unwrap();
        (handle.into(), view, state)
    });
    let direction = |cx: &mut TestAppContext, direction| {
        cx.update_window(handle, |_, window, cx| {
            let mut theme = window_material_theme(window, cx).clone();
            theme.layout.direction = direction;
            set_window_material_theme(Some(theme), window, cx);
            draw(window, cx);
            let rich = find(window, &[], &"rich".into()).unwrap();
            assert_eq!(rich.role(), Some(Role::Toolbar));
            assert_eq!(rich.label(), Some("Design workspace"));
            assert_eq!(view.read(cx).occupied.get(), px(150.));
            assert_eq!(rich.bounds().size.height, px(150.));
            assert!(find(window, &[], &"compact-title".into()).is_none());
            assert!(find(window, &[], &"compact-subtitle".into()).is_none());
            let brand = find(window, &[], &"brand".into()).unwrap();
            let supplement = find(window, &[], &"supplement".into()).unwrap();
            let (title, title_color) = view.read(cx).title.get().unwrap();
            let (subtitle, subtitle_color) = view.read(cx).subtitle.get().unwrap();
            let (navigation, navigation_color) = view.read(cx).navigation.get().unwrap();
            let (actions, actions_color) = view.read(cx).actions.get().unwrap();
            assert_eq!(title_color, pigment(TITLE));
            assert_eq!(subtitle_color, pigment(SUBTITLE));
            assert_eq!(navigation_color, pigment(NAVIGATION));
            assert_eq!(actions_color, pigment(ACTIONS));
            assert!(brand.bounds().bottom() <= title.top());
            assert!(subtitle.bottom() <= supplement.bounds().top());
            if direction == Direction::Ltr {
                assert_eq!(title.right(), rich.bounds().right() - px(23.));
                assert_eq!(subtitle.left(), rich.bounds().left() + px(11.));
                assert_eq!(navigation.left(), rich.bounds().left() + px(11.));
                assert_eq!(actions.right(), rich.bounds().right() - px(23.));
            } else {
                assert_eq!(title.left(), rich.bounds().left() + px(23.));
                assert_eq!(subtitle.right(), rich.bounds().right() - px(11.));
                assert_eq!(navigation.right(), rich.bounds().right() - px(11.));
                assert_eq!(actions.left(), rich.bounds().left() + px(23.));
            }
        })
        .unwrap();
    };
    direction(cx, Direction::Ltr);
    direction(cx, Direction::Rtl);

    let set_position = |cx: &mut TestAppContext, offset, expected, strategy| {
        cx.update_window(handle, |_, window, cx| {
            view.update(cx, |view, cx| {
                view.strategy = strategy;
                cx.notify();
            });
            state.update(cx, |state, cx| state.set_content_offset(px(offset), cx));
            draw(window, cx);
            assert_eq!(height(window, "rich"), expected);
            assert_eq!(view.read(cx).occupied.get(), px(expected));
            assert_eq!(state.read(cx).content_offset(), px(offset));
            assert_eq!(state.read(cx).height_offset_limit(), px(-90.));
            assert!(
                (state.read(cx).collapsed_fraction() - ((150. - expected) / 90.)).abs() < 0.00001
            );
            if offset >= 90. {
                assert_eq!(state.read(cx).overlapped_fraction(), 1.);
                assert!(
                    window
                        .painted_quads()
                        .iter()
                        .any(|quad| { quad.background.as_solid() == Some(pigment(SCROLLED)) })
                );
            }
            if expected == 60. {
                assert!(find(window, &[], &"compact-title".into()).is_some());
                assert!(find(window, &[], &"expanded-title".into()).is_none());
                assert!(find(window, &[], &"supplement".into()).is_none());
            }
        })
        .unwrap();
    };
    set_position(cx, 110., 60., AppBarScrollBehavior::EnterAlways);
    set_position(cx, 100., 70., AppBarScrollBehavior::EnterAlways);
    set_position(cx, 140., 150., AppBarScrollBehavior::Pinned);
    set_position(cx, 0., 150., AppBarScrollBehavior::ExitUntilCollapsed);
    set_position(cx, 130., 60., AppBarScrollBehavior::ExitUntilCollapsed);
    set_position(cx, 32., 60., AppBarScrollBehavior::ExitUntilCollapsed);
    set_position(cx, 0., 150., AppBarScrollBehavior::ExitUntilCollapsed);
    cx.update_window(handle, |_, window, cx| {
        view.update(cx, |view, cx| {
            view.strategy = AppBarScrollBehavior::EnterAlways;
            view.enabled = false;
            cx.notify();
        });
        state.update(cx, |state, cx| state.set_content_offset(px(110.), cx));
        draw(window, cx);
        assert_eq!(height(window, "rich"), 150.);
        assert_eq!(state.read(cx).content_offset(), px(0.));
        state.update(cx, |state, cx| state.set_collapsed_fraction(0.4, cx));
        draw(window, cx);
        assert_eq!(height(window, "rich"), 114.);
        state.update(cx, |state, cx| state.settle(100., cx));
        draw(window, cx);
        assert_eq!(height(window, "rich"), 60.);
    })
    .unwrap();

    cx.update(|cx| cx.set_reduce_motion(false));
    cx.update_window(handle, |_, window, cx| {
        state.update(cx, |state, cx| state.set_collapsed_fraction(0.4, cx));
        draw(window, cx);
        state.update(cx, |state, cx| state.settle(100., cx));
        draw(window, cx);
        assert_eq!(height(window, "rich"), 114.);
    })
    .unwrap();
    cx.executor().advance_clock(Duration::from_millis(100));
    cx.update_window(handle, |_, window, cx| {
        draw(window, cx);
        assert!((60. ..114.).contains(&height(window, "rich")));
    })
    .unwrap();
    cx.executor().advance_clock(Duration::from_secs(1));
    cx.update_window(handle, |_, window, cx| {
        draw(window, cx);
        assert_eq!(height(window, "rich"), 60.);
    })
    .unwrap();
}

struct Scrolling {
    scroll: ScrollHandle,
    headings: Capture,
}

impl Render for Scrolling {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(178.))
            .h(px(340.))
            .flex()
            .flex_col()
            .child(AppBar::new("default", "Simple heading"))
            .child(
                AppBar::new(
                    "multiline",
                    "First title line\nSecond title line\nThird title line",
                )
                .title_max_lines(3)
                .subtitle("First subtitle\nSecond subtitle")
                .subtitle_max_lines(2),
            )
            .child(
                AppBar::new("mixed-search", "Accessible search")
                    .variant(AppBarVariant::Centered)
                    .title_content(probe("not-a-search-slot", 20., 20., Capture::default()))
                    .logo(probe("search-logo", 16., 16., Capture::default()))
                    .search(probe("search-slot", 86., 28., self.headings.clone())),
            )
            .child(
                AppBar::new("tracked", "Tracked Library")
                    .variant(AppBarVariant::MediumFlexible)
                    .track_scroll(&self.scroll),
            )
            .child(
                div()
                    .id("content-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .child(div().h(px(1200.))),
            )
    }
}

#[gpui::test]
fn default_app_bar_scroll_and_search_stay_compatible_with_multiline_titles(
    cx: &mut TestAppContext,
) {
    let (handle, view) = cx.update(|cx| {
        crate::init_gallery(cx);
        let view = cx.new(|_| Scrolling {
            scroll: ScrollHandle::new(),
            headings: Capture::default(),
        });
        let handle =
            MaterialRoot::open_window(Default::default(), cx, |_, _| view.clone()).unwrap();
        (handle.into(), view)
    });
    cx.update_window(handle, |_, window, cx| {
        draw(window, cx);
        assert_eq!(height(window, "default"), 64.);
        assert!(height(window, "multiline") > 64.);
        assert_eq!(height(window, "mixed-search"), 64.);
        assert_eq!(height(window, "tracked"), 112.);
        assert!(find(window, &[], &"not-a-search-slot".into()).is_none());
        let logo = find(window, &[], &"search-logo".into()).unwrap();
        let search = find(window, &[], &"search-slot".into()).unwrap();
        assert!(logo.bounds().right() <= search.bounds().left());

        view.read(cx).scroll.set_offset(point(px(0.), px(-30.)));
        draw(window, cx);
        assert_eq!(height(window, "tracked"), 82.);
        view.read(cx).scroll.set_offset(point(px(0.), px(-100.)));
        draw(window, cx);
        assert_eq!(height(window, "tracked"), 64.);
        view.read(cx).scroll.set_offset(point(px(0.), px(-12.)));
        draw(window, cx);
        assert_eq!(height(window, "tracked"), 100.);
    })
    .unwrap();
}

struct Overflow {
    width: f32,
    command: Rc<Cell<usize>>,
    menu: Rc<Cell<Option<AppBarMenuState>>>,
}

impl Render for Overflow {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let command = self.command.clone();
        let menu = self.menu.clone();
        div()
            .flex()
            .flex_col()
            .child(
                div().w(px(self.width)).flex().child(
                    AppBarRow::new(
                        "row",
                        [
                            AppBarItem::clickable("first-action", "Search records", || {
                                Icon::new(IconName::Search)
                            }),
                            AppBarItem::custom(
                                "export-shell",
                                div()
                                    .w(px(72.))
                                    .child(Button::new("export-inline", "Export")),
                                move |menu| {
                                    MenuItem::new("export-menu", "Export records")
                                        .content(|| div().child("Export as CSV"))
                                        .on_activate(move |window, cx| {
                                            command.set(command.get() + 1);
                                            menu.dismiss(window, cx);
                                        })
                                },
                            ),
                            AppBarItem::clickable("last-action", "Show details", || {
                                Icon::new(IconName::Info)
                            }),
                            AppBarItem::clickable("unavailable", "Unavailable action", || {
                                Icon::new(IconName::Archive)
                            })
                            .disabled(true),
                        ],
                    )
                    .overflow_label("More record actions")
                    .overflow_indicator(move |state| {
                        menu.set(Some(state));
                        Icon::new(IconName::ArrowDropDown).size(px(18.))
                    }),
                ),
            )
            .child(
                div().w(px(60.)).h(px(100.)).flex().flex_col().child(
                    AppBarColumn::new(
                        "column",
                        [
                            AppBarItem::clickable("column-open", "Column open", || {
                                Icon::new(IconName::Inbox)
                            }),
                            AppBarItem::clickable("column-rename", "Column rename", || {
                                Icon::new(IconName::Edit)
                            }),
                            AppBarItem::clickable("column-remove", "Column remove", || {
                                Icon::new(IconName::Close)
                            }),
                        ],
                    )
                    .max_item_count(3)
                    .overflow_label("More column actions"),
                ),
            )
    }
}

#[gpui::test]
fn app_bar_actions_overflow_without_duplicate_controls_and_restore_keyboard_focus(
    cx: &mut TestAppContext,
) {
    let (handle, view, command, menu) = cx.update(|cx| {
        crate::init_gallery(cx);
        cx.set_reduce_motion(true);
        let command = Rc::new(Cell::new(0));
        let menu = Rc::new(Cell::new(None));
        let view = cx.new(|_| Overflow {
            width: 280.,
            command: command.clone(),
            menu: menu.clone(),
        });
        let handle =
            MaterialRoot::open_window(Default::default(), cx, |_, _| view.clone()).unwrap();
        (handle.into(), view, command, menu)
    });
    let row_overflow = ElementId::from((ElementId::from("row"), "overflow"));
    let column_overflow = ElementId::from((ElementId::from("column"), "overflow"));
    cx.update_window(handle, |_, window, cx| {
        draw(window, cx);
        assert!(find(window, &[], &row_overflow).is_none());
        let column = find(window, &[], &"column-open".into()).unwrap();
        let more = find(window, &[], &column_overflow).unwrap();
        assert!(column.bounds().bottom() <= more.bounds().top());
        assert!(find(window, &[], &"column-rename".into()).is_none());
        for _ in 0..3 {
            window.focus_next(cx);
            draw(window, cx);
        }
        assert_eq!(
            find(window, &[], &"last-action".into()).unwrap().focused(),
            Some(true)
        );
        view.update(cx, |view, cx| {
            view.width = 130.;
            cx.notify();
        });
        let mut theme = window_material_theme(window, cx).clone();
        theme.layout.direction = Direction::Rtl;
        set_window_material_theme(Some(theme), window, cx);
        draw(window, cx);
        window.simulate_next_frame(cx);
        draw(window, cx);
        let first = find(window, &[], &"first-action".into()).unwrap();
        let more = find(window, &[], &row_overflow).unwrap();
        assert!(more.bounds().right() <= first.bounds().left());
        assert_eq!(more.label(), Some("More record actions"));
        assert_eq!(more.focused(), Some(true));
        assert!(find(window, &[], &"last-action".into()).is_none());
        assert!(find(window, &[], &"export-menu".into()).is_none());

        key("enter", window, cx);
        draw(window, cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| draw(window, cx))
        .unwrap();
    cx.run_until_parked();
    let menu_state = menu.take().expect("overflow indicator received menu state");
    cx.update_window(handle, |_, window, cx| {
        draw(window, cx);
        assert!(menu_state.is_showing());
        assert!(find(window, &[], &"export-inline".into()).is_none());
        let export = find(window, &[], &"export-menu".into()).unwrap();
        assert_eq!(export.role(), Some(Role::MenuItem));
        assert_eq!(export.label(), Some("Export records"));
        assert_eq!(export.focused(), Some(true));
        assert_eq!(
            find(window, &[], &"unavailable".into()).unwrap().disabled(),
            Some(true)
        );
        assert_eq!(
            find(window, &[], &"last-action".into()).unwrap().role(),
            Some(Role::MenuItem)
        );

        key("enter", window, cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        draw(window, cx);
        assert_eq!(command.get(), 1);
        assert!(!menu_state.is_showing());
        assert!(find(window, &[], &"export-menu".into()).is_none());
        assert_eq!(
            find(window, &[], &row_overflow).unwrap().focused(),
            Some(true)
        );

        menu_state.show(window, cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        draw(window, cx);
        assert!(find(window, &[], &"export-menu".into()).is_some());
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        draw(window, cx);
        key("escape", window, cx);
        draw(window, cx);
        assert!(!menu_state.is_showing());
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        draw(window, cx);
        view.update(cx, |view, cx| {
            view.width = 280.;
            cx.notify();
        });
        draw(window, cx);
        window.simulate_next_frame(cx);
        draw(window, cx);
        assert!(find(window, &[], &row_overflow).is_none());
        assert_eq!(
            find(window, &[], &"last-action".into()).unwrap().focused(),
            Some(true)
        );
        assert_eq!(
            find(window, &[], &"last-action".into()).unwrap().role(),
            Some(Role::Button)
        );
    })
    .unwrap();
}
