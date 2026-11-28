use std::{collections::HashMap, f32::consts::FRAC_PI_2, rc::Rc};

use gpui::{
    AnyElement, App, AppContext as _, Axis, ClickEvent, Context, CursorStyle, ElementId, Entity,
    EventEmitter, FocusHandle, Hsla, InteractiveElement, IntoElement, KeyDownEvent, ParentElement,
    Pixels, RenderOnce, ScrollStrategy, SharedString, StatefulInteractiveElement, StyleRefinement,
    Styled, Subscription, UniformListScrollHandle, Window, canvas, div, prelude::FluentBuilder, px,
    radians, transparent_black,
};
use gpui_base::{
    StyledExt as _, TestSupportExt as _, Tree as BaseTree, TreeEntry, TreeEvent as BaseTreeEvent,
    TreeItem as BaseTreeItem, TreeState as BaseTreeState,
    actions::{Confirm, SelectDown, SelectLeft, SelectRight, SelectUp},
};
use gpui_material_components::{
    DisplayScrollbar, DisplayScrollbarFade, DisplayScrollbarStyle, Icon, IconName, Text,
};
use gpui_material_foundation::{
    FocusRingPlacement, InteractionFeedback, color,
    control_feedback::{activate_feedback, feedback_target, ripple_state},
    current_material_strings, current_material_theme,
};
use gpui_material_theme::{ColorRole, CornerRole, CornerShape, Density, Direction, TypeRole};

const DEFAULT_ROW_HEIGHT: f32 = 40.0;
const MIN_ROW_HEIGHT: f32 = 32.0;
const DEFAULT_INDENT: f32 = 20.0;
const DEFAULT_MAX_HEIGHT: f32 = 360.0;
const CHEVRON_SLOT_SIZE: f32 = 20.0;
const CONTAINER_PADDING_Y: f32 = 4.0;

type SelectHandler = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;
type ActivateHandler = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;
type ToggleHandler = Rc<dyn Fn(&SharedString, bool, &mut Window, &mut App)>;
type CustomRowRenderer = Rc<dyn Fn(&TreeRowInfo, &mut Window, &mut App) -> AnyElement>;

/// A Material-owned hierarchical node for [`Tree`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeNode {
    id: SharedString,
    label: SharedString,
    icon: Option<IconName>,
    expanded: bool,
    disabled: bool,
    children: Vec<TreeNode>,
}

impl TreeNode {
    /// Creates a tree node with a stable unique `id` and display `label`.
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            expanded: false,
            disabled: false,
            children: Vec::new(),
        }
    }

    /// Sets an optional leading Material Symbol icon for this node.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets whether this branch node starts expanded.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Disables selection, activation, and expansion changes for this node when `true`.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Appends a single child node.
    pub fn child(mut self, child: TreeNode) -> Self {
        self.children.push(child);
        self
    }

    /// Appends multiple child nodes.
    pub fn children(mut self, children: impl IntoIterator<Item = TreeNode>) -> Self {
        self.children.extend(children);
        self
    }

    /// Returns the node's stable identifier.
    pub fn id(&self) -> &SharedString {
        &self.id
    }

    /// Returns the node's display label.
    pub fn label(&self) -> &SharedString {
        &self.label
    }

    /// Returns the node's optional leading icon.
    pub fn icon_name(&self) -> Option<IconName> {
        self.icon
    }

    /// Returns `true` when the node has one or more children.
    pub fn is_branch(&self) -> bool {
        !self.children.is_empty()
    }

    /// Returns `true` when the node is currently expanded.
    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    /// Returns `true` when the node is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Returns the node's direct children.
    pub fn child_nodes(&self) -> &[TreeNode] {
        &self.children
    }

    fn to_base_item(&self) -> BaseTreeItem {
        let mut item = BaseTreeItem::new(self.id.clone(), self.label.clone())
            .expanded(self.expanded)
            .disabled(self.disabled);
        if !self.children.is_empty() {
            item = item.children(self.children.iter().map(Self::to_base_item));
        }
        item
    }

    fn collect_icons(&self, out: &mut HashMap<SharedString, IconName>) {
        if let Some(icon) = self.icon {
            out.insert(self.id.clone(), icon);
        }
        for child in &self.children {
            child.collect_icons(out);
        }
    }

    fn set_expanded_by_id(&mut self, target_id: &str, expanded: bool) -> Option<bool> {
        if self.id.as_ref() == target_id {
            if self.children.is_empty() || self.disabled || self.expanded == expanded {
                return Some(false);
            }
            self.expanded = expanded;
            return Some(true);
        }
        for child in &mut self.children {
            if let Some(changed) = child.set_expanded_by_id(target_id, expanded) {
                return Some(changed);
            }
        }
        None
    }

    fn is_expanded_by_id(&self, target_id: &str) -> Option<bool> {
        if self.id.as_ref() == target_id {
            return Some(self.expanded);
        }
        for child in &self.children {
            if let Some(expanded) = child.is_expanded_by_id(target_id) {
                return Some(expanded);
            }
        }
        None
    }

    fn expand_ancestors(&mut self, target_id: &str, expanded_ids: &mut Vec<SharedString>) -> bool {
        if self.id.as_ref() == target_id {
            return true;
        }
        for child in &mut self.children {
            if child.expand_ancestors(target_id, expanded_ids) {
                if !self.expanded {
                    self.expanded = true;
                    expanded_ids.push(self.id.clone());
                }
                return true;
            }
        }
        false
    }

    fn set_all_expanded(&mut self, expanded: bool) {
        if !self.children.is_empty() && !self.disabled {
            self.expanded = expanded;
        }
        for child in &mut self.children {
            child.set_all_expanded(expanded);
        }
    }

    fn find_enabled_by_id(&self, target_id: &str) -> Option<bool> {
        if self.id.as_ref() == target_id {
            return Some(!self.disabled);
        }
        for child in &self.children {
            if let Some(enabled) = child.find_enabled_by_id(target_id) {
                return Some(enabled);
            }
        }
        None
    }

    fn flatten_visible(&self, depth: usize, out: &mut Vec<FlatNode>) {
        out.push(FlatNode {
            id: self.id.clone(),
            depth,
            is_branch: !self.children.is_empty(),
            is_expanded: self.expanded,
            is_disabled: self.disabled,
        });
        if self.expanded {
            for child in &self.children {
                child.flatten_visible(depth + 1, out);
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FlatNode {
    id: SharedString,
    depth: usize,
    is_branch: bool,
    is_expanded: bool,
    is_disabled: bool,
}

/// Snapshot passed to [`Tree::render_row`] for customizing visible node content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeRowInfo {
    /// Stable node identifier.
    pub id: SharedString,
    /// Display label.
    pub label: SharedString,
    /// Optional leading icon associated with the node.
    pub icon: Option<IconName>,
    /// Zero-based nesting depth (`0` for root nodes).
    pub depth: usize,
    /// Whether this node has child nodes.
    pub is_branch: bool,
    /// Whether this branch is currently expanded.
    pub is_expanded: bool,
    /// Whether this row is currently selected.
    pub is_selected: bool,
    /// Whether this row is disabled.
    pub is_disabled: bool,
}

/// Events emitted by [`TreeState`] when expansion, selection, or activation changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TreeEvent {
    /// A branch node expanded.
    Expanded(SharedString),
    /// A branch node collapsed.
    Collapsed(SharedString),
    /// A node became the active selection.
    Selected(SharedString),
    /// A node was activated via pointer click or keyboard (`Enter`/`Space`).
    Activated(SharedString),
}

/// Optional per-instance color overrides for [`Tree`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TreeColors {
    /// Outer container fill (defaults to `SurfaceContainerLow`).
    pub container: Option<Hsla>,
    /// Outer container border color (defaults to `OutlineVariant`).
    pub border: Option<Hsla>,
    /// Default row label color (defaults to `OnSurface`).
    pub content: Option<Hsla>,
    /// Default icon and chevron color (defaults to `OnSurfaceVariant`).
    pub icon: Option<Hsla>,
    /// Selected row pill fill (defaults to `SecondaryContainer`).
    pub selected_container: Option<Hsla>,
    /// Selected row label, icon, and state-layer ink color (defaults to `OnSecondaryContainer`).
    pub selected_content: Option<Hsla>,
    /// Disabled row label and icon color (defaults to `OnSurface` at 38% opacity).
    pub disabled_content: Option<Hsla>,
}

