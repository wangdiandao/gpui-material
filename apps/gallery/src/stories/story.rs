use gpui::{
    AnyElement, AnyView, App, AppContext, Div, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, Role, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_material::{
    Text,
    foundation::{Surface, color, window_material_theme},
    theme::{ColorRole, CornerRole, Direction, TypeRole},
};

use super::{
    action_examples::ActionExamples,
    adaptive_examples::AdaptiveExamples,
    carousel_examples::CarouselExamples,
    command_palette_examples::CommandPaletteExamples,
    data_views_examples::DataViewsExamples,
    date_time_examples::DateTimeExamples,
    disclosure_examples::DisclosureExamples,
    display_examples::DisplayExamples,
    drag_handle_examples::DragHandleExamples,
    group_examples::GroupExamples,
    navigation_examples::NavigationExamples,
    overlay_examples::{MenuTooltipExamples, ModalExamples, SnackbarExamples},
    rich_text_examples::RichTextExamples,
    search_examples::SearchExamples,
    selection_examples::SelectionExamples,
    slider_examples::SliderExamples,
    specialized_input_examples::SpecializedInputExamples,
    surface_examples::SurfaceExamples,
    text_field_examples::{DropdownExamples, TextFieldExamples},
    toolbar_examples::ToolbarExamples,
};

/// An entry in the GPUI Material Gallery catalog.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum Story {
    #[default]
    Introduction,
    Accordion,
    AdaptiveLayout,
    AppBar,
    Avatar,
    Badge,
    Banner,
    BottomAppBar,
    Breadcrumbs,
    Button,
    ButtonGroup,
    Card,
    Carousel,
    Checkbox,
    Chip,
    ChipGroup,
    Collapsible,
    ColorPicker,
    CommandPalette,
    DataViews,
    DatePicker,
    DateRangePicker,
    Dialog,
    Divider,
    DropTarget,
    EmptyState,
    ExposedDropdown,
    ExtendedFab,
    Fab,
    FabMenu,
    Icon,
    IconButton,
    List,
    LoadingIndicator,
    Menu,
    NavigationBar,
    NavigationDrawer,
    NavigationRail,
    NumberInput,
    OtpInput,
    Pagination,
    Progress,
    Radio,
    RichText,
    Search,
    SelectableText,
    Sheets,
    Skeleton,
    Slider,
    Snackbar,
    SplitButton,
    Stepper,
    Surface,
    Switch,
    Tabs,
    Text,
    TextField,
    TimePicker,
    ToggleButton,
    Toolbars,
    Tooltip,
    VerticalDragHandle,
}

impl Story {
    /// The Gallery landing page followed by examples in display-name order.
    pub const ALL: &[Self] = &[
        Self::Introduction,
        Self::Accordion,
        Self::AdaptiveLayout,
        Self::AppBar,
        Self::Avatar,
        Self::Badge,
        Self::Banner,
        Self::BottomAppBar,
        Self::Breadcrumbs,
        Self::Button,
        Self::ButtonGroup,
        Self::Card,
        Self::Carousel,
        Self::Checkbox,
        Self::Chip,
        Self::ChipGroup,
        Self::Collapsible,
        Self::ColorPicker,
        Self::CommandPalette,
        Self::DataViews,
        Self::DatePicker,
        Self::DateRangePicker,
        Self::Dialog,
        Self::Divider,
        Self::DropTarget,
        Self::EmptyState,
        Self::ExposedDropdown,
        Self::ExtendedFab,
        Self::Fab,
        Self::FabMenu,
        Self::Icon,
        Self::IconButton,
        Self::List,
        Self::LoadingIndicator,
        Self::Menu,
        Self::NavigationBar,
        Self::NavigationDrawer,
        Self::NavigationRail,
        Self::NumberInput,
        Self::OtpInput,
        Self::Pagination,
        Self::Progress,
        Self::Radio,
        Self::RichText,
        Self::Search,
        Self::SelectableText,
        Self::Sheets,
        Self::Skeleton,
        Self::Slider,
        Self::Snackbar,
        Self::SplitButton,
        Self::Stepper,
        Self::Surface,
        Self::Switch,
        Self::Tabs,
        Self::Text,
        Self::TextField,
        Self::TimePicker,
        Self::ToggleButton,
        Self::Toolbars,
        Self::Tooltip,
        Self::VerticalDragHandle,
    ];

