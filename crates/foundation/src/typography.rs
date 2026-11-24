use std::{cell::RefCell, collections::HashSet, sync::Arc};

use gpui::{
    App, Font, FontFallbacks, FontWeight, Global, Hsla, ParentElement, Pixels, Point, ShapedLine,
    SharedString, Styled, TextAlign, TextRun, Window, div, px,
};
use gpui_material_theme::{Direction, FontFamilies, LayoutPolicy, TypeStyle, Typography};

/// Native wrapping with Material font metrics and fallback, including explicit
/// line breaks. GPUI owns shaping, Unicode line breaking, measurement and paint.
/// The pinned GPUI has no letter-spacing option for wrapped text; unlike
/// [`shape_text`], this path uses the font's native tracking.
pub fn wrapped_text(
    text: impl Into<SharedString>,
    style: TypeStyle,
    font_scale: f32,
    tint: Hsla,
    window: &Window,
    cx: &App,
) -> gpui::Div {
    text_style(div().min_w_0(), style, font_scale, window, cx)
        .text_color(tint)
        .whitespace_normal()
        .child(text.into())
}

/// Applies Material family, fallback, weight, size, line height and direction to
/// native GPUI text. Native wrapping does not support custom tracking.
pub fn text_style<E: Styled>(
    element: E,
    style: TypeStyle,
    font_scale: f32,
    window: &Window,
    cx: &App,
) -> E {
    let theme = crate::current_material_theme(window, cx);
    let style = LayoutPolicy::scale_text(theme.typography.scale_style(style), font_scale);
    let font = font_for_style(&theme.typography, style, cx);
    element
        .font(font)
        .text_size(px(style.size))
        .line_height(px(style.line_height))
        .text_align(if theme.layout.direction == Direction::Rtl {
            TextAlign::Right
        } else {
            TextAlign::Left
        })
}

type ResolvedFontEntry = (FontFamilies, SharedString, Option<FontFallbacks>);

struct InstalledFonts {
    families: HashSet<String>,
    revision: u64,
    resolved: RefCell<Vec<ResolvedFontEntry>>,
}

impl Global for InstalledFonts {}

thread_local! {
    static FALLBACK_INSTALLED_FONTS: RefCell<Option<InstalledFonts>> = const { RefCell::new(None) };
}

impl InstalledFonts {
    fn enumerate(cx: &App) -> Self {
        Self {
            families: cx
                .text_system()
                .all_font_names()
                .into_iter()
                .map(|name| name.to_ascii_lowercase())
                .collect(),
            revision: font_cache_revision(cx).wrapping_add(1),
            resolved: RefCell::new(Vec::new()),
        }
    }

    fn resolve_family(&self, families: &FontFamilies) -> (SharedString, Option<FontFallbacks>) {
        if let Some((_, family, fallbacks)) = self
            .resolved
            .borrow()
            .iter()
            .find(|(cached, _, _)| cached == families)
        {
            return (family.clone(), fallbacks.clone());
        }
        let family = std::iter::once(&families.primary)
            .chain(families.fallback.iter())
            .find(|name| {
                name.as_ref() == ".SystemUIFont"
                    || self.families.contains(&name.to_ascii_lowercase())
            })
            .map_or_else(
                || SharedString::new_static(".SystemUIFont"),
                |name| SharedString::from(name.as_ref().to_owned()),
            );
        let fallbacks = Some(FontFallbacks::from_fonts(
            families
                .fallback
                .iter()
                .map(|name| name.as_ref().to_owned())
                .collect(),
        ));
        let mut resolved = self.resolved.borrow_mut();
        if resolved.len() < 32 {
            resolved.push((families.clone(), family.clone(), fallbacks.clone()));
        }
        (family, fallbacks)
    }
}

/// Refreshes the installed-font snapshot and bumps [`font_cache_revision`].
/// Returns `true` when the set of installed font families changed from the previous snapshot.
pub(crate) fn refresh_installed_fonts(cx: &mut App) -> bool {
    let next = InstalledFonts::enumerate(cx);
    let changed = cx
        .try_global::<InstalledFonts>()
        .is_none_or(|prev| prev.families != next.families);
    cx.set_global(next);
    FALLBACK_INSTALLED_FONTS.with_borrow_mut(|slot| *slot = None);
    changed
}

/// Cache invalidation key for Material's installed-font snapshot. Setting an
/// application or window theme refreshes this key, even for identical values,
/// so registering fonts and reapplying a theme invalidates retained metrics.
#[doc(hidden)]
pub fn font_cache_revision(cx: &App) -> u64 {
    cx.try_global::<InstalledFonts>()
        .map_or(0, |fonts| fonts.revision)
}

/// Resolves the first available Material font family for a text style.
///
/// Material snapshots installed families when setting the application or window
/// theme. Set the theme again after registering additional fonts with GPUI.
pub fn font_for_style(typography: &Typography, style: TypeStyle, cx: &App) -> Font {
    let families = typography.families(style.typeface);
    let (family, fallbacks) = match cx.try_global::<InstalledFonts>() {
        Some(installed) => installed.resolve_family(families),
        None => FALLBACK_INSTALLED_FONTS.with_borrow_mut(|slot| {
            slot.get_or_insert_with(|| InstalledFonts::enumerate(cx))
                .resolve_family(families)
        }),
    };
    Font {
        family,
        features: Default::default(),
        fallbacks,
        weight: FontWeight(f32::from(style.weight)),
        style: Default::default(),
    }
}

