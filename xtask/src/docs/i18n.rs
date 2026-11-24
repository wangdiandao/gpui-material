use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
};

use markdown::mdast::Node;

use crate::{Result, output};

use super::{ParsedDoc, heading_slug, unique_id};

pub(super) fn check(markdown: &str, docs: &BTreeMap<PathBuf, ParsedDoc>) -> Result<Vec<String>> {
    let files = markdown.split_terminator('\0').collect::<BTreeSet<_>>();
    let records = output("git", &["ls-files", "-z", "--", "*.i18n.yaml"])?;
    let mut records = records.split_terminator('\0').collect::<BTreeSet<_>>();
    let mut count = 0;
    let mut errors = Vec::new();
    for path in &files {
        if !in_scope(path) {
            continue;
        }
        if let Some(stem) = path.strip_suffix(".zh.md") {
            if !files.contains(format!("{stem}.md").as_str()) {
                errors.push(format!("{path} has no tracked English counterpart"));
            }
            continue;
        }
        let pair = match Pair::new(path) {
            Ok(pair) => pair,
            Err(error) => {
                errors.push(error.to_string());
                continue;
            }
        };
        if !files.contains(pair.chinese.as_str()) {
            errors.push(format!("{path} has no tracked Chinese counterpart"));
            continue;
        }
        if !records.remove(pair.record.as_str()) {
            errors.push(format!("{path} has no tracked {}", pair.record));
            continue;
        }
        let expected = match pair.render(docs) {
            Ok(expected) => expected,
            Err(error) => {
                errors.push(error.to_string());
                continue;
            }
        };
        let current = match fs::read_to_string(&pair.record) {
            Ok(text) => text.replace("\r\n", "\n"),
            Err(error) => {
                errors.push(format!("{}: {error}", pair.record));
                continue;
            }
        };
        if current != expected {
            let mut section = "/";
            let mut current_lines = current.lines();
            for line in expected.lines() {
                if line.starts_with('/') {
                    section = line.trim_end_matches(':');
                }
                if current_lines.next() != Some(line) {
                    break;
                }
            }
            errors.push(format!(
                "{} is stale at {section}. Update both pages and run cargo xtask docs --write {path}",
                pair.record
            ));
            continue;
        }
        count += 1;
    }
    for path in records {
        errors.push(format!("unpaired documentation record: {path}"));
    }
    println!("Checked {count} bilingual Markdown pairs");
    Ok(errors)
}

pub(crate) fn write_records(paths: &[&str]) -> Result {
    let docs = BTreeMap::new();
    for path in paths {
        let normalized = path.replace('\\', "/");
        let pair = Pair::new(&normalized)?;
        fs::write(&pair.record, pair.render(&docs)?)?;
        println!("Updated {}", pair.record);
    }
    Ok(())
}

fn in_scope(path: &str) -> bool {
    matches!(
        path,
        "README.md"
            | "README.zh.md"
            | "CONTRIBUTING.md"
            | "CONTRIBUTING.zh.md"
            | "THIRD_PARTY_NOTICES.md"
            | "THIRD_PARTY_NOTICES.zh.md"
    ) || path.starts_with("docs/")
}

struct Pair<'a> {
    english: &'a str,
    chinese: String,
    record: String,
}

impl<'a> Pair<'a> {
    fn new(english: &'a str) -> Result<Self> {
        let stem = english
            .strip_suffix(".md")
            .ok_or("expected a Markdown path")?;
        if !in_scope(english)
            || stem.ends_with(".zh")
            || english
                .split('/')
                .any(|part| matches!(part, "" | "." | ".."))
        {
            return Err(format!("expected a repository-relative English page: {english}").into());
        }
        Ok(Self {
            english,
            chinese: format!("{stem}.zh.md"),
            record: format!("{stem}.i18n.yaml"),
        })
    }

