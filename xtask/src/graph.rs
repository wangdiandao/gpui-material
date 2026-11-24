use std::{
    collections::{BTreeMap, VecDeque},
    fs,
    sync::Mutex,
};

use serde::Deserialize;

use crate::{Result, output};

const GALLERY: &str = "gpui-material-gallery";
const REGISTRY: &str = "registry+https://github.com/rust-lang/crates.io-index";
const MAX_CONCURRENT_CARGO: usize = 6;
const TARGETS: [(&str, &str, &[&str]); 3] = [
    ("x86_64-pc-windows-msvc", "gpui-pre-windows", &[]),
    (
        "aarch64-apple-darwin",
        "gpui-pre-macos",
        &["font-kit", "runtime_shaders"],
    ),
    (
        "x86_64-unknown-linux-gnu",
        "gpui-pre-linux",
        &["x11", "wayland"],
    ),
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct PinnedVersions {
    gpui_base: String,
    gpui: String,
    gpui_platform: String,
    material_color_utils: String,
}

#[derive(Deserialize)]
struct Metadata {
    packages: Vec<Package>,
    workspace_default_members: Vec<String>,
    resolve: Resolve,
}

#[derive(Deserialize)]
struct Package {
    id: String,
    name: String,
    version: String,
    source: Option<String>,
}

#[derive(Deserialize)]
struct Resolve {
    nodes: Vec<Node>,
}

#[derive(Deserialize)]
struct Node {
    id: String,
    features: Vec<String>,
}

pub(crate) fn check() -> Result {
    const THEME_FEATURES: [&[&str]; 4] = [
        &[],
        &["dynamic-color"],
        &["serde"],
        &["dynamic-color,serde"],
    ];

    let root_toml = fs::read_to_string("Cargo.toml")?;
    let theme_toml = fs::read_to_string("crates/theme/Cargo.toml")?;
    let versions = parse_pinned_versions(&root_toml, &theme_toml)?;

    let mut tasks: Vec<Box<dyn FnOnce() -> std::result::Result<(), String> + Send>> = Vec::new();
    for (target, _, _) in TARGETS {
        let mcu_version = versions.material_color_utils.clone();
        tasks.push(Box::new(move || {
            verify_pure(Some(target), &[], false, &mcu_version).map_err(|e| e.to_string())
        }));
    }
    for features in THEME_FEATURES {
        let mcu_version = versions.material_color_utils.clone();
        tasks.push(Box::new(move || {
            verify_pure(None, features, true, &mcu_version).map_err(|e| e.to_string())
        }));
    }
    for (target, backend, backend_features) in TARGETS {
        for test in [false, true] {
            let versions = versions.clone();
            tasks.push(Box::new(move || {
                verify(target, backend, backend_features, test, &versions)
                    .map_err(|e| e.to_string())
            }));
        }
    }
    for (package, features, expected) in component_feature_cases() {
        tasks.push(Box::new(move || {
            verify_component_feature_case(package, features, expected).map_err(|e| e.to_string())
        }));
    }

    run_bounded(MAX_CONCURRENT_CARGO, tasks)?;

    for (target, _, _) in TARGETS {
        println!("{target}: default and theme dependency graphs verified");
    }
    for (target, backend, _) in TARGETS {
        for test in [false, true] {
            println!(
                "{target} {}: pinned GPUI/Base and {backend} verified",
                if test { "test-support" } else { "native" }
            );
        }
    }
    println!("component feature-group graphs verified");
    Ok(())
}

fn run_bounded<F>(max_concurrency: usize, tasks: Vec<F>) -> Result
where
    F: FnOnce() -> std::result::Result<(), String> + Send,
{
    let workers = max_concurrency.max(1).min(tasks.len());
    if workers == 0 {
        return Ok(());
    }
    let queue = Mutex::new(VecDeque::from(tasks));
    let errors = Mutex::new(Vec::<String>::new());

    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                loop {
                    let task = queue.lock().expect("task queue poisoned").pop_front();
                    let Some(task) = task else {
                        break;
                    };
                    if let Err(err) = task() {
                        errors.lock().expect("error list poisoned").push(err);
                    }
                }
            });
        }
    });

    let errors = errors.into_inner().expect("error list poisoned");
    if !errors.is_empty() {
        return Err(errors.join("\n").into());
    }
    Ok(())
}

type FeatureCase = (
    &'static str,
    Option<&'static str>,
    Option<&'static [&'static str]>,
);

