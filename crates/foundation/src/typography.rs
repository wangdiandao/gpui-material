use std::{collections::HashSet, sync::Arc};

use gpui::{
    App, Font, FontFallbacks, FontWeight, Global, Hsla, Pixels, Point, ShapedLine, SharedString,
    TextAlign, TextRun, Window, font, px,
};
use gpui_material_theme::{LayoutPolicy, TypeStyle, Typography};

struct InstalledFonts(HashSet<String>);

impl Global for InstalledFonts {}

impl InstalledFonts {
    fn enumerate(cx: &App) -> Self {
        Self(
            cx.text_system()
                .all_font_names()
                .into_iter()
                .map(|name| name.to_ascii_lowercase())
                .collect(),
        )
    }
}

pub(crate) fn refresh_installed_fonts(cx: &mut App) {
    cx.set_global(InstalledFonts::enumerate(cx));
}

/// Resolves the first available Material font family for a text style.
///
/// Material snapshots installed families when setting the application or window
/// theme. Set the theme again after registering additional fonts with GPUI.
pub fn font_for_style(typography: &Typography, style: TypeStyle, cx: &App) -> Font {
    let families = typography.families(style.typeface);
    let uncached;
    let installed = match cx.try_global::<InstalledFonts>() {
        Some(installed) => installed,
        None => {
            uncached = InstalledFonts::enumerate(cx);
            &uncached
        }
    };
    let family = std::iter::once(&families.primary)
        .chain(&families.fallback)
        .find(|name| {
            name.as_str() == ".SystemUIFont" || installed.0.contains(&name.to_ascii_lowercase())
        })
        .map_or(".SystemUIFont", String::as_str);
    let mut font = font(family.to_owned());
    font.weight = FontWeight(f32::from(style.weight));
    font.fallbacks = Some(FontFallbacks::from_fonts(families.fallback.clone()));
    font
}

/// A single line shaped by GPUI. The same glyph layout supplies metrics and paint.
pub struct ShapedText {
    line: ShapedLine,
    line_height: Pixels,
}

impl ShapedText {
    pub fn line(&self) -> &ShapedLine {
        &self.line
    }

    pub fn line_height(&self) -> Pixels {
        self.line_height
    }

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
    let style = LayoutPolicy::scale_text(style, font_scale);
    let run = TextRun {
        len: text.len(),
        font: font_for_style(typography, style, cx),
        color,
        ..Default::default()
    };
    let mut line = window
        .text_system()
        .shape_line(text, px(style.size), &[run], None);
    if style.tracking != 0. {
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
        *line = Arc::new(layout);
    }
    Some(ShapedText {
        line,
        line_height: px(style.line_height),
    })
}
