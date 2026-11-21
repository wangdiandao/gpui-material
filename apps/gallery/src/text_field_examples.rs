use gpui::{
    AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render, Styled,
    Window, div, px,
};
use gpui_base::TestSupportExt;
use gpui_material::{
    Icon, IconName, TextAreaState, TextField, TextFieldLabelAlignment, TextFieldLabelPosition,
    TextFieldMaskPattern, TextFieldObfuscationMode, TextFieldState, TextFieldVariant,
};

use crate::story::{section, stack};

use gpui_material::{
    Button, ButtonVariant, DropdownAnchor, DropdownOption, ExposedDropdown, ExposedDropdownState,
    MenuSurfaceStyle, Text,
    theme::{ColorRole, CornerShape, Elevation},
};

#[cfg(all(test, feature = "test-support"))]
mod dropdown_customization_tests;
#[cfg(all(test, feature = "test-support"))]
mod dropdown_tests;
#[cfg(all(test, feature = "test-support"))]
mod text_field_tests;

pub(crate) struct DropdownExamples {
    city: Entity<ExposedDropdownState>,
    language: Entity<ExposedDropdownState>,
    custom: Entity<ExposedDropdownState>,
    custom_open: bool,
    destination: gpui::SharedString,
}

impl DropdownExamples {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let city = cx.new(|cx| ExposedDropdownState::new("", true, window, cx));
        let language = cx.new(|cx| ExposedDropdownState::new("English", false, window, cx));
        let editor = cx.new(|cx| TextFieldState::new(window, cx).default_value("Oslo"));
        let custom = cx.new(|cx| ExposedDropdownState::with_editor(&editor, true, window, cx));
        cx.observe(&city, |_, _, cx| cx.notify()).detach();
        cx.observe(&language, |_, _, cx| cx.notify()).detach();
        cx.observe(&custom, |_, _, cx| cx.notify()).detach();
        Self {
            city,
            language,
            custom,
            custom_open: false,
            destination: "".into(),
        }
    }
}

impl Render for DropdownExamples {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.entity().downgrade();
        let custom_owner = owner.clone();
        let custom_toggle = owner.clone();
        let custom_open = self.custom_open;
        stack()
            .max_w(px(440.))
            .child(
                section("Editable")
                    .description("Type 北京, 上海 or 香港 to filter the destinations.")
                    .child(
                        ExposedDropdown::new(
                            "city",
                            "Destination",
                            &self.city,
                            [
                                DropdownOption::new("hk", "香港"),
                                DropdownOption::new("bj", "北京"),
                                DropdownOption::new("sh", "上海"),
                            ],
                        )
                        .on_select(move |option, _, cx| {
                            let _ = owner.update(cx, |view, cx| {
                                view.destination = option.label.clone();
                                cx.notify();
                            });
                        }),
                    ),
            )
            .child(
                section("Selection only")
                    .description("Open the list to choose the display language.")
                    .child(ExposedDropdown::new(
                        "language",
                        "Display language",
                        &self.language,
                        [
                            DropdownOption::new("en", "English"),
                            DropdownOption::new("zh", "简体中文"),
                            DropdownOption::new("ja", "日本語"),
                        ],
                    )),
            )
            .child(
                section("Custom editor and controlled popup")
                    .child(
                        Button::new(
                            "destination-options",
                            if custom_open {
                                "Hide destination options"
                            } else {
                                "Show destination options"
                            },
                        )
                        .variant(ButtonVariant::Text)
                        .on_activate(move |_, cx| {
                            let _ = custom_toggle.update(cx, |view, cx| {
                                view.custom_open = !view.custom_open;
                                cx.notify();
                            });
                        }),
                    )
                    .child(
                        ExposedDropdown::new(
                            "custom-destination",
                            "Preset destination",
                            &self.custom,
                            ["Oslo", "Paris", "Tokyo", "Vienna"]
                                .map(|name| DropdownOption::new(name, name)),
                        )
                        .primary_anchor(DropdownAnchor::PrimaryEditable)
                        .secondary_anchor("Browse preset destinations", |open| {
                            Icon::new(if open {
                                IconName::Remove
                            } else {
                                IconName::Add
                            })
                        })
                        .editor(|field| {
                            field
                                .variant(TextFieldVariant::Filled)
                                .label_content(Text::new("Preset destination").id("preset-label"))
                        })
                        .open(custom_open)
                        .on_open_change(move |open, _, cx| {
                            let _ = custom_owner.update(cx, |view, cx| {
                                view.custom_open = open;
                                cx.notify();
                            });
                        })
                        .option_content(|option, _, _, _| {
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(Icon::new(IconName::Favorite).size(px(18.)))
                                .child(Text::new(option.label.clone()).id("preset-name"))
                        })
                        .match_anchor_width(false)
                        .popup_width(px(200.))
                        .appearance(MenuSurfaceStyle {
                            shape: Some(CornerShape::Radii([16.; 4])),
                            container: Some(ColorRole::Surface),
                            border: Some((px(1.), ColorRole::OutlineVariant)),
                            tonal_elevation: Elevation::Level1,
                            shadow_elevation: Some(Elevation::Level3),
                            ..Default::default()
                        }),
                    ),
            )
            .children(
                (!self.destination.is_empty())
                    .then(|| Text::new(format!("Destination selected: {}", self.destination))),
            )
    }
}