struct ResolvedTreeColors {
    container: Hsla,
    border: Hsla,
    content: Hsla,
    icon: Hsla,
    selected_container: Hsla,
    selected_content: Hsla,
    disabled_content: Hsla,
    focus_ring: Hsla,
}

impl TreeColors {
    fn resolve(self, scheme: &gpui_material_theme::ColorScheme) -> ResolvedTreeColors {
        let tone = |role| color(scheme.color(role));
        ResolvedTreeColors {
            container: self
                .container
                .unwrap_or_else(|| tone(ColorRole::SurfaceContainerLow)),
            border: self
                .border
                .unwrap_or_else(|| tone(ColorRole::OutlineVariant)),
            content: self.content.unwrap_or_else(|| tone(ColorRole::OnSurface)),
            icon: self
                .icon
                .unwrap_or_else(|| tone(ColorRole::OnSurfaceVariant)),
            selected_container: self
                .selected_container
                .unwrap_or_else(|| tone(ColorRole::SecondaryContainer)),
            selected_content: self
                .selected_content
                .unwrap_or_else(|| tone(ColorRole::OnSecondaryContainer)),
            disabled_content: self
                .disabled_content
                .unwrap_or_else(|| tone(ColorRole::OnSurface).opacity(0.38)),
            focus_ring: tone(ColorRole::Secondary),
        }
    }
}

/// Retained state entity for a Material [`Tree`].
///
/// Wraps `gpui-base`'s virtualized tree state while exposing Material-owned nodes,
/// events, and WAI-ARIA keyboard helpers that honor local `ThemeScope` direction
/// and skip disabled nodes.
pub struct TreeState {
    base: Entity<BaseTreeState>,
    container_focus: FocusHandle,
    nodes: Vec<TreeNode>,
    visible: Vec<FlatNode>,
    icons: HashMap<SharedString, IconName>,
    _base_subscription: Subscription,
}

impl EventEmitter<TreeEvent> for TreeState {}

impl TreeState {
    /// Creates an empty [`TreeState`].
    pub fn new(cx: &mut Context<Self>) -> Self {
        let base = cx.new(|cx| BaseTreeState::new(cx));
        let container_focus = cx.focus_handle();
        let subscription = cx.subscribe(&base, |this, _, event: &BaseTreeEvent, cx| match event {
            BaseTreeEvent::Expanded(id) => {
                for node in &mut this.nodes {
                    if node.set_expanded_by_id(id.as_ref(), true).is_some() {
                        break;
                    }
                }
                this.rebuild_visible();
                cx.emit(TreeEvent::Expanded(id.clone()));
                cx.notify();
            }
            BaseTreeEvent::Collapsed(id) => {
                for node in &mut this.nodes {
                    if node.set_expanded_by_id(id.as_ref(), false).is_some() {
                        break;
                    }
                }
                this.rebuild_visible();
                cx.emit(TreeEvent::Collapsed(id.clone()));
                cx.notify();
            }
        });
        Self {
            base,
            container_focus,
            nodes: Vec::new(),
            visible: Vec::new(),
            icons: HashMap::new(),
            _base_subscription: subscription,
        }
    }

