use std::{
    collections::BTreeSet,
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
};

use markdown::{ParseOptions, mdast::Node, to_mdast};

use crate::{Result, output};

use super::{heading_slug, unique_id};

pub(super) fn check(markdown: &str) -> Result {
    let files = markdown.split_terminator('\0').collect::<BTreeSet<_>>();
    let records = output("git", &["ls-files", "-z", "--", "*.i18n.yaml"])?;
    let mut records = records.split_terminator('\0').collect::<BTreeSet<_>>();
    let mut count = 0;
    for path in &files {
        if !in_scope(path) {
            continue;
        }
        if let Some(stem) = path.strip_suffix(".zh.md") {
            if !files.contains(format!("{stem}.md").as_str()) {
                return Err(format!("{path} has no tracked English counterpart").into());
            }
            continue;
        }
        let pair = Pair::new(path)?;
        if !files.contains(pair.chinese.as_str()) {
            return Err(format!("{path} has no tracked Chinese counterpart").into());
        }
        if !records.remove(pair.record.as_str()) {
            return Err(format!("{path} has no tracked {}", pair.record).into());
        }
        let expected = pair.render()?;
        let current = fs::read_to_string(&pair.record)?.replace("\r\n", "\n");
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
            return Err(format!(
                "{} is stale at {section}. Update both pages and run cargo xtask docs --write {path}",
                pair.record
            )
            .into());
        }
        count += 1;
    }
    if let Some(path) = records.first() {
        return Err(format!("unpaired documentation record: {path}").into());
    }
    println!("Checked {count} bilingual Markdown pairs");
    Ok(())
}

pub(crate) fn write_records(paths: &[&str]) -> Result {
    for path in paths {
        let normalized = path.replace('\\', "/");
        let pair = Pair::new(&normalized)?;
        fs::write(&pair.record, pair.render()?)?;
        println!("Updated {}", pair.record);
    }
    Ok(())
}

fn in_scope(path: &str) -> bool {
    matches!(
        path,
        "README.md" | "README.zh.md" | "THIRD_PARTY_NOTICES.md" | "THIRD_PARTY_NOTICES.zh.md"
    ) || path.starts_with("docs/")
        || path.starts_with("experiments/")
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

    fn render(&self) -> Result<String> {
        let file_name = |path: &str| PathBuf::from(path).file_name().unwrap().to_owned();
        let en_switch = format!("English | [中文]({})", file_name(&self.chinese).display());
        let zh_switch = format!("[English]({}) | 中文", file_name(self.english).display());
        let en = read_sections(Path::new(self.english), &en_switch)?;
        let zh = read_sections(Path::new(&self.chinese), &zh_switch)?;
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

fn read_sections(path: &Path, switcher: &str) -> Result<Vec<Section>> {
    let text = fs::read_to_string(path)?.replace("\r\n", "\n");
    let tree = to_mdast(&text, &ParseOptions::gfm())
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let blocks = tree.children().ok_or("expected Markdown document")?;
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
