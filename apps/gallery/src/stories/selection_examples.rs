use gpui::{
    AppContext, Axis, Context, ElementId, Entity, FocusHandle, IntoElement, ParentElement, Render,
    SharedString, StrokeOptions, Styled, Window, div, prelude::FluentBuilder, px, rgb,
};
use gpui_material::foundation::{color, window_material_theme};
use gpui_material::theme::{ColorRole, Direction};
use gpui_material::{
    Checkbox, CheckboxColors, CheckboxState, CheckboxStateColors, Chip, ChipGroup, ChipGroupItem,
    ChipGroupSelection, ChipVariant, Icon, IconName, InteractionSource, Radio, RadioColors,
    RadioGroup, SemanticTone, Switch, SwitchColors, SwitchStateColors, Text,
};

use crate::story::{Story, section};

#[cfg(all(test, feature = "test-support"))]
mod checkbox_tests;
#[cfg(all(test, feature = "test-support"))]
mod chip_group_tests;
#[cfg(all(test, feature = "test-support"))]
mod chip_tests;
#[cfg(all(test, feature = "test-support"))]
mod radio_tests;
#[cfg(all(test, feature = "test-support"))]
mod switch_tests;

#[derive(IntoElement)]
struct SwitchThumbMarker(bool);

impl gpui::RenderOnce for SwitchThumbMarker {
    fn render(self, window: &mut Window, _: &mut gpui::App) -> impl IntoElement {
        let ink = window.text_style().color;
        div()
            .size(px(10.))
            .rounded_full()
            .border_1()
            .border_color(ink)
            .when(self.0, |marker| marker.bg(ink))
    }
}

pub(crate) struct SelectionExamples {
    story: Option<Story>,
    colors: [CheckboxState; 2],
    alignment: ElementId,
    density: ElementId,
    notifications: bool,
    plain_switch: bool,
    custom_switch: bool,
    switch_interactions: Entity<InteractionSource>,
    filter: bool,
    filter_removed: bool,
    chip_selected: bool,
    chip_visible: bool,
    restore: FocusHandle,
    styled_checkbox: CheckboxState,
    checkbox_source: Option<Entity<InteractionSource>>,
    group_multi: Vec<SharedString>,
    group_single: SharedString,
}