pub(crate) struct TextFieldExamples {
    name: Entity<TextFieldState>,
    email: Entity<TextFieldState>,
    notes: Entity<TextAreaState>,
    password: Entity<TextFieldState>,
    password_visible: bool,
    account: Entity<TextFieldState>,
    disabled: Entity<TextFieldState>,
    scaled: Entity<TextFieldState>,
    amount: Entity<TextFieldState>,
    phone: Entity<TextFieldState>,
    code: Entity<TextFieldState>,
    clearable_query: Entity<TextFieldState>,
    api_secret: Entity<TextFieldState>,
    api_secret_visible: bool,
}

impl TextFieldExamples {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let phone = cx.new(|cx| {
            TextFieldState::new(window, cx)
                .mask_pattern(TextFieldMaskPattern::new("(999) 999-9999"))
                .placeholder("(415) 555-0199")
        });
        cx.observe(&phone, |_, _, cx| cx.notify()).detach();
        Self {
            name: cx.new(|cx| TextFieldState::new(window, cx).placeholder("王小明")),
            email: cx.new(|cx| TextFieldState::new(window, cx).default_value("name@")),
            notes: cx.new(|cx| {
                TextAreaState::new(window, cx)
                    .rows(3)
                    .placeholder("输入备注")
            }),
            password: cx.new(|cx| {
                TextFieldState::new(window, cx)
                    .masked(true)
                    .placeholder("Enter a sample password")
            }),
            password_visible: false,
            account: cx.new(|cx| {
                let mut state = TextFieldState::new(window, cx).default_value("Demo");
                state.set_readonly(true, cx);
                state
            }),
            disabled: cx.new(|cx| {
                let mut state = TextFieldState::new(window, cx).default_value("Pending");
                state.set_disabled(true, cx);
                state
            }),
            scaled: cx.new(|cx| TextFieldState::new(window, cx)),
            amount: cx.new(|cx| TextFieldState::new(window, cx)),
            phone,
            code: cx.new(|cx| {
                TextFieldState::new(window, cx)
                    .placeholder("Up to 6 letters or digits")
                    .validate(|text, _| {
                        text.chars().count() <= 6
                            && text.chars().all(|ch| ch.is_ascii_alphanumeric())
                    })
            }),
            clearable_query: cx.new(|cx| {
                TextFieldState::new(window, cx)
                    .default_value("Material 3")
                    .placeholder("Type to search")
            }),
            api_secret: cx.new(|cx| {
                TextFieldState::new(window, cx)
                    .masked(true)
                    .default_value("sk-demo-421")
                    .placeholder("API secret")
            }),
            api_secret_visible: false,
        }
    }
}

