mod i18n;

use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path},
};

use markdown::{ParseOptions, mdast::Node, to_mdast};

use crate::{Result, output};

pub(crate) use i18n::write_records;

pub(crate) fn check() -> Result {
    let files = output("git", &["ls-files", "-z", "--", "*.md"])?;
    let mut count = 0;
    for name in files.split_terminator('\0') {
        let path = Path::new(name);
        let tree = parse(path)?;
        visit_links(&tree, &mut |url| {
            if let Err(error) = check_link(path, url) {
                return Err(format!("{name}: {url}: {error}").into());
            }
            count += 1;
            Ok(())
        })?;
    }
    println!("Checked {count} Markdown link destinations");
    i18n::check(&files)
}

fn parse(path: &Path) -> Result<Node> {
    let text = fs::read_to_string(path)?;
    to_mdast(&text, &ParseOptions::gfm())
        .map_err(|error| format!("{}: {error}", path.display()).into())
}

fn visit_links(node: &Node, visit: &mut impl FnMut(&str) -> Result) -> Result {
    match node {
        Node::Link(link) => visit(&link.url)?,
        Node::Image(image) => visit(&image.url)?,
        Node::Definition(definition) => visit(&definition.url)?,
        _ => {}
    }
    for child in node.children().into_iter().flatten() {
        visit_links(child, visit)?;
    }
    Ok(())
}

fn check_link(source: &Path, url: &str) -> Result {
    if url.starts_with("//") || url.split('/').next().is_some_and(|part| part.contains(':')) {
        return Ok(());
    }
    let (location, fragment) = url.split_once('#').unwrap_or((url, ""));
    let location = decode(location.split('?').next().unwrap_or_default())?;
    let path = if location.is_empty() {
        source.to_owned()
    } else {
        let location = Path::new(&location);
        let base = if location.has_root() {
            Path::new("")
        } else {
            source.parent().unwrap_or(Path::new(""))
        };
        let mut path = base.to_owned();
        for component in location.components() {
            match component {
                Component::RootDir | Component::CurDir => {}
                Component::ParentDir if path.pop() => {}
                Component::Normal(name) => path.push(name),
                _ => return Err("link escapes the repository".into()),
            }
        }
        path
    };
    if !path.exists() {
        return Err(format!("missing path {}", path.display()).into());
    }
    if !fragment.is_empty()
        && path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
    {
        let mut ids = BTreeSet::new();
        collect_anchors(&parse(&path)?, &mut ids);
        if !ids.contains(&decode(fragment)?) {
            return Err(format!("missing heading in {}", path.display()).into());
        }
    }
    Ok(())
}

fn collect_anchors(node: &Node, ids: &mut BTreeSet<String>) {
    if let Node::Html(html) = node
        && let Some(anchor) = html.value.trim().strip_prefix("<a id=\"")
        && let Some((id, closing)) = anchor.split_once("\">")
        && matches!(closing, "" | "</a>")
    {
        ids.insert(id.into());
    }
    if let Node::Heading(_) = node {
        let slug = heading_slug(node);
        ids.insert(unique_id(&slug, ids, "-"));
    }
    for child in node.children().into_iter().flatten() {
        collect_anchors(child, ids);
    }
}

fn heading_slug(node: &Node) -> String {
    let mut text = String::new();
    heading_text(node, &mut text);
    text.to_lowercase()
        .chars()
        .filter_map(|c| match c {
            ' ' => Some('-'),
            '-' | '_' => Some(c),
            c if c.is_alphanumeric() => Some(c),
            _ => None,
        })
        .collect()
}

fn unique_id(id: &str, ids: &BTreeSet<String>, separator: &str) -> String {
    let mut unique = id.to_owned();
    let mut suffix = 1;
    while ids.contains(&unique) {
        unique = format!("{id}{separator}{suffix}");
        suffix += 1;
    }
    unique
}

fn heading_text(node: &Node, text: &mut String) {
    match node {
        Node::Text(value) => text.push_str(&value.value),
        Node::InlineCode(value) => text.push_str(&value.value),
        Node::Image(value) => text.push_str(&value.alt),
        _ => {
            for child in node.children().into_iter().flatten() {
                heading_text(child, text);
            }
        }
    }
}

fn decode(value: &str) -> Result<String> {
    let mut decoded = Vec::with_capacity(value.len());
    let mut bytes = value.bytes();
    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            let mut hex = || bytes.next().and_then(|b| char::from(b).to_digit(16));
            let high = hex().ok_or("invalid percent escape")?;
            let low = hex().ok_or("invalid percent escape")?;
            decoded.push((high * 16 + low) as u8);
        } else {
            decoded.push(byte);
        }
    }
    Ok(String::from_utf8(decoded)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_unicode_and_duplicate_headings() {
        let tree = to_mdast(
            "# GPUI 与 Base\n\n<a id=\"gpui-and-base\"></a>\n\n# `GPUI` 与 *Base*",
            &ParseOptions::gfm(),
        )
        .unwrap();
        let mut ids = BTreeSet::new();
        collect_anchors(&tree, &mut ids);
        assert!(ids.contains("gpui-与-base"));
        assert!(ids.contains("gpui-与-base-1"));
        assert!(ids.contains("gpui-and-base"));
        assert_eq!(decode("gpui-%E4%B8%8E-base").unwrap(), "gpui-与-base");
    }

    #[test]
    fn checks_real_links_but_skips_code_samples() {
        let tree = to_mdast(
            "[local](p.md)\n\n~~~md\n[example](missing.md)\n~~~",
            &ParseOptions::gfm(),
        )
        .unwrap();
        let mut urls = Vec::new();
        visit_links(&tree, &mut |url| {
            urls.push(url.to_owned());
            Ok(())
        })
        .unwrap();
        assert_eq!(urls, ["p.md"]);
    }
}
