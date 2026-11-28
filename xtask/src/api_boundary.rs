use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use crate::Result;

const GUARDED_CRATES: [&str; 7] = [
    "gpui_base",
    "web_time",
    "jiff",
    "polymorpher",
    "material_color_utils",
    "windows",
    "raw_window_handle",
];

/// Explicit allowlist of intentional public third-party re-exports documented in
/// `docs/architecture.md` under Dependency policy.
///
/// Any new `pub use` of a guarded third-party crate outside this list fails
/// `cargo xtask check`. Converge lightweight value types into Material-owned
/// types instead of expanding this list.
const ALLOWLIST: [(&str, &str); 7] = [
    (
        "crates/components/src/display_scrollbar.rs",
        "pub use gpui_base::ScrollbarHandle;",
    ),
    (
        "crates/components/src/input.rs",
        "pub use gpui_base::input::{InputEvent as TextFieldEvent, InputState as TextFieldState, MaskPattern as TextFieldMaskPattern, TextareaState as TextAreaState};",
    ),
    (
        "crates/components/src/slider.rs",
        "pub use gpui_base::slider::{SliderEvent, SliderScale, SliderState, SliderValue};",
    ),
    (
        "crates/extras/src/date_time.rs",
        "pub use jiff::civil::{Date, Time, Weekday};",
    ),
    ("crates/foundation/src/lib.rs", "pub use web_time::Instant;"),
    (
        "crates/foundation/src/test.rs",
        "pub use gpui_base::TestSupportExt;",
    ),
    (
        "crates/foundation/src/test.rs",
        "pub use gpui_base::test_support::{ElementSnapshot, TextSnapshot};",
    ),
];

pub(crate) fn check() -> Result {
    let mut files = Vec::new();
    collect_crate_source_files(Path::new("crates"), &mut files)?;
    files.sort();

    let mut sources = Vec::with_capacity(files.len());
    for path in files {
        let rel = normalize_path(&path);
        let content = fs::read_to_string(&path)?;
        sources.push((rel, content));
    }

    verify_sources(&sources, &ALLOWLIST)?;
    println!(
        "public third-party API boundary verified ({} intentional re-exports)",
        ALLOWLIST.len()
    );
    Ok(())
}

fn collect_crate_source_files(crates_dir: &Path, out: &mut Vec<PathBuf>) -> Result {
    for entry in fs::read_dir(crates_dir)? {
        let entry = entry?;
        let src_dir = entry.path().join("src");
        if src_dir.is_dir() {
            collect_rs_files(&src_dir, out)?;
        }
    }
    Ok(())
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            out.push(path);
        }
    }
    Ok(())
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn verify_sources(sources: &[(String, String)], allowlist: &[(&str, &str)]) -> Result {
    let allowed: BTreeSet<(&str, &str)> = allowlist.iter().copied().collect();
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut violations = Vec::new();

    for (path, content) in sources {
        for stmt in extract_guarded_pub_uses(content) {
            if !allowed.contains(&(path.as_str(), stmt.as_str())) {
                violations.push(format!(
                    "{path}: unexpected public third-party re-export `{stmt}` \
                     (wrap in a Material-owned type or document and allowlist an intentional boundary)"
                ));
            }
            seen.insert((path.clone(), stmt));
        }

        if path == "crates/material/src/lib.rs" && facade_reexports_instant(content) {
            violations.push(
                "crates/material/src/lib.rs: `Instant` must stay scoped to `gpui_material::foundation`, not re-exported at the facade root"
                    .to_owned(),
            );
        }
    }

    for (path, stmt) in allowlist {
        if !seen.contains(&((*path).to_owned(), (*stmt).to_owned())) {
            violations.push(format!(
                "{path}: stale allowlist entry `{stmt}` no longer matches source; remove it from xtask/src/api_boundary.rs"
            ));
        }
    }

    if !violations.is_empty() {
        return Err(format!(
            "third-party public API boundary violations:\n  - {}",
            violations.join("\n  - ")
        )
        .into());
    }
    Ok(())
}

