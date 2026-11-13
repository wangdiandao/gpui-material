use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use serde::Deserialize;
use serde_json::Value;

use crate::Result;

const INPUT: &str = "tokens/material.json";
const RUST: &str = "crates/theme/src/generated.rs";
const INDEX: &str = "docs/theme/token-index.md";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    groups: Vec<Group>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Group {
    source: String,
    tokens: Vec<Token>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum Kind {
    Argb,
    ColorRole,
    Corners,
    Dp,
    DpOffset,
    Easing,
    Ms,
    Opacity,
    Polygon,
    Sp,
    Spring,
    Typeface,
    TypeStyle,
    Weight,
}

#[derive(Deserialize)]
#[serde(tag = "shape", rename_all = "kebab-case", deny_unknown_fields)]
enum Polygon {
    Circle {
        vertices: u8,
        radius: f32,
    },
    Rectangle {
        width: f32,
        height: f32,
        rounding_radius: f32,
        smoothing: f32,
    },
    Regular {
        vertices: u8,
        radius: f32,
        rounding_radius: f32,
        smoothing: f32,
        rotation_degrees: f32,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Token {
    name: String,
    kind: Kind,
    value: Option<Value>,
    #[serde(rename = "ref")]
    reference: Option<String>,
    upstream: String,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Layer {
    Reference,
    System,
    Component,
}

impl Token {
    fn layer(&self) -> Layer {
        match self.name.split('.').nth(1) {
            Some("ref") => Layer::Reference,
            Some("sys") => Layer::System,
            Some("comp") => Layer::Component,
            _ => unreachable!("token name has been validated"),
        }
    }
}

impl Kind {
    fn rust_type(self) -> &'static str {
        match self {
            Self::Argb => "Argb",
            Self::ColorRole => "ColorRole",
            Self::Corners => "CornerShape",
            Self::Dp | Self::DpOffset | Self::Opacity | Self::Sp => "f32",
            Self::Easing => "CubicBezier",
            Self::Ms => "u16",
            Self::Polygon => "PolygonSpec",
            Self::Spring => "Spring",
            Self::Typeface => "Typeface",
            Self::TypeStyle => "TypeStyle",
            Self::Weight => "u16",
        }
    }

    fn variant(self) -> &'static str {
        match self {
            Self::Argb => "Color",
            Self::ColorRole => "ColorRole",
            Self::Corners => "Corners",
            Self::Dp => "Dp",
            Self::DpOffset => "DpOffset",
            Self::Easing => "Easing",
            Self::Ms => "Milliseconds",
            Self::Opacity => "Opacity",
            Self::Polygon => "Polygon",
            Self::Sp => "Sp",
            Self::Spring => "Spring",
            Self::Typeface => "Typeface",
            Self::TypeStyle => "TypeStyle",
            Self::Weight => "Weight",
        }
    }
}

pub(crate) fn run(check: bool) -> Result {
    let manifest = serde_json::from_str::<Manifest>(&fs::read_to_string(INPUT)?)?;
    let (rust, index) = generate(&manifest)?;
    for (path, contents) in [(RUST, rust), (INDEX, index)] {
        if check {
            let matches = fs::read_to_string(path)
                .is_ok_and(|current| current.replace("\r\n", "\n") == contents);
            if !matches {
                return Err(format!("{path} is stale. Run cargo xtask tokens").into());
            }
        } else if !Path::new(path).exists()
            || fs::read_to_string(path)?.replace("\r\n", "\n") != contents
        {
            fs::write(path, contents)?;
        }
    }
    println!(
        "Material tokens verified ({})",
        if check { "check" } else { "generate" }
    );
    Ok(())
}

fn generate(manifest: &Manifest) -> Result<(String, String)> {
    use std::fmt::Write as _;

    let mut tokens = BTreeMap::new();
    let mut identifiers = BTreeSet::new();
    for group in &manifest.groups {
        if !(group.source.starts_with("https://") || group.source == "docs/theme/theme.md")
            || group
                .source
                .contains(|c: char| c.is_whitespace() || "()|<>".contains(c))
            || group.tokens.is_empty()
        {
            return Err(format!("invalid source group: {}", group.source).into());
        }
        for token in &group.tokens {
            let parts = token.name.split('.').collect::<Vec<_>>();
            if !matches!(parts.as_slice(), ["md", "ref" | "sys" | "comp", _, ..])
                || !parts.iter().all(|part| {
                    part.split('-').all(|segment| {
                        !segment.is_empty()
                            && segment
                                .bytes()
                                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
                    })
                })
                || token.upstream.is_empty()
                || token.upstream.contains(['\n', '\r', '|', '\x60'])
            {
                return Err(format!("invalid name or upstream symbol: {}", token.name).into());
            }
            if tokens
                .insert(token.name.as_str(), (token, group.source.as_str()))
                .is_some()
                || !identifiers.insert(constant(&token.name))
            {
                return Err(format!("duplicate name or Rust identifier: {}", token.name).into());
            }
        }
    }
    if tokens.is_empty() {
        return Err("no Material tokens".into());
    }

    let mut expressions = BTreeMap::new();
    let mut visiting = BTreeSet::new();
    for &(token, _) in tokens.values() {
        compile(token, &tokens, &mut visiting, &mut expressions)?;
    }

    let mut rust =
        String::from("// Generated by cargo xtask tokens from tokens/material.json.\n\n");
    let mut roles = Vec::new();
    for &(token, _) in tokens.values() {
        if token.kind == Kind::ColorRole && token.value.is_some() {
            roles.push(token);
        }
        writeln!(
            rust,
            "pub(crate) const {}: {} = {};",
            constant(&token.name),
            token.kind.rust_type(),
            expressions[token.name.as_str()]
        )?;
    }
    write_roles(&mut rust, &roles)?;
    write_type_styles(&mut rust, &tokens)?;
    writeln!(
        rust,
        "\npub(crate) static TOKENS: &[(&str, TokenValue)] = &["
    )?;
    for &(token, _) in tokens.values() {
        writeln!(
            rust,
            "    ({:?}, TokenValue::{}({})),",
            token.name,
            token.kind.variant(),
            constant(&token.name)
        )?;
    }
    writeln!(rust, "];")?;

    let sources: BTreeSet<_> = tokens.values().map(|(_, source)| *source).collect();
    let sources: BTreeMap<_, _> = sources
        .into_iter()
        .enumerate()
        .map(|(i, s)| (s, i + 1))
        .collect();
    let mut index = String::from(
        "# Material token index\n\nGenerated from the token manifest by cargo xtask tokens.\n\n\
         | Token | Rust constant | Upstream symbol | Source |\n| --- | --- | --- | --- |\n",
    );
    for &(token, source) in tokens.values() {
        writeln!(
            index,
            "| \x60{}\x60 | \x60{}\x60 | \x60{}\x60 | [S{}][s{}] |",
            token.name,
            constant(&token.name),
            token.upstream,
            sources[source],
            sources[source]
        )?;
    }
    writeln!(index)?;
    for (source, id) in sources {
        let destination = source.strip_prefix("docs/theme/").unwrap_or(source);
        writeln!(index, "[s{id}]: {destination}")?;
    }

    Ok((rust, index))
}

type Tokens<'a> = BTreeMap<&'a str, (&'a Token, &'a str)>;

fn compile<'a>(
    token: &'a Token,
    tokens: &Tokens<'a>,
    visiting: &mut BTreeSet<&'a str>,
    compiled: &mut BTreeMap<&'a str, String>,
) -> Result {
    if compiled.contains_key(token.name.as_str()) {
        return Ok(());
    }
    if !visiting.insert(&token.name) {
        return Err(format!("token reference cycle at {}", token.name).into());
    }
    let mut reference = |name: &str, kind| -> Result<String> {
        let &(target, _) = tokens
            .get(name)
            .ok_or_else(|| format!("{} references missing token {name}", token.name))?;
        if target.kind != kind {
            return Err(format!(
                "{} expects {kind:?}, but {name} has type {:?}",
                token.name, target.kind
            )
            .into());
        }
        if target.layer() > token.layer()
            || (target.layer() == Layer::Component && token.layer() == Layer::Component)
        {
            return Err(format!("{} has an invalid layer reference to {name}", token.name).into());
        }
        compile(target, tokens, visiting, compiled)?;
        Ok(constant(name))
    };
    let expression = match (&token.value, &token.reference) {
        (None, Some(name)) => reference(name, token.kind)?,
        (Some(value), None) => literal(token, value, &mut reference)?,
        _ => return Err(format!("{} requires exactly one of value or ref", token.name).into()),
    };
    visiting.remove(token.name.as_str());
    compiled.insert(&token.name, expression);
    Ok(())
}

fn literal(
    token: &Token,
    value: &Value,
    reference: &mut impl FnMut(&str, Kind) -> Result<String>,
) -> Result<String> {
    use Kind::*;

    let invalid = || {
        format!(
            "{} has an invalid {:?} value: {value}",
            token.name, token.kind
        )
    };
    let number = || {
        value
            .as_f64()
            .map(|n| n as f32)
            .filter(|n| n.is_finite())
            .ok_or_else(invalid)
    };
    let string = || value.as_str().ok_or_else(invalid);
    Ok(match token.kind {
        Argb => {
            let hex = string()?;
            if hex.len() != 8 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(invalid().into());
            }
            format!("Argb::from_u32(0x{})", hex.to_ascii_lowercase())
        }
        ColorRole => {
            let [light, dark]: [String; 2] =
                serde_json::from_value(value.clone()).map_err(|_| invalid())?;
            let name = role_name(token)?;
            if light != format!("md.sys.color.light.{name}")
                || dark != format!("md.sys.color.dark.{name}")
            {
                return Err(invalid().into());
            }
            reference(&light, Argb)?;
            reference(&dark, Argb)?;
            format!("ColorRole::{}", variant(name))
        }
        Corners if value.as_str() == Some("full") => "CornerShape::Full".into(),
        Corners => {
            let corners: [String; 4] =
                serde_json::from_value(value.clone()).map_err(|_| invalid())?;
            let radii = corners
                .iter()
                .map(|name| reference(name, Dp))
                .collect::<Result<Vec<_>>>()?;
            format!("CornerShape::Radii([{}])", radii.join(", "))
        }
        Dp if number()? >= 0.0 => format!("{:?}", number()?),
        DpOffset | Sp => format!("{:?}", number()?),
        Opacity if (0.0..=1.0).contains(&number()?) => format!("{:?}", number()?),
        Ms | Weight => {
            let limit = if token.kind == Weight {
                1000
            } else {
                u16::MAX.into()
            };
            let n = value
                .as_u64()
                .filter(|n| (1..=limit).contains(n))
                .ok_or_else(invalid)?;
            n.to_string()
        }
        Easing => {
            let [x1, y1, x2, y2]: [f32; 4] =
                serde_json::from_value(value.clone()).map_err(|_| invalid())?;
            if ![y1, y2].into_iter().all(f32::is_finite)
                || ![x1, x2].into_iter().all(|x| (0.0..=1.0).contains(&x))
            {
                return Err(invalid().into());
            }
            format!("CubicBezier {{ x1: {x1:?}, y1: {y1:?}, x2: {x2:?}, y2: {y2:?} }}")
        }
        Spring => {
            let [damping_ratio, stiffness]: [f32; 2] =
                serde_json::from_value(value.clone()).map_err(|_| invalid())?;
            if ![damping_ratio, stiffness]
                .into_iter()
                .all(|n| n.is_finite() && n > 0.0)
            {
                return Err(invalid().into());
            }
            format!("Spring {{ damping_ratio: {damping_ratio:?}, stiffness: {stiffness:?} }}")
        }
        Polygon => polygon_expression(value).ok_or_else(invalid)?,
        TypeStyle => {
            let [font, size, line_height, tracking, weight]: [String; 5] =
                serde_json::from_value(value.clone()).map_err(|_| invalid())?;
            let font = reference(&font, Typeface)?;
            let size = reference(&size, Sp)?;
            let line_height = reference(&line_height, Sp)?;
            let tracking = reference(&tracking, Sp)?;
            let weight = reference(&weight, Weight)?;
            format!(
                "TypeStyle {{ typeface: {font}, size: {size}, line_height: {line_height}, tracking: {tracking}, weight: {weight} }}"
            )
        }
        Typeface => match string()? {
            "brand" => "Typeface::Brand".into(),
            "plain" => "Typeface::Plain".into(),
            _ => return Err(invalid().into()),
        },
        _ => return Err(invalid().into()),
    })
}

