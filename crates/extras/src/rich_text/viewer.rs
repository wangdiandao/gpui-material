use std::{cell::RefCell, ops::Range, sync::Arc, time::Duration};

use gpui::{
    App, AppContext, ClickEvent, Context, ElementId, Empty, Entity, FontWeight, Global,
    HighlightStyle, Hsla, InteractiveElement, IntoElement, ParentElement, Refineable, Rems, Render,
    RenderOnce, Role, SharedString, StatefulInteractiveElement, StyleRefinement, Styled,
    Subscription, TextAlign, Window, div, px, rems,
};
use gpui_base::{
    SelectionFormat as BaseSelectionFormat, TaskToggleEvent as BaseTaskToggleEvent, TestSupportExt,
    TextView as BaseTextView, TextViewMotion as BaseTextViewMotion,
    TextViewState as BaseTextViewState, TextViewStyle as BaseTextViewStyle,
};
use gpui_material_foundation::{
    color, current_material_theme, font_cache_revision, font_for_style,
};
use gpui_material_theme::{
    Appearance, ColorRole, CornerRole, Direction, LayoutPolicy, MaterialTheme, TypeRole,
    TypographyEmphasis,
};

/// Format used when reading or copying selected content from [`RichText`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RichTextSelectionFormat {
    /// Copies rendered plain text without Markdown punctuation (default).
    #[default]
    Plain,
    /// Copies the original Markdown source (or reconstructed Markdown for partial selections).
    Source,
    /// Copies plain text within a single inline/block style and Markdown source across styles.
    SourceAcrossStyles,
}

impl From<RichTextSelectionFormat> for BaseSelectionFormat {
    fn from(format: RichTextSelectionFormat) -> Self {
        match format {
            RichTextSelectionFormat::Plain => Self::Plain,
            RichTextSelectionFormat::Source => Self::Source,
            RichTextSelectionFormat::SourceAcrossStyles => Self::SourceAcrossStyles,
        }
    }
}

/// Payload delivered to [`RichText::on_task_toggle`] when a Markdown task-list checkbox is clicked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RichTextTaskToggle {
    checked: bool,
    item_range: Range<usize>,
    marker_range: Range<usize>,
}

impl RichTextTaskToggle {
    fn from_base(event: &BaseTaskToggleEvent) -> Self {
        Self {
            checked: event.checked(),
            item_range: event.item_range(),
            marker_range: event.marker_range(),
        }
    }

    /// Returns the requested checked state (the opposite of the rendered checkbox).
    pub fn checked(&self) -> bool {
        self.checked
    }

    /// Returns the list item's byte range in the Markdown source.
    pub fn item_range(&self) -> Range<usize> {
        self.item_range.clone()
    }

    /// Returns the `[ ]` or `[x]` marker's byte range in the Markdown source.
    pub fn marker_range(&self) -> Range<usize> {
        self.marker_range.clone()
    }
}

/// Per-instance style overrides for [`RichText`]. Unset fields project automatically
/// from the active window or `ThemeScope` [`MaterialTheme`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RichTextStyle {
    /// Body text color (defaults to `OnSurface`).
    pub body_color: Option<Hsla>,
    /// Secondary/muted text color (defaults to `OnSurfaceVariant`).
    pub muted_color: Option<Hsla>,
    /// Hyperlink color (defaults to `Primary`).
    pub link_color: Option<Hsla>,
    /// Text selection highlight color (defaults to translucent `PrimaryContainer`).
    pub selection_color: Option<Hsla>,
    /// Background for fenced code blocks and inline code spans (defaults to `SurfaceContainerHighest`).
    pub code_background: Option<Hsla>,
    /// Foreground for code spans and code blocks (defaults to `OnSurface`).
    pub code_color: Option<Hsla>,
    /// Horizontal rule and table border color (defaults to `OutlineVariant`).
    pub border_color: Option<Hsla>,
    /// Leading accent bar color for blockquotes (defaults to `OutlineVariant`).
    pub blockquote_border_color: Option<Hsla>,
    /// Hovered table row tint (defaults to `SurfaceContainerLow`).
    pub table_row_hover: Option<Hsla>,
    /// Vertical spacing between paragraphs (defaults to `0.75 rem`).
    pub paragraph_gap: Option<Rems>,
    /// Optional monospace font family override for code spans and blocks.
    pub code_font_family: Option<SharedString>,
}

