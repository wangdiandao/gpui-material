use gpui::{
    AnyElement, App, Div, ElementId, Entity, Hsla, InteractiveElement, IntoElement, ParentElement,
    Pixels, RenderOnce, Role, ScrollHandle, SharedString, StatefulInteractiveElement, Styled,
    TextAlign, Window, div, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{animate, color, text_style, window_material_theme};
use gpui_material_theme::{
    ColorRole, Direction, MotionKind, MotionSpec, MotionSpeed, TypeRole, TypographyEmphasis,
};

mod actions;
mod scroll;

pub use actions::{AppBarColumn, AppBarItem, AppBarMenuState, AppBarRow};
use scroll::length;
pub use scroll::{AppBarScrollBehavior, AppBarScrollState};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AppBarVariant {
    #[default]
    Small,
    Centered,
    MediumFlexible,
    LargeFlexible,
    TwoRows,
}

impl AppBarVariant {
    fn two_rows(self) -> bool {
        matches!(
            self,
            Self::MediumFlexible | Self::LargeFlexible | Self::TwoRows
        )
    }

    fn expansion(self, subtitle: bool) -> f32 {
        match (self, subtitle) {
            (Self::MediumFlexible | Self::TwoRows, false) => 48.,
            (Self::MediumFlexible | Self::TwoRows, true) => 72.,
            (Self::LargeFlexible, false) => 56.,
            (Self::LargeFlexible, true) => 88.,
            _ => 0.,
        }
    }

    fn expanded_roles(self) -> (TypeRole, TypeRole) {
        if self == Self::LargeFlexible {
            (TypeRole::DisplaySmall, TypeRole::TitleMedium)
        } else {
            (TypeRole::HeadlineMedium, TypeRole::LabelLarge)
        }
    }
}

/// Logical alignment for a title or subtitle.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AppBarAlignment {
    #[default]
    Start,
    Center,
    End,
}

impl AppBarAlignment {
    fn resolve(self, direction: Direction) -> TextAlign {
        match (self, direction) {
            (Self::Center, _) => TextAlign::Center,
            (Self::Start, Direction::Ltr) | (Self::End, Direction::Rtl) => TextAlign::Left,
            _ => TextAlign::Right,
        }
    }
}

/// Per-instance pigments. Missing fields use the window theme.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AppBarColors {
    pub container: Option<Hsla>,
    pub scrolled_container: Option<Hsla>,
    pub navigation: Option<Hsla>,
    pub title: Option<Hsla>,
    pub subtitle: Option<Hsla>,
    pub actions: Option<Hsla>,
}

/// Logical insets within each app bar row.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AppBarPadding {
    pub top: Pixels,
    pub end: Pixels,
    pub bottom: Pixels,
    pub start: Pixels,
}

impl AppBarPadding {
    fn apply(self, row: Div, direction: Direction) -> Div {
        let (left, right) = if direction == Direction::Rtl {
            (self.end, self.start)
        } else {
            (self.start, self.end)
        };
        let resolve = |value| px(length(value).unwrap_or(0.));
        row.pl(resolve(left))
            .pr(resolve(right))
            .pt(resolve(self.top))
            .pb(resolve(self.bottom))
    }
}

type HeadingContent = Box<dyn FnOnce(bool) -> AnyElement>;

#[derive(Default)]
struct Heading {
    text: Option<SharedString>,
    content: Option<HeadingContent>,
    alignment: Option<AppBarAlignment>,
    max_lines: Option<usize>,
}

impl Heading {
    fn has_content(&self) -> bool {
        self.text.is_some() || self.content.is_some()
    }

    fn additional_height(&self, role: TypeRole) -> f32 {
        if self.content.is_some() || self.text.is_none() {
            return 0.;
        }
        self.max_lines.unwrap_or(1).saturating_sub(1) as f32
            * role.style(TypographyEmphasis::Baseline).line_height
    }

    fn render(
        self,
        expanded: bool,
        role: TypeRole,
        style: HeadingStyle,
        window: &Window,
        cx: &App,
    ) -> Option<Div> {
        let alignment = self
            .alignment
            .unwrap_or(style.alignment)
            .resolve(style.direction);
        let label = text_style(
            div().min_w_0().text_align(alignment),
            role.style(TypographyEmphasis::Baseline),
            style.font_scale,
            window,
            cx,
        )
        .text_color(style.color);
        if let Some(build) = self.content {
            return Some(aligned_content(alignment).child(label.child(build(expanded))));
        }
        let lines = self.max_lines.unwrap_or(1);
        self.text.map(|text| {
            label
                .when_else(lines == 1, Styled::truncate, |label| {
                    label.line_clamp(lines)
                })
                .child(text)
        })
    }
}