fn polygon_expression(value: &Value) -> Option<String> {
    let positive = |value: f32| value.is_finite() && value > 0.0;
    let rounding = |radius: f32, smoothing| {
        radius.is_finite() && radius >= 0.0 && (0.0..=1.0).contains(&smoothing)
    };
    match serde_json::from_value(value.clone()).ok()? {
        Polygon::Circle { vertices, radius } if vertices >= 3 && positive(radius) => Some(format!(
            "PolygonSpec::Circle {{ vertices: {vertices}, radius: {radius:?} }}"
        )),
        Polygon::Rectangle {
            width,
            height,
            rounding_radius,
            smoothing,
        } if positive(width) && positive(height) && rounding(rounding_radius, smoothing) => {
            Some(format!(
                "PolygonSpec::Rectangle {{ width: {width:?}, height: {height:?}, rounding_radius: {rounding_radius:?}, smoothing: {smoothing:?} }}"
            ))
        }
        Polygon::Regular {
            vertices,
            radius,
            rounding_radius,
            smoothing,
            rotation_degrees,
        } if vertices >= 3
            && positive(radius)
            && rotation_degrees.is_finite()
            && rounding(rounding_radius, smoothing) =>
        {
            Some(format!(
                "PolygonSpec::Regular {{ vertices: {vertices}, radius: {radius:?}, rounding_radius: {rounding_radius:?}, smoothing: {smoothing:?}, rotation_degrees: {rotation_degrees:?} }}"
            ))
        }
        _ => None,
    }
}

