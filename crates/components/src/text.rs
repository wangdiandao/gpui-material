use std::{
    cell::RefCell,
    hash::{DefaultHasher, Hash, Hasher},
    rc::Rc,
};

use gpui::{
    App, ElementId, Global, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels,
    RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled, TextAlign, Window, canvas,
    div, prelude::FluentBuilder, px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{
    ShapedText, color, current_material_theme, font_cache_revision, shape_text, wrapped_text,
};
use gpui_material_theme::{
    ColorRole, Direction, TypeRole, TypeStyle, Typography, TypographyEmphasis,
};

const ELLIPSIS_CACHE_SLOTS: usize = 256;
const ELLIPSIS_CACHE_MAX_BYTES: usize = 1024;

#[derive(PartialEq)]
struct EllipsisKey {
    text: SharedString,
    width: Pixels,
    style: TypeStyle,
    typography: Typography,
    font_scale: f32,
    scale_factor: f32,
    font_revision: u64,
    tint: Hsla,
}

impl EllipsisKey {
    fn slot(&self) -> usize {
        let mut hasher = DefaultHasher::new();
        self.text.hash(&mut hasher);
        f32::from(self.width).to_bits().hash(&mut hasher);
        hasher.finish() as usize % ELLIPSIS_CACHE_SLOTS
    }
}

type EllipsisEntry = (EllipsisKey, Option<Rc<ShapedText>>);

struct EllipsisCache(RefCell<Vec<Option<EllipsisEntry>>>);

impl Global for EllipsisCache {}

impl Default for EllipsisCache {
    fn default() -> Self {
        Self(RefCell::new(
            std::iter::repeat_with(|| None)
                .take(ELLIPSIS_CACHE_SLOTS)
                .collect(),
        ))
    }
}

impl EllipsisCache {
    fn get_or_shape(
        &self,
        key: EllipsisKey,
        shape: impl FnOnce(&EllipsisKey) -> Option<ShapedText>,
    ) -> Option<Rc<ShapedText>> {
        if key.text.len() > ELLIPSIS_CACHE_MAX_BYTES {
            return shape(&key).map(Rc::new);
        }
        let slot = key.slot();
        if let Some((stored, result)) = &self.0.borrow()[slot]
            && *stored == key
        {
            return result.clone();
        }
        let result = shape(&key).map(Rc::new);
        self.0.borrow_mut()[slot] = Some((key, result.clone()));
        result
    }
}

/// Single-line Material text. Measurement and painting share Foundation's
/// shaped glyphs, including font fallback and letter spacing.
#[derive(IntoElement)]
pub struct Text {
    value: SharedString,
    id: Option<ElementId>,
    role: TypeRole,
    emphasis: TypographyEmphasis,
    font_scale: f32,
    color: Option<ColorRole>,
    color_override: Option<Hsla>,
    align: Option<TextAlign>,
    max_lines: Option<usize>,
    max_width: Option<gpui::Pixels>,
    wrap: bool,
}

impl Text {
    /// Creates text with the default [`TypeRole::BodyLarge`] typography role.
    pub fn new(value: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            id: None,
            role: TypeRole::BodyLarge,
            emphasis: TypographyEmphasis::Baseline,
            font_scale: 1.,
            color: None,
            color_override: None,
            align: None,
            max_lines: None,
            max_width: None,
            wrap: false,
        }
    }

    /// Creates text styled with [`TypeRole::DisplayLarge`].
    pub fn display_large(value: impl Into<SharedString>) -> Self {
        Self::new(value).role(TypeRole::DisplayLarge)
    }

    /// Creates text styled with [`TypeRole::DisplayMedium`].
    pub fn display_medium(value: impl Into<SharedString>) -> Self {
        Self::new(value).role(TypeRole::DisplayMedium)
    }

    /// Creates text styled with [`TypeRole::DisplaySmall`].
    pub fn display_small(value: impl Into<SharedString>) -> Self {
        Self::new(value).role(TypeRole::DisplaySmall)
    }

    /// Creates text styled with [`TypeRole::HeadlineLarge`].
    pub fn headline_large(value: impl Into<SharedString>) -> Self {
        Self::new(value).role(TypeRole::HeadlineLarge)
    }

    /// Creates text styled with [`TypeRole::HeadlineMedium`].
    pub fn headline_medium(value: impl Into<SharedString>) -> Self {
        Self::new(value).role(TypeRole::HeadlineMedium)
    }

    /// Creates text styled with [`TypeRole::HeadlineSmall`].
    pub fn headline_small(value: impl Into<SharedString>) -> Self {
        Self::new(value).role(TypeRole::HeadlineSmall)
    }

    /// Creates text styled with [`TypeRole::TitleLarge`].
    pub fn title_large(value: impl Into<SharedString>) -> Self {
        Self::new(value).role(TypeRole::TitleLarge)
    }

    /// Creates text styled with [`TypeRole::TitleMedium`].
    pub fn title_medium(value: impl Into<SharedString>) -> Self {
        Self::new(value).role(TypeRole::TitleMedium)
    }

    /// Creates text styled with [`TypeRole::TitleSmall`].
    pub fn title_small(value: impl Into<SharedString>) -> Self {
        Self::new(value).role(TypeRole::TitleSmall)
    }

    /// Creates text styled with [`TypeRole::BodyLarge`].
    pub fn body_large(value: impl Into<SharedString>) -> Self {
        Self::new(value).role(TypeRole::BodyLarge)
    }

    /// Creates text styled with [`TypeRole::BodyMedium`].
    pub fn body_medium(value: impl Into<SharedString>) -> Self {
        Self::new(value).role(TypeRole::BodyMedium)
    }

    /// Creates text styled with [`TypeRole::BodySmall`].
    pub fn body_small(value: impl Into<SharedString>) -> Self {
        Self::new(value).role(TypeRole::BodySmall)
    }

    /// Creates text styled with [`TypeRole::LabelLarge`].
    pub fn label_large(value: impl Into<SharedString>) -> Self {
        Self::new(value).role(TypeRole::LabelLarge)
    }

    /// Creates text styled with [`TypeRole::LabelMedium`].
    pub fn label_medium(value: impl Into<SharedString>) -> Self {
        Self::new(value).role(TypeRole::LabelMedium)
    }

    /// Creates text styled with [`TypeRole::LabelSmall`].
    pub fn label_small(value: impl Into<SharedString>) -> Self {
        Self::new(value).role(TypeRole::LabelSmall)
    }

    /// Supplies a stable ID for this label within its parent.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the Material typography scale role. Defaults to [`TypeRole::BodyLarge`].
    pub fn role(mut self, role: TypeRole) -> Self {
        self.role = role;
        self
    }

    /// Selects emphasized typography weight when `true`, or baseline weight when `false`.
    pub fn emphasized(mut self, emphasized: bool) -> Self {
        self.emphasis = if emphasized {
            TypographyEmphasis::Emphasized
        } else {
            TypographyEmphasis::Baseline
        };
        self
    }

    /// Applies an additional font-size multiplier on top of the window theme's typography scale. Defaults to `1.0`.
    pub fn font_scale(mut self, scale: f32) -> Self {
        self.font_scale = scale;
        self
    }

    /// Resolves the text color from the window theme's semantic color role.
    pub fn color(mut self, role: ColorRole) -> Self {
        self.color = Some(role);
        self.color_override = None;
        self
    }

    /// Sets a literal text color. A later call to [`Self::color`] restores theme-role lookup.
    pub fn color_override(mut self, color: impl Into<Hsla>) -> Self {
        self.color_override = Some(color.into());
        self
    }

    /// Sets the text color to [`ColorRole::OnSurfaceVariant`].
    pub fn muted(self) -> Self {
        self.color(ColorRole::OnSurfaceVariant)
    }

    /// Sets explicit horizontal text alignment. Defaults to the window's logical start edge.
    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = Some(align);
        self
    }

    /// Clamps wrapped text to at most `lines` lines (minimum 1) when `.wrap()` is enabled.
    pub fn max_lines(mut self, lines: usize) -> Self {
        self.max_lines = Some(lines.max(1));
        self
    }

    /// Limits single-line text to `max_width` logical pixels, ellipsizing overflow and disabling wrapping.
    pub fn truncate(mut self, max_width: gpui::Pixels) -> Self {
        self.max_width = Some(max_width.max(px(0.)));
        self.wrap = false;
        self
    }

    /// Wraps at the available width and preserves explicit line breaks.
    /// This uses GPUI's native Unicode layout, whose pinned version does not
    /// expose custom tracking for wrapped text. Single-line text retains tracking.
    pub fn wrap(mut self) -> Self {
        self.wrap = true;
        self.max_width = None;
        self
    }
}

