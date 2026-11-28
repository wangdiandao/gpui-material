//! Material 3 Expressive modal command palette (`Ctrl+K` / `Cmd+K` style).

mod render;
mod state;

use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, Entity, Hsla, IntoElement, Pixels, SharedString, Window, px,
};
use gpui_material_components::{DialogCloseReason, DialogProperties, IconName};
use gpui_material_theme::CornerShape;

pub use state::{CommandPaletteEvent, CommandPaletteState};

type FilterPredicate = Rc<dyn Fn(&CommandItem, &str) -> bool>;
type SelectHandler = Rc<dyn Fn(&CommandItem, &mut Window, &mut App)>;
type DismissRequestHandler = Rc<dyn Fn(DialogCloseReason, &mut Window, &mut App) -> bool>;
type ClosedHandler = Rc<dyn Fn(DialogCloseReason, &mut Window, &mut App)>;

/// One typed command entry rendered in a [`CommandPalette`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandItem {
    id: SharedString,
    label: SharedString,
    supporting: Option<SharedString>,
    group: Option<SharedString>,
    icon: Option<IconName>,
    shortcut: Option<SharedString>,
    keywords: Vec<SharedString>,
    disabled: bool,
}

impl CommandItem {
    /// Creates an enabled command with a stable `id` and primary display `label`.
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            supporting: None,
            group: None,
            icon: None,
            shortcut: None,
            keywords: Vec::new(),
            disabled: false,
        }
    }

    /// Sets optional secondary description text rendered beneath the command label.
    pub fn supporting(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting = Some(text.into());
        self
    }

    /// Assigns this command to a visual group heading.
    pub fn group(mut self, heading: impl Into<SharedString>) -> Self {
        self.group = Some(heading.into());
        self
    }

    /// Sets an optional leading bundled Material Symbol icon.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets an optional keyboard shortcut hint badge (for example `"Ctrl+K"` or `"⌘P"`).
    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Appends one extra search keyword matched by the default command filter.
    pub fn keyword(mut self, keyword: impl Into<SharedString>) -> Self {
        self.keywords.push(keyword.into());
        self
    }

    /// Appends multiple extra search keywords matched by the default command filter.
    pub fn keywords<I, S>(mut self, keywords: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.keywords.extend(keywords.into_iter().map(Into::into));
        self
    }

    /// Marks this command as unavailable (greyed out, non-clickable, skipped by keyboard navigation).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Returns the stable command identifier.
    pub fn id(&self) -> &SharedString {
        &self.id
    }

    /// Returns the primary display label.
    pub fn label(&self) -> &SharedString {
        &self.label
    }

    /// Returns the optional supporting description.
    pub fn supporting_text(&self) -> Option<&SharedString> {
        self.supporting.as_ref()
    }

    /// Returns the optional group heading assigned to this command.
    pub fn group_label(&self) -> Option<&SharedString> {
        self.group.as_ref()
    }

    /// Returns the optional leading [`IconName`].
    pub fn icon_name(&self) -> Option<IconName> {
        self.icon
    }

    /// Returns the optional shortcut hint string.
    pub fn shortcut_label(&self) -> Option<&SharedString> {
        self.shortcut.as_ref()
    }

    /// Returns the extra search keywords associated with this command.
    pub fn keywords_slice(&self) -> &[SharedString] {
        &self.keywords
    }

    /// Returns `true` when this command is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

/// A named group of [`CommandItem`] entries for [`CommandPalette::grouped`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandGroup {
    heading: SharedString,
    items: Vec<CommandItem>,
}

impl CommandGroup {
    /// Creates a command group whose `heading` is applied to items without an explicit group.
    pub fn new(
        heading: impl Into<SharedString>,
        items: impl IntoIterator<Item = CommandItem>,
    ) -> Self {
        let heading = heading.into();
        let items = items
            .into_iter()
            .map(|item| {
                if item.group.is_some() {
                    item
                } else {
                    item.group(heading.clone())
                }
            })
            .collect();
        Self { heading, items }
    }