impl SelectionExamples {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            story: None,
            colors: [CheckboxState::Checked, CheckboxState::Unchecked],
            alignment: "start".into(),
            density: "standard".into(),
            notifications: false,
            plain_switch: false,
            custom_switch: false,
            switch_interactions: cx.new(|_| InteractionSource::new()),
            filter: false,
            filter_removed: false,
            chip_selected: false,
            chip_visible: true,
            restore: cx.focus_handle(),
            styled_checkbox: CheckboxState::Indeterminate,
            checkbox_source: None,
            group_multi: vec!["open".into(), "urgent".into()],
            group_single: "week".into(),
        }
    }
    pub(crate) fn for_story(story: Story, cx: &mut Context<Self>) -> Self {
        Self {
            story: Some(story),
            ..Self::new(cx)
        }
    }

    fn shows(&self, story: Story) -> bool {
        self.story.is_none_or(|active| active == story)
    }

    fn custom_checkboxes(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> crate::story::StorySection {
        let source = self.checkbox_source.get_or_insert_with(|| {
            let source = cx.new(|_| InteractionSource::new());
            cx.observe(&source, |_, _, cx| cx.notify()).detach();
            source
        });
        let window_id = window.window_handle().window_id();
        let interaction = source.read(cx).state(window_id, &"styled-mixed".into());
        let status = match interaction {
            Some(state) if state.pressed => "Pressed",
            Some(state) if state.focus_visible => "Keyboard focus",
            Some(state) if state.hovered => "Hovered",
            _ => "Ready",
        };
        let colors = custom_checkbox_colors();
        let stroke = StrokeOptions {
            line_width: 3.5,
            ..StrokeOptions::default()
        };
        let owner = cx.entity();
        let examples = [
            (
                "styled-unchecked",
                "Unchecked",
                CheckboxState::Unchecked,
                false,
                false,
            ),
            (
                "styled-checked",
                "Checked",
                CheckboxState::Checked,
                false,
                false,
            ),
            (
                "styled-mixed",
                "Change selection",
                self.styled_checkbox,
                false,
                false,
            ),
            (
                "styled-error",
                "Error",
                CheckboxState::Indeterminate,
                true,
                false,
            ),
            (
                "styled-disabled",
                "Disabled",
                CheckboxState::Checked,
                false,
                true,
            ),
        ];
        section("Custom appearance")
            .child(div().flex().flex_wrap().gap_2().children(examples.map(
                |(id, label, state, error, disabled)| {
                    let control = Checkbox::new(id, label, state)
                        .colors(colors)
                        .checkmark_stroke(stroke)
                        .outline_stroke(px(1.))
                        .error(error)
                        .disabled(disabled)
                        .interaction_source(source);
                    if id == "styled-mixed" {
                        let owner = owner.clone();
                        control.on_change(move |next, _, cx| {
                            owner.update(cx, |this, cx| {
                                this.styled_checkbox = next;
                                cx.notify();
                            });
                        })
                    } else {
                        control
                    }
                },
            )))
            .child(Text::new(format!("Change selection · {status}")))
    }

    fn checkbox_section(&self, rtl: bool, cx: &mut Context<Self>) -> crate::story::StorySection {
        let owner = cx.entity();
        section("Group selection and validation")
            .description("Select Warm or Cool to see how All colors reflects a partial selection.")
            .child(
                Checkbox::new(
                    "palette-all",
                    "All colors",
                    Checkbox::aggregate(self.colors),
                )
                .on_change(move |next, _, cx| {
                    owner.update(cx, |this, cx| {
                        this.colors.fill(next);
                        cx.notify();
                    })
                }),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .when(rtl, Styled::flex_row_reverse)
                    .gap_2()
                    .children(
                        [("warm", "Warm"), ("cool", "Cool")]
                            .into_iter()
                            .enumerate()
                            .map(|(ix, (id, label))| {
                                let owner = cx.entity();
                                Checkbox::new(id, label, self.colors[ix]).on_change(
                                    move |next, _, cx| {
                                        owner.update(cx, |this, cx| {
                                            this.colors[ix] = next;
                                            cx.notify();
                                        });
                                    },
                                )
                            }),
                    ),
            )
            .child(
                Checkbox::new(
                    "invalid-option",
                    "Required option",
                    CheckboxState::Unchecked,
                )
                .error(true),
            )
            .child(
                Checkbox::new("disabled-option", "Unavailable", CheckboxState::Checked)
                    .disabled(true),
            )
    }

    fn radio_section(&self, window: &Window, cx: &mut Context<Self>) -> crate::story::StorySection {
        let theme = window_material_theme(window, cx);
        let tone = |role| color(theme.colors.color(role));
        let alignment_colors = RadioColors {
            selected: Some(tone(ColorRole::Secondary)),
            unselected: Some(tone(ColorRole::Outline)),
            ..Default::default()
        };
        let density_colors = RadioColors {
            selected: Some(tone(ColorRole::Tertiary)),
            unselected: Some(tone(ColorRole::OnSurfaceVariant)),
            ..Default::default()
        };
        let radio_story = self.story == Some(Story::Radio);
        section(if radio_story {
            "Alignment and density"
        } else {
            "Horizontal group"
        })
        .description("Use the arrow keys to navigate each group.")
        .child({
            let owner = cx.entity();
            RadioGroup::new("alignment", "Alignment")
                .axis(Axis::Horizontal)
                .selected(self.alignment.clone())
                .options(
                    [
                        Radio::new("start", "Start"),
                        Radio::new("center", "Center").disabled(true),
                        Radio::new("end", "End"),
                    ]
                    .map(|radio| radio.colors(alignment_colors)),
                )
                .on_select(move |id, _, cx| {
                    owner.update(cx, |this, cx| {
                        this.alignment = id.clone();
                        cx.notify();
                    });
                })
        })
        .children(radio_story.then(|| {
            let owner = cx.entity();
            RadioGroup::new("density", "Density")
                .selected(self.density.clone())
                .options(
                    [
                        Radio::new("compact", "Compact"),
                        Radio::new("standard", "Standard"),
                        Radio::new("comfortable", "Comfortable"),
                    ]
                    .map(|radio| radio.colors(density_colors)),
                )
                .on_select(move |id, _, cx| {
                    owner.update(cx, |this, cx| {
                        this.density = id.clone();
                        cx.notify();
                    });
                })
        }))
    }

    fn switch_section(&self, cx: &mut Context<Self>) -> crate::story::StorySection {
        section("With icons, without icons and disabled")
            .child({
                let owner = cx.entity();
                Switch::new("notifications", "Notifications", self.notifications)
                    .checked_icon(Icon::new(IconName::Check))
                    .unchecked_icon(Icon::new(IconName::Remove))
                    .on_change(move |next, _, cx| {
                        owner.update(cx, |this, cx| {
                            this.notifications = next;
                            cx.notify();
                        });
                    })
            })
            .child({
                let owner = cx.entity();
                Switch::new("sync-switch", "Sync automatically", self.plain_switch).on_change(
                    move |next, _, cx| {
                        owner.update(cx, |this, cx| {
                            this.plain_switch = next;
                            cx.notify();
                        })
                    },
                )
            })
            .child(Switch::new("disabled-switch", "Sync locked on", true).disabled(true))
    }

    fn custom_switch_section(&self, cx: &mut Context<Self>) -> crate::story::StorySection {
        let owner = cx.entity();
        let colors = SwitchColors {
            checked: SwitchStateColors {
                track: Some(rgb(0x236449).into()),
                thumb: Some(rgb(0xe9f6ed).into()),
                content: Some(rgb(0x24523d).into()),
                ..Default::default()
            },
            unchecked: SwitchStateColors {
                track: Some(rgb(0xece5f7).into()),
                border: Some(rgb(0x665084).into()),
                thumb: Some(rgb(0x665084).into()),
                content: Some(rgb(0xffffff).into()),
            },
            ..Default::default()
        };
        section("Custom thumb and colors")
            .child(
                Switch::new(
                    "custom-switch",
                    "Save drafts automatically",
                    self.custom_switch,
                )
                .colors(colors)
                .thumb_content(SwitchThumbMarker(self.custom_switch))
                .interaction_source(&self.switch_interactions)
                .on_change(move |next, _, cx| {
                    owner.update(cx, |selection, cx| {
                        selection.custom_switch = next;
                        cx.notify();
                    })
                }),
            )
            .child(
                Switch::new(
                    "locked-custom-switch",
                    "Managed by your organization",
                    false,
                )
                .disabled(true)
                .colors(colors)
                .thumb_content(SwitchThumbMarker(false))
                .interaction_source(&self.switch_interactions),
            )
    }

    fn chip_section(&self, rtl: bool, cx: &mut Context<Self>) -> crate::story::StorySection {
        section("Assist, filter, input and suggestion")
            .description("Toggle Recent, or remove the tag and use Restore tag to bring it back.")
            .child(
                div()
                    .flex()
                    .items_center()
                    .flex_wrap()
                    .when(rtl, Styled::flex_row_reverse)
                    .gap_2()
                    .child({
                        let owner = cx.entity();
                        gpui_material::Button::new("restore-chip", "Restore tag")
                            .variant(gpui_material::ButtonVariant::Text)
                            .track_focus(&self.restore)
                            .on_activate(move |_, cx| {
                                owner.update(cx, |this, cx| {
                                    this.chip_visible = true;
                                    cx.notify();
                                })
                            })
                    })
                    .child(
                        Chip::new("assist", "Browse")
                            .leading_icon(Icon::new(IconName::Add))
                            .elevated(true),
                    )
                    .child({
                        let owner = cx.entity();
                        Chip::new("filter", "Recent")
                            .variant(ChipVariant::Filter)
                            .selected(self.filter)
                            .on_change(move |value, _, cx| {
                                owner.update(cx, |this, cx| {
                                    this.filter = value;
                                    cx.notify();
                                })
                            })
                    })
                    .children((!self.filter_removed).then(|| {
                        let owner = cx.entity();
                        let remove = owner.clone();
                        Chip::new("removable-filter", "Downloads")
                            .variant(ChipVariant::Filter)
                            .selected(self.filter)
                            .on_change(move |value, _, cx| {
                                owner.update(cx, |this, cx| {
                                    this.filter = value;
                                    cx.notify();
                                })
                            })
                            .on_remove(
                                "Remove downloads filter",
                                Some(self.restore.clone()),
                                move |_, cx| {
                                    remove.update(cx, |this, cx| {
                                        this.filter_removed = true;
                                        cx.notify();
                                    })
                                },
                            )
                    }))
                    .children(self.chip_visible.then(|| {
                        let remove = cx.entity();
                        let select = cx.entity();
                        Chip::new("tag", "Project")
                            .variant(ChipVariant::Input)
                            .selected(self.chip_selected)
                            .on_change(move |value, _, cx| {
                                select.update(cx, |this, cx| {
                                    this.chip_selected = value;
                                    cx.notify();
                                })
                            })
                            .on_remove(
                                "Remove Project tag",
                                Some(self.restore.clone()),
                                move |_, cx| {
                                    remove.update(cx, |this, cx| {
                                        this.chip_visible = false;
                                        cx.notify();
                                    });
                                },
                            )
                    }))
                    .child(Chip::new("suggestion", "Suggested").variant(ChipVariant::Suggestion)),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .flex_wrap()
                    .when(rtl, Styled::flex_row_reverse)
                    .gap_2()
                    .child(
                        Chip::new("disabled-tag", "Locked tag")
                            .variant(ChipVariant::Input)
                            .selected(true)
                            .disabled(true)
                            .on_remove("Remove locked tag", None, |_, _| {}),
                    )
                    .child(
                        Chip::new("elevated-suggestion", "Nearby")
                            .variant(ChipVariant::Suggestion)
                            .elevated(true),
                    ),
            )
    }
}