fn aligned_content(alignment: TextAlign) -> Div {
    div()
        .flex()
        .min_w_0()
        .when(alignment == TextAlign::Center, Styled::justify_center)
        .when(alignment == TextAlign::Right, Styled::justify_end)
}

#[derive(Clone, Copy)]
struct HeadingStyle {
    direction: Direction,
    alignment: AppBarAlignment,
    font_scale: f32,
    color: Hsla,
}

/// Top app bar with caller-owned navigation, headings, actions, and optional search.
#[derive(IntoElement)]
pub struct AppBar {
    id: ElementId,
    title: Heading,
    subtitle: Heading,
    variant: AppBarVariant,
    navigation: Option<AnyElement>,
    actions: Option<AnyElement>,
    search: Option<AnyElement>,
    logo: Option<AnyElement>,
    second_row: Option<AnyElement>,
    colors: AppBarColors,
    padding: Option<AppBarPadding>,
    collapsed_height: Option<f32>,
    expanded_height: Option<f32>,
    scroll: Option<ScrollHandle>,
    scroll_state: Option<Entity<AppBarScrollState>>,
    scroll_behavior: Option<AppBarScrollBehavior>,
    scroll_enabled: bool,
    scroll_motion: Option<MotionSpec>,
    font_scale: f32,
}

