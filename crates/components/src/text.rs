use std::{
    cell::RefCell,
    hash::{DefaultHasher, Hash, Hasher},
    rc::Rc,
};

use gpui::{
    App, ElementId, Global, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels,
    RenderOnce, Role, SharedString, StatefulInteractiveElement, Styled, TextAlign, Window, canvas,
    div, px,
};
use gpui_base::TestSupportExt;
use gpui_material_foundation::{
    ShapedText, color, font_cache_revision, shape_text, window_material_theme, wrapped_text,
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
    max_width: Option<gpui::Pixels>,
    wrap: bool,
}

impl Text {
    pub fn new(value: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            id: None,
            role: TypeRole::BodyLarge,
            emphasis: TypographyEmphasis::Baseline,
            font_scale: 1.,
            color: None,
            max_width: None,
            wrap: false,
        }
    }

    /// Supplies a stable ID for this label within its parent.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn role(mut self, role: TypeRole) -> Self {
        self.role = role;
        self
    }

    pub fn emphasized(mut self, emphasized: bool) -> Self {
        self.emphasis = if emphasized {
            TypographyEmphasis::Emphasized
        } else {
            TypographyEmphasis::Baseline
        };
        self
    }

    pub fn font_scale(mut self, scale: f32) -> Self {
        self.font_scale = scale;
        self
    }

    pub fn color(mut self, role: ColorRole) -> Self {
        self.color = Some(role);
        self
    }

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
        let tint = self.color.map_or_else(
            || window.text_style().color,
            |role| color(window_material_theme(window, cx).colors.color(role)),
        );
        let text = if self.wrap {
            wrapped_text(
                self.value.clone(),
                self.role.style(self.emphasis),
                self.font_scale,
                tint,
                window,
                cx,
            )
        } else {
            single_line(
                self.value.clone(),
                self.role.style(self.emphasis),
                self.font_scale,
                tint,
                (self.max_width, 1., 1.),
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

pub(crate) fn single_line_width(
    value: &SharedString,
    style: TypeStyle,
    tint: Hsla,
    window: &Window,
    cx: &App,
) -> Pixels {
    let theme = window_material_theme(window, cx);
    shape_text(
        clean_line(value.clone()),
        style,
        &theme.typography,
        1.,
        tint,
        window,
        cx,
    )
    .expect("single-line text")
    .line()
    .width
    .ceil()
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
    let clean = clean_line(value);
    let theme = window_material_theme(window, cx);
    let typography = theme.typography.clone();
    let align = if theme.layout.direction == Direction::Rtl {
        TextAlign::Right
    } else {
        TextAlign::Left
    };
    let shaped = shape_text(
        clean.clone(),
        style,
        &typography,
        font_scale,
        tint,
        window,
        cx,
    )
    .expect("single-line text");
    let width = shaped.line().width.ceil();
    let height = shaped.line_height();
    div()
        .min_w_0()
        .w(max_width.map_or(width, |max| width.min(max)) * expansion)
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
        .filter(|&index| key.text.is_char_boundary(index))
        .collect::<Vec<_>>();
    ends.sort_unstable();
    ends.dedup();
    ends.into_iter()
        .rev()
        .filter(|&end| end > 0)
        .map(|end| shape(format!("{}…", &key.text[..end]).into()))
        .find(|line| line.line().width <= key.width)
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
}
