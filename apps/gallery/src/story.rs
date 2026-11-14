use gpui::{Div, ParentElement, Styled, div};
use gpui_material::{
    Text,
    theme::{ColorRole, TypeRole},
};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(crate) enum Story {
    #[default]
    Foundation,
    Button,
    ToggleButton,
    IconButton,
    Fab,
    Checkbox,
    Radio,
    Switch,
    Chip,
    Slider,
    TextIcon,
    Card,
    List,
    DividerBadge,
    Progress,
    Dialog,
    Sheets,
    TextField,
    ExposedDropdown,
    Snackbar,
}

impl Story {
    pub(crate) const GROUPS: &[(&str, &[Self])] = &[
        ("Overview", &[Self::Foundation]),
        (
            "Actions",
            &[
                Self::Button,
                Self::ToggleButton,
                Self::IconButton,
                Self::Fab,
            ],
        ),
        (
            "Selection",
            &[
                Self::Checkbox,
                Self::Radio,
                Self::Switch,
                Self::Chip,
                Self::Slider,
            ],
        ),
        (
            "Display",
            &[Self::TextIcon, Self::Card, Self::List, Self::DividerBadge],
        ),
        ("Overlays", &[Self::Dialog, Self::Sheets]),
        ("Input", &[Self::TextField, Self::ExposedDropdown]),
        ("Feedback", &[Self::Progress, Self::Snackbar]),
    ];

    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::Foundation => "foundation",
            Self::Button => "button",
            Self::ToggleButton => "toggle-button",
            Self::IconButton => "icon-button",
            Self::Fab => "fab",
            Self::Checkbox => "checkbox",
            Self::Radio => "radio",
            Self::Switch => "switch",
            Self::Chip => "chip",
            Self::Slider => "slider",
            Self::TextIcon => "text-icon",
            Self::Card => "card",
            Self::List => "list",
            Self::DividerBadge => "divider-badge",
            Self::Progress => "progress",
            Self::Dialog => "dialog",
            Self::Sheets => "sheets",
            Self::TextField => "text-field",
            Self::ExposedDropdown => "exposed-dropdown",
            Self::Snackbar => "snackbar",
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Foundation => "Foundation",
            Self::Button => "Button",
            Self::ToggleButton => "Toggle button",
            Self::IconButton => "Icon buttons",
            Self::Fab => "FAB & Extended FAB",
            Self::Checkbox => "Checkbox",
            Self::Radio => "Radio group",
            Self::Switch => "Switch",
            Self::Chip => "Chips",
            Self::Slider => "Sliders",
            Self::TextIcon => "Text & Icons",
            Self::Card => "Cards",
            Self::List => "Lists",
            Self::DividerBadge => "Divider & Badge",
            Self::Progress => "Progress & Loading",
            Self::Dialog => "Dialogs",
            Self::Sheets => "Sheets",
            Self::TextField => "Text fields",
            Self::ExposedDropdown => "Exposed dropdown",
            Self::Snackbar => "Snackbar",
        }
    }

    pub(crate) fn matches(self, query: &str) -> bool {
        query
            .split_whitespace()
            .all(|word| self.name().to_ascii_lowercase().contains(word) || self.id().contains(word))
    }
}

pub(crate) fn stack() -> Div {
    div().flex().flex_col().min_w_0().gap_4()
}

pub(crate) fn section(title: &'static str) -> Div {
    stack().gap_2().child(
        Text::new(title)
            .id(title)
            .role(TypeRole::TitleSmall)
            .color(ColorRole::OnSurfaceVariant),
    )
}