impl RenderOnce for Text {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tint = self.color_override.unwrap_or_else(|| {
            self.color.map_or_else(
                || window.text_style().color,
                |role| color(current_material_theme(window, cx).colors.color(role)),
            )
        });
        let text = if self.wrap {
            wrapped_text(
                self.value.clone(),
                self.role.style(self.emphasis),
                self.font_scale,
                tint,
                window,
                cx,
            )
            .when_some(self.align, |text, align| text.w_full().text_align(align))
            .when_some(self.max_lines, |text, lines| {
                text.line_clamp(lines).text_ellipsis()
            })
        } else {
            single_line_aligned(
                self.value.clone(),
                self.role.style(self.emphasis),
                self.font_scale,
                tint,
                (self.max_width, 1., 1.),
                self.align,
                window,
                cx,
            )
        };
        text.id(self.id.unwrap_or_else(|| "material-text".into()))
            .test_support()
            .role(Role::Label)
            .aria_label(self.value)
    }
}

fn clean_line(value: SharedString) -> SharedString {
    if value.contains(['\r', '\n']) {
        value.replace(['\r', '\n'], " ").into()
    } else {
        value
    }
}

pub(crate) fn shape_single_line(
    value: SharedString,
    style: TypeStyle,
    font_scale: f32,
    tint: Hsla,
    window: &Window,
    cx: &App,
) -> (SharedString, ShapedText) {
    let clean = clean_line(value);
    let theme = current_material_theme(window, cx);
    let shaped = shape_text(
        clean.clone(),
        style,
        &theme.typography,
        font_scale,
        tint,
        window,
        cx,
    )
    .expect("single-line text");
    (clean, shaped)
}