    /// Populates the initial tree items during construction.
    pub fn items(mut self, items: impl Into<Vec<TreeNode>>, cx: &mut Context<Self>) -> Self {
        self.set_items(items, cx);
        self
    }

    /// Replaces the tree model, clearing the previous selection.
    pub fn set_items(&mut self, items: impl Into<Vec<TreeNode>>, cx: &mut Context<Self>) {
        self.nodes = items.into();
        self.rebuild_icons();
        self.rebuild_visible();
        let base_items: Vec<BaseTreeItem> = self.nodes.iter().map(TreeNode::to_base_item).collect();
        self.base.update(cx, |base, cx| {
            base.set_items(base_items, cx);
        });
        cx.notify();
    }

    /// Returns the root [`TreeNode`] slice.
    pub fn nodes(&self) -> &[TreeNode] {
        &self.nodes
    }

    /// Returns the number of currently visible flattened rows.
    pub fn visible_count(&self) -> usize {
        self.visible.len()
    }

    /// Returns the currently selected visible row index, if any.
    pub fn selected_index(&self, cx: &App) -> Option<usize> {
        self.base.read(cx).selected_index()
    }

    /// Returns the ID of the currently selected node, if any.
    pub fn selected_id(&self, cx: &App) -> Option<SharedString> {
        self.base
            .read(cx)
            .selected_item()
            .map(|item| item.id.clone())
    }

    /// Returns whether the node with `id` is currently expanded.
    pub fn is_expanded(&self, id: &str) -> bool {
        self.nodes
            .iter()
            .find_map(|node| node.is_expanded_by_id(id))
            .unwrap_or(false)
    }

    /// Selects the node with `id` (expanding hidden ancestors if needed) or clears selection when `None`.
    ///
    /// Disabled nodes cannot be selected.
    pub fn select(&mut self, id: Option<&str>, cx: &mut Context<Self>) {
        let Some(target_id) = id else {
            self.base
                .update(cx, |base, cx| base.set_selected_index(None, cx));
            cx.notify();
            return;
        };
        let enabled = self
            .nodes
            .iter()
            .find_map(|node| node.find_enabled_by_id(target_id))
            .unwrap_or(false);
        if !enabled {
            return;
        }
        let mut expanded_ids = Vec::new();
        for node in &mut self.nodes {
            if node.expand_ancestors(target_id, &mut expanded_ids) {
                break;
            }
        }
        if !expanded_ids.is_empty() {
            self.rebuild_visible();
            self.sync_base_preserving_selection(Some(target_id), cx);
            for expanded in expanded_ids.into_iter().rev() {
                cx.emit(TreeEvent::Expanded(expanded));
            }
        } else {
            let target_ix = self.visible.iter().position(|n| n.id.as_ref() == target_id);
            self.base.update(cx, |base, cx| {
                base.set_selected_index(target_ix, cx);
                if let Some(ix) = target_ix {
                    base.scroll_to_item(ix, ScrollStrategy::Nearest);
                }
            });
        }
        cx.emit(TreeEvent::Selected(SharedString::from(
            target_id.to_owned(),
        )));
        cx.notify();
    }

    /// Expands or collapses the branch node with `id`, returning `true` when state changed.
    pub fn set_expanded(&mut self, id: &str, expanded: bool, cx: &mut Context<Self>) -> bool {
        let mut changed = false;
        for node in &mut self.nodes {
            if let Some(did_change) = node.set_expanded_by_id(id, expanded) {
                changed = did_change;
                break;
            }
        }
        if !changed {
            return false;
        }
        let prev_selected = self.selected_id(cx);
        self.rebuild_visible();
        self.sync_base_preserving_selection(prev_selected.as_deref(), cx);
        let shared = SharedString::from(id.to_owned());
        cx.emit(if expanded {
            TreeEvent::Expanded(shared)
        } else {
            TreeEvent::Collapsed(shared)
        });
        cx.notify();
        true
    }

    /// Toggles the expansion state of the branch node with `id`, returning the new expanded state
    /// if the node is an enabled branch.
    pub fn toggle_expanded(&mut self, id: &str, cx: &mut Context<Self>) -> Option<bool> {
        let next = !self.is_expanded(id);
        self.set_expanded(id, next, cx).then_some(next)
    }

    /// Expands all enabled branch nodes.
    pub fn expand_all(&mut self, cx: &mut Context<Self>) {
        let prev_selected = self.selected_id(cx);
        for node in &mut self.nodes {
            node.set_all_expanded(true);
        }
        self.rebuild_visible();
        self.sync_base_preserving_selection(prev_selected.as_deref(), cx);
        cx.notify();
    }

    /// Collapses all enabled branch nodes.
    pub fn collapse_all(&mut self, cx: &mut Context<Self>) {
        let prev_selected = self.selected_id(cx);
        for node in &mut self.nodes {
            node.set_all_expanded(false);
        }
        self.rebuild_visible();
        self.sync_base_preserving_selection(prev_selected.as_deref(), cx);
        cx.notify();
    }

