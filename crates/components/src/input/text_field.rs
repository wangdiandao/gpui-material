use std::rc::Rc;

use gpui::{
    AnyElement, App, Bounds, Edges, ElementId, Entity, Hsla, IntoElement, Pixels, SharedString,
    Window,
};
use gpui_base::input::{InputBaseState, InputModeKind};
use gpui_material_foundation::InteractionSource;
use gpui_material_theme::{CornerShape, TypeStyle};

use super::{TextAreaState, TextFieldState};

mod render;

/// Material text field container treatment. The default is Filled.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextFieldVariant {
    #[default]
    Filled,
    Outlined,
}

/// Position of the visual label. Filled uses Inside and Outlined uses Cutout by default.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextFieldLabelPosition {
    /// The minimized label remains within the container, above the editor.
    Inside,
    /// The minimized label crosses the container's top edge.
    Cutout,
    /// The label is always minimized, with its own row above the container.
    Above,
}

/// Horizontal label alignment. Start and End follow the Material theme direction.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextFieldLabelAlignment {
    #[default]
    Start,
    Center,
    End,
}

/// Optional colors for a single text field state. An unset role uses the window theme.
#[derive(Clone, Copy, Debug, Default)]
pub struct TextFieldColorOverrides {
    pub container: Option<Hsla>,
    pub text: Option<Hsla>,
    pub label: Option<Hsla>,
    pub placeholder: Option<Hsla>,
    pub leading: Option<Hsla>,
    pub trailing: Option<Hsla>,
    pub prefix: Option<Hsla>,
    pub suffix: Option<Hsla>,
    pub supporting: Option<Hsla>,
    pub indicator: Option<Hsla>,
    pub caret: Option<Hsla>,
    pub selection: Option<Hsla>,
}

/// Per-instance colors. Disabled takes precedence over invalid, then focused.
#[derive(Clone, Copy, Debug, Default)]
pub struct TextFieldColors {
    pub unfocused: TextFieldColorOverrides,
    pub focused: TextFieldColorOverrides,
    pub disabled: TextFieldColorOverrides,
    pub invalid: TextFieldColorOverrides,
    pub hovered_indicator: Option<Hsla>,
}

/// Geometry reported after the retained Base editor lays out the field.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextFieldLayout {
    /// Bounds of the input content area in the window.
    pub editor_bounds: Bounds<Pixels>,
    /// Bounds reported by Base for the laid-out text viewport, when available.
    pub text_bounds: Option<Bounds<Pixels>>,
    /// Number of wrapped visual lines, independent of row limits and scrolling.
    pub line_count: usize,
}

type OnTextLayout = Box<dyn FnOnce(TextFieldLayout, &mut Window, &mut App)>;
type DropdownRequest = Rc<dyn Fn(bool, &mut Window, &mut App)>;

struct FieldDescription {
    text: SharedString,
    content: Option<AnyElement>,
}

/// A Material presentation for a caller-owned Base text editing entity.
///
/// Keep the entity across renders. It owns selection, undo, composition, focus,
/// and text. Subscribe to [`super::TextFieldEvent`] on the entity to observe user
/// edits. Set disabled and read-only on the entity before rendering.
#[derive(IntoElement)]
pub struct TextField {
    id: ElementId,
    label: SharedString,
    editor: Editor,
    variant: TextFieldVariant,
    label_content: Option<AnyElement>,
    label_position: Option<TextFieldLabelPosition>,
    label_minimized: bool,
    expanded_alignment: TextFieldLabelAlignment,
    minimized_alignment: TextFieldLabelAlignment,
    placeholder: Option<(SharedString, AnyElement)>,
    supporting: Option<FieldDescription>,
    error: Option<SharedString>,
    invalid: bool,
    required: bool,
    character_count: Option<usize>,
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
    prefix: Option<AnyElement>,
    suffix: Option<AnyElement>,
    dropdown_editor: bool,
    expanded: Option<bool>,
    dropdown_request: Option<DropdownRequest>,
    font_scale: f32,
    text_style: Option<TypeStyle>,
    shape: Option<CornerShape>,
    content_padding: Option<Edges<Pixels>>,
    colors: TextFieldColors,
    on_text_layout: Option<OnTextLayout>,
    interaction_source: Option<Entity<InteractionSource>>,
}

#[derive(Clone)]
enum Editor {
    Single(Entity<TextFieldState>),
    Multiple(Entity<TextAreaState>),
    Secure(Entity<TextFieldState>),
}

impl Editor {
    fn layout(&self, cx: &App) -> TextFieldLayout {
        fn snapshot(state: &InputBaseState<impl InputModeKind>) -> TextFieldLayout {
            TextFieldLayout {
                editor_bounds: state.input_bounds(),
                text_bounds: state.text_bounds(),
                line_count: state.display_map().wrap_row_count(),
            }
        }
        match self {
            Self::Single(state) | Self::Secure(state) => snapshot(state.read(cx)),
            Self::Multiple(state) => snapshot(state.read(cx)),
        }
    }
}