fn facade_reexports_instant(source: &str) -> bool {
    let stripped = strip_comments_and_strings(source);
    for stmt in extract_all_pub_uses(&stripped) {
        if stmt.contains("gpui_material_foundation")
            && stmt
                .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .any(|token| token == "Instant")
        {
            return true;
        }
    }
    false
}

fn extract_guarded_pub_uses(source: &str) -> Vec<String> {
    let stripped = strip_comments_and_strings(source);
    extract_all_pub_uses(&stripped)
        .into_iter()
        .filter(|stmt| is_guarded_pub_use(stmt))
        .collect()
}

fn extract_all_pub_uses(stripped: &str) -> Vec<String> {
    let mut results = Vec::new();
    let bytes = stripped.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if starts_with_word(bytes, i, b"pub") {
            let mut j = skip_ws(bytes, i + 3);
            if starts_with_word(bytes, j, b"use") {
                let start = i;
                j = skip_ws(bytes, j + 3);
                while j < len && bytes[j] != b';' {
                    j += 1;
                }
                if j < len {
                    let raw = &stripped[start..=j];
                    results.push(normalize_use_statement(raw));
                    i = j + 1;
                    continue;
                }
            }
        }
        i += 1;
    }

    results
}

fn is_guarded_pub_use(normalized_stmt: &str) -> bool {
    let Some(rest) = normalized_stmt.strip_prefix("pub use ") else {
        return false;
    };
    let rest = rest.strip_prefix("::").unwrap_or(rest);
    let root = rest
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .next()
        .unwrap_or_default();
    GUARDED_CRATES.contains(&root)
}

fn starts_with_word(bytes: &[u8], pos: usize, word: &[u8]) -> bool {
    if pos + word.len() > bytes.len() || &bytes[pos..pos + word.len()] != word {
        return false;
    }
    let before_ok = pos == 0 || !is_ident_char(bytes[pos - 1]);
    let after_ok = pos + word.len() == bytes.len() || !is_ident_char(bytes[pos + word.len()]);
    before_ok && after_ok
}

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn skip_ws(bytes: &[u8], mut pos: usize) -> usize {
    while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
        pos += 1;
    }
    pos
}

fn normalize_use_statement(raw: &str) -> String {
    let mut collapsed = String::with_capacity(raw.len());
    let mut prev_ws = false;
    for ch in raw.chars() {
        if ch.is_whitespace() {
            if !prev_ws {
                collapsed.push(' ');
                prev_ws = true;
            }
        } else {
            collapsed.push(ch);
            prev_ws = false;
        }
    }
    let normalized = collapsed
        .replace("{ ", "{")
        .replace(" }", "}")
        .replace(",}", "}")
        .replace(" ,", ",")
        .replace(" ;", ";")
        .replace(" ::", "::")
        .replace(":: ", "::");
    let normalized = normalized.trim();
    if let Some(rest) = normalized.strip_prefix("pub use::") {
        format!("pub use ::{rest}")
    } else {
        normalized.to_owned()
    }
}

