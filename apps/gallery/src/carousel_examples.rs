use gpui::{Context, ParentElement, Render, SharedString, Styled, Window, div, px};
use gpui_material::{
    Carousel, CarouselItem, CarouselVariant, Icon, IconName, Surface, Text,
    theme::{ColorRole, TypeRole},
};

use crate::story::{section, stack};

struct Place {
    id: &'static str,
    title: &'static str,
    icon: IconName,
    color: ColorRole,
    aspect: f32,
}

const PLACES: [Place; 6] = [
    Place {
        id: "cedar",
        title: "Cedar trail",
        icon: IconName::Favorite,
        color: ColorRole::PrimaryContainer,
        aspect: 1.65,
    },
    Place {
        id: "harbor",
        title: "Old harbor",
        icon: IconName::Archive,
        color: ColorRole::TertiaryContainer,
        aspect: 0.68,
    },
    Place {
        id: "market",
        title: "Saturday market",
        icon: IconName::Add,
        color: ColorRole::SecondaryContainer,
        aspect: 1.20,
    },
    Place {
        id: "ridge",
        title: "Granite ridge",
        icon: IconName::Contrast,
        color: ColorRole::PrimaryContainer,
        aspect: 0.90,
    },
    Place {
        id: "orchard",
        title: "Apple orchard",
        icon: IconName::Edit,
        color: ColorRole::TertiaryContainer,
        aspect: 1.48,
    },
    Place {
        id: "canal",
        title: "Canal path",
        icon: IconName::Inbox,
        color: ColorRole::SecondaryContainer,
        aspect: 0.74,
    },
];

fn foreground(background: ColorRole) -> ColorRole {
    match background {
        ColorRole::PrimaryContainer => ColorRole::OnPrimaryContainer,
        ColorRole::TertiaryContainer => ColorRole::OnTertiaryContainer,
        _ => ColorRole::OnSecondaryContainer,
    }
}

const STRATEGIES: [(&str, &str, CarouselVariant); 6] = [
    ("multi-browse", "Multi-browse", CarouselVariant::MultiBrowse),
    ("uncontained", "Uncontained", CarouselVariant::Uncontained),
    (
        "multi-aspect",
        "Uncontained multi-aspect ratio",
        CarouselVariant::UncontainedMultiAspectRatio,
    ),
    ("hero", "Hero", CarouselVariant::Hero),
    (
        "centered-hero",
        "Centered hero",
        CarouselVariant::CenteredHero,
    ),
    ("full-screen", "Full-screen", CarouselVariant::FullScreen),
];

#[derive(Default)]
pub(crate) struct CarouselExamples {
    active: Option<SharedString>,
}