/// Retained state for a [`RichText`] document supporting incremental streaming,
/// selection inspection, and clamping checks.
pub struct RichTextState {
    base: Entity<BaseTextViewState>,
    _subscription: Subscription,
}

impl RichTextState {
    /// Creates a retained Markdown rich-text state.
    pub fn markdown(text: &str, cx: &mut Context<Self>) -> Self {
        let base = cx.new(|cx| BaseTextViewState::markdown(text, cx).selectable(true));
        let subscription = cx.observe(&base, |_, _, cx| cx.notify());
        Self {
            base,
            _subscription: subscription,
        }
    }

    /// Creates a retained HTML rich-text state.
    pub fn html(text: &str, cx: &mut Context<Self>) -> Self {
        let base = cx.new(|cx| BaseTextViewState::html(text, cx).selectable(true));
        let subscription = cx.observe(&base, |_, _, cx| cx.notify());
        Self {
            base,
            _subscription: subscription,
        }
    }

    /// Replaces the document source (or extends incrementally when `text` starts with the
    /// current Markdown content).
    pub fn set_text(&mut self, text: &str, cx: &mut Context<Self>) {
        self.base.update(cx, |base, cx| base.set_text(text, cx));
        cx.notify();
    }

    /// Appends a streaming text delta to the document.
    pub fn push_str(&mut self, chunk: &str, cx: &mut Context<Self>) {
        self.base.update(cx, |base, cx| base.push_str(chunk, cx));
        cx.notify();
    }

    /// Returns the currently selected text in this rich-text view.
    pub fn selected_text(&self, cx: &App) -> String {
        self.base.read(cx).selected_text()
    }

    /// Returns the plain rendered text of the latest committed parse.
    pub fn rendered_text(&self, cx: &App) -> String {
        self.base.read(cx).rendered_text().as_str().to_owned()
    }

    /// Returns whether the last painted frame clipped content because of [`RichText::max_lines`].
    pub fn is_clamped(&self, cx: &App) -> bool {
        self.base.read(cx).is_clamped()
    }

    /// Sets whether text selection is enabled.
    pub fn set_selectable(&mut self, selectable: bool, cx: &mut Context<Self>) {
        self.base
            .update(cx, |base, cx| base.set_selectable(selectable, cx));
        cx.notify();
    }

    /// Sets whether the view uses internal virtualized scrolling.
    pub fn set_scrollable(&mut self, scrollable: bool, cx: &mut Context<Self>) {
        self.base
            .update(cx, |base, cx| base.set_scrollable(scrollable, cx));
        cx.notify();
    }

    /// Sets the selection copy format.
    pub fn set_selection_format(
        &mut self,
        format: RichTextSelectionFormat,
        cx: &mut Context<Self>,
    ) {
        self.base
            .update(cx, |base, cx| base.set_selection_format(format.into(), cx));
        cx.notify();
    }
}

impl Render for RichTextState {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

type LinkClickHandler = dyn Fn(&SharedString, &ClickEvent, &mut Window, &mut App) + Send + Sync;
type TaskToggleHandler = dyn Fn(RichTextTaskToggle, &mut Window, &mut App) + Send + Sync;

enum RichTextSource {
    Markdown(SharedString),
    Html(SharedString),
    State(Entity<RichTextState>),
}

/// Material 3 styled Markdown and HTML rich-text viewer.
///
/// Automatically projects the active window or `ThemeScope` [`MaterialTheme`] onto
/// headings (`HeadlineLarge`–`TitleSmall`), body typography, `SurfaceContainerHighest`
/// monospace code blocks, `Primary` links, `OutlineVariant` blockquotes and tables,
/// and `PrimaryContainer` selection highlights.
#[derive(IntoElement)]
pub struct RichText {
    id: ElementId,
    source: RichTextSource,
    body_role: TypeRole,
    font_scale: f32,
    rich_style: RichTextStyle,
    selectable: bool,
    selection_format: RichTextSelectionFormat,
    scrollable: bool,
    max_lines: Option<usize>,
    stream_fade: bool,
    on_link_click: Option<Arc<LinkClickHandler>>,
    on_task_toggle: Option<Arc<TaskToggleHandler>>,
    accessible_label: Option<SharedString>,
    style: StyleRefinement,
}

impl RichText {
    /// Creates a stateless Material 3 Markdown rich-text view.
    pub fn markdown(id: impl Into<ElementId>, source: impl Into<SharedString>) -> Self {
        Self::with_source(id.into(), RichTextSource::Markdown(source.into()))
    }