    /// Expands ancestors of `id` and scrolls it into view.
    pub fn reveal(&mut self, id: &str, strategy: ScrollStrategy, cx: &mut Context<Self>) {
        let mut expanded_ids = Vec::new();
        for node in &mut self.nodes {
            if node.expand_ancestors(id, &mut expanded_ids) {
                break;
            }
        }
        if !expanded_ids.is_empty() {
            let prev_selected = self.selected_id(cx);
            self.rebuild_visible();
            self.sync_base_preserving_selection(prev_selected.as_deref(), cx);
            for expanded in expanded_ids.into_iter().rev() {
                cx.emit(TreeEvent::Expanded(expanded));
            }
        }
        if let Some(ix) = self.visible.iter().position(|n| n.id.as_ref() == id) {
            self.base.update(cx, |base, _| {
                base.scroll_to_item(ix, strategy);
            });
        }
        cx.notify();
    }

    /// Returns a clone of the underlying list scroll handle (compatible with `DisplayScrollbar`).
    pub fn scroll_handle(&self, cx: &App) -> UniformListScrollHandle {
        self.base.read(cx).scroll_handle().clone()
    }

    /// Focuses the tree for keyboard navigation.
    pub fn focus(&self, window: &mut Window, cx: &mut App) {
        self.base.update(cx, |base, cx| {
            base.focus(window, cx);
        });
    }

    /// Returns `true` when this tree currently owns keyboard focus.
    pub fn is_focused(&self, window: &Window, cx: &App) -> bool {
        self.container_focus.contains_focused(window, cx)
    }

    fn rebuild_icons(&mut self) {
        self.icons.clear();
        for node in &self.nodes {
            node.collect_icons(&mut self.icons);
        }
    }

    fn rebuild_visible(&mut self) {
        self.visible.clear();
        for node in &self.nodes {
            node.flatten_visible(0, &mut self.visible);
        }
    }

    fn sync_base_preserving_selection(
        &mut self,
        selected_id: Option<&str>,
        cx: &mut Context<Self>,
    ) {
        let base_items: Vec<BaseTreeItem> = self.nodes.iter().map(TreeNode::to_base_item).collect();
        let selected_ix =
            selected_id.and_then(|id| self.visible.iter().position(|n| n.id.as_ref() == id));
        self.base.update(cx, |base, cx| {
            base.set_items(base_items, cx);
            base.set_selected_index(selected_ix, cx);
            if let Some(ix) = selected_ix {
                base.scroll_to_item(ix, ScrollStrategy::Nearest);
            }
        });
    }

    fn step_selection(&mut self, forward: bool, cx: &mut Context<Self>) -> Option<SharedString> {
        let current = self.selected_index(cx);
        let next_ix = next_enabled_index(&self.visible, current, forward)?;
        let id = self.visible[next_ix].id.clone();
        self.base.update(cx, |base, cx| {
            base.set_selected_index(Some(next_ix), cx);
            base.scroll_to_item(
                next_ix,
                if forward {
                    ScrollStrategy::Bottom
                } else {
                    ScrollStrategy::Top
                },
            );
        });
        cx.emit(TreeEvent::Selected(id.clone()));
        cx.notify();
        Some(id)
    }

    fn jump_selection(&mut self, first: bool, cx: &mut Context<Self>) -> Option<SharedString> {
        let next_ix = edge_enabled_index(&self.visible, first)?;
        let id = self.visible[next_ix].id.clone();
        self.base.update(cx, |base, cx| {
            base.set_selected_index(Some(next_ix), cx);
            base.scroll_to_item(
                next_ix,
                if first {
                    ScrollStrategy::Top
                } else {
                    ScrollStrategy::Bottom
                },
            );
        });
        cx.emit(TreeEvent::Selected(id.clone()));
        cx.notify();
        Some(id)
    }

    fn handle_expand_key(
        &mut self,
        cx: &mut Context<Self>,
    ) -> (Option<SharedString>, Option<(SharedString, bool)>) {
        let Some(ix) = self.selected_index(cx) else {
            return (self.jump_selection(true, cx), None);
        };
        let Some(node) = self.visible.get(ix).cloned() else {
            return (None, None);
        };
        if node.is_disabled {
            return (None, None);
        }
        if node.is_branch && !node.is_expanded {
            if self.set_expanded(node.id.as_ref(), true, cx) {
                return (None, Some((node.id, true)));
            }
        } else if node.is_branch && node.is_expanded {
            // Move focus to the first enabled descendant.
            if let Some(child_ix) = first_enabled_child_index(&self.visible, ix) {
                let id = self.visible[child_ix].id.clone();
                self.base.update(cx, |base, cx| {
                    base.set_selected_index(Some(child_ix), cx);
                    base.scroll_to_item(child_ix, ScrollStrategy::Nearest);
                });
                cx.emit(TreeEvent::Selected(id.clone()));
                cx.notify();
                return (Some(id), None);
            }
        }
        (None, None)
    }

    fn handle_collapse_key(
        &mut self,
        cx: &mut Context<Self>,
    ) -> (Option<SharedString>, Option<(SharedString, bool)>) {
        let Some(ix) = self.selected_index(cx) else {
            return (None, None);
        };
        let Some(node) = self.visible.get(ix).cloned() else {
            return (None, None);
        };
        if node.is_disabled {
            return (None, None);
        }
        if node.is_branch && node.is_expanded {
            if self.set_expanded(node.id.as_ref(), false, cx) {
                return (None, Some((node.id, false)));
            }
        } else if let Some(parent_ix) = nearest_enabled_ancestor_index(&self.visible, ix) {
            let id = self.visible[parent_ix].id.clone();
            self.base.update(cx, |base, cx| {
                base.set_selected_index(Some(parent_ix), cx);
                base.scroll_to_item(parent_ix, ScrollStrategy::Nearest);
            });
            cx.emit(TreeEvent::Selected(id.clone()));
            cx.notify();
            return (Some(id), None);
        }
        (None, None)
    }

