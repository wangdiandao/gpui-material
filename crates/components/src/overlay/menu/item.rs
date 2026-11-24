use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, SharedString,
    Styled, Window, div,
};
use gpui_base::TestSupportExt;
use gpui_material_theme::{CornerShape, TypeRole, TypographyEmphasis};

use crate::menu_appearance::{MenuItemStyle, MenuSurfaceStyle, Paint};
use crate::{Icon, IconName, SemanticTone};

pub(super) type Action = Rc<dyn Fn(&mut Window, &mut App)>;
pub(super) type Change = Rc<dyn Fn(bool, &mut Window, &mut App)>;
pub(super) type ItemContent = Rc<dyn Fn() -> AnyElement>;
pub(super) type ComposedTrigger = Box<dyn FnOnce(bool, Action) -> AnyElement>;
pub(super) type LazyItems = Box<dyn FnOnce(&mut Window, &mut App) -> Vec<MenuItem>>;

pub(super) enum MenuItems {
    Eager(Vec<MenuItem>),
    Lazy(LazyItems),
}

impl MenuItems {
    pub(super) fn resolve(self, window: &mut Window, cx: &mut App) -> Vec<MenuItem> {
        match self {
            Self::Eager(items) => items,
            Self::Lazy(build) => build(window, cx),
        }
    }
}

/// An item in a Material menu. Callers retain command and selection state.
pub struct MenuItem {
    pub(super) id: ElementId,
    pub(super) label: SharedString,
    pub(super) disabled: bool,
    pub(super) kind: Kind,
    pub(super) slots: ItemSlots,
    pub(super) on_checked_change: Option<Change>,
    pub(super) tone: SemanticTone,
    pub(super) style: MenuItemStyle,
}

pub(super) enum Kind {
    Action(Option<Action>),
    Check(bool, Option<Action>),
    Select(bool, Option<Action>),
    Heading,
    Divider,
    Section {
        rows: Vec<MenuItem>,
        style: MenuSurfaceStyle,
        inactive_shape: Option<CornerShape>,
    },
    Submenu(Vec<MenuItem>),
}

#[derive(Clone, Default)]
pub(super) struct ItemSlots {
    pub(super) content: Option<ItemContent>,
    pub(super) leading: Option<ItemContent>,
    pub(super) supporting: Option<ItemContent>,
    pub(super) trailing: Option<ItemContent>,
    pub(super) shortcut: Option<SharedString>,
    pub(super) selection_icon: Option<ItemContent>,
}

impl ItemSlots {
    pub(super) fn label(&self, label: SharedString) -> AnyElement {
        self.content
            .as_ref()
            .map_or_else(|| label.into_any_element(), |content| content())
    }

    pub(super) fn children(
        &self,
        label: SharedString,
        marked: bool,
        submenu: bool,
        paint: Paint<'_>,
        (supporting_color, shortcut_color): (Hsla, Hsla),
        active: bool,
    ) -> Vec<AnyElement> {
        let motion = paint.motion;
        let icon_size = motion.px(18.);
        let leading = if marked {
            Some(self.selection_icon.as_ref().map_or_else(
                || {
                    Icon::new(IconName::Check)
                        .size(icon_size)
                        .into_any_element()
                },
                |icon| icon(),
            ))
        } else {
            self.leading.as_ref().map(|icon| icon())
        };
        let supporting_scale = paint.theme.typography.text_scale();
        let body = div()
            .flex_1()
            .min_w_0()
            .overflow_hidden()
            .child(div().truncate().child(self.label(label)))
            .children(self.supporting.as_ref().map(|supporting| {
                div()
                    .text_size(motion.px(12. * supporting_scale))
                    .line_height(motion.px(16. * supporting_scale))
                    .text_color(supporting_color)
                    .child(supporting())
            }));
        let mut children = vec![
            div()
                .min_w(icon_size)
                .flex_none()
                .children(leading)
                .into_any_element(),
            body.into_any_element(),
        ];
        if let Some(trailing) = self.trailing.as_ref() {
            children.push(trailing());
        } else if let Some(shortcut) = self.shortcut.as_ref().filter(|text| !text.is_empty()) {
            let label_small = paint
                .theme
                .typography
                .scale_style(TypeRole::LabelSmall.style(TypographyEmphasis::Baseline));
            let hint = div()
                .flex_none()
                .text_size(motion.px(label_small.size))
                .line_height(motion.px(label_small.line_height))
                .text_color(shortcut_color)
                .child(shortcut.clone());
            children.push(if active {
                hint.id("shortcut").test_support().into_any_element()
            } else {
                hint.into_any_element()
            });
        }
        children.extend(submenu.then(|| {
            Icon::new(IconName::ChevronRight)
                .size(icon_size)
                .mirror_in_rtl(true)
                .into_any_element()
        }));
        children
    }
}