fn custom_checkbox_colors() -> CheckboxColors {
    let paint = |box_color, border, mark| CheckboxStateColors {
        box_color: Some(rgb(box_color).into()),
        border: Some(rgb(border).into()),
        checkmark: Some(rgb(mark).into()),
    };
    CheckboxColors {
        unchecked: paint(0xe8ecf6, 0x4b536b, 0x29364f),
        checked: paint(0x285844, 0x182f29, 0xffeed6),
        indeterminate: paint(0x415380, 0x29364f, 0xffeed6),
        error_indeterminate: paint(0x8b3a31, 0x682c32, 0xffffff),
        disabled_checked: paint(0xbbb7be, 0x76717a, 0xf6f1f5),
        ..Default::default()
    }
}

impl Render for SelectionExamples {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rtl = window_material_theme(window, cx).layout.direction == Direction::Rtl;
        div()
            .flex()
            .flex_col()
            .gap_2()
            .children(
                self.shows(Story::Checkbox)
                    .then(|| self.checkbox_section(rtl, cx)),
            )
            .children(
                (self.story == Some(Story::Checkbox)).then(|| self.custom_checkboxes(window, cx)),
            )
            .children(
                self.shows(Story::Radio)
                    .then(|| self.radio_section(window, cx)),
            )
            .children(self.shows(Story::Switch).then(|| self.switch_section(cx)))
            .children((self.story == Some(Story::Switch)).then(|| self.custom_switch_section(cx)))
            .children(self.shows(Story::Chip).then(|| self.chip_section(rtl, cx)))
            .children((self.story == Some(Story::Chip)).then(chip_content_examples))
            .children(
                self.shows(Story::ChipGroup)
                    .then(|| self.chip_group_section(cx)),
            )
    }
}