impl AppBar {
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: Heading {
                text: Some(title.into()),
                ..Heading::default()
            },
            subtitle: Heading::default(),
            variant: AppBarVariant::Small,
            navigation: None,
            actions: None,
            search: None,
            logo: None,
            second_row: None,
            colors: AppBarColors::default(),
            padding: None,
            collapsed_height: None,
            expanded_height: None,
            scroll: None,
            scroll_state: None,
            scroll_behavior: None,
            scroll_enabled: true,
            scroll_motion: None,
            font_scale: 1.,
        }
    }

    pub fn variant(mut self, variant: AppBarVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle.text = Some(subtitle.into());
        self.subtitle.content = None;
        self
    }

    /// Replaces the visual title while keeping the constructor's accessible name.
    pub fn title_content(self, content: impl IntoElement + 'static) -> Self {
        self.title_for_rows(move |_| content)
    }

    pub fn subtitle_content(self, content: impl IntoElement + 'static) -> Self {
        self.subtitle_for_rows(move |_| content)
    }

    /// Builds the title for the currently displayed row.
    pub fn title_for_rows<E: IntoElement>(
        mut self,
        build: impl FnOnce(bool) -> E + 'static,
    ) -> Self {
        self.title.content = Some(Box::new(move |expanded| build(expanded).into_any_element()));
        self
    }

    /// Builds the subtitle for the currently displayed row.
    pub fn subtitle_for_rows<E: IntoElement>(
        mut self,
        build: impl FnOnce(bool) -> E + 'static,
    ) -> Self {
        self.subtitle.content = Some(Box::new(move |expanded| build(expanded).into_any_element()));
        self
    }

    pub fn title_alignment(mut self, alignment: AppBarAlignment) -> Self {
        self.title.alignment = Some(alignment);
        self
    }

    pub fn subtitle_alignment(mut self, alignment: AppBarAlignment) -> Self {
        self.subtitle.alignment = Some(alignment);
        self
    }

    pub fn title_max_lines(mut self, lines: usize) -> Self {
        self.title.max_lines = Some(lines.max(1));
        self
    }

    pub fn subtitle_max_lines(mut self, lines: usize) -> Self {
        self.subtitle.max_lines = Some(lines.max(1));
        self
    }

    pub fn navigation(mut self, element: impl IntoElement) -> Self {
        self.navigation = Some(element.into_any_element());
        self
    }

    pub fn actions(mut self, element: impl IntoElement) -> Self {
        self.actions = Some(element.into_any_element());
        self
    }

    /// Supplies the search control that replaces the heading.
    pub fn search(mut self, element: impl IntoElement) -> Self {
        self.search = Some(element.into_any_element());
        self
    }

    pub fn logo(mut self, element: impl IntoElement) -> Self {
        self.logo = Some(element.into_any_element());
        self
    }

    /// Adds presentation content below the expanded heading in TwoRows.
    pub fn second_row(mut self, element: impl IntoElement) -> Self {
        self.second_row = Some(element.into_any_element());
        self
    }

    pub fn colors(mut self, colors: AppBarColors) -> Self {
        self.colors = colors;
        self
    }

    pub fn content_padding(mut self, padding: AppBarPadding) -> Self {
        self.padding = Some(padding);
        self
    }

    pub fn collapsed_height(mut self, height: Pixels) -> Self {
        self.collapsed_height = length(height).filter(|height| *height > 0.);
        self
    }

    pub fn expanded_height(mut self, height: Pixels) -> Self {
        self.expanded_height = length(height).filter(|height| *height > 0.);
        self
    }

    pub fn track_scroll(mut self, handle: &ScrollHandle) -> Self {
        self.scroll = Some(handle.clone());
        self
    }

    pub fn scroll_state(mut self, state: &Entity<AppBarScrollState>) -> Self {
        self.scroll_state = Some(state.clone());
        self
    }

    pub fn scroll_behavior(mut self, behavior: AppBarScrollBehavior) -> Self {
        self.scroll_behavior = Some(behavior);
        self
    }

    pub fn scroll_enabled(mut self, enabled: bool) -> Self {
        self.scroll_enabled = enabled;
        self
    }

    /// Overrides motion when the caller asks the linked scroll state to settle.
    pub fn scroll_motion(mut self, motion: MotionSpec) -> Self {
        self.scroll_motion = Some(motion);
        self
    }

    pub fn font_scale(mut self, scale: f32) -> Self {
        self.font_scale = if scale.is_finite() && scale > 0. {
            scale
        } else {
            1.
        };
        self
    }

    fn scroll_position(&self, limit: f32, window: &mut Window, cx: &mut App) -> (f32, f32) {
        if self.scroll.is_none() && self.scroll_state.is_none() {
            return (0., 0.);
        }
        let state = self.scroll_state.clone().unwrap_or_else(|| {
            window.use_keyed_state((self.id.clone(), "app-bar-scroll"), cx, |_, _| {
                AppBarScrollState::new()
            })
        });
        let content_offset = self
            .scroll
            .as_ref()
            .map(|handle| length(-handle.offset().y).unwrap_or(0.));
        let position = state.update(cx, |state, _| {
            state.prepare(
                content_offset,
                limit,
                self.scroll_behavior,
                self.scroll_enabled,
            )
        });
        let motion = if position.settling && !cx.reduce_motion() {
            self.scroll_motion.unwrap_or_else(|| {
                window_material_theme(window, cx).motion.spec(
                    MotionKind::Spatial,
                    MotionSpeed::Default,
                    false,
                )
            })
        } else {
            MotionSpec::Instant
        };
        let fraction = animate(
            (self.id.clone(), "app-bar-height"),
            position.target,
            motion,
            window,
            cx,
        )
        .clamp(0., 1.);
        if state.update(cx, |state, _| state.display(position, fraction))
            && self.scroll_state.is_some()
        {
            let state = state.downgrade();
            window.defer(cx, move |_, cx| {
                let _ = state.update(cx, |_, cx| cx.notify());
            });
        }
        (fraction, position.overlap)
    }
}