#[allow(clippy::too_many_arguments)]
fn single_line_aligned(
    value: SharedString,
    style: TypeStyle,
    font_scale: f32,
    tint: Hsla,
    (max_width, expansion, opacity): (Option<Pixels>, f32, f32),
    align_override: Option<TextAlign>,
    window: &Window,
    cx: &App,
) -> gpui::Div {
    let shaped = shape_single_line(value, style, font_scale, tint, window, cx);
    single_line_with_shaped_and_align(
        shaped,
        style,
        font_scale,
        tint,
        (max_width, expansion, opacity),
        align_override,
        window,
        cx,
    )
}

pub(crate) fn single_line(
    value: SharedString,
    style: TypeStyle,
    font_scale: f32,
    tint: Hsla,
    (max_width, expansion, opacity): (Option<Pixels>, f32, f32),
    window: &Window,
    cx: &App,
) -> gpui::Div {
    single_line_aligned(
        value,
        style,
        font_scale,
        tint,
        (max_width, expansion, opacity),
        None,
        window,
        cx,
    )
}

pub(crate) fn single_line_with_shaped(
    shaped: (SharedString, ShapedText),
    style: TypeStyle,
    font_scale: f32,
    tint: Hsla,
    (max_width, expansion, opacity): (Option<Pixels>, f32, f32),
    window: &Window,
    cx: &App,
) -> gpui::Div {
    single_line_with_shaped_and_align(
        shaped,
        style,
        font_scale,
        tint,
        (max_width, expansion, opacity),
        None,
        window,
        cx,
    )
}

