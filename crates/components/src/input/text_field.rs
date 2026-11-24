use std::rc::Rc;

use gpui::{
    AnyElement, App, Bounds, ElementId, Entity, Hsla, IntoElement, Pixels, SharedString, Window, px,
};
use gpui_base::input::{InputBaseState, InputModeKind};
use gpui_material_foundation::InteractionSource;
use gpui_material_theme::{CornerShape, LogicalEdges, TypeStyle};

use super::{TextAreaState, TextFieldState};

mod controls;
mod editor;
mod label;
mod metrics;
mod outline;
mod palette;
mod render;

/// Material text field container treatment. The default is Filled.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextFieldVariant {
    /// Filled container with top rounded corners and a bottom active indicator (default).
    #[default]
    Filled,
    /// Transparent container framed by a rounded perimeter outline with a label cutout.
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
    /// Aligned to the logical start edge (left in LTR, right in RTL; default).
    #[default]
    Start,
    /// Horizontally centered.
    Center,
    /// Aligned to the logical end edge (right in LTR, left in RTL).
    End,
}

/// Visibility mode for [`TextField::secure`] editors.
///
/// Desktop platforms treat a revealed password as ordinary plain text. `Hidden`
/// masks the editor with Base's fixed bullet (`•`), disables copy/cut, and omits
/// the accessible value, accessible `SetValue`, and character count. `Visible`
/// renders and behaves as plain text.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextFieldObfuscationMode {
    /// Masked with bullets (`•`), copy/cut disabled, and value omitted from accessibility (default).
    #[default]
    Hidden,
    /// Rendered and exposed as ordinary plain text.
    Visible,
}

/// Optional colors for a single text field state. An unset role uses the window theme.
#[derive(Clone, Copy, Debug, Default)]
pub struct TextFieldColorOverrides {
    /// Field container background color.
    pub container: Option<Hsla>,
    /// Input text color.
    pub text: Option<Hsla>,
    /// Floating or resting label color.
    pub label: Option<Hsla>,
    /// Placeholder hint text color.
    pub placeholder: Option<Hsla>,
    /// Leading adornment color.
    pub leading: Option<Hsla>,
    /// Trailing adornment color.
    pub trailing: Option<Hsla>,
    /// Prefix text/element color.
    pub prefix: Option<Hsla>,
    /// Suffix text/element color.
    pub suffix: Option<Hsla>,
    /// Supporting or error text color below the field.
    pub supporting: Option<Hsla>,
    /// Active indicator line (Filled) or perimeter outline (Outlined) color.
    pub indicator: Option<Hsla>,
    /// Text cursor (caret) color.
    pub caret: Option<Hsla>,
    /// Text selection highlight color.
    pub selection: Option<Hsla>,
}

/// Per-instance colors. Disabled takes precedence over invalid, then focused.
#[derive(Clone, Copy, Debug, Default)]
pub struct TextFieldColors {
    /// Colors when enabled, valid, and unfocused.
    pub unfocused: TextFieldColorOverrides,
    /// Colors when enabled, valid, and focused.
    pub focused: TextFieldColorOverrides,
    /// Colors when disabled.
    pub disabled: TextFieldColorOverrides,
    /// Colors when enabled and marked invalid/error.
    pub invalid: TextFieldColorOverrides,
    /// Indicator or outline color when hovered while unfocused and valid.
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
type VisibilityCallback = Rc<dyn Fn(bool, &mut Window, &mut App)>;

struct VisibilityToggle {
    visible: bool,
    on_toggle: VisibilityCallback,
}

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
    obfuscation_mode: TextFieldObfuscationMode,
    show_label: bool,
    label_content: Option<AnyElement>,
    label_position: Option<TextFieldLabelPosition>,
    label_minimized: bool,
    expanded_alignment: TextFieldLabelAlignment,
    minimized_alignment: TextFieldLabelAlignment,
    placeholder: Option<(SharedString, AnyElement)>,
    supporting: Option<FieldDescription>,
    error: Option<SharedString>,
    disabled: bool,
    invalid: bool,
    required: bool,
    character_count: Option<usize>,
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
    clearable: bool,
    clear_label: Option<SharedString>,
    visibility_toggle: Option<VisibilityToggle>,
    visibility_labels: Option<(SharedString, SharedString)>,
    prefix: Option<AnyElement>,
    suffix: Option<AnyElement>,
    dropdown_editor: bool,
    expanded: Option<bool>,
    dropdown_request: Option<DropdownRequest>,
    font_scale: f32,
    text_style: Option<TypeStyle>,
    shape: Option<CornerShape>,
    content_padding: Option<LogicalEdges>,
    min_height: Option<Pixels>,
    indicator_width: (Pixels, Pixels),
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