fn role_name(token: &Token) -> Result<&str> {
    token
        .name
        .strip_prefix("md.sys.color.")
        .filter(|name| !name.contains('.') && name.starts_with(|c: char| c.is_ascii_lowercase()))
        .ok_or_else(|| format!("invalid color role: {}", token.name).into())
}

fn write_roles(rust: &mut String, roles: &[&Token]) -> Result {
    use std::fmt::Write as _;

    writeln!(rust, "\n/// Semantic Material color roles.")?;
    writeln!(
        rust,
        "#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]"
    )?;
    writeln!(
        rust,
        "#[cfg_attr(feature = \"serde\", derive(serde::Serialize, serde::Deserialize))]"
    )?;
    writeln!(rust, "pub enum ColorRole {{")?;
    for token in roles {
        writeln!(rust, "    {},", variant(role_name(token)?))?;
    }
    writeln!(rust, "}}\n")?;
    writeln!(rust, "impl ColorRole {{")?;
    writeln!(
        rust,
        "    /// All supported roles, in a stable order for this crate version."
    )?;
    writeln!(rust, "    pub const ALL: [Self; {}] = [", roles.len())?;
    for token in roles {
        writeln!(rust, "        Self::{},", variant(role_name(token)?))?;
    }
    writeln!(rust, "    ];\n}}")?;
    for mode in ["light", "dark"] {
        writeln!(
            rust,
            "\npub(crate) const {}: [Argb; ColorRole::ALL.len()] = [",
            mode.to_uppercase()
        )?;
        for token in roles {
            writeln!(
                rust,
                "    {},",
                constant(&format!("md.sys.color.{mode}.{}", role_name(token)?))
            )?;
        }
        writeln!(rust, "];")?;
    }
    writeln!(rust, "\n#[cfg(feature = \"dynamic-color\")]")?;
    writeln!(
        rust,
        "pub(crate) fn dynamic_colors(scheme: &material_color_utils::dynamic::dynamic_scheme::DynamicScheme) -> [Argb; ColorRole::ALL.len()] {{"
    )?;
    writeln!(rust, "    [")?;
    for token in roles {
        writeln!(
            rust,
            "        Argb::from_u32(scheme.{}().0),",
            role_name(token)?.replace('-', "_")
        )?;
    }
    writeln!(rust, "    ]\n}}")?;
    Ok(())
}