    /// Returns the stable identifier used for routes and command-line arguments.
    pub fn id(self) -> &'static str {
        match self {
            Self::ButtonGroup => "button-group",
            Self::Introduction => "introduction",
            Self::Accordion => "accordion",
            Self::AdaptiveLayout => "adaptive-layout",
            Self::AppBar => "app-bar",
            Self::Avatar => "avatar",
            Self::Badge => "badge",
            Self::Banner => "banner",
            Self::BottomAppBar => "bottom-app-bar",
            Self::Breadcrumbs => "breadcrumbs",
            Self::Button => "button",
            Self::Card => "card",
            Self::Carousel => "carousel",
            Self::Checkbox => "checkbox",
            Self::Chip => "chip",
            Self::ChipGroup => "chip-group",
            Self::Collapsible => "collapsible",
            Self::ColorPicker => "color-picker",
            Self::CommandPalette => "command-palette",
            Self::DataViews => "data-views",
            Self::DatePicker => "date-picker",
            Self::DateRangePicker => "date-range-picker",
            Self::Dialog => "dialog",
            Self::Divider => "divider",
            Self::DropTarget => "drop-target",
            Self::EmptyState => "empty-state",
            Self::ExposedDropdown => "exposed-dropdown",
            Self::ExtendedFab => "extended-fab",
            Self::Fab => "fab",
            Self::FabMenu => "fab-menu",
            Self::Icon => "icon",
            Self::IconButton => "icon-button",
            Self::List => "list",
            Self::LoadingIndicator => "loading-indicator",
            Self::Menu => "menu",
            Self::NavigationBar => "navigation-bar",
            Self::NavigationDrawer => "navigation-drawer",
            Self::NavigationRail => "navigation-rail",
            Self::NumberInput => "number-input",
            Self::OtpInput => "otp-input",
            Self::Pagination => "pagination",
            Self::Progress => "progress",
            Self::Radio => "radio",
            Self::RichText => "rich-text",
            Self::Search => "search",
            Self::SelectableText => "selectable-text",
            Self::Sheets => "sheets",
            Self::Skeleton => "skeleton",
            Self::Slider => "slider",
            Self::Snackbar => "snackbar",
            Self::SplitButton => "split-button",
            Self::Stepper => "stepper",
            Self::Surface => "surface",
            Self::Switch => "switch",
            Self::Tabs => "tabs",
            Self::Text => "text",
            Self::TextField => "text-field",
            Self::TimePicker => "time-picker",
            Self::ToggleButton => "toggle-button",
            Self::Toolbars => "toolbars",
            Self::Tooltip => "tooltip",
            Self::VerticalDragHandle => "vertical-drag-handle",
        }
    }

    /// Returns the display name of this catalog entry.
    pub fn name(self) -> &'static str {
        match self {
            Self::ButtonGroup => "Button group",
            Self::Introduction => "Introduction",
            Self::Accordion => "Accordion",
            Self::AdaptiveLayout => "Adaptive layout",
            Self::AppBar => "App bar",
            Self::Avatar => "Avatar",
            Self::Badge => "Badge",
            Self::Banner => "Banner",
            Self::BottomAppBar => "Bottom app bar",
            Self::Breadcrumbs => "Breadcrumbs",
            Self::Button => "Button",
            Self::Card => "Card",
            Self::Carousel => "Carousel",
            Self::Checkbox => "Checkbox",
            Self::Chip => "Chip",
            Self::ChipGroup => "Chip group",
            Self::Collapsible => "Collapsible",
            Self::ColorPicker => "Color picker",
            Self::CommandPalette => "Command palette",
            Self::DataViews => "Data views",
            Self::DatePicker => "Date picker",
            Self::DateRangePicker => "Date range picker",
            Self::Dialog => "Dialog",
            Self::Divider => "Divider",
            Self::DropTarget => "Drop target",
            Self::EmptyState => "Empty state",
            Self::ExposedDropdown => "Exposed dropdown",
            Self::ExtendedFab => "Extended FAB",
            Self::Fab => "FAB",
            Self::FabMenu => "FAB menu",
            Self::Icon => "Icon",
            Self::IconButton => "Icon button",
            Self::List => "List",
            Self::LoadingIndicator => "Loading indicator",
            Self::Menu => "Menu",
            Self::NavigationBar => "Navigation bar",
            Self::NavigationDrawer => "Navigation drawer",
            Self::NavigationRail => "Navigation rail",
            Self::NumberInput => "Number input",
            Self::OtpInput => "OTP input",
            Self::Pagination => "Pagination",
            Self::Progress => "Progress",
            Self::Radio => "Radio group",
            Self::RichText => "Rich text",
            Self::Search => "Search",
            Self::SelectableText => "Selectable text",
            Self::Sheets => "Sheet",
            Self::Skeleton => "Skeleton",
            Self::Slider => "Slider",
            Self::Snackbar => "Snackbar",
            Self::SplitButton => "Split button",
            Self::Stepper => "Stepper",
            Self::Surface => "Surface",
            Self::Switch => "Switch",
            Self::Tabs => "Tabs",
            Self::Text => "Text",
            Self::TextField => "Text field",
            Self::TimePicker => "Time picker",
            Self::ToggleButton => "Toggle button",
            Self::Toolbars => "Toolbars",
            Self::Tooltip => "Tooltip",
            Self::VerticalDragHandle => "Vertical drag handle",
        }
    }

    /// Returns the description shown in the Gallery.
    pub fn description(self) -> &'static str {
        match self {
            Self::ButtonGroup => {
                "Arrange actions or controlled choices in standard and connected rows."
            }
            Self::Introduction => {
                "Interactive examples from the Material 3 Expressive library for GPUI."
            }
            Self::Accordion => {
                "Expand and collapse grouped sections in Standard or Segmented Material 3 layouts."
            }
            Self::AdaptiveLayout => {
                "Resize the window and switch between Inbox, Activity, and Feed to compare the layouts."
            }
            Self::AppBar => "Place navigation, titles, and commands above scrolling content.",
            Self::Avatar => "Display user or entity initials, icons, images, and fallback avatars.",
            Self::Badge => "Annotate icons with a dot, label, or count.",
            Self::Banner => {
                "Present in-flow Material 3 banners with semantic tones, actions, and optional dismissal."
            }
            Self::BottomAppBar => {
                "Dock primary actions and a floating action button along the bottom edge."
            }
            Self::Breadcrumbs => {
                "Navigate hierarchical desktop paths with automatic middle-segment overflow menus."
            }
            Self::Button => {
                "Trigger commands with Material button variants, sizes, and icon slots."
            }
            Self::Card => "Group related content and actions on a Material surface.",
            Self::Carousel => "Browse a collection using six Material layouts.",
            Self::Checkbox => "Select multiple options and represent a mixed selection.",
            Self::Chip => "Offer suggestions, filter results, or represent removable inputs.",
            Self::ChipGroup => {
                "Coordinate controlled chip selection with roving focus and wrapping or scrolling rows."
            }
            Self::Collapsible => {
                "Reveal or hide supplementary content inside a Material 3 disclosure container."
            }
            Self::ColorPicker => {
                "Pick colors with Material 3 presets, HSLA sliders, and live hex synced with Argb."
            }
            Self::CommandPalette => {
                "Filter and execute grouped desktop commands in a Ctrl/Cmd+K modal palette."
            }
            Self::DataViews => {
                "Inspect sortable data tables, virtualized hierarchical trees, and variable-size virtual lists."
            }
            Self::DatePicker => "Choose an available date in a docked calendar or a dialog.",
            Self::DateRangePicker => {
                "Choose two date endpoints on the calendar or enter them by hand."
            }
            Self::Dialog => "Request a decision or input and restore focus when dismissed.",
            Self::Divider => "Separate related content with horizontal or vertical rules.",
            Self::DropTarget => {
                "Accept external OS file drops or typed in-app drag payloads with validation feedback."
            }
            Self::EmptyState => {
                "Present an expressive empty-state shell with optional icon backdrop, copy, and actions."
            }
            Self::ExposedDropdown => "Pick an option from an editable or fixed list.",
            Self::ExtendedFab => "Give a prominent action a label that can expand or collapse.",
            Self::Fab => "Make the primary action easy to reach with a floating button.",
            Self::FabMenu => "Open a stack of related commands from a floating action button.",
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
            Self::NavigationDrawer => {
                "Present grouped destinations in a docked or modal 360 dp navigation drawer."
            }
            Self::NavigationRail => {
                "Keep destinations at the leading edge or open them in a modal rail."
            }
            Self::NumberInput => {
                "Enter bounded numeric values with decrement/increment steppers and Up/Down stepping."
            }
            Self::OtpInput => {
                "Capture fixed-length verification codes with paste, masking, and slot navigation."
            }
            Self::Pagination => {
                "Navigate multi-page desktop collections with active page pills and ellipsis truncation."
            }
            Self::Progress => "Visualize measured or indeterminate work as a line or a circle.",
            Self::Radio => "Choose one option from a group.",
            Self::RichText => {
                "Render Markdown and HTML with Material typography, code surfaces, links, and ThemeScope."
            }
            Self::Search => "Search from a docked surface or a full-screen app bar.",
            Self::SelectableText => {
                "Display Material typography with drag selection, keyboard copy, and PrimaryContainer highlights."
            }
            Self::Sheets => "Reveal supplementary content at the bottom or trailing edge.",
            Self::Skeleton => {
                "Display animated Material 3 loading placeholders with offscreen and reduced-motion discipline."
            }
            Self::Slider => "Adjust a value or range with a pointer or keyboard.",
            Self::Snackbar => "Announce an outcome and offer a follow-up action such as Undo.",
            Self::SplitButton => "Run a primary command or choose a related action from its menu.",
            Self::Stepper => "Guide users through horizontal or vertical multi-step workflows.",
            Self::Surface => "Compose passive surfaces and named controls with custom content.",
            Self::Switch => "Turn a setting on or off.",
            Self::Tabs => "Switch between related panels with fixed or scrolling tabs.",
            Self::Text => {
                "Explore Material type roles and see how emphasis, scaling, and overflow affect text."
            }
            Self::TextField => {
                "Edit single-line, multiline, and secure text with supporting feedback."
            }
            Self::TimePicker => {
                "Set hours and minutes with a clock face, keyboard entry, or scroll fields."
            }
            Self::ToggleButton => "Keep the on or off state of an action visible.",
            Self::Toolbars => "Group editing commands in floating and docked toolbars.",
            Self::Tooltip => "Provide contextual help for a control on hover or focus.",
            Self::VerticalDragHandle => "Resize two panes with a pointer or the keyboard.",
        }
    }

    /// Finds an entry by identifier or display name, ignoring ASCII case.
    pub fn find(name: &str) -> Option<Self> {
        let name = name.trim();
        Self::ALL.iter().copied().find(|story| {
            story.id().eq_ignore_ascii_case(name) || story.name().eq_ignore_ascii_case(name)
        })
    }

    /// Creates one example with fresh state for the given window.
    /// The Gallery renders [`Self::Introduction`] itself, so it returns `None`.
    pub fn new_view(self, window: &mut Window, cx: &mut App) -> Option<AnyView> {
        let view = match self {
            Self::Introduction => return None,
            Self::AdaptiveLayout => cx.new(AdaptiveExamples::new).into(),
            Self::DatePicker | Self::DateRangePicker | Self::TimePicker => {
                cx.new(|cx| DateTimeExamples::new(self, window, cx)).into()
            }
            Self::Carousel => cx.new(|_| CarouselExamples::default()).into(),
            Self::Accordion | Self::Collapsible | Self::Pagination => {
                cx.new(|_| DisclosureExamples::new(self)).into()
            }
            Self::CommandPalette => cx.new(|cx| CommandPaletteExamples::new(window, cx)).into(),
            Self::DataViews => cx.new(DataViewsExamples::new).into(),
            Self::ButtonGroup | Self::SplitButton => {
                cx.new(|cx| GroupExamples::new(self, cx)).into()
            }
            Self::Button
            | Self::ToggleButton
            | Self::IconButton
            | Self::Fab
            | Self::ExtendedFab => cx.new(|_| ActionExamples::for_story(self)).into(),
            Self::Checkbox | Self::Radio | Self::Switch | Self::Chip | Self::ChipGroup => {
                cx.new(|cx| SelectionExamples::for_story(self, cx)).into()
            }
            Self::Slider => cx.new(SliderExamples::new).into(),
            Self::Surface => cx.new(SurfaceExamples::new).into(),
            Self::VerticalDragHandle => cx.new(DragHandleExamples::new).into(),
            Self::Search => cx.new(|cx| SearchExamples::new(window, cx)).into(),
            Self::AppBar
            | Self::BottomAppBar
            | Self::Breadcrumbs
            | Self::NavigationBar
            | Self::NavigationDrawer
            | Self::NavigationRail
            | Self::Stepper
            | Self::Tabs => cx.new(|cx| NavigationExamples::new(self, cx)).into(),
            Self::Menu | Self::Tooltip => cx.new(|cx| MenuTooltipExamples::new(self, cx)).into(),
            Self::Dialog | Self::Sheets => cx.new(|cx| ModalExamples::new(self, window, cx)).into(),
            Self::TextField => cx.new(|cx| TextFieldExamples::new(window, cx)).into(),
            Self::NumberInput | Self::OtpInput | Self::ColorPicker => cx
                .new(|cx| SpecializedInputExamples::new(self, window, cx))
                .into(),
            Self::SelectableText | Self::RichText => {
                cx.new(|cx| RichTextExamples::new(self, cx)).into()
            }
            Self::ExposedDropdown => cx.new(|cx| DropdownExamples::new(window, cx)).into(),
            Self::Snackbar => cx.new(SnackbarExamples::new).into(),
            Self::FabMenu | Self::Toolbars => cx.new(|cx| ToolbarExamples::new(self, cx)).into(),
            Self::Avatar
            | Self::Badge
            | Self::Banner
            | Self::Card
            | Self::Divider
            | Self::DropTarget
            | Self::EmptyState
            | Self::Icon
            | Self::List
            | Self::LoadingIndicator
            | Self::Progress
            | Self::Skeleton
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
    description: Option<&'static str>,
    examples: Vec<AnyElement>,
}

impl StorySection {
    pub(crate) fn description(mut self, description: &'static str) -> Self {
        self.description = Some(description);
        self
    }
}

impl ParentElement for StorySection {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.examples.extend(elements);
    }
}