impl Render for TextFieldExamples {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.entity().downgrade();
        let secret_owner = cx.entity().downgrade();
        let password_visible = self.password_visible;
        let api_secret_visible = self.api_secret_visible;
        let unmasked_phone = self.phone.read(cx).unmask_value();
        let phone_supporting = if unmasked_phone.is_empty() {
            "US phone mask; unmasked digits appear here".to_string()
        } else {
            format!("Unmasked value: {unmasked_phone}")
        };
        stack()
            .max_w(px(440.))
            .child(
                section("Filled and outlined")
                    .child(
                        TextField::new("name", "姓名", &self.name)
                            .supporting_text("可使用微软拼音输入"),
                    )
                    .child(
                        TextField::new("email", "Email", &self.email)
                            .variant(TextFieldVariant::Outlined)
                            .error("Enter a domain after @"),
                    ),
            )
            .child(
                section("Multiline and secure")
                    .child(
                        TextField::multiline("notes", "备注", &self.notes)
                            .variant(TextFieldVariant::Outlined),
                    )
                    .child(
                        TextField::secure("password", "Password", &self.password)
                            .obfuscation_mode(if password_visible {
                                TextFieldObfuscationMode::Visible
                            } else {
                                TextFieldObfuscationMode::Hidden
                            })
                            .label_always_minimized(true)
                            .placeholder_content(
                                "Sample password",
                                Text::new("Enter a sample password"),
                            )
                            .trailing(
                                Button::new(
                                    "password-visibility",
                                    if password_visible { "Hide" } else { "Show" },
                                )
                                .variant(ButtonVariant::Text)
                                .on_activate(move |_, cx| {
                                    let _ = owner.update(cx, |view, cx| {
                                        view.password_visible = !view.password_visible;
                                        cx.notify();
                                    });
                                }),
                            )
                            .character_count(24),
                    ),
            )
            .child(
                section("Readonly and disabled")
                    .child(TextField::new(
                        "account",
                        "Account (readonly)",
                        &self.account,
                    ))
                    .child(TextField::new("status", "Account status", &self.disabled)),
            )
            .child(
                section("Label at 150% scale").child(
                    TextField::new(
                        "scaled-label",
                        "Preferred name for project correspondence",
                        &self.scaled,
                    )
                    .font_scale(1.5)
                    .supporting_text("用于项目沟通，可输入中英文"),
                ),
            )
            .child(
                section("Outside label and affixes").child(
                    TextField::new("amount", "Budget amount", &self.amount)
                        .variant(TextFieldVariant::Outlined)
                        .shape(gpui_material::theme::CornerRole::Medium.shape())
                        .content_padding(gpui::Edges {
                            left: px(16.),
                            right: px(20.),
                            top: px(8.),
                            bottom: px(8.),
                        })
                        .label_position(TextFieldLabelPosition::Above)
                        .label_alignment(
                            TextFieldLabelAlignment::Start,
                            TextFieldLabelAlignment::End,
                        )
                        .label_content(Text::new("Project budget"))
                        .placeholder_content("Amount", Text::new("0.00"))
                        .leading(Icon::new(IconName::Edit))
                        .prefix(div().id("currency-prefix").test_support().child("$"))
                        .suffix(div().id("currency-suffix").test_support().child("USD"))
                        .supporting_content("Amount in US dollars", Text::new("US dollars"))
                        .required(true)
                        .character_count(8),
                ),
            )
            .child(
                section("Formatting and validation")
                    .child(
                        TextField::new("phone", "Phone number", &self.phone)
                            .variant(TextFieldVariant::Outlined)
                            .supporting_text(phone_supporting),
                    )
                    .child(
                        TextField::new("code", "Invite code", &self.code)
                            .supporting_text(
                                "Rejects non-alphanumeric characters and excess length",
                            )
                            .character_count(6),
                    ),
            )
            .child(
                section("Clearable and visibility toggle")
                    .child(
                        TextField::outlined(
                            "clearable-search",
                            "Search query",
                            &self.clearable_query,
                        )
                        .leading(Icon::new(IconName::Search))
                        .clearable(true)
                        .supporting_text("Trailing clear button appears while non-empty"),
                    )
                    .child(
                        TextField::secure("api-secret", "API secret", &self.api_secret)
                            .variant(TextFieldVariant::Outlined)
                            .clearable(true)
                            .visibility_toggle(api_secret_visible, move |visible, _, cx| {
                                let _ = secret_owner.update(cx, |view, cx| {
                                    view.api_secret_visible = visible;
                                    cx.notify();
                                });
                            })
                            .supporting_text(
                                "Combines clearable and built-in Visibility/VisibilityOff toggle",
                            ),
                    ),
            )
    }
}
