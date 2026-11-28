use gpui::{Context, IntoElement, ParentElement, Styled, Window, div, px};
use gpui_material::foundation::{color, window_material_theme};
use gpui_material::theme::{ColorRole, CornerRole, Elevation, TypeRole};
use gpui_material::{
    Avatar, AvatarGroup, AvatarShape, AvatarSize, Badge, Button, ButtonVariant, Card, CardColors,
    CardVariant, ControlBorder, ControlElevations, Divider, Icon, IconName, Text,
};

use super::DisplayExamples;
use crate::story::{StorySection, section, stack};

impl DisplayExamples {
    pub(super) fn icon_symbols_section(&self) -> StorySection {
        section("Outlined and filled symbols").child(
            div().flex().flex_wrap().gap_4().items_center().children(
                [20., 24., 40., 48.].into_iter().map(|side| {
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(Icon::new(IconName::Favorite).size(px(side)))
                        .child(
                            Icon::new(IconName::Favorite)
                                .filled(true)
                                .size(px(side))
                                .color(ColorRole::Primary),
                        )
                }),
            ),
        )
    }

    pub(super) fn icon_inherited_section(&self) -> StorySection {
        section("Shared size and color").child(
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap_3()
                .child(
                    Button::new("icon-inherited", "Add document")
                        .leading_icon(Icon::new(IconName::Add)),
                )
                .child(
                    Button::new("icon-inherited-disabled", "Unavailable")
                        .leading_icon(Icon::new(IconName::Add))
                        .disabled(true),
                ),
        )
    }

    pub(super) fn icon_directional_section(&self) -> StorySection {
        section("Directional and decorative SVGs")
            .description(
                "The first arrow mirrors with right-to-left layout. The second retains its direction.",
            )
            .child(
                div()
                    .flex()
                    .gap_3()
                    .h_10()
                    .items_center()
                    .child(
                        Icon::new(IconName::ArrowForward)
                            .size(px(24.))
                            .mirror_in_rtl(true)
                            .label("forward", "Forward"),
                    )
                    .child(Divider::vertical().inset(px(4.), px(8.)))
                    .child(Icon::new(IconName::ArrowForward).size(px(24.))),
            )
    }

    pub(super) fn card_section(&self, window: &Window, cx: &mut Context<Self>) -> StorySection {
        let card = cx.entity();
        let child = cx.entity();
        let theme = window_material_theme(window, cx);
        let tone = |role| color(theme.colors.color(role));
        section("Elevated, filled and outlined")
            .description("Open the project card or activate its Pin button independently.")
            .child(
                Card::new("elevated-card")
                    .variant(CardVariant::Elevated)
                    .shape(CornerRole::Large.shape())
                    .child(Text::new("Elevated card")),
            )
            .child(
                Card::new("filled-card")
                    .colors(CardColors {
                        container: Some(tone(ColorRole::SecondaryContainer)),
                        content: Some(tone(ColorRole::OnSecondaryContainer)),
                        ..Default::default()
                    })
                    .elevations(ControlElevations {
                        hovered: Some(Elevation::Level2),
                        focused: Some(Elevation::Level1),
                        ..Default::default()
                    })
                    .accessible_label("Open project")
                    .on_activate(move |_, cx| {
                        card.update(cx, |this, cx| {
                            this.opened = !this.opened;
                            cx.notify();
                        });
                    })
                    .child(Text::new(if self.opened {
                        "Project open"
                    } else {
                        "Project"
                    }))
                    .child(
                        Button::new("card-child", format!("Pin ({})", self.child_count))
                            .variant(ButtonVariant::Outlined)
                            .on_activate(move |_, cx| {
                                child.update(cx, |this, cx| {
                                    this.child_count += 1;
                                    cx.notify();
                                })
                            }),
                    ),
            )
            .child(
                Card::new("outlined-card")
                    .variant(CardVariant::Outlined)
                    .border(ControlBorder {
                        width: px(2.),
                        color: tone(ColorRole::OutlineVariant),
                    })
                    .disabled(true)
                    .accessible_label("Unavailable card")
                    .on_activate(|_, _| {})
                    .child(Text::new("Unavailable card")),
            )
    }