impl RenderOnce for AppBar {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let collapsed = self.collapsed_height.unwrap_or(64.);
        let two_rows = self.variant.two_rows();
        let (expanded_title_role, expanded_subtitle_role) = self.variant.expanded_roles();
        let default_expansion = if two_rows {
            (self.variant.expansion(self.subtitle.has_content())
                + self.title.additional_height(expanded_title_role)
                + self.subtitle.additional_height(expanded_subtitle_role))
                * self.font_scale.max(1.)
        } else {
            0.
        };
        let limit = self
            .expanded_height
            .map_or(default_expansion, |height| (height - collapsed).max(0.));
        let (fraction, overlap) = self.scroll_position(limit, window, cx);
        let remaining = limit * (1. - fraction);
        let theme = window_material_theme(window, cx);
        let direction = theme.layout.direction;
        let rtl = direction == Direction::Rtl;
        let centered = self.variant == AppBarVariant::Centered;
        let has_search = self.search.is_some();
        let expanded_row = two_rows && remaining > 0. && !has_search;
        let expanded_heading = expanded_row && remaining > limit / 2.;
        let default_alignment = if centered {
            AppBarAlignment::Center
        } else {
            AppBarAlignment::Start
        };
        let role = |role| color(theme.colors.color(role));
        let surface = self
            .colors
            .container
            .unwrap_or_else(|| role(ColorRole::Surface));
        let scrolled = self
            .colors
            .scrolled_container
            .unwrap_or_else(|| role(ColorRole::SurfaceContainer));
        let background = match overlap {
            0. => surface,
            1. => scrolled,
            _ => crate::motion::mix(surface, scrolled, overlap),
        };
        let title_color = self
            .colors
            .title
            .unwrap_or_else(|| role(ColorRole::OnSurface));
        let subtitle_color = self
            .colors
            .subtitle
            .unwrap_or_else(|| role(ColorRole::OnSurfaceVariant));
        let mut navigation = div()
            .flex()
            .items_center()
            .min_w_0()
            .gap(px(4.))
            .when(rtl, Styled::flex_row_reverse);
        if let Some(navigation_color) = self.colors.navigation {
            navigation = navigation.text_color(navigation_color);
        }
        let search_logo = has_search.then(|| self.logo.take()).flatten();
        navigation = navigation.children(self.navigation).children(search_logo);
        let mut actions = div()
            .flex()
            .justify_end()
            .min_w_0()
            .when(rtl, Styled::flex_row_reverse)
            .children(self.actions);
        if let Some(actions_color) = self.colors.actions {
            actions = actions.text_color(actions_color);
        }
        let label = self.title.text.clone().unwrap_or_default();
        let title_alignment = self
            .title
            .alignment
            .unwrap_or(default_alignment)
            .resolve(direction);
        let heading =
            if has_search {
                None
            } else if !two_rows && self.logo.is_some() {
                Some(
                    aligned_content(title_alignment)
                        .text_color(title_color)
                        .children(self.logo.take()),
                )
            } else {
                let heading_style = HeadingStyle {
                    direction,
                    alignment: default_alignment,
                    font_scale: self.font_scale,
                    color: title_color,
                };
                let (title_role, subtitle_role) = if expanded_heading {
                    (expanded_title_role, expanded_subtitle_role)
                } else {
                    (TypeRole::TitleLarge, TypeRole::LabelMedium)
                };
                Some(
                    div()
                        .flex()
                        .flex_col()
                        .min_w_0()
                        .children(expanded_heading.then(|| self.logo.take()).flatten().map(
                            |logo| {
                                aligned_content(title_alignment)
                                    .text_color(title_color)
                                    .child(logo)
                            },
                        ))
                        .children(self.title.render(
                            expanded_heading,
                            title_role,
                            heading_style,
                            window,
                            cx,
                        ))
                        .children(self.subtitle.render(
                            expanded_heading,
                            subtitle_role,
                            HeadingStyle {
                                color: subtitle_color,
                                ..heading_style
                            },
                            window,
                            cx,
                        )),
                )
            };
        let (top_heading, lower_heading) = if expanded_heading {
            (None, heading)
        } else {
            (heading, None)
        };
        let center_slots = centered && !has_search;
        let row = div()
            .flex()
            .items_center()
            .min_h(px(if two_rows || has_search {
                collapsed
            } else {
                collapsed + remaining
            }))
            .gap(px(4.))
            .when(rtl, Styled::flex_row_reverse)
            .child(navigation.when(center_slots, |lane| lane.flex_1()))
            .child(
                div()
                    .min_w_0()
                    .when_else(center_slots, |lane| lane.w_1_2(), |lane| lane.flex_1())
                    .children(self.search)
                    .children(top_heading),
            )
            .child(actions.when(center_slots, |lane| lane.flex_1()));
        let primary = self
            .padding
            .unwrap_or(AppBarPadding {
                start: px(4.),
                end: px(4.),
                ..Default::default()
            })
            .apply(row, direction);
        let lower = expanded_row.then(|| {
            let body = div()
                .h(px(remaining))
                .overflow_hidden()
                .flex()
                .flex_col()
                .justify_end()
                .children(lower_heading)
                .children(
                    self.second_row
                        .filter(|_| self.variant == AppBarVariant::TwoRows && expanded_heading),
                );
            self.padding
                .unwrap_or(AppBarPadding {
                    start: px(16.),
                    end: px(16.),
                    bottom: px(4.),
                    top: px(0.),
                })
                .apply(body, direction)
        });
        div()
            .id(self.id)
            .test_support()
            .role(Role::Toolbar)
            .aria_label(label)
            .w_full()
            .flex_none()
            .flex()
            .flex_col()
            .bg(background)
            .child(primary)
            .children(lower)
    }
}