    /// Creates a stateless Material 3 HTML rich-text view.
    pub fn html(id: impl Into<ElementId>, source: impl Into<SharedString>) -> Self {
        Self::with_source(id.into(), RichTextSource::Html(source.into()))
    }

    /// Creates a Material 3 rich-text view bound to a retained [`RichTextState`] entity.
    pub fn from_state(id: impl Into<ElementId>, state: &Entity<RichTextState>) -> Self {
        Self::with_source(id.into(), RichTextSource::State(state.clone()))
    }

    fn with_source(id: ElementId, source: RichTextSource) -> Self {
        Self {
            id,
            source,
            body_role: TypeRole::BodyLarge,
            font_scale: 1.0,
            rich_style: RichTextStyle::default(),
            selectable: true,
            selection_format: RichTextSelectionFormat::Plain,
            scrollable: false,
            max_lines: None,
            stream_fade: false,
            on_link_click: None,
            on_task_toggle: None,
            accessible_label: None,
            style: StyleRefinement::default(),
        }
    }

    /// Selects the Material typography role used for body paragraphs (defaults to [`TypeRole::BodyLarge`]).
    pub fn body_role(mut self, role: TypeRole) -> Self {
        self.body_role = role;
        self
    }

    /// Applies an additional font-size multiplier across body and headings.
    /// Non-finite or non-positive scales are ignored.
    pub fn font_scale(mut self, scale: f32) -> Self {
        if scale.is_finite() && scale > 0.0 {
            self.font_scale = scale;
        }
        self
    }

    /// Supplies per-instance color and spacing overrides.
    pub fn rich_style(mut self, style: RichTextStyle) -> Self {
        self.rich_style = style;
        self
    }

    /// Sets whether text selection is enabled (defaults to `true`).
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    /// Sets the clipboard/selection copy format (defaults to [`RichTextSelectionFormat::Plain`]).
    pub fn selection_format(mut self, format: RichTextSelectionFormat) -> Self {
        self.selection_format = format;
        self
    }

    /// Enables internal virtualized scrolling for long documents (requires a bounded parent height).
    pub fn scrollable(mut self, scrollable: bool) -> Self {
        self.scrollable = scrollable;
        self
    }

    /// Caps the rendered view height to at most `lines` body lines (ignored when scrollable).
    pub fn max_lines(mut self, lines: usize) -> Self {
        self.max_lines = Some(lines.max(1));
        self
    }

    /// Enables a gentle fade-in for newly streamed chunks (automatically disabled under reduced motion).
    pub fn stream_fade(mut self, enabled: bool) -> Self {
        self.stream_fade = enabled;
        self
    }

    /// Handles clicks on rendered Markdown/HTML links. Without a handler, links open via `App::open_url`.
    pub fn on_link_click<F>(mut self, handler: F) -> Self
    where
        F: Fn(&SharedString, &ClickEvent, &mut Window, &mut App) + Send + Sync + 'static,
    {
        self.on_link_click = Some(Arc::new(handler));
        self
    }

    /// Makes Markdown task-list checkboxes (`- [ ]` / `- [x]`) interactive and delivers toggle events.
    pub fn on_task_toggle<F>(mut self, handler: F) -> Self
    where
        F: Fn(RichTextTaskToggle, &mut Window, &mut App) + Send + Sync + 'static,
    {
        self.on_task_toggle = Some(Arc::new(handler));
        self
    }

    /// Sets an accessible name on the rich-text container.
    pub fn accessible_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }
}

impl Styled for RichText {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for RichText {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if !cx.has_global::<MonospaceCache>() {
            cx.set_global(MonospaceCache(RefCell::new(None)));
        }
        let theme = current_material_theme(window, cx);
        let rtl = theme.layout.direction == Direction::Rtl;
        let body_type = LayoutPolicy::scale_text(
            theme
                .typography
                .style(self.body_role, TypographyEmphasis::Baseline),
            self.font_scale,
        );
        let body_font = font_for_style(&theme.typography, body_type, cx);
        let tv_style = project_text_view_style(&theme, &self.rich_style, self.font_scale, cx);

