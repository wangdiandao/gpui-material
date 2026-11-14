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
    adaptive_examples::AdaptiveExamples,
    carousel_examples::CarouselExamples,
    date_time_examples::DateTimeExamples,
    display_examples::DisplayExamples,
    group_examples::GroupExamples,
    navigation_examples::NavigationExamples,
    overlay_examples::{MenuTooltipExamples, ModalExamples, SnackbarExamples},
    search_examples::SearchExamples,
    selection_examples::SelectionExamples,
    slider_examples::SliderExamples,
    text_field_examples::{DropdownExamples, TextFieldExamples},
    toolbar_examples::ToolbarExamples,
};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(crate) enum Story {
    #[default]
    Introduction,
    AdaptiveLayout,
    AppBar,
    Badge,
    Button,
    ButtonGroup,
    Card,
    Carousel,
    Checkbox,
    Chip,
    DatePicker,
    DateRangePicker,
    Dialog,
    Divider,
    ExposedDropdown,
    ExtendedFab,
    Fab,
    FabMenu,
    Foundation,
    Icon,
    IconButton,
    List,
    LoadingIndicator,
    Menu,
    NavigationBar,
    NavigationRail,
    Progress,
    Radio,
    Search,
    Sheets,
    Slider,
    Snackbar,
    SplitButton,
    Switch,
    Tabs,
    Text,
    TextField,
    TimePicker,
    ToggleButton,
    Toolbars,
    Tooltip,
}

