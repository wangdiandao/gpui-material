use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
    prelude::FluentBuilder, px,
};
use gpui_material::{
    Button, ButtonVariant, ControlElevations, ControlSize, Icon, IconName, List, ListItem,
    ListItemAlignment, ListItemColors, ListItemShapes, ListItemStateColors, ListVariant,
    LogicalEdges, SwipeToDismissState, SwipeToDismissValue, Text, foundation::InteractionSource,
    foundation::color, foundation::window_material_theme, theme::ColorRole, theme::CornerRole,
    theme::Elevation, theme::TypeRole,
};

use crate::story::section;

#[cfg(all(test, feature = "test-support"))]
mod tests;

pub(super) struct ListExamples {
    checked: bool,
    expanded: bool,
    pin: bool,
    details: usize,
    swipe: Entity<SwipeToDismissState>,
    source: Entity<InteractionSource>,
    dismissed: bool,
}

impl ListExamples {
    pub(super) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            checked: false,
            expanded: false,
            pin: false,
            details: 0,
            swipe: cx.new(|_| SwipeToDismissState::new()),
            source: cx.new(|_| InteractionSource::default()),
            dismissed: false,
        }
    }
}

impl Render for ListExamples {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.entity();
        let check = owner.clone();
        let expand = owner.clone();
        let details = owner.clone();
        let pin = owner.clone();
        let dismissed = owner.clone();
        let remove = self.swipe.clone();
        let restore = self.swipe.clone();
        let scheme = &window_material_theme(window, cx).colors;
        let secondary_container = color(scheme.color(ColorRole::SecondaryContainer));
        let color_on_secondary = color(scheme.color(ColorRole::OnSecondaryContainer));
        let detail_label = if self.details == 0 {
            "View release details".to_owned()
        } else {
            format!("Release details opened {}", self.details)
        };
        let details_row = ListItem::new("release-notes", "Release notes")
            .content(Text::new("Release notes").emphasized(true))
            .overline_content(Text::new("Current version").role(TypeRole::LabelSmall))
            .supporting_content(
                Text::new("The download includes the desktop assets.")
                    .role(TypeRole::BodyMedium)
                    .wrap(),
            )
            .leading(Icon::new(IconName::Description).size(px(24.)))
            .trailing(
                Button::new("pin-notes", if self.pin { "Unpin" } else { "Pin" })
                    .variant(ButtonVariant::Outlined)
                    .size(ControlSize::ExtraSmall)
                    .on_activate(move |_, cx| {
                        pin.update(cx, |this, cx| {
                            this.pin = !this.pin;
                            cx.notify();
                        })
                    }),
            )
            .expanded_content(
                self.expanded,
                div()
                    .p_3()
                    .child(Text::new("Build artifacts are available for this release.")),
            )
            .on_expanded_change(move |expanded, _, cx| {
                expand.update(cx, |this, cx| {
                    this.expanded = expanded;
                    cx.notify();
                })
            })
            .on_long_press(detail_label, move |_, cx| {
                details.update(cx, |this, cx| {
                    this.details += 1;
                    cx.notify();
                })
            })
            .long_press_button("notes-details")
            .vertical_alignment(ListItemAlignment::Start)
            .content_padding(LogicalEdges {
                start: 20.,
                ..Default::default()
            })
            .shapes(ListItemShapes {
                hovered: Some(CornerRole::Medium.shape()),
                pressed: Some(CornerRole::Large.shape()),
                ..Default::default()
            })
            .elevations(ControlElevations {
                hovered: Some(Elevation::Level1),
                ..Default::default()
            })
            .interaction_source(&self.source);

        section("Release files")
            .child(
                List::new("release-files")
                    .variant(ListVariant::Segmented)
                    .items([
                        details_row,
                        ListItem::new("include-symbols", "Include debug symbols")
                            .checked(self.checked)
                            .on_checked_change(move |checked, _, cx| {
                                check.update(cx, |this, cx| {
                                    this.checked = checked;
                                    cx.notify();
                                })
                            })
                            .colors(ListItemColors {
                                selected: ListItemStateColors {
                                    container: Some(secondary_container),
                                    content: Some(color_on_secondary),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }),
                    ])
                    .when(!self.dismissed, |list| {
                        list.items([ListItem::new("release-download", "Release download")
                            .supporting("Swipe toward the start to remove")
                            .trailing(
                                Button::new("remove-download", "Remove download")
                                    .variant(ButtonVariant::Text)
                                    .size(ControlSize::ExtraSmall)
                                    .on_activate(move |window, cx| {
                                        remove.update(cx, |state, cx| {
                                            state.dismiss(
                                                SwipeToDismissValue::EndToStart,
                                                window,
                                                cx,
                                            );
                                        });
                                    }),
                            )
                            .swipe_to_dismiss(&self.swipe, move |swipe| {
                                swipe
                                    .enable_start_to_end(false)
                                    .background(
                                        div()
                                            .size_full()
                                            .p_4()
                                            .bg(secondary_container)
                                            .text_color(color_on_secondary)
                                            .child("Removing download"),
                                    )
                                    .on_dismiss(move |_, _, cx| {
                                        dismissed.update(cx, |this, cx| {
                                            this.dismissed = true;
                                            cx.notify();
                                        })
                                    })
                            })])
                    }),
            )
            .children(self.dismissed.then(|| {
                Button::new("restore-download", "Restore download").on_activate(
                    move |window, cx| {
                        restore.update(cx, |state, cx| state.reset(window, cx));
                        owner.update(cx, |this, cx| {
                            this.dismissed = false;
                            cx.notify();
                        });
                    },
                )
            }))
    }
}