impl RenderOnce for StorySection {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = window_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let radius = CornerRole::Large
            .shape()
            .radii(100., 100., theme.layout.direction)[0];
        let outline = color(theme.colors.color(ColorRole::OutlineVariant));
        div()
            .id(self.title)
            .w_full()
            .min_w_0()
            .rounded(px(radius))
            .border_1()
            .border_color(outline)
            .bg(color(theme.colors.color(ColorRole::SurfaceContainerLow)))
            .flex()
            .flex_col()
            .child(
                div()
                    .w_full()
                    .px_4()
                    .py_1p5()
                    .border_b_1()
                    .border_color(outline)
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .when(rtl, Styled::items_end)
                    .child(
                        div()
                            .id("section-title")
                            .role(Role::Heading)
                            .aria_label(self.title)
                            .child(
                                Text::new(self.title)
                                    .id("title-text")
                                    .role(TypeRole::TitleSmall)
                                    .emphasized(true)
                                    .wrap(),
                            ),
                    )
                    .children(self.description.map(|description| {
                        Text::new(description)
                            .id("section-description")
                            .role(TypeRole::BodySmall)
                            .color(ColorRole::OnSurfaceVariant)
                            .wrap()
                    })),
            )
            .child(
                Surface::new(ColorRole::Surface)
                    .w_full()
                    .min_w_0()
                    .rounded_b(px(radius))
                    .p_4()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .children(self.examples),
            )
    }
}

pub(crate) fn section(title: &'static str) -> StorySection {
    StorySection {
        title,
        description: None,
        examples: Vec::new(),
    }
}
