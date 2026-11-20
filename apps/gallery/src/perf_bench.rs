//! Release frame-time benchmark suite separating Engine, isolated Component,
//! standalone Story, and full Gallery shell overhead.
//!
//! Run in locked Release mode:
//! `cargo test --locked --release -p gpui-material-gallery --features test-support --lib perf_bench -- --ignored --nocapture --test-threads=1`

use std::{
    cell::Cell,
    rc::Rc,
    time::{Duration, Instant},
};

use gpui::{
    App, AppContext, Context, Entity, InputEvent as _, IntoElement, KeyDownEvent, Keystroke,
    ListAlignment, ListState, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    ParentElement, Render, Styled, TestAppContext, Window, WindowBounds, WindowOptions, div, list,
    point, px, size,
};
use gpui_base::test_support::find;
use gpui_material::{
    AppBar, AppBarVariant, Button, ButtonVariant, CircularProgress, DisplayScrollbar, Icon,
    IconName, LinearProgress, ListItem, LoadingIndicator, MaterialRoot, NavigationBar,
    NavigationItem, NavigationItems, NavigationRail, NavigationRailState, ProgressShape, RailMode,
    Scaffold, Switch, TabVariant, Tabs, Text, TextAreaState, TextField, TextFieldState,
    TextFieldVariant, foundation::color, theme::ColorRole,
};

use crate::{Gallery, Story, init_gallery};

const VIEWPORT_WIDTH: f32 = 1040.;
const VIEWPORT_HEIGHT: f32 = 744.;
const WARMUP_FRAMES: usize = 20;
const SAMPLE_FRAMES: usize = 100;
const FRAME_STEP: Duration = Duration::from_millis(16);

#[derive(Clone, Debug)]
struct BenchStats {
    name: &'static str,
    layer: &'static str,
    p50_us: f64,
    p95_us: f64,
    p99_us: f64,
    mean_us: f64,
    quads: usize,
    scheduled_frames: usize,
}

fn bench_window_options() -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::new(
            point(px(0.), px(0.)),
            size(px(VIEWPORT_WIDTH), px(VIEWPORT_HEIGHT)),
        ))),
        ..Default::default()
    }
}

fn summarize(
    name: &'static str,
    layer: &'static str,
    samples: &mut [Duration],
    quads: usize,
    scheduled_frames: usize,
) -> BenchStats {
    samples.sort_unstable();
    let count = samples.len();
    let pct = |p: usize| samples[(count * p).div_ceil(100) - 1].as_secs_f64() * 1e6;
    let mean_us = samples.iter().map(Duration::as_secs_f64).sum::<f64>() * 1e6 / count as f64;
    BenchStats {
        name,
        layer,
        p50_us: pct(50),
        p95_us: pct(95),
        p99_us: pct(99),
        mean_us,
        quads,
        scheduled_frames,
    }
}