        let motion = if self.stream_fade && !cx.reduce_motion() {
            BaseTextViewMotion::default().with_stream_fade(Duration::from_millis(120))
        } else {
            BaseTextViewMotion::default()
        };

        let view_id = (self.id.clone(), "view");
        let mut view = match self.source {
            RichTextSource::Markdown(source) => BaseTextView::markdown(view_id, source),
            RichTextSource::Html(source) => BaseTextView::html(view_id, source),
            RichTextSource::State(state) => BaseTextView::new(&state.read(cx).base),
        }
        .style(tv_style)
        .selectable(self.selectable)
        .selection_format(self.selection_format.into())
        .scrollable(self.scrollable)
        .motion(motion)
        .font(body_font)
        .text_size(px(body_type.size))
        .line_height(px(body_type.line_height))
        .text_align(if rtl {
            TextAlign::Right
        } else {
            TextAlign::Left
        });

        if let Some(max_lines) = self.max_lines {
            view = view.max_lines(max_lines);
        }
        if let Some(handler) = self.on_link_click {
            view = view.on_link_click(move |url, event, window, cx| {
                handler(url, event, window, cx);
            });
        }
        if let Some(handler) = self.on_task_toggle {
            view = view.on_task_toggle(move |event, window, cx| {
                handler(RichTextTaskToggle::from_base(event), window, cx);
            });
        }

        let mut container = div()
            .id(self.id)
            .test_support()
            .role(Role::Document)
            .min_w_0()
            .w_full();
        if let Some(label) = self.accessible_label {
            container = container.aria_label(label);
        }
        container.style().refine(&self.style);
        container.child(view)
    }
}

fn project_text_view_style(
    theme: &MaterialTheme,
    overrides: &RichTextStyle,
    font_scale: f32,
    cx: &App,
) -> BaseTextViewStyle {
    let is_dark = theme.colors.appearance() == Appearance::Dark;
    let body_color = overrides
        .body_color
        .unwrap_or_else(|| color(theme.colors.color(ColorRole::OnSurface)));
    let muted_color = overrides
        .muted_color
        .unwrap_or_else(|| color(theme.colors.color(ColorRole::OnSurfaceVariant)));
    let link_color = overrides
        .link_color
        .unwrap_or_else(|| color(theme.colors.color(ColorRole::Primary)));
    let selection_color = overrides
        .selection_color
        .unwrap_or_else(|| color(theme.colors.color(ColorRole::PrimaryContainer)).opacity(0.75));
    let code_bg = overrides
        .code_background
        .unwrap_or_else(|| color(theme.colors.color(ColorRole::SurfaceContainerHighest)));
    let code_fg = overrides.code_color.unwrap_or(body_color);
    let border_color = overrides
        .border_color
        .unwrap_or_else(|| color(theme.colors.color(ColorRole::OutlineVariant)));
    let blockquote_border = overrides.blockquote_border_color.unwrap_or(border_color);
    let table_head_bg = color(theme.colors.color(ColorRole::SurfaceContainerHigh));
    let table_row_hover = overrides
        .table_row_hover
        .unwrap_or_else(|| color(theme.colors.color(ColorRole::SurfaceContainerLow)));
    let paragraph_gap = overrides.paragraph_gap.unwrap_or(rems(0.75));
    let code_family = overrides
        .code_font_family
        .clone()
        .unwrap_or_else(|| resolve_monospace_family(cx));

    let heading_roles = [
        TypeRole::HeadlineLarge,
        TypeRole::HeadlineMedium,
        TypeRole::HeadlineSmall,
        TypeRole::TitleLarge,
        TypeRole::TitleMedium,
        TypeRole::TitleSmall,
    ];
    let headings: [StyleRefinement; 6] = heading_roles.map(|role| {
        let style = LayoutPolicy::scale_text(
            theme.typography.style(role, TypographyEmphasis::Emphasized),
            font_scale,
        );
        let font = font_for_style(&theme.typography, style, cx);
        let mut refinement = StyleRefinement::default()
            .font(font)
            .text_size(px(style.size))
            .line_height(px(style.line_height))
            .font_weight(FontWeight(f32::from(style.weight)))
            .text_color(body_color);
        refinement.margin.top = Some(px(8.).into());
        refinement.margin.bottom = Some(px(4.).into());
        refinement
    });

    let code_radius = px(CornerRole::Medium
        .shape()
        .radii(200., 80., theme.layout.direction)[0]);
    let inline_code_radius =
        px(CornerRole::ExtraSmall
            .shape()
            .radii(40., 20., theme.layout.direction)[0]);
    let code_type = LayoutPolicy::scale_text(
        theme
            .typography
            .style(TypeRole::BodyMedium, TypographyEmphasis::Baseline),
        font_scale,
    );

    let code_block_style = StyleRefinement::default()
        .bg(code_bg)
        .text_color(code_fg)
        .font_family(code_family.clone())
        .text_size(px(code_type.size))
        .line_height(px(code_type.line_height))
        .rounded(code_radius)
        .border_1()
        .border_color(border_color)
        .p_3();

    let inline_code_style = StyleRefinement::default()
        .bg(code_bg)
        .text_color(code_fg)
        .font_family(code_family)
        .rounded(inline_code_radius)
        .px_1();

    let blockquote_style = StyleRefinement::default()
        .text_color(muted_color)
        .border_l(px(3.))
        .border_color(blockquote_border)
        .pl_3();

    let table_style = StyleRefinement::default()
        .rounded(code_radius)
        .border_1()
        .border_color(border_color);

    let table_head_style = StyleRefinement::default()
        .bg(table_head_bg)
        .text_color(body_color)
        .font_weight(FontWeight::MEDIUM);

    BaseTextViewStyle::default()
        .with_foreground(body_color)
        .with_muted_foreground(muted_color)
        .with_link(link_color)
        .with_selection(selection_color)
        .with_code_background(code_bg)
        .with_border(border_color)
        .with_paragraph_gap(paragraph_gap)
        .with_heading(move |level| {
            let idx = level.clamp(1, 6) as usize - 1;
            headings[idx].clone()
        })
        .with_code_block(code_block_style)
        .with_inline_code(HighlightStyle {
            color: Some(code_fg),
            background_color: Some(code_bg),
            ..Default::default()
        })
        .with_inline_code_style(inline_code_style)
        .with_blockquote_style(blockquote_style)
        .with_table(table_style)
        .with_table_head(table_head_style)
        .with_table_row_hover(table_row_hover)
        .with_dark(is_dark)
}