impl TextField {
    /// Renders a single-line editor.
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<TextFieldState>,
    ) -> Self {
        Self::with_editor(id, label, Editor::Single(state.clone()))
    }

    /// Renders a multi-line editor using the row policy configured in its state.
    pub fn multiline(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<TextAreaState>,
    ) -> Self {
        Self::with_editor(id, label, Editor::Multiple(state.clone()))
    }

    /// Always masks the editor and suppresses its value from accessibility.
    /// Use a dedicated entity initialized with `TextFieldState::masked(true)`
    /// so the text is protected even before this field is first rendered.
    pub fn secure(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<TextFieldState>,
    ) -> Self {
        Self::with_editor(id, label, Editor::Secure(state.clone()))
    }

    fn with_editor(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        editor: Editor,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            editor,
            variant: TextFieldVariant::Filled,
            label_content: None,
            label_position: None,
            label_minimized: false,
            expanded_alignment: TextFieldLabelAlignment::Start,
            minimized_alignment: TextFieldLabelAlignment::Start,
            placeholder: None,
            supporting: None,
            error: None,
            invalid: false,
            required: false,
            character_count: None,
            leading: None,
            trailing: None,
            prefix: None,
            suffix: None,
            dropdown_editor: false,
            expanded: None,
            dropdown_request: None,
            font_scale: 1.,
            text_style: None,
            shape: None,
            content_padding: None,
            colors: TextFieldColors::default(),
            on_text_layout: None,
            interaction_source: None,
        }
    }

    pub fn variant(mut self, variant: TextFieldVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Applies the caller's user text scale to the editor, label, and supporting text.
    pub fn font_scale(mut self, scale: f32) -> Self {
        self.font_scale = scale;
        self
    }

    /// Overrides the Material type metrics for input, hint, and affixes before applying font scale.
    pub fn text_style(mut self, style: TypeStyle) -> Self {
        self.text_style = Some(style);
        self
    }

    /// Overrides the container's logical corner shape. Full uses the laid-out bounds.
    pub fn shape(mut self, shape: CornerShape) -> Self {
        self.shape = Some(shape);
        self
    }

    /// Sets physical editor insets. An Inside label reserves additional height above the editor.
    pub fn content_padding(mut self, padding: Edges<Pixels>) -> Self {
        self.content_padding = Some(padding);
        self
    }

    /// Applies per-state color overrides while preserving theme defaults for unset colors.
    pub fn colors(mut self, colors: TextFieldColors) -> Self {
        self.colors = colors;
        self
    }

    /// Called after paint when the editor's reported layout changes. Subscribe to the Base entity
    /// for text changes that do not affect layout.
    pub fn on_text_layout(
        mut self,
        listener: impl FnOnce(TextFieldLayout, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_text_layout = Some(Box::new(listener));
        self
    }

    /// Observes hover, keyboard-visible focus, and disabled state through a shared source.
    /// Editing and selection stay with the Base entity.
    pub fn interaction_source(mut self, source: &Entity<InteractionSource>) -> Self {
        self.interaction_source = Some(source.clone());
        self
    }

    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting = Some(FieldDescription {
            text: text.into(),
            content: None,
        });
        self
    }

    /// Replaces the visible label. The constructor label remains the accessible name.
    pub fn label_content(mut self, element: impl IntoElement) -> Self {
        self.label_content = Some(element.into_any_element());
        self
    }

    pub fn label_position(mut self, position: TextFieldLabelPosition) -> Self {
        self.label_position = Some(position);
        self
    }

    /// Keeps an Inside or Cutout label minimized so affixes and hints can remain visible.
    pub fn label_always_minimized(mut self, minimized: bool) -> Self {
        self.label_minimized = minimized;
        self
    }

    /// Configures the alignment in each label state. Above uses `minimized`.
    pub fn label_alignment(
        mut self,
        expanded: TextFieldLabelAlignment,
        minimized: TextFieldLabelAlignment,
    ) -> Self {
        self.expanded_alignment = expanded;
        self.minimized_alignment = minimized;
        self
    }

    /// Replaces the Base placeholder's presentation. `hint` is the accessible placeholder.
    pub fn placeholder_content(
        mut self,
        hint: impl Into<SharedString>,
        element: impl IntoElement,
    ) -> Self {
        self.placeholder = Some((hint.into(), element.into_any_element()));
        self
    }

    /// Supplies custom supporting content and the field's plain-text accessible description.
    pub fn supporting_content(
        mut self,
        description: impl Into<SharedString>,
        element: impl IntoElement,
    ) -> Self {
        self.supporting = Some(FieldDescription {
            text: description.into(),
            content: Some(element.into_any_element()),
        });
        self
    }

    /// Displays an error message in place of the supporting text.
    pub fn error(mut self, message: impl Into<SharedString>) -> Self {
        self.error = Some(message.into());
        self.invalid = true;
        self
    }

    /// Sets the invalid visual and accessibility state independently of the supporting content.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Displays a required mark and reports the required state to assistive technology.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Shows a Unicode scalar count and the supplied limit. Base still owns validation.
    /// The count is omitted for secure or masked fields.
    pub fn character_count(mut self, limit: usize) -> Self {
        self.character_count = Some(limit);
        self
    }

    /// Places an adornment in the 48 dp leading icon slot.
    pub fn leading(mut self, element: impl IntoElement) -> Self {
        self.leading = Some(element.into_any_element());
        self
    }

    /// Places an addon at the logical end of the editing region.
    pub fn trailing(mut self, element: impl IntoElement) -> Self {
        self.trailing = Some(element.into_any_element());
        self
    }

    /// Places an element before the value while the label is minimized.
    pub fn prefix(mut self, element: impl IntoElement) -> Self {
        self.prefix = Some(element.into_any_element());
        self
    }

    /// Places an element after the value while the label is minimized.
    pub fn suffix(mut self, element: impl IntoElement) -> Self {
        self.suffix = Some(element.into_any_element());
        self
    }

    pub(super) fn dropdown(mut self, open: bool, request: Option<DropdownRequest>) -> Self {
        self.dropdown_editor = true;
        self.expanded = request.is_some().then_some(open);
        self.dropdown_request = request;
        self
    }
}