impl SelectionExamples {
    fn chip_group_section(&self, cx: &Context<Self>) -> crate::story::StorySection {
        let on_multi = cx.entity();
        let on_single = cx.entity();
        let active_multi = if self.group_multi.is_empty() {
            "None".to_string()
        } else {
            self.group_multi
                .iter()
                .map(|id| id.as_ref())
                .collect::<Vec<_>>()
                .join(", ")
        };
        section("Chip group")
            .description("Controlled chip collections with M3 8 dp spacing, roving focus, and wrapping or scrollable rows.")
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(Text::new(format!("Multi-select filters: {active_multi}")).muted())
                    .child(
                        ChipGroup::new(
                            "gallery-chip-group-multi",
                            "Issue filters",
                            [
                                ChipGroupItem::new("open", "Open"),
                                ChipGroupItem::new("review", "In review"),
                                ChipGroupItem::new("urgent", "Urgent")
                                    .tone(SemanticTone::Danger),
                                ChipGroupItem::new("archived", "Archived").disabled(true),
                            ],
                        )
                        .selection(ChipGroupSelection::Multiple, self.group_multi.clone())
                        .on_change(move |selected, _, cx| {
                            on_multi.update(cx, |this, cx| {
                                this.group_multi = selected;
                                cx.notify();
                            });
                        }),
                    )
                    .child(
                        Text::new(format!("Single-select timeframe: {}", self.group_single))
                            .muted(),
                    )
                    .child(
                        ChipGroup::new(
                            "gallery-chip-group-single",
                            "Timeframe",
                            [
                                ChipGroupItem::new("day", "Today"),
                                ChipGroupItem::new("week", "This week"),
                                ChipGroupItem::new("month", "This month"),
                                ChipGroupItem::new("quarter", "This quarter"),
                                ChipGroupItem::new("year", "This year"),
                            ],
                        )
                        .scrollable(true)
                        .single_select(self.group_single.clone(), move |id, _, cx| {
                            on_single.update(cx, |this, cx| {
                                this.group_single = id;
                                cx.notify();
                            });
                        }),
                    ),
            )
    }
}