impl Story {
    // Introduction is the landing page. The component index follows display-name order.
    pub(crate) const ALL: &[Self] = &[
        Self::Introduction,
        Self::AdaptiveLayout,
        Self::AppBar,
        Self::Badge,
        Self::Button,
        Self::ButtonGroup,
        Self::Card,
        Self::Carousel,
        Self::Checkbox,
        Self::Chip,
        Self::DatePicker,
        Self::DateRangePicker,
        Self::Dialog,
        Self::Divider,
        Self::ExposedDropdown,
        Self::ExtendedFab,
        Self::Fab,
        Self::FabMenu,
        Self::Foundation,
        Self::Icon,
        Self::IconButton,
        Self::List,
        Self::LoadingIndicator,
        Self::Menu,
        Self::NavigationBar,
        Self::NavigationRail,
        Self::Progress,
        Self::Radio,
        Self::Search,
        Self::Sheets,
        Self::Slider,
        Self::Snackbar,
        Self::SplitButton,
        Self::Switch,
        Self::Tabs,
        Self::Text,
        Self::TextField,
        Self::TimePicker,
        Self::ToggleButton,
        Self::Toolbars,
        Self::Tooltip,
    ];

    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::ButtonGroup => "button-group",
            Self::Introduction => "introduction",
            Self::AdaptiveLayout => "adaptive-layout",
            Self::AppBar => "app-bar",
            Self::Badge => "badge",
            Self::Button => "button",
            Self::Card => "card",
            Self::Carousel => "carousel",
            Self::Checkbox => "checkbox",
            Self::Chip => "chip",
            Self::DatePicker => "date-picker",
            Self::DateRangePicker => "date-range-picker",
            Self::Dialog => "dialog",
            Self::Divider => "divider",
            Self::ExposedDropdown => "exposed-dropdown",
            Self::ExtendedFab => "extended-fab",
            Self::Fab => "fab",
            Self::FabMenu => "fab-menu",
            Self::Foundation => "foundation",
            Self::Icon => "icon",
            Self::IconButton => "icon-button",
            Self::List => "list",
            Self::LoadingIndicator => "loading-indicator",
            Self::Menu => "menu",
            Self::NavigationBar => "navigation-bar",
            Self::NavigationRail => "navigation-rail",
            Self::Progress => "progress",
            Self::Radio => "radio",
            Self::Search => "search",
            Self::Sheets => "sheets",
            Self::Slider => "slider",
            Self::Snackbar => "snackbar",
            Self::SplitButton => "split-button",
            Self::Switch => "switch",
            Self::Tabs => "tabs",
            Self::Text => "text",
            Self::TextField => "text-field",
            Self::TimePicker => "time-picker",
            Self::ToggleButton => "toggle-button",
            Self::Toolbars => "toolbars",
            Self::Tooltip => "tooltip",
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::ButtonGroup => "Button group",
            Self::Introduction => "Introduction",
            Self::AdaptiveLayout => "Adaptive layout",
            Self::AppBar => "App bar",
            Self::Badge => "Badge",
            Self::Button => "Button",
            Self::Card => "Card",
            Self::Carousel => "Carousel",
            Self::Checkbox => "Checkbox",
            Self::Chip => "Chip",
            Self::DatePicker => "Date picker",
            Self::DateRangePicker => "Date range picker",
            Self::Dialog => "Dialog",
            Self::Divider => "Divider",
            Self::ExposedDropdown => "Exposed dropdown",
            Self::ExtendedFab => "Extended FAB",
            Self::Fab => "FAB",
            Self::FabMenu => "FAB menu",
            Self::Foundation => "Foundation",
            Self::Icon => "Icon",
            Self::IconButton => "Icon button",
            Self::List => "List",
            Self::LoadingIndicator => "Loading indicator",
            Self::Menu => "Menu",
            Self::NavigationBar => "Navigation bar",
            Self::NavigationRail => "Navigation rail",
            Self::Progress => "Progress",
            Self::Radio => "Radio group",
            Self::Search => "Search",
            Self::Sheets => "Sheet",
            Self::Slider => "Slider",
            Self::Snackbar => "Snackbar",
            Self::SplitButton => "Split button",
            Self::Switch => "Switch",
            Self::Tabs => "Tabs",
            Self::Text => "Text",
            Self::TextField => "Text field",
            Self::TimePicker => "Time picker",
            Self::ToggleButton => "Toggle button",
            Self::Toolbars => "Toolbars",
            Self::Tooltip => "Tooltip",
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::ButtonGroup => {
                "Arrange actions or controlled choices in standard and connected rows."
            }
            Self::Introduction => {
                "Interactive examples from the Material 3 Expressive library for GPUI."
            }
            Self::AdaptiveLayout => {
                "Resize a scaffold to see navigation, panes, and feed columns adapt."
            }
            Self::AppBar => "Place navigation, titles, and commands above scrolling content.",
            Self::Badge => "Annotate icons with a dot, label, or count.",
            Self::Button => {
                "Trigger commands with Material button variants, sizes, and icon slots."
            }
            Self::Card => "Group related content and actions on a Material surface.",
            Self::Carousel => "Browse a collection using six Material layouts.",
            Self::Checkbox => "Select multiple options and represent a mixed selection.",
            Self::Chip => "Offer suggestions, filter results, or represent removable inputs.",
            Self::DatePicker => "Choose an available date in a docked calendar or a dialog.",
            Self::DateRangePicker => {
                "Choose two date endpoints on the calendar or enter them by hand."
            }
            Self::Dialog => "Request a decision or input and restore focus when dismissed.",
            Self::Divider => "Separate related content with horizontal or vertical rules.",
            Self::ExposedDropdown => "Pick an option from an editable or fixed list.",
            Self::ExtendedFab => "Give a prominent action a label that can expand or collapse.",
            Self::Fab => "Make the primary action easy to reach with a floating button.",
            Self::FabMenu => "Open a stack of related commands from a floating action button.",
            Self::Foundation => "Inspect window scaling, typography, drawing, and input adapters.",
            Self::Icon => {
                "Use official Material Symbols with shared sizing, color and right-to-left mirroring."
            }
            Self::IconButton => "Trigger or toggle actions with an icon.",
            Self::List => "Display selectable rows with supporting text and secondary actions.",
            Self::LoadingIndicator => {
                "Show that an operation is underway while waiting for a result."
            }
            Self::Menu => "Offer commands through a trigger, context menu, or submenu.",
            Self::NavigationBar => "Switch between primary destinations from a short bottom bar.",
            Self::NavigationRail => {
                "Keep destinations at the leading edge or open them in a modal rail."
            }
            Self::Progress => "Visualize measured or indeterminate work as a line or a circle.",
            Self::Radio => "Choose one option from a group.",
            Self::Search => "Search from a docked surface or a full-screen app bar.",
            Self::Sheets => "Reveal supplementary content at the bottom or trailing edge.",
            Self::Slider => "Adjust a value or range with a pointer or keyboard.",
            Self::Snackbar => "Announce an outcome and offer a follow-up action such as Undo.",
            Self::SplitButton => "Run a primary command or choose a related action from its menu.",
            Self::Switch => "Turn a setting on or off.",
            Self::Tabs => "Switch between related panels with fixed or scrolling tabs.",
            Self::Text => "Preview Material typography, font scaling, and mixed scripts.",
            Self::TextField => {
                "Edit single-line, multiline, and secure text with supporting feedback."
            }
            Self::TimePicker => "Set hours and minutes with a clock face or keyboard entry.",
            Self::ToggleButton => "Keep the on or off state of an action visible.",
            Self::Toolbars => "Group editing commands in floating and docked toolbars.",
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
            Self::AdaptiveLayout => cx.new(AdaptiveExamples::new).into(),
            Self::DatePicker | Self::DateRangePicker | Self::TimePicker => {
                cx.new(|cx| DateTimeExamples::new(self, window, cx)).into()
            }
            Self::Carousel => cx.new(|_| CarouselExamples::default()).into(),
            Self::ButtonGroup | Self::SplitButton => {
                cx.new(|cx| GroupExamples::new(self, cx)).into()
            }
            Self::Button
            | Self::ToggleButton
            | Self::IconButton
            | Self::Fab
            | Self::ExtendedFab => cx.new(|_| ActionExamples::for_story(self)).into(),
            Self::Checkbox | Self::Radio | Self::Switch | Self::Chip => {
                cx.new(|cx| SelectionExamples::for_story(self, cx)).into()
            }
            Self::Slider => cx.new(SliderExamples::new).into(),
            Self::Search => cx.new(|cx| SearchExamples::new(window, cx)).into(),
            Self::AppBar | Self::NavigationBar | Self::NavigationRail | Self::Tabs => {
                cx.new(|cx| NavigationExamples::new(self, cx)).into()
            }
            Self::Menu | Self::Tooltip => cx.new(|cx| MenuTooltipExamples::new(self, cx)).into(),
            Self::Dialog | Self::Sheets => cx.new(|cx| ModalExamples::new(self, window, cx)).into(),
            Self::TextField => cx.new(|cx| TextFieldExamples::new(window, cx)).into(),
            Self::ExposedDropdown => cx.new(|cx| DropdownExamples::new(window, cx)).into(),
            Self::Snackbar => cx.new(SnackbarExamples::new).into(),
            Self::FabMenu | Self::Toolbars => cx.new(|cx| ToolbarExamples::new(self, cx)).into(),
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