fn write_type_styles(rust: &mut String, tokens: &Tokens<'_>) -> Result {
    use std::fmt::Write as _;

    for (mode, prefix) in [
        ("BASELINE", "md.sys.typescale."),
        ("EMPHASIZED", "md.sys.typescale.emphasized."),
    ] {
        writeln!(
            rust,
            "\npub(crate) const {mode}_TYPE_STYLES: [TypeStyle; 15] = ["
        )?;
        for family in ["body", "display", "headline", "label", "title"] {
            for size in ["large", "medium", "small"] {
                let name = format!("{prefix}{family}-{size}");
                if !tokens
                    .get(name.as_str())
                    .is_some_and(|(token, _)| token.kind == Kind::TypeStyle)
                {
                    return Err(format!("missing type style {name}").into());
                }
                writeln!(rust, "    {},", constant(&name))?;
            }
        }
        writeln!(rust, "];")?;
    }
    Ok(())
}

fn constant(name: &str) -> String {
    name.replace(['.', '-'], "_").to_ascii_uppercase()
}

fn variant(name: &str) -> String {
    name.split('-')
        .map(|part| {
            let (first, rest) = part.split_at(1);
            first.to_ascii_uppercase() + rest
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn error(tokens: &str) -> String {
        let manifest = serde_json::from_str(&format!(
            r#"{{"groups":[{{"source":"https://example.com/revision","tokens":[{tokens}]}}]}}"#
        ))
        .unwrap();
        generate(&manifest).unwrap_err().to_string()
    }

    #[test]
    fn checks_reference_graph_and_types() {
        assert!(
            error(r#"{"name":"md.sys.a","kind":"dp","ref":"md.ref.absent","upstream":"A"}"#)
                .contains("missing token")
        );
        assert!(
            error(
                r#"
            {"name":"md.sys.a","kind":"dp","ref":"md.sys.b","upstream":"A"},
            {"name":"md.sys.b","kind":"dp","ref":"md.sys.a","upstream":"B"}
        "#
            )
            .contains("cycle")
        );
        assert!(
            error(
                r#"
            {"name":"md.sys.a","kind":"dp","ref":"md.ref.b","upstream":"A"},
            {"name":"md.ref.b","kind":"sp","value":2,"upstream":"B"}
        "#
            )
            .contains("has type Sp")
        );
        assert!(
            error(
                r#"
            {"name":"md.ref.a","kind":"dp","ref":"md.sys.b","upstream":"A"},
            {"name":"md.sys.b","kind":"dp","value":2,"upstream":"B"}
        "#
            )
            .contains("invalid layer")
        );
    }

    #[test]
    fn rejects_ambiguous_or_invalid_data() {
        assert!(
            error(
                r#"
            {"name":"md.ref.a.b","kind":"dp","value":1,"upstream":"A"},
            {"name":"md.ref.a-b","kind":"dp","value":2,"upstream":"B"}
        "#
            )
            .contains("duplicate")
        );
        assert!(
            error(r#"{"name":"md.sys.a","kind":"dp","value":-1,"upstream":"A"}"#)
                .contains("invalid Dp")
        );
        assert!(
            error(r#"{"name":"md.sys.a","kind":"dp","value":1,"ref":"md.ref.b","upstream":"A"}"#)
                .contains("exactly one")
        );
    }
}