/// Replaces Rust comments and string literals with spaces while preserving line
/// structure and token separators.
fn strip_comments_and_strings(source: &str) -> String {
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut out = String::with_capacity(len);
    let mut i = 0;

    while i < len {
        // Line comment
        if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            while i < len && bytes[i] != b'\n' {
                out.push(' ');
                i += 1;
            }
            continue;
        }
        // Nested block comment
        if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            let mut depth = 1usize;
            out.push_str("  ");
            i += 2;
            while i < len && depth > 0 {
                if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'*' {
                    depth += 1;
                    out.push_str("  ");
                    i += 2;
                } else if i + 1 < len && bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    depth -= 1;
                    out.push_str("  ");
                    i += 2;
                } else {
                    out.push(if bytes[i] == b'\n' { '\n' } else { ' ' });
                    i += 1;
                }
            }
            continue;
        }
        // Raw string literal: r"...", r#"..."#
        if bytes[i] == b'r' && (i == 0 || !is_ident_char(bytes[i - 1])) {
            let mut j = i + 1;
            let mut hashes = 0usize;
            while j < len && bytes[j] == b'#' {
                hashes += 1;
                j += 1;
            }
            if j < len && bytes[j] == b'"' {
                for _ in i..=j {
                    out.push(' ');
                }
                i = j + 1;
                while i < len {
                    if bytes[i] == b'"' {
                        let mut k = i + 1;
                        let mut matched = 0usize;
                        while k < len && matched < hashes && bytes[k] == b'#' {
                            matched += 1;
                            k += 1;
                        }
                        if matched == hashes {
                            for _ in i..k {
                                out.push(' ');
                            }
                            i = k;
                            break;
                        }
                    }
                    out.push(if bytes[i] == b'\n' { '\n' } else { ' ' });
                    i += 1;
                }
                continue;
            }
        }
        // Regular string literal
        if bytes[i] == b'"' {
            out.push(' ');
            i += 1;
            while i < len {
                if bytes[i] == b'\\' && i + 1 < len {
                    out.push_str("  ");
                    i += 2;
                    continue;
                }
                let ch = bytes[i];
                out.push(if ch == b'\n' { '\n' } else { ' ' });
                i += 1;
                if ch == b'"' {
                    break;
                }
            }
            continue;
        }
        // Preserve UTF-8 characters safely
        let ch = source[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_single_and_multiline_guarded_pub_uses_and_ignores_private_or_commented() {
        let src = r##"
            // pub use gpui_base::CheckboxState;
            /* pub use web_time::Instant; */
            const DOC: &str = "pub use polymorpher::Cubic;";
            const RAW: &str = r#"pub use jiff::civil::Date;"#;
            pub(crate) use gpui_base::Button;
            pub(super) use windows::Win32::Foundation::HWND;
            pub use gpui_base::input::{
                InputEvent as TextFieldEvent,
                InputState as TextFieldState,
            };
            pub use crate::local::Thing;
            pub use ::raw_window_handle::RawWindowHandle;
        "##;
        assert_eq!(
            extract_guarded_pub_uses(src),
            vec![
                "pub use gpui_base::input::{InputEvent as TextFieldEvent, InputState as TextFieldState};",
                "pub use ::raw_window_handle::RawWindowHandle;",
            ]
        );
    }

    #[test]
    fn verify_sources_rejects_unlisted_reexports_stale_entries_and_facade_instant() {
        let allow = [(
            "crates/components/src/slider.rs",
            "pub use gpui_base::slider::SliderState;",
        )];

        let ok_sources = vec![(
            "crates/components/src/slider.rs".to_owned(),
            "pub use gpui_base::slider::SliderState;".to_owned(),
        )];
        verify_sources(&ok_sources, &allow).unwrap();

        let leaked = vec![
            ok_sources[0].clone(),
            (
                "crates/foundation/src/motion.rs".to_owned(),
                "pub use gpui_base::motion::MotionStatus;".to_owned(),
            ),
        ];
        let err = verify_sources(&leaked, &allow).unwrap_err().to_string();
        assert!(err.contains("MotionStatus"), "{err}");

        let stale_err = verify_sources(&[], &allow).unwrap_err().to_string();
        assert!(stale_err.contains("stale allowlist entry"), "{stale_err}");

        let facade_instant = vec![
            ok_sources[0].clone(),
            (
                "crates/material/src/lib.rs".to_owned(),
                "pub use gpui_material_foundation::{Instant, MaterialRoot};".to_owned(),
            ),
        ];
        let err = verify_sources(&facade_instant, &allow)
            .unwrap_err()
            .to_string();
        assert!(err.contains("Instant"), "{err}");
    }
}
