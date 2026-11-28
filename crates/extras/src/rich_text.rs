//! Material 3 selectable text and Markdown/HTML rich-text rendering.

mod selectable_text;
mod viewer;

pub use selectable_text::{SelectableText, SelectableTextGroup};
pub use viewer::{
    RichText, RichTextSelectionFormat, RichTextState, RichTextStyle, RichTextTaskToggle,
};