fn measure_frames<V: Render + 'static>(
    name: &'static str,
    layer: &'static str,
    cx: &mut TestAppContext,
    reduce_motion: bool,
    build: impl FnOnce(&mut Window, &mut App) -> Entity<V> + 'static,
    mut step: impl FnMut(usize, &Entity<V>, &mut Window, &mut App),
) -> BenchStats {
    let mut built: Option<Entity<V>> = None;
    let window = cx.update(|cx| {
        init_gallery(cx);
        cx.set_reduce_motion(reduce_motion);
        MaterialRoot::open_window(bench_window_options(), cx, |window, cx| {
            let entity = build(window, cx);
            built = Some(entity.clone());
            entity
        })
        .unwrap()
    });
    let entity = built.expect("view entity");

    cx.update_window(window.into(), |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();

    for i in 0..WARMUP_FRAMES {
        cx.executor().advance_clock(FRAME_STEP);
        cx.update_window(window.into(), |_, window, cx| {
            window.simulate_next_frame(cx);
            step(i, &entity, window, cx);
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
    }

    let mut samples = Vec::with_capacity(SAMPLE_FRAMES);
    let mut last_quads = 0;
    let mut total_scheduled = 0;

    for i in 0..SAMPLE_FRAMES {
        cx.executor().advance_clock(FRAME_STEP);
        cx.update_window(window.into(), |_, window, cx| {
            let drained = window.simulate_next_frame(cx);
            if i > 0 {
                total_scheduled += drained;
            }
            step(WARMUP_FRAMES + i, &entity, window, cx);
            let start = Instant::now();
            window.refresh();
            window.draw(cx).clear(cx);
            samples.push(start.elapsed());
            last_quads = window.painted_quads().len();
        })
        .unwrap();
    }

    cx.update_window(window.into(), |_, window, _| window.remove_window())
        .unwrap();
    cx.run_until_parked();

    summarize(name, layer, &mut samples, last_quads, total_scheduled)
}

// --- Layer 1: Bare GPUI Engine Views (zero Material presentation) ---

struct EmptyEngineView;

impl Render for EmptyEngineView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full()
    }
}

struct RawQuads64View;

impl Render for RawQuads64View {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .p_4()
            .flex()
            .flex_wrap()
            .gap_2()
            .children((0..64).map(|i| {
                div()
                    .w(px(110.))
                    .h(px(40.))
                    .rounded(px(20.))
                    .bg(gpui::rgb(0x6750a4))
                    .text_color(gpui::white())
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(format!("Item {i:02}"))
            }))
    }
}

struct RawVirtualListView {
    list: ListState,
}

impl Render for RawVirtualListView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().p_4().child(
            list(self.list.clone(), |ix, _, _| {
                div()
                    .h(px(64.))
                    .w_full()
                    .px_4()
                    .flex()
                    .items_center()
                    .justify_between()
                    .bg(gpui::rgb(0xf7f2fa))
                    .border_b_1()
                    .border_color(gpui::rgb(0xcac4d0))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(format!("Row headline {ix:03}"))
                            .child(format!("Supporting description for item {ix:03}")),
                    )
                    .child(format!("#{ix:03}"))
                    .into_any_element()
            })
            .size_full(),
        )
    }
}

// --- Layer 2: Isolated Material Component Views (zero Gallery shell) ---

struct ButtonGridView {
    count: usize,
}

impl Render for ButtonGridView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let variants = [
            ButtonVariant::Filled,
            ButtonVariant::FilledTonal,
            ButtonVariant::Elevated,
            ButtonVariant::Outlined,
        ];
        div()
            .size_full()
            .p_4()
            .flex()
            .flex_wrap()
            .gap_2()
            .children((0..self.count).map(|i| {
                Button::new(("bench-btn", i), format!("Action {i:02}"))
                    .variant(variants[i % variants.len()])
                    .on_activate(|_, _| {})
            }))
    }
}

struct TextFieldBenchView {
    filled: Entity<TextFieldState>,
    outlined: Entity<TextFieldState>,
    multiline: Entity<TextAreaState>,
}

impl Render for TextFieldBenchView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .p_4()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                TextField::new("bench-filled", "Project title", &self.filled)
                    .variant(TextFieldVariant::Filled)
                    .supporting_text("Material 3 filled field"),
            )
            .child(
                TextField::new("bench-outlined", "Description", &self.outlined)
                    .variant(TextFieldVariant::Outlined)
                    .supporting_text("Material 3 outlined field")
                    .character_count(120),
            )
            .child(
                div().h(px(160.)).child(
                    TextField::multiline("bench-multiline", "Notes", &self.multiline)
                        .variant(TextFieldVariant::Outlined),
                ),
            )
    }
}

struct ComplexListBenchView {
    list: ListState,
    selected: Rc<Cell<usize>>,
}