    pub(super) fn divider_section(&self) -> StorySection {
        section("Horizontal and vertical dividers")
            .child(
                Text::new("Workspace")
                    .id("divider-from")
                    .role(TypeRole::LabelLarge),
            )
            .child(Divider::horizontal().inset(px(16.), px(0.)))
            .child(
                Text::new("Documents")
                    .id("divider-to")
                    .role(TypeRole::BodyMedium),
            )
            .child(
                Divider::horizontal()
                    .thickness(px(3.))
                    .color(ColorRole::Primary),
            )
            .child(Divider::horizontal().thickness(px(0.)))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .h_10()
                    .child(Text::new("Before").id("divider-before"))
                    .child(Divider::vertical().inset(px(4.), px(8.)))
                    .child(Text::new("After").id("divider-after"))
                    .child(
                        Divider::vertical()
                            .thickness(px(3.))
                            .color(ColorRole::Primary),
                    )
                    .child(Divider::vertical().thickness(px(0.))),
            )
    }

    pub(super) fn badge_section(&self) -> impl IntoElement {
        stack()
            .child(
                section("Anchored badges")
                    .description("The archive badge caps the displayed count at 99+.")
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .flex_wrap()
                            .gap_5()
                            .p_2()
                            .child(Badge::dot(
                                Icon::new(IconName::Notifications)
                                    .size(px(24.))
                                    .label("new-activity", "New activity"),
                            ))
                            .child(Badge::count(
                                Icon::new(IconName::Inbox)
                                    .size(px(24.))
                                    .label("inbox", "Inbox, 8 unread"),
                                8,
                                99,
                            ))
                            .child(Badge::label(
                                Icon::new(IconName::Description)
                                    .size(px(24.))
                                    .label("new-file", "Files"),
                                "New",
                            ))
                            .child(Badge::count(
                                Icon::new(IconName::Archive)
                                    .size(px(24.))
                                    .label("archive-overflow", "More than 99 archived items"),
                                125,
                                99,
                            ))
                            .child(
                                Badge::dot(
                                    Icon::new(IconName::Favorite)
                                        .size(px(24.))
                                        .label("favorite-status", "Favorites, synced"),
                                )
                                .content("synced-marker", "✓")
                                .container_color(gpui::rgb(0x21644a))
                                .content_color(gpui::rgb(0xffffff)),
                            ),
                    ),
            )
            .child(
                section("Standalone badges").child(
                    div()
                        .flex()
                        .items_center()
                        .gap_5()
                        .p_2()
                        .child(Badge::standalone())
                        .child(
                            Badge::standalone()
                                .content(
                                    "priority-marker",
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_1()
                                        .child(div().size(px(4.)).rounded_full().bg(gpui::white()))
                                        .child("P1"),
                                )
                                .container_color(gpui::rgb(0x334e68))
                                .content_color(gpui::rgb(0xffffff))
                                .accessible_label("priority-badge", "Priority 1"),
                        ),
                ),
            )
    }

    pub(super) fn avatar_section(&self) -> impl IntoElement {
        stack()
            .child(
                section("Sizes, shapes, and content")
                    .description(
                        "Avatars scale typography with size and support initials, icons, images with fallbacks, and rounded shapes.",
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap_4()
                            .child(Avatar::initials("Ada Lovelace").size(AvatarSize::ExtraSmall))
                            .child(Avatar::initials("Grace Hopper").size(AvatarSize::Small))
                            .child(
                                Avatar::initials("Alan Turing")
                                    .size(AvatarSize::Medium)
                                    .accessible_label("avatar-at", "Alan Turing"),
                            )
                            .child(
                                Avatar::icon(Icon::new(IconName::Favorite))
                                    .size(AvatarSize::Large)
                                    .color_role(
                                        ColorRole::SecondaryContainer,
                                        ColorRole::OnSecondaryContainer,
                                    ),
                            )
                            .child(
                                Avatar::initials("Katherine Johnson")
                                    .size(AvatarSize::ExtraLarge)
                                    .shape(AvatarShape::Rounded(CornerRole::Large.shape()))
                                    .color_role(
                                        ColorRole::TertiaryContainer,
                                        ColorRole::OnTertiaryContainer,
                                    )
                                    .border(px(2.)),
                            ),
                    ),
            )
            .child(
                section("Avatar group")
                    .description(
                        "AvatarGroup overlaps members with surface-matched cutout rings, RTL mirroring, and +N overflow indicators.",
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap_6()
                            .child(
                                AvatarGroup::new(
                                    "gallery-team-avatars",
                                    [
                                        Avatar::initials("Ada Lovelace"),
                                        Avatar::initials("Grace Hopper").color_role(
                                            ColorRole::SecondaryContainer,
                                            ColorRole::OnSecondaryContainer,
                                        ),
                                        Avatar::initials("Alan Turing").color_role(
                                            ColorRole::TertiaryContainer,
                                            ColorRole::OnTertiaryContainer,
                                        ),
                                        Avatar::initials("Katherine Johnson"),
                                        Avatar::initials("Margaret Hamilton"),
                                    ],
                                )
                                .size(AvatarSize::Medium)
                                .max_visible(3)
                                .ring_role(ColorRole::SurfaceContainerLow)
                                .accessible_label("5 team members"),
                            )
                            .child(
                                AvatarGroup::new(
                                    "gallery-compact-avatars",
                                    [
                                        Avatar::icon(Icon::new(IconName::Favorite)),
                                        Avatar::initials("Design").color_role(
                                            ColorRole::SecondaryContainer,
                                            ColorRole::OnSecondaryContainer,
                                        ),
                                        Avatar::initials("Engineering"),
                                        Avatar::initials("Research"),
                                    ],
                                )
                                .size(AvatarSize::Small)
                                .overlap(px(10.))
                                .ring_width(px(2.))
                                .ring_role(ColorRole::SurfaceContainerLow)
                                .max_visible(2)
                                .accessible_label("4 contributors"),
                            ),
                    ),
            )
    }
}