    fn replace_all(&self, value: SharedString, window: &mut Window, cx: &mut App) {
        fn apply<M: InputModeKind>(
            state: &Entity<InputBaseState<M>>,
            value: SharedString,
            window: &mut Window,
            cx: &mut App,
        ) {
            state.update(cx, |state, cx| {
                if state.is_editable() {
                    state.replace_all(value, window, cx);
                }
            });
        }
        match self {
            Self::Single(state) | Self::Secure(state) => apply(state, value, window, cx),
            Self::Multiple(state) => apply(state, value, window, cx),
        }
    }

    fn set_disabled(&self, disabled: bool, cx: &mut App) {
        match self {
            Self::Single(state) | Self::Secure(state) => {
                state.update(cx, |state, cx| state.set_disabled(disabled, cx));
            }
            Self::Multiple(state) => {
                state.update(cx, |state, cx| state.set_disabled(disabled, cx));
            }
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

    /// Renders a Filled single-line editor.
    pub fn filled(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<TextFieldState>,
    ) -> Self {
        Self::new(id, label, state).variant(TextFieldVariant::Filled)
    }

    /// Renders an Outlined single-line editor.
    pub fn outlined(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<TextFieldState>,
    ) -> Self {
        Self::new(id, label, state).variant(TextFieldVariant::Outlined)
    }

    /// Renders a multi-line editor using the row policy configured in its state.
    pub fn multiline(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        state: &Entity<TextAreaState>,
    ) -> Self {
        Self::with_editor(id, label, Editor::Multiple(state.clone()))
    }

    /// Renders a secure password editor. Defaults to [`TextFieldObfuscationMode::Hidden`],
    /// masking the value with `•`, blocking copy/cut, and suppressing the accessible value
    /// and character count. Call [`Self::obfuscation_mode`] with
    /// [`TextFieldObfuscationMode::Visible`] to reveal plain text.
    ///
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
            obfuscation_mode: TextFieldObfuscationMode::Hidden,
            show_label: true,
            label_content: None,
            label_position: None,
            label_minimized: false,
            expanded_alignment: TextFieldLabelAlignment::Start,
            minimized_alignment: TextFieldLabelAlignment::Start,
            placeholder: None,
            supporting: None,
            error: None,
            disabled: false,
            invalid: false,
            required: false,
            character_count: None,
            leading: None,
            trailing: None,
            clearable: false,
            clear_label: None,
            visibility_toggle: None,
            visibility_labels: None,
            prefix: None,
            suffix: None,
            dropdown_editor: false,
            expanded: None,
            dropdown_request: None,
            font_scale: 1.,
            text_style: None,
            shape: None,
            content_padding: None,
            min_height: None,
            indicator_width: (px(1.), px(2.)),
            colors: TextFieldColors::default(),
            on_text_layout: None,
            interaction_source: None,
        }
    }

