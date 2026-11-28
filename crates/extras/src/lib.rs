//! Optional composite Material components for GPUI desktop applications.
#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

#[cfg(any(feature = "expressive-groups", feature = "toolbars"))]
mod arrangement;
#[cfg(any(feature = "expressive-groups", feature = "toolbars"))]
pub use arrangement::ContentArrangement;

#[cfg(feature = "carousel")]
mod carousel;
#[cfg(feature = "date-time")]
pub mod date_time;
#[cfg(feature = "carousel")]
pub use carousel::{
    Carousel, CarouselAlignment, CarouselFling, CarouselFlingContext, CarouselItem,
    CarouselParallax, CarouselState, CarouselVariant,
};

#[cfg(feature = "expressive-groups")]
mod groups;
#[cfg(feature = "expressive-groups")]
pub use groups::{
    ButtonGroup, ButtonGroupConnectedShapes, ButtonGroupItem, ButtonGroupSelection,
    ButtonGroupVariant, ButtonGroupVerticalAlignment, ButtonSegmentStyle,
};
#[cfg(feature = "expressive-groups")]
mod split_button;
#[cfg(feature = "expressive-groups")]
pub use split_button::SplitButton;

#[cfg(feature = "rich-text")]
mod rich_text;
#[cfg(feature = "rich-text")]
pub use rich_text::{
    RichText, RichTextSelectionFormat, RichTextState, RichTextStyle, RichTextTaskToggle,
    SelectableText, SelectableTextGroup,
};

#[cfg(feature = "search")]
mod search;
#[cfg(feature = "search")]
pub use search::{
    SearchBar, SearchEvent, SearchMotion, SearchOutcome, SearchPopupOptions, SearchRequest,
    SearchResult, SearchScrim, SearchScrollState, SearchState, SearchStyle, SearchValue,
    SearchView, SearchViewVariant, SearchWindowOptions,
};

#[cfg(feature = "specialized-inputs")]
mod specialized_inputs;
#[cfg(feature = "specialized-inputs")]
pub use specialized_inputs::{
    ColorPicker, ColorPickerColors, ColorPickerEvent, ColorPickerPreset, ColorPickerState,
    NumberInput, NumberInputEvent, NumberInputState, NumberInputStepper, NumberStepAction,
    OtpInput, OtpInputColors, OtpInputEvent, OtpInputState, OtpInputVariant, argb_to_hsla,
    format_hex_argb, hsla_to_argb, parse_hex_argb,
};

#[cfg(feature = "toolbars")]
mod toolbar;
#[cfg(feature = "toolbars")]
pub use toolbar::{
    FabMenu, FabMenuAlignment, FabMenuItem, FabMenuItemColors, FabMenuTriggerAppearance,
    FabMenuTriggerStyle, Toolbar, ToolbarAction, ToolbarColor, ToolbarColors, ToolbarExitDirection,
    ToolbarFabPosition, ToolbarFling, ToolbarOrientation, ToolbarPlacement, ToolbarScrollBehavior,
    ToolbarScrollState,
};

#[cfg(feature = "window-chrome")]
pub mod window_chrome;