#[allow(clippy::too_many_arguments)]
fn single_line_with_shaped_and_align(
    (clean, shaped): (SharedString, ShapedText),
    style: TypeStyle,
    font_scale: f32,
    tint: Hsla,
    (max_width, expansion, opacity): (Option<Pixels>, f32, f32),
    align_override: Option<TextAlign>,
    window: &Window,
    cx: &App,
) -> gpui::Div {
    let theme = current_material_theme(window, cx);
    let typography = theme.typography.clone();
    let align = align_override.unwrap_or(if theme.layout.direction == Direction::Rtl {
        TextAlign::Right
    } else {
        TextAlign::Left
    });
    let width = shaped.line().width.ceil();
    let height = shaped.line_height();
    let container = div().min_w_0();
    let container = if align_override.is_some() {
        match max_width {
            Some(max) => container.w(max * expansion),
            None => container.w_full(),
        }
    } else {
        container.w(max_width.map_or(width, |max| width.min(max)) * expansion)
    };
    container
        .max_w_full()
        .h(height)
        .overflow_hidden()
        .opacity(opacity)
        .child(
            canvas(
                move |bounds, window, cx| {
                    let maximum = bounds.size.width;
                    if shaped.line().width <= maximum || expansion < 1. {
                        return Some(Ok(shaped));
                    }
                    let key = EllipsisKey {
                        text: clean,
                        width: maximum,
                        style,
                        typography,
                        font_scale,
                        scale_factor: window.scale_factor(),
                        font_revision: font_cache_revision(cx),
                        tint,
                    };
                    if cx.try_global::<EllipsisCache>().is_none() {
                        cx.set_global(EllipsisCache::default());
                    }
                    cx.global::<EllipsisCache>()
                        .get_or_shape(key, |key| ellipsize(&shaped, key, window, cx))
                        .map(Err)
                },
                move |bounds, shaped, window, cx| {
                    if let Some(shaped) = shaped {
                        let shaped = match &shaped {
                            Ok(line) => line,
                            Err(line) => line.as_ref(),
                        };
                        let _ =
                            shaped.paint(bounds.origin, align, Some(bounds.size.width), window, cx);
                    }
                },
            )
            .size_full(),
        )
}

fn find_ellipsis_fit<T>(ends: &[usize], mut try_fit: impl FnMut(usize) -> Option<T>) -> Option<T> {
    let mut last_fitting: Option<(usize, T)> = None;
    let cut = ends.partition_point(|&end| {
        if let Some(fitted) = try_fit(end) {
            last_fitting = Some((end, fitted));
            true
        } else {
            false
        }
    });
    if cut == 0 {
        return None;
    }
    let target = ends[cut - 1];
    if let Some((end, fitted)) = last_fitting
        && end == target
    {
        Some(fitted)
    } else {
        try_fit(target)
    }
}