    /// Returns the group heading label.
    pub fn heading(&self) -> &SharedString {
        &self.heading
    }

    /// Returns the ordered commands in this group.
    pub fn items(&self) -> &[CommandItem] {
        &self.items
    }

    /// Consumes the group and returns its flattened [`CommandItem`] list.
    pub fn into_items(self) -> Vec<CommandItem> {
        self.items
    }
}

/// Per-instance color overrides for [`CommandPalette`]. Omitted fields use the window theme.
#[derive(Clone, Copy, Debug, Default)]
pub struct CommandPaletteColors {
    /// Background color for the modal surface (defaults to `SurfaceContainerHigh`).
    pub container: Option<Hsla>,
    /// Divider color between the search header, results list, and optional footer.
    pub divider: Option<Hsla>,
    /// Foreground color for group headings (defaults to `OnSurfaceVariant`).
    pub group_heading: Option<Hsla>,
    /// Background fill for the currently highlighted command row (defaults to `SecondaryContainer`).
    pub active_item_container: Option<Hsla>,
    /// Foreground color for the currently highlighted command row (defaults to `OnSecondaryContainer`).
    pub active_item_content: Option<Hsla>,
    /// Background fill for shortcut hint badges (defaults to `SurfaceContainerHighest`).
    pub shortcut_container: Option<Hsla>,
    /// Foreground color for shortcut hint badges (defaults to `OnSurfaceVariant`).
    pub shortcut_content: Option<Hsla>,
    /// Optional override for the modal backdrop scrim tint.
    pub scrim: Option<Hsla>,
}

/// Material 3 modal command palette built on `Dialog` and `TextField`.
///
/// Retain an [`Entity<CommandPaletteState>`] in your view, mount `CommandPalette` in the
/// view tree, and call [`CommandPaletteState::open`] or [`CommandPaletteState::toggle`]
/// from a button or `Ctrl+K` / `Cmd+K` keybinding.
#[derive(IntoElement)]
pub struct CommandPalette {
    id: ElementId,
    state: Entity<CommandPaletteState>,
    items: Vec<CommandItem>,
    accessible_label: Option<SharedString>,
    placeholder: Option<SharedString>,
    empty_label: Option<SharedString>,
    close_label: Option<SharedString>,
    filter: Option<FilterPredicate>,
    close_on_select: bool,
    width: Pixels,
    max_results_height: Pixels,
    layer: usize,
    properties: DialogProperties,
    shape: Option<CornerShape>,
    colors: CommandPaletteColors,
    scrim_opacity: f32,
    footer: Option<AnyElement>,
    on_select: Option<SelectHandler>,
    on_dismiss_request: Option<DismissRequestHandler>,
    on_closed: Option<ClosedHandler>,
}