impl Render for ComplexListBenchView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let selected = self.selected.clone();
        let scroll = self.list.clone();
        div()
            .size_full()
            .p_4()
            .relative()
            .child(
                list(self.list.clone(), move |ix, _, _| {
                    let is_selected = selected.get() == ix;
                    let checked = ix % 2 == 0;
                    let selected_ref = selected.clone();
                    ListItem::new(("bench-row", ix), format!("Workflow pipeline {ix:03}"))
                        .overline("MATERIAL")
                        .supporting_text(format!(
                            "Automated verification row #{ix:03} with metadata"
                        ))
                        .leading(Icon::new(IconName::Favorite))
                        .trailing(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    Switch::new(("bench-sw", ix), format!("Enable {ix}"), checked)
                                        .on_change(|_, _, _| {}),
                                )
                                .child(
                                    Button::new(("bench-row-btn", ix), "Open")
                                        .variant(ButtonVariant::Text)
                                        .on_activate(|_, _| {}),
                                ),
                        )
                        .selected(is_selected)
                        .on_activate(move |_, _| selected_ref.set(ix))
                        .into_any_element()
                })
                .size_full(),
            )
            .child(DisplayScrollbar::new(
                "bench-list-scroll",
                gpui::Axis::Vertical,
                &scroll,
            ))
    }
}

struct NavigationBenchView {
    rail_state: Entity<NavigationRailState>,
    selected_dest: gpui::SharedString,
    selected_tab: gpui::SharedString,
}

impl Render for NavigationBenchView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let destinations = [
            ("inbox", "Inbox", IconName::Favorite),
            ("search", "Search", IconName::Search),
            ("checks", "Checks", IconName::Check),
            ("more", "More", IconName::ArrowDropDown),
        ];
        let rail_items = NavigationItems::new(
            "bench-rail-items",
            "Primary destinations",
            destinations
                .iter()
                .map(|(id, label, icon)| NavigationItem::new(*id, *label).icon(Icon::new(*icon))),
        )
        .selected(self.selected_dest.clone())
        .on_select(cx.listener(|this, id: &gpui::SharedString, _, cx| {
            this.selected_dest = id.clone();
            cx.notify();
        }));
        let bar_items = NavigationItems::new(
            "bench-bar-items",
            "Bottom destinations",
            destinations
                .iter()
                .map(|(id, label, icon)| NavigationItem::new(*id, *label).icon(Icon::new(*icon))),
        )
        .selected(self.selected_dest.clone())
        .on_select(cx.listener(|this, id: &gpui::SharedString, _, cx| {
            this.selected_dest = id.clone();
            cx.notify();
        }));
        let tab_items = NavigationItems::new(
            "bench-tabs-items",
            "Workspace tabs",
            [
                ("overview", "Overview"),
                ("metrics", "Metrics"),
                ("traces", "Traces"),
                ("logs", "Logs"),
            ]
            .into_iter()
            .map(|(id, label)| NavigationItem::new(id, label).icon(Icon::new(IconName::Check))),
        )
        .selected(self.selected_tab.clone())
        .on_select(cx.listener(|this, id: &gpui::SharedString, _, cx| {
            this.selected_tab = id.clone();
            cx.notify();
        }));

        let body = div()
            .size_full()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .child(Tabs::new(tab_items).variant(TabVariant::Primary))
            .child(Text::new("Navigation benchmark content"));

        Scaffold::new("bench-scaffold", body)
            .app_bar(AppBar::new("bench-app-bar", "Workspace").variant(AppBarVariant::Small))
            .rail(NavigationRail::new("bench-rail", rail_items).state(&self.rail_state))
            .bottom_bar(NavigationBar::new(bar_items))
    }
}

struct AnimatedBenchView {
    offscreen: bool,
}

impl Render for AnimatedBenchView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let surface = color(
            gpui_material::foundation::current_material_theme(window, cx)
                .colors
                .color(ColorRole::Surface),
        );
        div()
            .w(px(400.))
            .h(px(220.))
            .bg(surface)
            .overflow_hidden()
            .flex()
            .flex_col()
            .gap_4()
            .p_4()
            .child(
                div()
                    .h(if self.offscreen { px(400.) } else { px(0.) })
                    .flex_none(),
            )
            .child(LoadingIndicator::new("bench-load", "Loading").contained(true))
            .child(
                LinearProgress::new("bench-linear", "Syncing")
                    .indeterminate()
                    .shape(ProgressShape::Wavy),
            )
            .child(
                CircularProgress::new("bench-circular", "Processing")
                    .indeterminate()
                    .shape(ProgressShape::Wavy),
            )
    }
}