fn chip_content_examples() -> impl IntoElement {
    use gpui::{px, rgb};
    use gpui_material::{ChipColors, ChipStateColors, LogicalEdges};

    section("Custom chip content").child(
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_2()
            .child(
                Chip::new("chip-avatar-example", "Workspace attendee")
                    .variant(ChipVariant::Input)
                    .tonal(true)
                    .avatar(
                        div()
                            .size_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .bg(rgb(0xd6e4ff))
                            .text_color(rgb(0x153a65))
                            .text_size(px(12.))
                            .child("JD"),
                    )
                    .content(
                        div()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Jordan"),
                    ),
            )
            .child(
                Chip::new("chip-trailing-example", "Pending items")
                    .variant(ChipVariant::Assist)
                    .trailing(
                        div()
                            .rounded_full()
                            .px_2()
                            .bg(rgb(0xd6eee1))
                            .text_size(px(12.))
                            .child("3"),
                    )
                    .colors(ChipColors {
                        enabled: ChipStateColors {
                            trailing: Some(rgb(0x204e3d).into()),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .content_padding(LogicalEdges {
                        start: 12.,
                        end: 4.,
                        ..Default::default()
                    }),
            )
            .child(
                Chip::new("chip-tonal-example", "Selected label")
                    .variant(ChipVariant::Filter)
                    .selected(true)
                    .tonal(true),
            ),
    )
}