struct MonospaceCache(RefCell<Option<(u64, SharedString)>>);

impl Global for MonospaceCache {}

fn resolve_monospace_family(cx: &App) -> SharedString {
    const CANDIDATES: [&str; 6] = [
        "Cascadia Mono",
        "JetBrains Mono",
        "Consolas",
        "SF Mono",
        "DejaVu Sans Mono",
        "Liberation Mono",
    ];
    let revision = font_cache_revision(cx);
    if let Some(cache) = cx.try_global::<MonospaceCache>()
        && let Some((cached_rev, family)) = cache.0.borrow().as_ref()
        && *cached_rev == revision
    {
        return family.clone();
    }
    let installed = cx.text_system().all_font_names();
    let resolved = CANDIDATES
        .iter()
        .find(|candidate| {
            installed
                .iter()
                .any(|name| name.eq_ignore_ascii_case(candidate))
        })
        .copied()
        .map(SharedString::new_static)
        .unwrap_or_else(|| SharedString::new_static("monospace"));
    if let Some(cache) = cx.try_global::<MonospaceCache>() {
        *cache.0.borrow_mut() = Some((revision, resolved.clone()));
    }
    resolved
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_format_maps_all_variants() {
        assert_eq!(
            BaseSelectionFormat::from(RichTextSelectionFormat::Plain),
            BaseSelectionFormat::Plain
        );
        assert_eq!(
            BaseSelectionFormat::from(RichTextSelectionFormat::Source),
            BaseSelectionFormat::Source
        );
        assert_eq!(
            BaseSelectionFormat::from(RichTextSelectionFormat::SourceAcrossStyles),
            BaseSelectionFormat::SourceAcrossStyles
        );
    }

    #[test]
    fn rich_text_builders_sanitize_scale_and_max_lines() {
        let view = RichText::markdown("doc", "# Hello")
            .font_scale(-2.0)
            .font_scale(f32::NAN)
            .max_lines(0);
        assert_eq!(view.font_scale, 1.0);
        assert_eq!(view.max_lines, Some(1));
    }
}