#[gpui::test]
#[ignore = "Release performance benchmark"]
fn release_performance_baseline(cx: &mut TestAppContext) {
    let mut results = Vec::new();

    // --- Layer 1: Engine Baseline ---
    results.push(measure_frames(
        "engine/empty_window",
        "Engine",
        cx,
        false,
        |_, cx| cx.new(|_| EmptyEngineView),
        |_, _, _, _| {},
    ));
    results.push(measure_frames(
        "engine/raw_quads_64",
        "Engine",
        cx,
        false,
        |_, cx| cx.new(|_| RawQuads64View),
        |_, _, _, _| {},
    ));
    results.push(measure_frames(
        "engine/raw_virtual_list_200",
        "Engine",
        cx,
        false,
        |_, cx| {
            let list = ListState::new(200, ListAlignment::Top, px(256.));
            cx.new(|_| RawVirtualListView { list })
        },
        |frame, view, _, cx| {
            view.update(cx, |v, _| {
                v.list.scroll_to_reveal_item((frame * 3) % 200);
            });
        },
    ));

    // --- Layer 2: Isolated Material Components ---
    results.push(measure_frames(
        "component/button_grid_64_idle",
        "Component",
        cx,
        false,
        |_, cx| cx.new(|_| ButtonGridView { count: 64 }),
        |_, _, _, _| {},
    ));
    results.push(measure_frames(
        "component/button_grid_64_active",
        "Component",
        cx,
        false,
        |_, cx| cx.new(|_| ButtonGridView { count: 64 }),
        |frame, _, window, cx| {
            let target_ix = (frame / 4) % 16;
            if let Some(node) = find(window, &[], &("bench-btn", target_ix).into()) {
                let pos = node.bounds().center();
                let phase = frame % 4;
                if phase == 0 {
                    window.dispatch_event(
                        MouseMoveEvent {
                            position: pos,
                            pressed_button: None,
                            modifiers: Default::default(),
                        }
                        .to_platform_input(),
                        cx,
                    );
                } else if phase == 1 {
                    window.dispatch_event(
                        MouseDownEvent {
                            button: MouseButton::Left,
                            position: pos,
                            click_count: 1,
                            first_mouse: false,
                            modifiers: Default::default(),
                        }
                        .to_platform_input(),
                        cx,
                    );
                } else if phase == 2 {
                    window.dispatch_event(
                        MouseUpEvent {
                            button: MouseButton::Left,
                            position: pos,
                            click_count: 1,
                            modifiers: Default::default(),
                        }
                        .to_platform_input(),
                        cx,
                    );
                }
            }
        },
    ));

    let long_notes = (0..30)
        .map(|i| format!("Line {i:02} — Release benchmark multiline notes"))
        .collect::<Vec<_>>()
        .join("\n");
    let notes_idle = long_notes.clone();
    results.push(measure_frames(
        "component/text_field_idle",
        "Component",
        cx,
        false,
        move |window, cx| {
            let filled =
                cx.new(|cx| TextFieldState::new(window, cx).default_value("GPUI Material"));
            let outlined = cx.new(|cx| {
                TextFieldState::new(window, cx).default_value("Release performance baseline")
            });
            let multiline = cx.new(|cx| {
                TextAreaState::new(window, cx)
                    .rows(4)
                    .default_value(notes_idle)
            });
            cx.new(|_| TextFieldBenchView {
                filled,
                outlined,
                multiline,
            })
        },
        |_, _, _, _| {},
    ));

    let notes_active = long_notes;
    results.push(measure_frames(
        "component/text_field_typing",
        "Component",
        cx,
        false,
        move |window, cx| {
            let filled = cx.new(|cx| TextFieldState::new(window, cx).default_value("Bench"));
            let outlined = cx.new(|cx| TextFieldState::new(window, cx).default_value("Input"));
            let multiline = cx.new(|cx| {
                TextAreaState::new(window, cx)
                    .rows(4)
                    .default_value(notes_active)
            });
            filled.update(cx, |state, cx| state.focus(window, cx));
            cx.new(|_| TextFieldBenchView {
                filled,
                outlined,
                multiline,
            })
        },
        |frame, _, window, cx| {
            let key = if frame % 2 == 0 {
                Keystroke::parse("a").unwrap()
            } else {
                Keystroke::parse("backspace").unwrap()
            };
            window.dispatch_event(
                KeyDownEvent {
                    keystroke: key,
                    is_held: false,
                    prefer_character_input: true,
                }
                .to_platform_input(),
                cx,
            );
        },
    ));

    results.push(measure_frames(
        "component/complex_list_200_idle",
        "Component",
        cx,
        false,
        |_, cx| {
            let list = ListState::new(200, ListAlignment::Top, px(256.));
            cx.new(|_| ComplexListBenchView {
                list,
                selected: Rc::new(Cell::new(0)),
            })
        },
        |_, _, _, _| {},
    ));

    results.push(measure_frames(
        "component/complex_list_200_scroll",
        "Component",
        cx,
        false,
        |_, cx| {
            let list = ListState::new(200, ListAlignment::Top, px(256.));
            cx.new(|_| ComplexListBenchView {
                list,
                selected: Rc::new(Cell::new(0)),
            })
        },
        |frame, view, _, cx| {
            view.update(cx, |v, _| {
                let target = (frame * 3) % 200;
                v.selected.set(target);
                v.list.scroll_to_reveal_item(target);
            });
        },
    ));

    results.push(measure_frames(
        "component/navigation_scaffold_idle",
        "Component",
        cx,
        false,
        |_, cx| {
            let rail_state = cx.new(|_| NavigationRailState::new(RailMode::Collapsed));
            cx.new(|_| NavigationBenchView {
                rail_state,
                selected_dest: "inbox".into(),
                selected_tab: "overview".into(),
            })
        },
        |_, _, _, _| {},
    ));

    let tab_ids = ["overview", "metrics", "traces", "logs"];
    results.push(measure_frames(
        "component/navigation_rail_transition",
        "Component",
        cx,
        false,
        |_, cx| {
            let rail_state = cx.new(|_| {
                NavigationRailState::new(RailMode::Collapsed)
                    .transition_duration(Duration::from_millis(240))
            });
            cx.new(|_| NavigationBenchView {
                rail_state,
                selected_dest: "inbox".into(),
                selected_tab: "overview".into(),
            })
        },
        move |frame, view, _, cx| {
            if frame % 12 == 0 {
                let next_tab = tab_ids[(frame / 12) % tab_ids.len()];
                view.update(cx, |v, cx| {
                    v.rail_state.update(cx, |rail, cx| rail.toggle(cx));
                    v.selected_tab = next_tab.into();
                    cx.notify();
                });
            }
        },
    ));

    results.push(measure_frames(
        "component/animated_visible",
        "Component",
        cx,
        false,
        |_, cx| cx.new(|_| AnimatedBenchView { offscreen: false }),
        |_, _, _, _| {},
    ));
    results.push(measure_frames(
        "component/animated_clipped_offscreen",
        "Component",
        cx,
        false,
        |_, cx| cx.new(|_| AnimatedBenchView { offscreen: true }),
        |_, _, _, _| {},
    ));
    results.push(measure_frames(
        "component/animated_reduced_motion",
        "Component",
        cx,
        true,
        |_, cx| cx.new(|_| AnimatedBenchView { offscreen: false }),
        |_, _, _, _| {},
    ));

    // --- Layer 3: Standalone Story vs Full Gallery Shell ---
    for (story, story_name, gallery_name) in [
        (Story::Button, "story/button", "gallery/button"),
        (Story::TextField, "story/text_field", "gallery/text_field"),
        (Story::List, "story/list", "gallery/list"),
        (
            Story::NavigationRail,
            "story/navigation_rail",
            "gallery/navigation_rail",
        ),
        (
            Story::LoadingIndicator,
            "story/loading_indicator",
            "gallery/loading_indicator",
        ),
    ] {
        results.push(measure_frames(
            story_name,
            "Story",
            cx,
            false,
            move |window, cx| {
                let view = story.new_view(window, cx).expect("story view");
                cx.new(|_| StoryHostView { view })
            },
            |_, _, _, _| {},
        ));
        results.push(measure_frames(
            gallery_name,
            "Gallery",
            cx,
            false,
            move |window, cx| cx.new(|cx| Gallery::new(story, window, cx)),
            |_, _, _, _| {},
        ));
    }

    println!(
        "\n=== GPUI Material Release Performance Baseline ({VIEWPORT_WIDTH}x{VIEWPORT_HEIGHT}, {WARMUP_FRAMES} warmup + {SAMPLE_FRAMES} frames) ==="
    );
    println!(
        "| {:<36} | {:<9} | {:>8} | {:>8} | {:>8} | {:>8} | {:>5} | {:>6} |",
        "Benchmark", "Layer", "p50 us", "p95 us", "p99 us", "mean us", "quads", "frames"
    );
    println!(
        "| {:-<36} | {:-<9} | {:-<8} | {:-<8} | {:-<8} | {:-<8} | {:-<5} | {:-<6} |",
        "", "", "", "", "", "", "", ""
    );
    for r in &results {
        println!(
            "| {:<36} | {:<9} | {:>8.1} | {:>8.1} | {:>8.1} | {:>8.1} | {:>5} | {:>6} |",
            r.name, r.layer, r.p50_us, r.p95_us, r.p99_us, r.mean_us, r.quads, r.scheduled_frames
        );
    }

    let find_stat = |name: &str| results.iter().find(|r| r.name == name).unwrap();
    println!("\n=== Layer Attribution (p50 us) ===");
    let raw_64 = find_stat("engine/raw_quads_64").p50_us;
    let btn_64 = find_stat("component/button_grid_64_idle").p50_us;
    let raw_list = find_stat("engine/raw_virtual_list_200").p50_us;
    let mat_list = find_stat("component/complex_list_200_scroll").p50_us;
    println!(
        "Button 64 grid Material component overhead over raw 64 GPUI quads: {:+.1} us (component {:.1} us vs engine {:.1} us)",
        btn_64 - raw_64,
        btn_64,
        raw_64
    );
    println!(
        "Complex 200-row virtual list Material overhead over raw GPUI list: {:+.1} us (component {:.1} us vs engine {:.1} us)",
        mat_list - raw_list,
        mat_list,
        raw_list
    );
    for (story_name, gallery_name) in [
        ("story/button", "gallery/button"),
        ("story/text_field", "gallery/text_field"),
        ("story/list", "gallery/list"),
        ("story/navigation_rail", "gallery/navigation_rail"),
        ("story/loading_indicator", "gallery/loading_indicator"),
    ] {
        let s = find_stat(story_name).p50_us;
        let g = find_stat(gallery_name).p50_us;
        println!(
            "Gallery shell overhead on {:<24}: {:+.1} us (gallery {:.1} us vs standalone story {:.1} us)",
            gallery_name,
            g - s,
            g,
            s
        );
    }

    // Sanity invariant: off-screen clipped animated indicators must schedule zero frames.
    assert_eq!(
        find_stat("component/animated_clipped_offscreen").scheduled_frames,
        0
    );
    assert_eq!(
        find_stat("component/animated_reduced_motion").scheduled_frames,
        0
    );
    assert!(find_stat("component/animated_visible").scheduled_frames > 0);
}

struct StoryHostView {
    view: gpui::AnyView,
}

impl Render for StoryHostView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.view.clone())
    }
}