fn ellipsize(
    shaped: &ShapedText,
    key: &EllipsisKey,
    window: &Window,
    cx: &App,
) -> Option<ShapedText> {
    let shape = |value: SharedString| {
        shape_text(
            value,
            key.style,
            &key.typography,
            key.font_scale,
            key.tint,
            window,
            cx,
        )
        .expect("single-line text")
    };
    let ellipsis = shape("…".into());
    if ellipsis.line().width > key.width {
        return None;
    }
    let mut ends = shaped
        .line()
        .runs
        .iter()
        .flat_map(|run| &run.glyphs)
        .map(|glyph| glyph.index)
        .filter(|&index| index > 0 && key.text.is_char_boundary(index))
        .collect::<Vec<_>>();
    ends.sort_unstable();
    ends.dedup();
    find_ellipsis_fit(&ends, |end| {
        let line = shape(format!("{}…", &key.text[..end]).into());
        (line.line().width <= key.width).then_some(line)
    })
    .or(Some(ellipsis))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ellipsis_cache_reuses_results_and_misses_on_layout_changes() {
        let cache = EllipsisCache::default();
        let key = || EllipsisKey {
            text: "office 中文 العربية e\u{301} 👨‍👩‍👧‍👦".into(),
            width: px(120.),
            style: TypeRole::BodyLarge.style(TypographyEmphasis::Baseline),
            typography: Typography::default(),
            font_scale: 1.,
            scale_factor: 1.,
            font_revision: 0,
            tint: gpui::white(),
        };
        assert!(cache.get_or_shape(key(), |_| None).is_none());
        assert!(
            cache
                .get_or_shape(key(), |_| panic!("same layout must be reused"))
                .is_none()
        );

        for changed in [
            EllipsisKey {
                width: px(121.),
                ..key()
            },
            EllipsisKey {
                font_scale: 1.25,
                ..key()
            },
            EllipsisKey {
                scale_factor: 2.,
                ..key()
            },
            EllipsisKey {
                font_revision: 1,
                ..key()
            },
            EllipsisKey {
                tint: gpui::black(),
                ..key()
            },
        ] {
            let mut missed = false;
            cache.get_or_shape(changed, |_| {
                missed = true;
                None
            });
            assert!(missed);
        }
    }

    #[test]
    fn binary_search_ellipsis_fit_matches_linear_reverse_truncation() {
        let text =
            "Material 3 Expressive 中文排版与省略号测试 — longer sample for logarithmic search";
        let ends: Vec<usize> = text
            .char_indices()
            .map(|(ix, _)| ix)
            .filter(|&ix| ix > 0)
            .collect();
        let max_probes = (usize::BITS - ends.len().leading_zeros()) as usize + 1;
        assert_eq!(find_ellipsis_fit(&[], Some), None);
        for max_bytes in 0..=text.len() + 5 {
            let linear = ends.iter().copied().rev().find(|&end| end <= max_bytes);
            let mut probes = 0usize;
            let binary = find_ellipsis_fit(&ends, |end| {
                probes += 1;
                (end <= max_bytes).then_some(end)
            });
            assert_eq!(binary, linear, "mismatch for max_bytes={max_bytes}");
            assert!(
                probes <= max_probes,
                "expected O(log N) probes <= {max_probes}, got {probes}"
            );
        }
    }

    #[test]
    fn text_shorthands_and_muted_set_expected_role_and_color() {
        let cases = [
            (Text::display_large("a"), TypeRole::DisplayLarge),
            (Text::display_medium("a"), TypeRole::DisplayMedium),
            (Text::display_small("a"), TypeRole::DisplaySmall),
            (Text::headline_large("a"), TypeRole::HeadlineLarge),
            (Text::headline_medium("a"), TypeRole::HeadlineMedium),
            (Text::headline_small("a"), TypeRole::HeadlineSmall),
            (Text::title_large("a"), TypeRole::TitleLarge),
            (Text::title_medium("a"), TypeRole::TitleMedium),
            (Text::title_small("a"), TypeRole::TitleSmall),
            (Text::body_large("a"), TypeRole::BodyLarge),
            (Text::body_medium("a"), TypeRole::BodyMedium),
            (Text::body_small("a"), TypeRole::BodySmall),
            (Text::label_large("a"), TypeRole::LabelLarge),
            (Text::label_medium("a"), TypeRole::LabelMedium),
            (Text::label_small("a"), TypeRole::LabelSmall),
        ];
        for (text, expected_role) in cases {
            assert_eq!(text.role, expected_role);
            assert_eq!(text.color, None);
        }
        let muted = Text::body_medium("hint").muted();
        assert_eq!(muted.role, TypeRole::BodyMedium);
        assert_eq!(muted.color, Some(ColorRole::OnSurfaceVariant));
    }
}