impl Render for CarouselExamples {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let mut body = stack().child(
            Text::new("Use arrow keys to browse and Enter to open an item.").id("carousel-hint"),
        );
        if let Some(active) = self
            .active
            .as_ref()
            .and_then(|active| PLACES.iter().find(|place| place.id == active.as_ref()))
        {
            body = body.child(Text::new(format!("Opened {}", active.title)).id("carousel-status"));
        }
        body.children(STRATEGIES.iter().map(|&(id, name, variant)| {
            let view = cx.weak_entity();
            section(name).child(
                Carousel::new(
                    id,
                    name,
                    PLACES.iter().map(|place| {
                        CarouselItem::new(
                            place.id,
                            place.title,
                            Surface::new(place.color)
                                .size_full()
                                .rounded(px(if variant == CarouselVariant::FullScreen {
                                    0.
                                } else {
                                    28.
                                }))
                                .flex()
                                .flex_col()
                                .items_center()
                                .justify_center()
                                .gap_2()
                                .child(
                                    Icon::new(place.icon)
                                        .size(px(36.))
                                        .color(foreground(place.color)),
                                )
                                .child(
                                    div().min_w_0().max_w_full().overflow_hidden().child(
                                        Text::new(place.title)
                                            .role(TypeRole::TitleSmall)
                                            .color(foreground(place.color)),
                                    ),
                                ),
                        )
                        .aspect_ratio(place.aspect)
                    }),
                )
                .variant(variant)
                .height(px(if variant == CarouselVariant::FullScreen {
                    320.
                } else {
                    180.
                }))
                .on_activate(move |item, _, cx| {
                    let _ = view.update(cx, |this, cx| {
                        this.active = Some(item.clone());
                        cx.notify();
                    });
                }),
            )
        }))
    }
}
#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{
        AppContext, ElementId, InputEvent as _, MouseButton, MouseDownEvent, MouseUpEvent, Role,
        ScrollDelta, ScrollEvent, TestAppContext, TouchPhase, WindowBounds, WindowOptions, point,
        size,
    };
    use gpui_base::test_support::find;
    use gpui_material::{
        MaterialRoot, MaterialTheme, foundation::set_window_material_theme, theme::Direction,
    };
    use std::time::Duration;

    struct Fixture {
        variant: CarouselVariant,
        active: Option<SharedString>,
        count: usize,
    }

    impl Render for Fixture {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
            let view = cx.weak_entity();
            div().w(px(360.)).child(
                Carousel::new(
                    "carousel-test",
                    "Places",
                    PLACES.iter().map(|place| {
                        CarouselItem::new(place.id, place.title, div())
                            .disabled(place.id == "harbor")
                            .aspect_ratio(place.aspect)
                    }),
                )
                .variant(self.variant)
                .height(px(200.))
                .on_activate(move |item, _, cx| {
                    let _ = view.update(cx, |this, cx| {
                        this.count += 1;
                        this.active = Some(item.clone());
                        cx.notify();
                    });
                }),
            )
        }
    }

    fn open(
        cx: &mut TestAppContext,
        variant: CarouselVariant,
        rtl: bool,
        reduced_motion: bool,
    ) -> (gpui::WindowHandle<gpui_base::Root>, gpui::Entity<Fixture>) {
        cx.update(|cx| {
            crate::init_gallery(cx);
            cx.set_reduce_motion(reduced_motion);
            let mut theme = MaterialTheme::default();
            if rtl {
                theme.layout.direction = Direction::Rtl;
            }
            let mut view = None;
            let window = MaterialRoot::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::new(
                        point(px(0.), px(0.)),
                        size(px(600.), px(400.)),
                    ))),
                    ..WindowOptions::default()
                },
                cx,
                |window, cx| {
                    set_window_material_theme(Some(theme), window, cx);
                    let entity = cx.new(|_| Fixture {
                        variant,
                        active: None,
                        count: 0,
                    });
                    view = Some(entity.clone());
                    entity
                },
            )
            .unwrap();
            (window, view.unwrap())
        })
    }

    fn draw(cx: &mut TestAppContext, handle: gpui::WindowHandle<gpui_base::Root>) {
        cx.update_window(handle.into(), |_, window, cx| {
            window.activate_window();
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
        cx.run_until_parked();
    }

    fn item_id(id: &'static str) -> ElementId {
        (ElementId::from("carousel-test"), id).into()
    }

    fn scroll(
        cx: &mut TestAppContext,
        handle: gpui::WindowHandle<gpui_base::Root>,
        delta: ScrollDelta,
    ) {
        cx.update_window(handle.into(), |_, window, cx| {
            let region = find(window, &[], &"carousel-test".into()).unwrap();
            window.dispatch_event(
                ScrollEvent {
                    position: region.bounds().center(),
                    delta,
                    touch_phase: TouchPhase::Moved,
                    ..ScrollEvent::default()
                }
                .to_platform_input(),
                cx,
            );
        })
        .unwrap();
    }

    #[gpui::test]
    fn carousel_focus_is_revealed_and_activation_skips_disabled_items_in_rtl(
        cx: &mut TestAppContext,
    ) {
        let (handle, view) = open(cx, CarouselVariant::Hero, true, true);
        draw(cx, handle);
        cx.update_window(handle.into(), |_, window, cx| {
            window.focus_next(cx);
        })
        .unwrap();
        draw(cx, handle);
        cx.update_window(handle.into(), |_, window, _| {
            let first = find(window, &[], &item_id("cedar")).unwrap();
            assert_eq!(first.role(), Some(Role::Button));
            assert_eq!(first.focused(), Some(true));
        })
        .unwrap();

        cx.simulate_keystrokes(handle.into(), "left");
        draw(cx, handle);
        cx.update_window(handle.into(), |_, window, _| {
            let region = find(window, &[], &"carousel-test".into()).unwrap();
            let market = find(window, &[], &item_id("market")).unwrap();
            assert_eq!(market.focused(), Some(true));
            assert!(market.bounds().left() >= region.bounds().left());
            assert!(market.bounds().right() <= region.bounds().right());
            assert!(find(window, &[], &item_id("cedar")).is_none());
        })
        .unwrap();
        cx.update(|cx| assert_eq!(view.read(cx).count, 0));

        cx.update_window(handle.into(), |_, window, cx| {
            let keystroke = gpui::Keystroke::parse_with_os("enter", cx.platform_info().os).unwrap();
            window.dispatch_event(
                gpui::KeyDownEvent {
                    keystroke: keystroke.clone(),
                    is_held: false,
                    prefer_character_input: false,
                }
                .to_platform_input(),
                cx,
            );
            window.dispatch_event(gpui::KeyUpEvent { keystroke }.to_platform_input(), cx);
        })
        .unwrap();
        draw(cx, handle);
        cx.update(|cx| {
            assert_eq!(view.read(cx).active.as_deref(), Some("market"));
            assert_eq!(view.read(cx).count, 1);
        });

        let position = cx
            .update_window(handle.into(), |_, window, _| {
                find(window, &[], &item_id("market"))
                    .unwrap()
                    .bounds()
                    .center()
            })
            .unwrap();
        cx.update_window(handle.into(), |_, window, cx| {
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
        })
        .unwrap();
        draw(cx, handle);
        cx.update(|cx| assert_eq!(view.read(cx).count, 2));
    }

    #[gpui::test]
    fn full_screen_carousel_snaps_after_precise_scroll_with_reduced_motion(
        cx: &mut TestAppContext,
    ) {
        let (handle, view) = open(cx, CarouselVariant::FullScreen, false, true);
        draw(cx, handle);
        scroll(cx, handle, ScrollDelta::Pixels(point(px(0.), px(-150.))));
        draw(cx, handle);
        cx.update_window(handle.into(), |_, window, _| {
            assert!(find(window, &[], &item_id("cedar")).is_some());
            assert!(find(window, &[], &item_id("harbor")).is_some());
        })
        .unwrap();
        cx.executor().advance_clock(Duration::from_millis(150));
        cx.run_until_parked();
        draw(cx, handle);
        cx.update_window(handle.into(), |_, window, _| {
            let region = find(window, &[], &"carousel-test".into()).unwrap();
            let visible = find(window, &[], &item_id("harbor")).unwrap();
            assert_eq!(visible.bounds(), region.bounds());
            assert!(find(window, &[], &item_id("cedar")).is_none());
        })
        .unwrap();
        cx.update(|cx| assert_eq!(view.read(cx).count, 0));
    }
}