fn component_feature_cases() -> [FeatureCase; 16] {
    const COMPONENTS: &str = "gpui-material-components";
    const MATERIAL: &str = "gpui-material";
    const EXTRAS: &str = "gpui-material-extras";

    [
        (COMPONENTS, None, Some(&[])),
        (COMPONENTS, Some("actions"), Some(&["actions"])),
        (COMPONENTS, Some("selection"), Some(&["selection"])),
        (COMPONENTS, Some("inputs"), Some(&["actions", "inputs"])),
        (COMPONENTS, Some("surfaces"), Some(&["actions", "surfaces"])),
        (COMPONENTS, Some("feedback"), Some(&["actions", "feedback"])),
        (
            COMPONENTS,
            Some("navigation"),
            Some(&["actions", "feedback", "navigation"]),
        ),
        (
            MATERIAL,
            Some("actions,inputs"),
            Some(&["actions", "inputs"]),
        ),
        (MATERIAL, Some("carousel"), Some(&[])),
        (MATERIAL, Some("window-chrome"), Some(&[])),
        (EXTRAS, Some("carousel"), None),
        (EXTRAS, Some("window-chrome"), None),
        (
            EXTRAS,
            Some("expressive-groups"),
            Some(&["actions", "feedback"]),
        ),
        (EXTRAS, Some("toolbars"), Some(&["actions", "feedback"])),
        (
            EXTRAS,
            Some("date-time"),
            Some(&["actions", "feedback", "inputs"]),
        ),
        (
            EXTRAS,
            Some("search"),
            Some(&["actions", "feedback", "inputs", "surfaces"]),
        ),
    ]
}

fn verify_component_feature_case(
    package: &str,
    features: Option<&str>,
    expected: Option<&[&str]>,
) -> Result {
    const COMPONENTS: &str = "gpui-material-components";
    let mut args = vec![
        "tree",
        "--locked",
        "-p",
        package,
        "--no-default-features",
        "--edges",
        "normal",
        "--prefix",
        "none",
        "--format",
        "{p}|{f}",
    ];
    if let Some(features) = features {
        args.extend(["--features", features]);
    }
    let tree = output("cargo", &args)?;
    let actual = parse_components_features(&tree, COMPONENTS);
    let expected = expected.map(|list| {
        let mut sorted: Vec<String> = list.iter().map(|s| (*s).to_owned()).collect();
        sorted.sort();
        sorted
    });
    if actual != expected {
        return Err(format!(
            "{package} ({features:?}): expected {COMPONENTS} features {expected:?}, found {actual:?}"
        )
        .into());
    }
    Ok(())
}

fn parse_components_features(tree: &str, target_package: &str) -> Option<Vec<String>> {
    for line in tree.lines() {
        let trimmed = line.trim_end().trim_end_matches(" (*)").trim_end();
        let Some((pkg_part, feat_part)) = trimmed.split_once('|') else {
            continue;
        };
        if pkg_part.split_whitespace().next() == Some(target_package) {
            let mut active = feat_part
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(str::to_owned)
                .collect::<Vec<_>>();
            active.sort();
            return Some(active);
        }
    }
    None
}

fn parse_pinned_versions(root_toml: &str, theme_toml: &str) -> Result<PinnedVersions> {
    let gpui_base = parse_toml_dep_version(root_toml, "workspace.dependencies", "gpui-base")?;
    let gpui = parse_toml_dep_version(root_toml, "workspace.dependencies", "gpui")?;
    let gpui_platform =
        parse_toml_dep_version(root_toml, "workspace.dependencies", "gpui_platform")?;
    let material_color_utils =
        parse_toml_dep_version(theme_toml, "dependencies", "material-color-utils")?;
    Ok(PinnedVersions {
        gpui_base,
        gpui,
        gpui_platform,
        material_color_utils,
    })
}

fn parse_toml_dep_version(toml: &str, section: &str, key: &str) -> Result<String> {
    let mut in_section = false;
    let mut collecting: Option<String> = None;

    for raw_line in toml.lines() {
        let line = raw_line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        if let Some(buf) = collecting.as_mut() {
            buf.push(' ');
            buf.push_str(line);
            if buf.contains('}') {
                return extract_version_value(buf, section, key);
            }
            continue;
        }
        if let Some(header) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            in_section = header.trim() == section;
            continue;
        }
        if !in_section {
            continue;
        }
        let Some((lhs, rhs)) = line.split_once('=') else {
            continue;
        };
        if lhs.trim() != key {
            continue;
        }
        let rhs = rhs.trim();
        if rhs.starts_with('{') && !rhs.contains('}') {
            collecting = Some(rhs.to_owned());
            continue;
        }
        return extract_version_value(rhs, section, key);
    }

    Err(format!("missing [{section}].{key} version").into())
}