    fn handle_row_click(
        &mut self,
        id: &SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> (bool, Option<bool>) {
        let Some(ix) = self.visible.iter().position(|n| &n.id == id) else {
            return (false, None);
        };
        let node = self.visible[ix].clone();
        if node.is_disabled {
            return (false, None);
        }
        let toggled = if node.is_branch {
            let next_expanded = !node.is_expanded;
            for root in &mut self.nodes {
                if root
                    .set_expanded_by_id(id.as_ref(), next_expanded)
                    .is_some()
                {
                    break;
                }
            }
            self.rebuild_visible();
            self.sync_base_preserving_selection(Some(id.as_ref()), cx);
            cx.emit(if next_expanded {
                TreeEvent::Expanded(id.clone())
            } else {
                TreeEvent::Collapsed(id.clone())
            });
            Some(next_expanded)
        } else {
            self.base.update(cx, |base, cx| {
                base.set_selected_index(Some(ix), cx);
            });
            None
        };
        self.focus(window, cx);
        cx.emit(TreeEvent::Selected(id.clone()));
        cx.emit(TreeEvent::Activated(id.clone()));
        cx.notify();
        (true, toggled)
    }

    fn handle_activate_key(
        &mut self,
        cx: &mut Context<Self>,
    ) -> (Option<SharedString>, Option<(SharedString, bool)>) {
        let Some(ix) = self.selected_index(cx) else {
            return (None, None);
        };
        let Some(node) = self.visible.get(ix).cloned() else {
            return (None, None);
        };
        if node.is_disabled {
            return (None, None);
        }
        let toggled = if node.is_branch {
            self.toggle_expanded(node.id.as_ref(), cx)
                .map(|expanded| (node.id.clone(), expanded))
        } else {
            None
        };
        cx.emit(TreeEvent::Activated(node.id.clone()));
        (Some(node.id), toggled)
    }
}

fn next_enabled_index(
    visible: &[FlatNode],
    current: Option<usize>,
    forward: bool,
) -> Option<usize> {
    let len = visible.len();
    if len == 0 {
        return None;
    }
    let start = match current {
        Some(ix) if ix < len => ix,
        _ => return edge_enabled_index(visible, forward),
    };
    for offset in 1..=len {
        let candidate = if forward {
            (start + offset) % len
        } else {
            (start + len - offset) % len
        };
        if !visible[candidate].is_disabled {
            return Some(candidate);
        }
    }
    None
}

fn edge_enabled_index(visible: &[FlatNode], first: bool) -> Option<usize> {
    if first {
        visible.iter().position(|n| !n.is_disabled)
    } else {
        visible.iter().rposition(|n| !n.is_disabled)
    }
}

fn first_enabled_child_index(visible: &[FlatNode], parent_ix: usize) -> Option<usize> {
    let parent_depth = visible.get(parent_ix)?.depth;
    for (ix, node) in visible.iter().enumerate().skip(parent_ix + 1) {
        if node.depth <= parent_depth {
            break;
        }
        if !node.is_disabled {
            return Some(ix);
        }
    }
    None
}

fn nearest_enabled_ancestor_index(visible: &[FlatNode], child_ix: usize) -> Option<usize> {
    let mut target_depth = visible.get(child_ix)?.depth;
    if target_depth == 0 {
        return None;
    }
    for ix in (0..child_ix).rev() {
        let node = &visible[ix];
        if node.depth < target_depth {
            if !node.is_disabled {
                return Some(ix);
            }
            target_depth = node.depth;
            if target_depth == 0 {
                break;
            }
        }
    }
    None
}

pub(crate) fn tree_row_height(density: Density, override_height: Option<Pixels>) -> Pixels {
    override_height
        .unwrap_or_else(|| px(density.visual_size(DEFAULT_ROW_HEIGHT).max(MIN_ROW_HEIGHT)))
}

/// Material 3 Expressive virtualized hierarchical tree view over `gpui-base`'s [`BaseTree`].
#[derive(IntoElement)]
pub struct Tree {
    id: ElementId,
    state: Entity<TreeState>,
    row_height: Option<Pixels>,
    indent: Pixels,
    shape: Option<CornerShape>,
    colors: TreeColors,
    show_scrollbar: bool,
    scrollbar_style: DisplayScrollbarStyle,
    scrollbar_fade: DisplayScrollbarFade,
    accessible_label: Option<SharedString>,
    render_row: Option<CustomRowRenderer>,
    on_select: Option<SelectHandler>,
    on_activate: Option<ActivateHandler>,
    on_toggle: Option<ToggleHandler>,
    style: StyleRefinement,
}

impl Tree {
    /// Creates a Material 3 tree bound to the caller's retained [`Entity<TreeState>`].
    pub fn new(id: impl Into<ElementId>, state: &Entity<TreeState>) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            row_height: None,
            indent: px(DEFAULT_INDENT),
            shape: None,
            colors: TreeColors::default(),
            show_scrollbar: true,
            scrollbar_style: DisplayScrollbarStyle::default(),
            scrollbar_fade: DisplayScrollbarFade::default(),
            accessible_label: None,
            render_row: None,
            on_select: None,
            on_activate: None,
            on_toggle: None,
            style: StyleRefinement::default(),
        }
    }

    /// Overrides the density-aware row height. Non-finite or non-positive values are ignored.
    pub fn row_height(mut self, height: Pixels) -> Self {
        if f32::from(height).is_finite() && height > px(0.) {
            self.row_height = Some(height);
        }
        self
    }

    /// Sets logical indentation per depth level (defaults to `20 px`).
    pub fn indent(mut self, indent: Pixels) -> Self {
        if f32::from(indent).is_finite() && indent >= px(0.) {
            self.indent = indent;
        }
        self
    }

    /// Sets the container corner shape (defaults to `CornerRole::Medium`).
    pub fn shape(mut self, shape: CornerShape) -> Self {
        self.shape = Some(shape);
        self
    }

    /// Applies per-instance Material color overrides.
    pub fn colors(mut self, colors: TreeColors) -> Self {
        self.colors = colors;
        self
    }

    /// Shows or hides the Material [`DisplayScrollbar`] overlay (defaults to `true`).
    pub fn show_scrollbar(mut self, show: bool) -> Self {
        self.show_scrollbar = show;
        self
    }

    /// Customizes the [`DisplayScrollbar`] dimensions and colors.
    pub fn scrollbar_style(mut self, style: DisplayScrollbarStyle) -> Self {
        self.scrollbar_style = style;
        self
    }

    /// Customizes the [`DisplayScrollbar`] auto-hide fade behavior.
    pub fn scrollbar_fade(mut self, fade: DisplayScrollbarFade) -> Self {
        self.scrollbar_fade = fade;
        self
    }

    /// Sets an accessible name for the tree container.
    pub fn accessible_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }

    /// Supplies custom row content rendered inside each indented, state-layered Material row shell.
    pub fn render_row(
        mut self,
        renderer: impl Fn(&TreeRowInfo, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.render_row = Some(Rc::new(renderer));
        self
    }

    /// Invoked whenever an enabled node becomes selected via pointer click or keyboard navigation.
    pub fn on_select(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    /// Invoked when an enabled node is activated via pointer click or `Enter`/`Space`.
    pub fn on_activate(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_activate = Some(Rc::new(handler));
        self
    }

    /// Invoked when an enabled branch node expands (`true`) or collapses (`false`).
    pub fn on_toggle(
        mut self,
        handler: impl Fn(&SharedString, bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(Rc::new(handler));
        self
    }
}

impl Styled for Tree {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Tree {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = current_material_theme(window, cx).clone();
        let strings = current_material_strings(window, cx).clone();
        let direction = theme.layout.direction;
        let rtl = direction == Direction::Rtl;
        let density = theme.layout.density;
        let reduced_motion = cx.reduce_motion();
        let colors = self.colors.resolve(&theme.colors);
        let row_height = tree_row_height(density, self.row_height);
        let indent = self.indent;
        let shape = self.shape.unwrap_or_else(|| CornerRole::Medium.shape());
        let radii = shape.radii(100.0, 100.0, direction);

        let (base_entity, container_focus, visible_count, scroll_handle, icons) = {
            let state = self.state.read(cx);
            (
                state.base.clone(),
                state.container_focus.clone(),
                state.visible.len(),
                state.base.read(cx).scroll_handle().clone(),
                Rc::new(state.icons.clone()),
            )
        };
        let tree_focused = container_focus.contains_focused(window, cx);
        let focus_visible = tree_focused && window.last_input_was_keyboard();

        let on_select = self.on_select;
        let on_activate = self.on_activate;
        let on_toggle = self.on_toggle;

        let row_state = self.state.clone();
        let row_select = on_select.clone();
        let row_activate = on_activate.clone();
        let row_toggle = on_toggle.clone();
        let custom_renderer = self.render_row;
        let tree_id = self.id.clone();

        let act_up_state = self.state.clone();
        let act_up_select = on_select.clone();
        let act_down_state = self.state.clone();
        let act_down_select = on_select.clone();
        let act_left_state = self.state.clone();
        let act_left_select = on_select.clone();
        let act_left_toggle = on_toggle.clone();
        let act_right_state = self.state.clone();
        let act_right_select = on_select.clone();
        let act_right_toggle = on_toggle.clone();
        let act_confirm_state = self.state.clone();
        let act_confirm_activate = on_activate.clone();
        let act_confirm_toggle = on_toggle.clone();

        let key_state = self.state.clone();
        let key_select = on_select;
        let key_activate = on_activate;
        let key_toggle = on_toggle;

        let row_focus = container_focus.clone();
        let base_tree = BaseTree::new(&base_entity)
            .size_full()
            .list_style(StyleRefinement::default().size_full())
            .item(move |_ix, entry: &TreeEntry, entry_state, window, cx| {
                let item = entry.item();
                let id = item.id.clone();
                let label = item.label.clone();
                let depth = entry.depth();
                let is_branch = entry.is_folder();
                let is_expanded = entry.is_expanded();
                let is_disabled = entry.is_disabled();
                let is_selected = entry_state.is_selected();
                let icon_name = icons.get(&id).copied();

                let row_elem_id = ElementId::from((tree_id.clone(), format!("node-{id}")));
                let feedback = ripple_state(&row_elem_id, !is_disabled, window, cx);
                let interaction = {
                    let mut interaction =
                        feedback
                            .borrow()
                            .interaction(!is_disabled, &row_focus, window);
                    interaction.focus_visible = !is_disabled && is_selected && focus_visible;
                    interaction
                };

                let pill_bg = if is_selected {
                    colors.selected_container
                } else {
                    transparent_black()
                };
                let text_color = if is_disabled {
                    colors.disabled_content
                } else if is_selected {
                    colors.selected_content
                } else {
                    colors.content
                };
                let icon_color = if is_disabled {
                    colors.disabled_content
                } else if is_selected {
                    colors.selected_content
                } else {
                    colors.icon
                };

                let start_indent = px(8.0) + indent * (depth as f32);
                let end_padding = px(12.0);
                let (pl, pr) = if rtl {
                    (end_padding, start_indent)
                } else {
                    (start_indent, end_padding)
                };

                let chevron_el = if is_branch {
                    let mut icon = Icon::new(IconName::ChevronRight)
                        .size(px(18.))
                        .color_override(icon_color);
                    if is_expanded {
                        // Symmetric downward chevron works identically in LTR and RTL.
                        icon = icon.rotate(radians(FRAC_PI_2));
                    } else {
                        icon = icon.mirror_in_rtl(true);
                    }
                    div()
                        .w(px(CHEVRON_SLOT_SIZE))
                        .h(px(CHEVRON_SLOT_SIZE))
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(icon)
                        .into_any_element()
                } else {
                    div()
                        .w(px(CHEVRON_SLOT_SIZE))
                        .h(px(CHEVRON_SLOT_SIZE))
                        .flex_none()
                        .into_any_element()
                };

                let body = if let Some(renderer) = &custom_renderer {
                    let info = TreeRowInfo {
                        id: id.clone(),
                        label: label.clone(),
                        icon: icon_name,
                        depth,
                        is_branch,
                        is_expanded,
                        is_selected,
                        is_disabled,
                    };
                    renderer(&info, window, cx)
                } else {
                    let leading_icon = icon_name.map(|name| {
                        Icon::new(name)
                            .size(px(18.))
                            .color_override(icon_color)
                            .mirror_in_rtl(false)
                    });
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .when(rtl, Styled::flex_row_reverse)
                        .items_center()
                        .gap_2()
                        .children(leading_icon)
                        .child(
                            Text::new(label.clone())
                                .role(TypeRole::BodyMedium)
                                .emphasized(is_selected)
                                .color_override(text_color)
                                .max_lines(1),
                        )
                        .into_any_element()
                };

                let pill_shape = CornerRole::Full.shape();
                let prepaint_feedback = feedback.clone();
                let paint_feedback = feedback.clone();
                let feedback_layer = canvas(
                    move |bounds, window, _| {
                        prepaint_feedback
                            .borrow_mut()
                            .set_rounded_shape(bounds, pill_shape, direction, window);
                    },
                    move |_, _, window, cx| {
                        let state = paint_feedback.borrow();
                        let Some(shape) = state.shape() else {
                            return;
                        };
                        let now = cx.background_executor().now();
                        InteractionFeedback::new(interaction, text_color, colors.focus_ring)
                            .focus_ring_placement(FocusRingPlacement::Inset)
                            .paint(shape, state.ripple(), now, reduced_motion, window);
                    },
                )
                .absolute()
                .inset_0();

                let pill = div()
                    .id(row_elem_id)
                    .relative()
                    .size_full()
                    .rounded_full()
                    .bg(pill_bg)
                    .text_color(text_color)
                    .flex()
                    .when(rtl, Styled::flex_row_reverse)
                    .items_center()
                    .gap_1()
                    .pl(pl)
                    .pr(pr)
                    .overflow_hidden()
                    .when(!is_disabled, |el| el.cursor(CursorStyle::PointingHand))
                    .child(feedback_layer)
                    .child(chevron_el)
                    .child(body);

                let pill = if !is_disabled {
                    let click_feedback = feedback.clone();
                    let click_state = row_state.clone();
                    let click_id = id.clone();
                    let on_select = row_select.clone();
                    let on_activate = row_activate.clone();
                    let on_toggle = row_toggle.clone();
                    feedback_target(
                        pill.on_click(move |event: &ClickEvent, window, cx| {
                            if click_feedback.borrow().blocks_activation(event) {
                                return;
                            }
                            activate_feedback(event, &click_feedback, window, cx);
                            cx.stop_propagation();
                            let (handled, toggled) = click_state.update(cx, |state, cx| {
                                state.handle_row_click(&click_id, window, cx)
                            });
                            if handled {
                                if let Some(handler) = &on_select {
                                    handler(&click_id, window, cx);
                                }
                                if let Some(handler) = &on_activate {
                                    handler(&click_id, window, cx);
                                }
                                if let (Some(expanded), Some(handler)) = (toggled, &on_toggle) {
                                    handler(&click_id, expanded, window, cx);
                                }
                            }
                        }),
                        feedback,
                        true,
                        None,
                    )
                } else {
                    pill
                };

                div()
                    .w_full()
                    .h(row_height)
                    .px_1p5()
                    .py(px(1.))
                    .child(pill)
                    .into_any_element()
            });

        let natural_height = px((visible_count.max(1) as f32 * f32::from(row_height)
            + CONTAINER_PADDING_Y * 2.0)
            .min(DEFAULT_MAX_HEIGHT));
        let default_label = self
            .accessible_label
            .unwrap_or_else(|| SharedString::from(strings.navigation.as_ref().to_owned()));

        let mut container = div()
            .id(self.id.clone())
            .test_support()
            .aria_label(default_label)
            .track_focus(&container_focus)
            .tab_stop(false)
            .relative()
            .w_full()
            .h(natural_height)
            .py(px(CONTAINER_PADDING_Y))
            .bg(colors.container)
            .border_1()
            .border_color(colors.border)
            .rounded_tl(px(radii[0]))
            .rounded_tr(px(radii[1]))
            .rounded_br(px(radii[2]))
            .rounded_bl(px(radii[3]))
            .overflow_hidden()
            .capture_action(move |_: &SelectUp, window, cx| {
                cx.stop_propagation();
                let selected = act_up_state.update(cx, |state, cx| state.step_selection(false, cx));
                if let (Some(id), Some(handler)) = (selected, &act_up_select) {
                    handler(&id, window, cx);
                }
            })
            .capture_action(move |_: &SelectDown, window, cx| {
                cx.stop_propagation();
                let selected =
                    act_down_state.update(cx, |state, cx| state.step_selection(true, cx));
                if let (Some(id), Some(handler)) = (selected, &act_down_select) {
                    handler(&id, window, cx);
                }
            })
            .capture_action(move |_: &SelectLeft, window, cx| {
                cx.stop_propagation();
                let (selected, toggled) = act_left_state.update(cx, |state, cx| {
                    if rtl {
                        state.handle_expand_key(cx)
                    } else {
                        state.handle_collapse_key(cx)
                    }
                });
                if let (Some(id), Some(handler)) = (selected, &act_left_select) {
                    handler(&id, window, cx);
                }
                if let (Some((id, expanded)), Some(handler)) = (toggled, &act_left_toggle) {
                    handler(&id, expanded, window, cx);
                }
            })
            .capture_action(move |_: &SelectRight, window, cx| {
                cx.stop_propagation();
                let (selected, toggled) = act_right_state.update(cx, |state, cx| {
                    if rtl {
                        state.handle_collapse_key(cx)
                    } else {
                        state.handle_expand_key(cx)
                    }
                });
                if let (Some(id), Some(handler)) = (selected, &act_right_select) {
                    handler(&id, window, cx);
                }
                if let (Some((id, expanded)), Some(handler)) = (toggled, &act_right_toggle) {
                    handler(&id, expanded, window, cx);
                }
            })
            .capture_action(move |_: &Confirm, window, cx| {
                cx.stop_propagation();
                let (activated, toggled) =
                    act_confirm_state.update(cx, |state, cx| state.handle_activate_key(cx));
                if let (Some(id), Some(handler)) = (activated, &act_confirm_activate) {
                    handler(&id, window, cx);
                }
                if let (Some((id, expanded)), Some(handler)) = (toggled, &act_confirm_toggle) {
                    handler(&id, expanded, window, cx);
                }
            })
            .capture_key_down(move |event: &KeyDownEvent, window, cx| {
                if event.keystroke.modifiers.modified() {
                    return;
                }
                match event.keystroke.key.as_str() {
                    "home" => {
                        cx.stop_propagation();
                        let selected =
                            key_state.update(cx, |state, cx| state.jump_selection(true, cx));
                        if let (Some(id), Some(handler)) = (selected, &key_select) {
                            handler(&id, window, cx);
                        }
                    }
                    "end" => {
                        cx.stop_propagation();
                        let selected =
                            key_state.update(cx, |state, cx| state.jump_selection(false, cx));
                        if let (Some(id), Some(handler)) = (selected, &key_select) {
                            handler(&id, window, cx);
                        }
                    }
                    "space" => {
                        cx.stop_propagation();
                        let (activated, toggled) =
                            key_state.update(cx, |state, cx| state.handle_activate_key(cx));
                        if let (Some(id), Some(handler)) = (activated, &key_activate) {
                            handler(&id, window, cx);
                        }
                        if let (Some((id, expanded)), Some(handler)) = (toggled, &key_toggle) {
                            handler(&id, expanded, window, cx);
                        }
                    }
                    _ => {}
                }
            })
            .child(base_tree);

        if self.show_scrollbar {
            container = container.child(
                DisplayScrollbar::new((self.id, "scrollbar"), Axis::Vertical, &scroll_handle)
                    .style(self.scrollbar_style)
                    .fade(self.scrollbar_fade),
            );
        }

        container.refine_style(&self.style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_flat() -> Vec<FlatNode> {
        vec![
            FlatNode {
                id: "root".into(),
                depth: 0,
                is_branch: true,
                is_expanded: true,
                is_disabled: false,
            },
            FlatNode {
                id: "disabled-child".into(),
                depth: 1,
                is_branch: false,
                is_expanded: false,
                is_disabled: true,
            },
            FlatNode {
                id: "child".into(),
                depth: 1,
                is_branch: false,
                is_expanded: false,
                is_disabled: false,
            },
            FlatNode {
                id: "sibling".into(),
                depth: 0,
                is_branch: false,
                is_expanded: false,
                is_disabled: false,
            },
        ]
    }

    #[test]
    fn navigation_skips_disabled_nodes_and_resolves_parent_and_child() {
        let flat = sample_flat();
        assert_eq!(next_enabled_index(&flat, Some(0), true), Some(2));
        assert_eq!(next_enabled_index(&flat, Some(2), false), Some(0));
        assert_eq!(next_enabled_index(&flat, Some(3), true), Some(0));
        assert_eq!(first_enabled_child_index(&flat, 0), Some(2));
        assert_eq!(nearest_enabled_ancestor_index(&flat, 2), Some(0));
        assert_eq!(nearest_enabled_ancestor_index(&flat, 3), None);
    }

    #[test]
    fn density_scales_tree_row_height() {
        let compact = tree_row_height(Density::Compact, None);
        let standard = tree_row_height(Density::Standard, None);
        let comfortable = tree_row_height(Density::Comfortable, None);
        assert!(compact < standard);
        assert!(standard < comfortable);
        assert_eq!(tree_row_height(Density::Compact, Some(px(48.))), px(48.));
    }
}
