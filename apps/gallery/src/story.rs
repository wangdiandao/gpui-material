use gpui::{
    AnyElement, AnyView, App, AppContext, Div, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, Role, StatefulInteractiveElement, Styled, Window, div,
};
use gpui_material::{
    Card, CardVariant, Text,
    theme::{ColorRole, TypeRole},
};

use crate::{
    action_examples::ActionExamples,
    display_examples::DisplayExamples,
    overlay_examples::{MenuTooltipExamples, ModalExamples, SnackbarExamples},
    selection_examples::SelectionExamples,
    slider_examples::SliderExamples,
    text_field_examples::{DropdownExamples, TextFieldExamples},
};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(crate) enum Story {
    #[default]
    Introduction,
    Badge,
    Button,
    Card,
    Checkbox,
    Chip,
    Dialog,
    Divider,
    ExposedDropdown,
    ExtendedFab,
    Fab,
    Foundation,
    Icon,
    IconButton,
    List,
    LoadingIndicator,
    Menu,
    Progress,
    Radio,
    Sheets,
    Slider,
    Snackbar,
    Switch,
    Text,
    TextField,
    ToggleButton,
    Tooltip,
}

impl Story {
    // Introduction is the landing page. The component index follows display-name order.
    pub(crate) const ALL: &[Self] = &[
        Self::Introduction,
        Self::Badge,
        Self::Button,
        Self::Card,
        Self::Checkbox,
        Self::Chip,
        Self::Dialog,
        Self::Divider,
        Self::ExposedDropdown,
        Self::ExtendedFab,
        Self::Fab,
        Self::Foundation,
        Self::Icon,
        Self::IconButton,
        Self::List,
        Self::LoadingIndicator,
        Self::Menu,
        Self::Progress,
        Self::Radio,
        Self::Sheets,
        Self::Slider,
        Self::Snackbar,
        Self::Switch,
        Self::Text,
        Self::TextField,
        Self::ToggleButton,
        Self::Tooltip,
    ];

    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::Introduction => "introduction",
            Self::Badge => "badge",
            Self::Button => "button",
            Self::Card => "card",
            Self::Checkbox => "checkbox",
            Self::Chip => "chip",
            Self::Dialog => "dialog",
            Self::Divider => "divider",
            Self::ExposedDropdown => "exposed-dropdown",
            Self::ExtendedFab => "extended-fab",
            Self::Fab => "fab",
            Self::Foundation => "foundation",
            Self::Icon => "icon",
            Self::IconButton => "icon-button",
            Self::List => "list",
            Self::LoadingIndicator => "loading-indicator",
            Self::Menu => "menu",
            Self::Progress => "progress",
            Self::Radio => "radio",
            Self::Sheets => "sheets",
            Self::Slider => "slider",
            Self::Snackbar => "snackbar",
            Self::Switch => "switch",
            Self::Text => "text",
            Self::TextField => "text-field",
            Self::ToggleButton => "toggle-button",
            Self::Tooltip => "tooltip",
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Introduction => "Introduction",
            Self::Badge => "Badge",
            Self::Button => "Button",
            Self::Card => "Card",
            Self::Checkbox => "Checkbox",
            Self::Chip => "Chip",
            Self::Dialog => "Dialog",
            Self::Divider => "Divider",
            Self::ExposedDropdown => "Exposed dropdown",
            Self::ExtendedFab => "Extended FAB",
            Self::Fab => "FAB",
            Self::Foundation => "Foundation",
            Self::Icon => "Icon",
            Self::IconButton => "Icon button",
            Self::List => "List",
            Self::LoadingIndicator => "Loading indicator",
            Self::Menu => "Menu",
            Self::Progress => "Progress",
            Self::Radio => "Radio group",
            Self::Sheets => "Sheet",
            Self::Slider => "Slider",
            Self::Snackbar => "Snackbar",
            Self::Switch => "Switch",
            Self::Text => "Text",
            Self::TextField => "Text field",
            Self::ToggleButton => "Toggle button",
            Self::Tooltip => "Tooltip",
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::Introduction => {
                "Interactive examples from the Material 3 Expressive library for GPUI."
            }
            Self::Badge => "Annotate icons with a dot, label, or count.",
            Self::Button => {
                "Trigger commands with Material button variants, sizes, and icon slots."
            }
            Self::Card => "Group related content and actions on a Material surface.",
            Self::Checkbox => "Select multiple options and represent a mixed selection.",
            Self::Chip => "Offer suggestions, filter results, or represent removable inputs.",
            Self::Dialog => "Request a decision or input and restore focus when dismissed.",
            Self::Divider => "Separate related content with horizontal or vertical rules.",
            Self::ExposedDropdown => "Pick an option from an editable or fixed list.",
            Self::ExtendedFab => "Give a prominent action a label that can expand or collapse.",
            Self::Fab => "Make the primary action easy to reach with a floating button.",
            Self::Foundation => "Inspect window scaling, typography, drawing, and input adapters.",
            Self::Icon => "Render SVG icons with optional mirroring in right-to-left layouts.",
            Self::IconButton => "Trigger or toggle actions with an icon.",
            Self::List => "Display selectable rows with supporting text and secondary actions.",
            Self::LoadingIndicator => {
                "Show that an operation is underway while waiting for a result."
            }
            Self::Menu => "Offer commands through a trigger, context menu, or submenu.",
            Self::Progress => "Visualize measured or indeterminate work as a line or a circle.",
            Self::Radio => "Choose one option from a group.",
            Self::Sheets => "Reveal supplementary content at the bottom or trailing edge.",
            Self::Slider => "Adjust a value or range with a pointer or keyboard.",
            Self::Snackbar => "Announce an outcome and offer a follow-up action such as Undo.",
            Self::Switch => "Turn a setting on or off.",
            Self::Text => "Preview Material typography, font scaling, and mixed scripts.",
            Self::TextField => {
                "Edit single-line, multiline, and secure text with supporting feedback."
            }
            Self::ToggleButton => "Keep the on or off state of an action visible.",
            Self::Tooltip => "Provide contextual help for a control on hover or focus.",
        }
    }

    pub(crate) fn find(name: &str) -> Option<Self> {
        let name = name.trim();
        Self::ALL.iter().copied().find(|story| {
            story.id().eq_ignore_ascii_case(name) || story.name().eq_ignore_ascii_case(name)
        })
    }

    pub(crate) fn matches(self, query: &str) -> bool {
        let name = self.name().to_ascii_lowercase();
        query
            .split_whitespace()
            .all(|word| name.contains(word) || self.id().contains(word))
    }

    pub(crate) fn new_view(self, window: &mut Window, cx: &mut App) -> Option<AnyView> {
        let view = match self {
            Self::Introduction | Self::Foundation => return None,
            Self::Button
            | Self::ToggleButton
            | Self::IconButton
            | Self::Fab
            | Self::ExtendedFab => cx.new(|_| ActionExamples::for_story(self)).into(),
            Self::Checkbox | Self::Radio | Self::Switch | Self::Chip => {
                cx.new(|cx| SelectionExamples::for_story(self, cx)).into()
            }
            Self::Slider => cx.new(SliderExamples::new).into(),
            Self::Menu | Self::Tooltip => cx.new(|cx| MenuTooltipExamples::new(self, cx)).into(),
            Self::Dialog | Self::Sheets => cx.new(|cx| ModalExamples::new(self, window, cx)).into(),
            Self::TextField => cx.new(|cx| TextFieldExamples::new(window, cx)).into(),
            Self::ExposedDropdown => cx.new(|cx| DropdownExamples::new(window, cx)).into(),
            Self::Snackbar => cx.new(SnackbarExamples::new).into(),
            Self::Badge
            | Self::Card
            | Self::Divider
            | Self::Icon
            | Self::List
            | Self::LoadingIndicator
            | Self::Progress
            | Self::Text => cx.new(|_| DisplayExamples::for_story(self)).into(),
        };
        Some(view)
    }
}

pub(crate) fn stack() -> Div {
    div().flex().flex_col().min_w_0().gap_4()
}

#[derive(IntoElement)]
pub(crate) struct StorySection {
    title: &'static str,
    examples: Card,
}

impl ParentElement for StorySection {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.examples.extend(elements);
    }
}

impl RenderOnce for StorySection {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        stack()
            .id(self.title)
            .w_full()
            .gap_2()
            .child(
                div()
                    .id("section-title")
                    .role(Role::Heading)
                    .aria_label(self.title)
                    .child(
                        Text::new(self.title)
                            .role(TypeRole::TitleSmall)
                            .color(ColorRole::OnSurfaceVariant),
                    ),
            )
            .child(self.examples)
    }
}

pub(crate) fn section(title: &'static str) -> StorySection {
    StorySection {
        title,
        examples: Card::new("examples").variant(CardVariant::Outlined),
    }
}