impl MenuItem {
    /// Creates a standard action menu item with the given element ID and label.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            disabled: false,
            kind: Kind::Action(None),
            slots: ItemSlots::default(),
            on_checked_change: None,
            tone: SemanticTone::Default,
            style: MenuItemStyle::default(),
        }
    }

    /// Creates a non-interactive section heading row that is skipped by keyboard navigation.
    pub fn group(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            kind: Kind::Heading,
            ..Self::new(id, title)
        }
    }

    /// Creates a non-interactive horizontal divider row inside a menu that is
    /// skipped by keyboard navigation.
    pub fn divider(id: impl Into<ElementId>) -> Self {
        Self {
            kind: Kind::Divider,
            ..Self::new(id, "")
        }
    }

    /// Creates a surfaced group with rows in the owning menu's navigation order.
    pub fn section(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        rows: impl IntoIterator<Item = MenuItem>,
    ) -> Self {
        Self {
            kind: Kind::Section {
                rows: rows.into_iter().collect(),
                style: MenuSurfaceStyle::default(),
                inactive_shape: None,
            },
            ..Self::new(id, name)
        }
    }

    /// Sets the styled section's surface appearance.
    pub fn group_appearance(mut self, appearance: MenuSurfaceStyle) -> Self {
        if let Kind::Section { style, .. } = &mut self.kind {
            *style = appearance;
        }
        self
    }

    /// Sets the section shape used after the pointer leaves the section.
    pub fn inactive_shape(mut self, shape: CornerShape) -> Self {
        if let Kind::Section { inactive_shape, .. } = &mut self.kind {
            *inactive_shape = Some(shape);
        }
        self
    }

    /// Configures the appearance of an action, checked or selected row.
    pub fn appearance(mut self, style: MenuItemStyle) -> Self {
        self.style = style;
        self
    }

    /// Applies a Material 3 semantic tone to this row's foreground and selected roles.
    /// Explicit [`MenuItemStyle`] overrides and disabled styling still take precedence.
    pub fn tone(mut self, tone: SemanticTone) -> Self {
        self.tone = tone;
        self
    }

    /// Shorthand for `.tone(SemanticTone::Danger)` on destructive menu commands.
    pub fn destructive(self) -> Self {
        self.tone(SemanticTone::Danger)
    }

    /// Renders a desktop shortcut hint in the trailing slot using `LabelSmall`
    /// and `OnSurfaceVariant` (or disabled color when disabled), unless an
    /// explicit `.trailing(...)` overrides it.
    pub fn shortcut(mut self, text: impl Into<SharedString>) -> Self {
        self.slots.shortcut = Some(text.into());
        self
    }

    /// Turns this row into a submenu trigger containing the given child items.
    pub fn submenu(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.kind = Kind::Submenu(items.into_iter().collect());
        self
    }

    /// Marks the item as disabled, preventing activation and applying disabled styling.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Turns this row into a checkable menu item with the given caller-owned checked state.
    pub fn checked(mut self, checked: bool) -> Self {
        self.kind = Kind::Check(checked, self.action());
        self
    }

    /// Turns this row into a selectable menu item with the given caller-owned selected state.
    pub fn selected(mut self, selected: bool) -> Self {
        self.kind = Kind::Select(selected, self.action());
        self
    }

    /// Replaces the visible label while retaining the name passed to `new`.
    /// Supply a repeatable factory so the content also appears during exit.
    pub fn content<E: IntoElement>(mut self, build: impl Fn() -> E + 'static) -> Self {
        self.slots.content = Some(Rc::new(move || build().into_any_element()));
        self
    }

    /// Supplies a decorative element at the logical start of an unmarked row.
    pub fn leading<E: IntoElement>(mut self, build: impl Fn() -> E + 'static) -> Self {
        self.slots.leading = Some(Rc::new(move || build().into_any_element()));
        self
    }

    /// Supplies additional visual content below the label.
    pub fn supporting<E: IntoElement>(mut self, build: impl Fn() -> E + 'static) -> Self {
        self.slots.supporting = Some(Rc::new(move || build().into_any_element()));
        self
    }

    /// Supplies decorative content at the logical end of the row.
    pub fn trailing<E: IntoElement>(mut self, build: impl Fn() -> E + 'static) -> Self {
        self.slots.trailing = Some(Rc::new(move || build().into_any_element()));
        self
    }

    /// Replaces the leading check mark of a checked or selected row.
    pub fn selection_icon<E: IntoElement>(mut self, build: impl Fn() -> E + 'static) -> Self {
        self.slots.selection_icon = Some(Rc::new(move || build().into_any_element()));
        self
    }

    /// Requests the inverse of the supplied checked value on activation.
    /// The checked callback takes precedence over `on_activate` on checked rows.
    pub fn on_checked_change(
        mut self,
        callback: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_checked_change = Some(Rc::new(callback));
        self
    }

    fn action(&self) -> Option<Action> {
        match &self.kind {
            Kind::Action(action) | Kind::Check(_, action) | Kind::Select(_, action) => {
                action.clone()
            }
            _ => None,
        }
    }

    pub(super) fn activation(&self) -> Option<Action> {
        if let Kind::Check(checked, _) = self.kind
            && let Some(callback) = &self.on_checked_change
        {
            let callback = callback.clone();
            return Some(Rc::new(move |window, cx| callback(!checked, window, cx)));
        }
        self.action()
    }

    /// Registers a callback invoked when an action, checked, or selected row is activated.
    pub fn on_activate(mut self, action: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        match &mut self.kind {
            Kind::Action(handler) | Kind::Check(_, handler) | Kind::Select(_, handler) => {
                *handler = Some(Rc::new(action));
            }
            _ => {}
        }
        self
    }
}