fn extract_version_value(rhs: &str, section: &str, key: &str) -> Result<String> {
    let raw = if let Some(quoted) = extract_quoted_string(rhs) {
        if rhs.starts_with('"') {
            quoted
        } else if let Some(idx) = rhs.find("version") {
            let after = rhs[idx + "version".len()..].trim_start();
            let after = after
                .strip_prefix('=')
                .ok_or_else(|| format!("malformed version in [{section}].{key}"))?
                .trim_start();
            extract_quoted_string(after)
                .ok_or_else(|| format!("unquoted version in [{section}].{key}"))?
        } else {
            return Err(format!("missing version field in [{section}].{key}").into());
        }
    } else {
        return Err(format!("invalid dependency value in [{section}].{key}").into());
    };
    let version = raw.trim_start_matches('=').trim();
    if version.is_empty() {
        return Err(format!("empty version in [{section}].{key}").into());
    }
    Ok(version.to_owned())
}

fn extract_quoted_string(input: &str) -> Option<&str> {
    let start = input.find('"')?;
    let rest = &input[start + 1..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

fn verify_pure(
    target: Option<&str>,
    features: &[&str],
    theme_only: bool,
    mcu_version: &str,
) -> Result {
    const THEME: &str = "gpui-material-theme";
    const MCU: &str = "material-color-utils";

    let label = target.unwrap_or(THEME);
    let mut args = vec![
        "tree",
        "--locked",
        "--edges",
        "normal,build,dev",
        "--prefix",
        "none",
        "--format",
        "{p}|{f}",
        "--no-default-features",
    ];
    if let Some(target) = target {
        args.extend(["--target", target]);
    }
    if theme_only {
        args.extend(["--package", THEME]);
    }
    if let [features] = features {
        args.extend(["--features", features]);
    }
    let tree = output("cargo", &args)?;
    let expected_mcu = format!("{MCU} v{mcu_version}");
    let mut found_theme = false;
    let mut found_mcu = false;
    let mut found_serde = false;
    for line in tree.lines() {
        let trimmed = line.trim_end().trim_end_matches(" (*)").trim_end();
        let (pkg_part, feat_part) = trimmed.split_once('|').unwrap_or((trimmed, ""));
        let package = pkg_part.split_whitespace().next().unwrap_or_default();
        if package.starts_with("gpui") && package != THEME {
            return Err(format!("{label}: unexpected dependency in the pure graph: {line}").into());
        }
        found_theme |= package == THEME;
        found_serde |= package == "serde";
        if package == MCU {
            found_mcu = true;
            if pkg_part.trim() != expected_mcu || !feat_part.trim().is_empty() {
                return Err(format!("{label}: unexpected MCU version or features: {line}").into());
            }
        }
        if theme_only && matches!(package, "image" | "rayon") {
            return Err(format!("{label}: unexpected theme dependency: {line}").into());
        }
    }
    let active = features.first().copied().unwrap_or_default();
    if !found_theme
        || found_mcu != active.contains("dynamic-color")
        || (theme_only && found_serde != active.contains("serde"))
    {
        return Err(format!("{label}: unexpected pure dependency graph for {features:?}").into());
    }
    Ok(())
}

fn verify(
    target: &str,
    backend: &str,
    backend_features: &[&str],
    test: bool,
    versions: &PinnedVersions,
) -> Result {
    let mut metadata_args = vec![
        "metadata",
        "--locked",
        "--format-version",
        "1",
        "--filter-platform",
        target,
    ];
    let mut tree_args = vec![
        "tree",
        "--locked",
        "-p",
        GALLERY,
        "--target",
        target,
        "--edges",
        "normal,build",
        "--prefix",
        "none",
        "--format",
        "{p}",
    ];
    if test {
        metadata_args.extend(["--features", "gpui-material-gallery/test-support"]);
        tree_args.extend(["--features", "test-support"]);
    }
    let metadata_json = output("cargo", &metadata_args)?;
    let tree = output("cargo", &tree_args)?;
    let metadata: Metadata = serde_json::from_str(&metadata_json)?;
    let packages: BTreeMap<_, _> = metadata
        .packages
        .iter()
        .map(|p| (p.id.as_str(), p))
        .collect();
    if metadata
        .workspace_default_members
        .iter()
        .any(|id| packages[id.as_str()].name == GALLERY)
    {
        return Err("Gallery must stay outside the default workspace members".into());
    }
    for line in tree.lines() {
        let name = line.split_whitespace().next().unwrap_or_default();
        if name.starts_with("gpui-fast") || matches!(name, "gpui" | "gpui-component" | "gpui-kit") {
            return Err(format!("{target}: unexpected Gallery dependency {name}").into());
        }
    }
    for (name, version, features) in [
        ("gpui-base", versions.gpui_base.as_str(), &[][..]),
        ("gpui-pre", versions.gpui.as_str(), &[][..]),
        (
            "gpui-pre-platform",
            versions.gpui_platform.as_str(),
            &["font-kit", "x11", "wayland", "runtime_shaders"][..],
        ),
        (backend, versions.gpui.as_str(), backend_features),
    ] {
        let nodes = metadata
            .resolve
            .nodes
            .iter()
            .filter(|node| packages[node.id.as_str()].name == name)
            .collect::<Vec<_>>();
        let [node] = nodes.as_slice() else {
            return Err(format!(
                "{target}: expected one resolved {name}, found {}",
                nodes.len()
            )
            .into());
        };
        let package = packages[node.id.as_str()];
        let expected = format!("{name} v{version}");
        if package.version != version
            || package.source.as_deref() != Some(REGISTRY)
            || !tree
                .lines()
                .any(|line| line == expected || line.starts_with(&format!("{expected} ")))
        {
            return Err(format!(
                "{target}: expected active crates.io {name} {version}, found {}",
                package.id
            )
            .into());
        }
        if features
            .iter()
            .any(|feature| !node.features.iter().any(|active| active == feature))
            || (name != backend && node.features.iter().any(|f| f == "test-support") != test)
        {
            return Err(format!("{target}: unexpected {name} features {:?}", node.features).into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_components_features_with_spaces_in_path() {
        let tree = "\
gpui-material v0.3.3 (C:\\My Repos\\gpui material\\crates\\material)|actions,inputs
gpui-material-components v0.3.3 (C:\\My Repos\\gpui material\\crates\\components)|inputs,actions (*)
";
        assert_eq!(
            parse_components_features(tree, "gpui-material-components"),
            Some(vec!["actions".to_owned(), "inputs".to_owned()])
        );
    }

    #[test]
    fn parses_empty_and_missing_components_features() {
        let tree = "gpui-material-components v0.3.3 (/tmp/path with space/crates/components)|\n";
        assert_eq!(
            parse_components_features(tree, "gpui-material-components"),
            Some(Vec::new())
        );
        assert_eq!(
            parse_components_features(
                "gpui-material-extras v0.3.3 (/tmp/path with space/crates/extras)|carousel\n",
                "gpui-material-components"
            ),
            None
        );
    }

    #[test]
    fn parses_pinned_versions_from_manifests() {
        let root_toml = r#"
[workspace.dependencies]
gpui = { package = "gpui-pre", version = "=0.3.14" }
gpui-base = "=0.11.0"
gpui_platform = { package = "gpui-pre-platform", version = "=0.3.14", features = [
    "font-kit",
] }
"#;
        let theme_toml = r#"
[dependencies]
material-color-utils = { version = "0.1.3", default-features = false, optional = true }
"#;
        let parsed = parse_pinned_versions(root_toml, theme_toml).unwrap();
        assert_eq!(
            parsed,
            PinnedVersions {
                gpui_base: "0.11.0".into(),
                gpui: "0.3.14".into(),
                gpui_platform: "0.3.14".into(),
                material_color_utils: "0.1.3".into(),
            }
        );
    }

    #[test]
    fn run_bounded_executes_all_tasks_and_collects_errors() {
        let tasks: Vec<Box<dyn FnOnce() -> std::result::Result<(), String> + Send>> = vec![
            Box::new(|| Ok(())),
            Box::new(|| Err("boom-1".into())),
            Box::new(|| Err("boom-2".into())),
        ];
        let err = run_bounded(2, tasks).unwrap_err().to_string();
        assert!(err.contains("boom-1"));
        assert!(err.contains("boom-2"));
    }
}