impl CommandPalette {
    /// Creates a modal command palette bound to `state` with the supplied ordered `items`.
    pub fn new(
        id: impl Into<ElementId>,
        state: &Entity<CommandPaletteState>,
        items: impl IntoIterator<Item = CommandItem>,
    ) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            items: items.into_iter().collect(),
            accessible_label: None,
            placeholder: None,
            empty_label: None,
            close_label: None,
            filter: None,
            close_on_select: true,
            width: px(580.),
            max_results_height: px(360.),
            layer: 0,
            properties: DialogProperties::default(),
            shape: None,
            colors: CommandPaletteColors::default(),
            scrim_opacity: 0.32,
            footer: None,
            on_select: None,
            on_dismiss_request: None,
            on_closed: None,
        }
    }

    /// Creates a modal command palette from grouped [`CommandGroup`] collections.
    pub fn grouped(
        id: impl Into<ElementId>,
        state: &Entity<CommandPaletteState>,
        groups: impl IntoIterator<Item = CommandGroup>,
    ) -> Self {
        let items = groups.into_iter().flat_map(CommandGroup::into_items);
        Self::new(id, state, items)
    }

    /// Overrides the accessible dialog name (defaults to `"Command palette"`).
    pub fn accessible_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }

    /// Overrides the search field placeholder (defaults to
    /// [`gpui_material_theme::MaterialStrings::search_prompt`]).
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Overrides the empty-results message (defaults to
    /// [`gpui_material_theme::MaterialStrings::no_results`]).
    pub fn empty_label(mut self, label: impl Into<SharedString>) -> Self {
        self.empty_label = Some(label.into());
        self
    }

    /// Overrides the close button's accessible label (defaults to
    /// [`gpui_material_theme::MaterialStrings::close`]).
    pub fn close_label(mut self, label: impl Into<SharedString>) -> Self {
        self.close_label = Some(label.into());
        self
    }

    /// Supplies a custom filtering predicate `(item, query) -> bool` replacing the built-in
    /// case-insensitive multi-token substring and keyword matcher.
    pub fn filter(mut self, predicate: impl Fn(&CommandItem, &str) -> bool + 'static) -> Self {
        self.filter = Some(Rc::new(predicate));
        self
    }

    /// Sets whether executing a command automatically closes the palette (defaults to `true`).
    pub fn close_on_select(mut self, close: bool) -> Self {
        self.close_on_select = close;
        self
    }

    /// Sets the maximum width of the command palette surface in logical pixels (defaults to `580px`).
    pub fn width(mut self, width: Pixels) -> Self {
        if f32::from(width).is_finite() {
            self.width = width.max(px(280.));
        }
        self
    }

    /// Sets the maximum scrollable height of the results region in logical pixels (defaults to `360px`).
    pub fn max_results_height(mut self, height: Pixels) -> Self {
        if f32::from(height).is_finite() {
            self.max_results_height = height.max(px(120.));
        }
        self
    }

    /// Sets the modal z-layer forwarded to the underlying `Dialog`.
    pub fn layer(mut self, layer: usize) -> Self {
        self.layer = layer;
        self
    }

    /// Configures Escape and outside-click dismissal behavior.
    pub fn properties(mut self, properties: DialogProperties) -> Self {
        self.properties = properties;
        self
    }

    /// Overrides the corner shape of the command palette surface (defaults to `CornerRole::ExtraLarge`).
    pub fn shape(mut self, shape: CornerShape) -> Self {
        self.shape = Some(shape);
        self
    }

    /// Overrides container, divider, heading, active row, shortcut badge, and scrim colors.
    pub fn colors(mut self, colors: CommandPaletteColors) -> Self {
        self.colors = colors;
        self
    }

    /// Sets backdrop scrim opacity clamped to `0.0..=1.0` (defaults to `0.32`).
    pub fn scrim_opacity(mut self, opacity: f32) -> Self {
        self.scrim_opacity = if opacity.is_nan() {
            0.32
        } else {
            opacity.clamp(0., 1.)
        };
        self
    }

    /// Renders optional footer content beneath the results list (for example keyboard hints).
    pub fn footer(mut self, element: impl IntoElement) -> Self {
        self.footer = Some(element.into_any_element());
        self
    }

    /// Invoked when an enabled command is executed via `Enter` or pointer click.
    pub fn on_select(
        mut self,
        handler: impl Fn(&CommandItem, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    /// Handles Escape and outside-click dismissal requests. Return `false` to keep the palette open.
    pub fn on_dismiss_request(
        mut self,
        handler: impl Fn(DialogCloseReason, &mut Window, &mut App) -> bool + 'static,
    ) -> Self {
        self.on_dismiss_request = Some(Rc::new(handler));
        self
    }

    /// Invoked once when the palette closes due to user dismissal or selection.
    pub fn on_closed(
        mut self,
        handler: impl Fn(DialogCloseReason, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_closed = Some(Rc::new(handler));
        self
    }
}