/// A single line shaped by GPUI. The same glyph layout supplies metrics and paint.
pub struct ShapedText {
    line: ShapedLine,
    line_height: Pixels,
}

impl ShapedText {
    /// Returns the underlying GPUI [`ShapedLine`].
    pub fn line(&self) -> &ShapedLine {
        &self.line
    }

    /// Returns the resolved line height in logical pixels.
    pub fn line_height(&self) -> Pixels {
        self.line_height
    }

    /// Paints the shaped text line at `origin` with the given horizontal alignment.
    pub fn paint(
        &self,
        origin: Point<Pixels>,
        align: TextAlign,
        align_width: Option<Pixels>,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui::Result<()> {
        self.line
            .paint(origin, self.line_height, align, align_width, window, cx)
    }
}

const TRACKED_LAYOUT_CACHE_LEN: usize = 512;
type TrackedLayoutSlot = Option<(Arc<gpui::LineLayout>, u32, Arc<gpui::LineLayout>)>;

struct TrackedLayoutCache {
    revision: u64,
    slots: Vec<TrackedLayoutSlot>,
}

impl TrackedLayoutCache {
    fn new() -> Self {
        Self {
            revision: 0,
            slots: vec![None; TRACKED_LAYOUT_CACHE_LEN],
        }
    }

    fn sync_revision(&mut self, revision: u64) {
        if self.revision != revision {
            self.slots.fill(None);
            self.revision = revision;
        }
    }
}

thread_local! {
    static TRACKED_LAYOUT_CACHE: RefCell<TrackedLayoutCache> =
        RefCell::new(TrackedLayoutCache::new());
}

/// Shapes one line of application-localized text. Returns `None` for multiline input.
/// The caller supplies the current user font scale and may override font families.
/// Tracking is applied between shaped glyph clusters in visual order.
pub fn shape_text(
    text: impl Into<SharedString>,
    style: TypeStyle,
    typography: &Typography,
    font_scale: f32,
    color: Hsla,
    window: &Window,
    cx: &App,
) -> Option<ShapedText> {
    let text = text.into();
    if text.contains(['\r', '\n']) {
        return None;
    }
    let style = LayoutPolicy::scale_text(typography.scale_style(style), font_scale);
    let run = TextRun {
        len: text.len(),
        font: font_for_style(typography, style, cx),
        color,
        ..Default::default()
    };
    let mut line = window
        .text_system()
        .shape_line(text, px(style.size), &[run], None);
    if style.tracking != 0. && line.len() > 1 {
        let revision = font_cache_revision(cx);
        let tracking_bits = style.tracking.to_bits();
        let raw_ptr = Arc::as_ptr(&*line) as usize;
        let hash =
            (raw_ptr >> 4) ^ (raw_ptr >> 13) ^ (tracking_bits as usize).wrapping_mul(0x9e37_79b9);
        let slot_index = hash & (TRACKED_LAYOUT_CACHE_LEN - 1);
        let cached = TRACKED_LAYOUT_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            cache.sync_revision(revision);
            cache.slots[slot_index]
                .as_ref()
                .and_then(|(untracked, bits, tracked)| {
                    (*bits == tracking_bits && Arc::ptr_eq(untracked, &*line))
                        .then(|| tracked.clone())
                })
        });
        if let Some(tracked) = cached {
            *line = tracked;
        } else {
            let mut first_cluster = None;
            let has_multiple_clusters =
                line.runs
                    .iter()
                    .flat_map(|run| &run.glyphs)
                    .any(|glyph| match first_cluster {
                        None => {
                            first_cluster = Some(glyph.index);
                            false
                        }
                        Some(first) => glyph.index != first,
                    });
            let tracked = if has_multiple_clusters {
                let mut layout = gpui::LineLayout {
                    font_size: line.font_size,
                    width: line.width,
                    ascent: line.ascent,
                    descent: line.descent,
                    runs: line.runs.clone(),
                    len: line.len(),
                };
                let mut previous_cluster = None;
                let mut offset = px(0.);
                for glyph in layout.runs.iter_mut().flat_map(|run| &mut run.glyphs) {
                    if previous_cluster.is_some_and(|index| index != glyph.index) {
                        offset += px(style.tracking);
                    }
                    glyph.position.x += offset;
                    previous_cluster = Some(glyph.index);
                }
                layout.width = (layout.width + offset).max(px(0.));
                Arc::new(layout)
            } else {
                (*line).clone()
            };
            let untracked = std::mem::replace(&mut *line, tracked.clone());
            TRACKED_LAYOUT_CACHE.with(|cache| {
                let mut cache = cache.borrow_mut();
                cache.sync_revision(revision);
                cache.slots[slot_index] = Some((untracked, tracking_bits, tracked));
            });
        }
    }
    Some(ShapedText {
        line,
        line_height: px(style.line_height),
    })
}