    /// Sets the text field container style (`Filled` or `Outlined`).
    pub fn variant(mut self, variant: TextFieldVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets whether a [`TextField::secure`] editor is masked (`Hidden`) or revealed (`Visible`).
    /// Ignored for non-secure fields.
    pub fn obfuscation_mode(mut self, mode: TextFieldObfuscationMode) -> Self {
        self.obfuscation_mode = mode;
        self
    }

    /// Displays a trailing clear button when the editor is editable and non-empty.
    ///
    /// Activating the button clears the Base state via `replace_all("")` and keeps focus on the
    /// editor. When combined with `.visibility_toggle(...)` or `.trailing(...)`, built-in controls
    /// render in logical order before custom trailing content.
    pub fn clearable(mut self, clearable: bool) -> Self {
        self.clearable = clearable;
        self
    }

    /// Overrides the accessible label of the `.clearable(true)` button (default `"Clear"`).
    pub fn clear_label(mut self, label: impl Into<SharedString>) -> Self {
        self.clear_label = Some(label.into());
        self
    }

    /// Adds a controlled password reveal toggle on [`TextField::secure`] editors and syncs
    /// [`Self::obfuscation_mode`] automatically (`Visible` when `visible` is `true`, `Hidden`
    /// otherwise). Ignored for non-secure fields.
    pub fn visibility_toggle(
        mut self,
        visible: bool,
        on_toggle: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.obfuscation_mode = if visible {
            TextFieldObfuscationMode::Visible
        } else {
            TextFieldObfuscationMode::Hidden
        };
        self.visibility_toggle = Some(VisibilityToggle {
            visible,
            on_toggle: Rc::new(on_toggle),
        });
        self
    }

    /// Overrides the accessible labels of the `.visibility_toggle(...)` button
    /// (defaults to `"Show password"` and `"Hide password"`).
    pub fn visibility_labels(
        mut self,
        show: impl Into<SharedString>,
        hide: impl Into<SharedString>,
    ) -> Self {
        self.visibility_labels = Some((show.into(), hide.into()));
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

    /// Sets logical editor insets, mirrored automatically in RTL.
    /// An Inside label reserves additional height above the editor.
    pub fn content_padding(mut self, padding: LogicalEdges) -> Self {
        self.content_padding = Some(padding);
        self
    }

    /// Disables the text field at the component level.
    ///
    /// Combined via logical OR with the underlying Base editor state's disabled flag.
    /// When disabled, pointer and keyboard interaction are suppressed and active focus is blurred.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Overrides the container's minimum height in logical pixels.
    ///
    /// Negative or non-finite values are ignored and keep the density-aware default (`52 dp`
    /// Compact, `56 dp` Standard, `60 dp` Comfortable). The container still expands to fit its
    /// text line height, vertical padding, and any inside label reserve.
    pub fn min_height(mut self, height: Pixels) -> Self {
        let value = f32::from(height);
        if value.is_finite() && value >= 0. {
            self.min_height = Some(height);
        }
        self
    }

    /// Sets the indicator thickness for unfocused and focused states (defaults to `1 px` and `2 px`).
    ///
    /// Controls the Filled bottom indicator or the Outlined container border. Pass `px(0.)` to
    /// omit the indicator or border cleanly. Negative or non-finite values are ignored.
    pub fn indicator_width(mut self, unfocused: Pixels, focused: Pixels) -> Self {
        let unfocused_value = f32::from(unfocused);
        if unfocused_value.is_finite() && unfocused_value >= 0. {
            self.indicator_width.0 = unfocused;
        }
        let focused_value = f32::from(focused);
        if focused_value.is_finite() && focused_value >= 0. {
            self.indicator_width.1 = focused;
        }
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

    /// Displays helper text below the field and exposes it as the field's accessible description.
    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting = Some(FieldDescription {
            text: text.into(),
            content: None,
        });
        self
    }

    /// Controls whether the visual label is rendered. Defaults to `true`.
    ///
    /// When `false`, the constructor's label remains the accessible name (`aria_label`), vertical
    /// label space is omitted, and placeholders, prefixes, and suffixes stay immediately visible.
    pub fn show_label(mut self, show: bool) -> Self {
        self.show_label = show;
        self
    }

    /// Replaces the visible label. The constructor label remains the accessible name.
    pub fn label_content(mut self, element: impl IntoElement) -> Self {
        self.label_content = Some(element.into_any_element());
        self
    }

    /// Overrides where the minimized label is positioned (`Inside`, `Cutout`, or `Above`).
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