    fn render(&self, docs: &BTreeMap<PathBuf, ParsedDoc>) -> Result<String> {
        let file_name = |path: &str| PathBuf::from(path).file_name().unwrap().to_owned();
        let en_switch = format!("English | [中文]({})", file_name(&self.chinese).display());
        let zh_switch = format!("[English]({}) | 中文", file_name(self.english).display());
        let en = read_sections(Path::new(self.english), &en_switch, docs)?;
        let zh = read_sections(Path::new(&self.chinese), &zh_switch, docs)?;
        if en.len() != zh.len() {
            return Err(format!(
                "{} and {} have different heading counts",
                self.english, self.chinese
            )
            .into());
        }

        let mut record =
            String::from("# Section fingerprints for the English and Chinese pages.\n");
        let mut parents = Vec::new();
        let mut ids = BTreeSet::new();
        for (en, zh) in en.into_iter().zip(zh) {
            while parents.last().is_some_and(|(depth, _)| *depth >= en.depth) {
                parents.pop();
            }
            let parent = parents
                .last()
                .map_or("", |(_, path): &(u8, String)| path.as_str());
            let path = unique_id(&format!("{parent}/{}", en.slug), &ids, "~");
            ids.insert(path.clone());
            if en.depth != zh.depth || en.code != zh.code {
                return Err(format!(
                    "{}: heading depth or code blocks differ from {} at {path}",
                    self.english, self.chinese,
                )
                .into());
            }
            writeln!(
                record,
                "{path}:\n  en: \"{:016x}\"\n  zh: \"{:016x}\"",
                en.hash, zh.hash
            )?;
            parents.push((en.depth, path));
        }
        Ok(record)
    }
}

struct Section {
    depth: u8,
    slug: String,
    hash: u64,
    code: Vec<(Option<String>, Option<String>, String)>,
}

fn read_sections(
    path: &Path,
    switcher: &str,
    docs: &BTreeMap<PathBuf, ParsedDoc>,
) -> Result<Vec<Section>> {
    let owned;
    let doc = if let Some(doc) = docs.get(path) {
        doc
    } else {
        owned = ParsedDoc::parse(path)?;
        &owned
    };
    let text = &doc.text;
    let blocks = doc.tree.children().ok_or("expected Markdown document")?;
    let [Node::Heading(title), Node::Paragraph(navigation), ..] = blocks.as_slice() else {
        return Err(format!(
            "{} must start with a title and language switcher",
            path.display()
        )
        .into());
    };
    let nav = navigation
        .position
        .as_ref()
        .ok_or("missing switcher position")?;
    if title.depth != 1 || &text[nav.start.offset..nav.end.offset] != switcher {
        return Err(format!("{}: expected language switcher {switcher}", path.display()).into());
    }

    let mut sections = Vec::new();
    for (index, block) in blocks.iter().enumerate() {
        let Node::Heading(heading) = block else {
            continue;
        };
        let slug = heading_slug(block);
        if slug.is_empty() {
            return Err(format!("{} has a heading without a slug", path.display()).into());
        }
        let mut body = String::new();
        let mut code = Vec::new();
        for child in std::iter::once(block).chain(
            blocks[index + 1..]
                .iter()
                .take_while(|node| !matches!(node, Node::Heading(_))),
        ) {
            if let Node::Code(fence) = child {
                code.push((fence.lang.clone(), fence.meta.clone(), fence.value.clone()));
            } else if let Some(position) = child.position()
                && position.start.offset != nav.start.offset
            {
                body.push_str(&text[position.start.offset..position.end.offset]);
                body.push('\n');
            }
        }
        sections.push(Section {
            depth: heading.depth,
            slug,
            hash: fingerprint(&body),
            code,
        });
    }
    Ok(sections)
}

fn fingerprint(text: &str) -> u64 {
    // FNV-1a gives stable fingerprints across hosts and Rust versions.
    text.bytes().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_scope_excludes_removed_experiments_dir() {
        assert!(in_scope("README.md"));
        assert!(in_scope("CONTRIBUTING.zh.md"));
        assert!(in_scope("docs/development.md"));
        assert!(!in_scope("experiments/demo.md"));
        assert!(!in_scope("CHANGELOG.md"));
    }

    #[test]
    fn pair_rejects_chinese_and_out_of_scope_paths() {
        let pair = Pair::new("docs/development.md").unwrap();
        assert_eq!(pair.chinese, "docs/development.zh.md");
        assert_eq!(pair.record, "docs/development.i18n.yaml");
        assert!(Pair::new("docs/development.zh.md").is_err());
        assert!(Pair::new("experiments/notes.md").is_err());
        assert!(Pair::new("docs/../development.md").is_err());
    }

    #[test]
    fn fingerprint_is_stable() {
        assert_eq!(fingerprint("hello"), 0xa430d84680aabd0b);
        assert_ne!(fingerprint("hello"), fingerprint("hello!"));
    }
}
