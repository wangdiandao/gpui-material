use std::collections::BTreeMap;

use serde::Deserialize;

use crate::{Result, output};

const GALLERY: &str = "gpui-material-gallery";
const PROBE_PREFIX: &str = "gpui-material-probe-";
const REGISTRY: &str = "registry+https://github.com/rust-lang/crates.io-index";
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

    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for (target, _, _) in TARGETS {
            handles.push(
                scope.spawn(move || {
                    verify_pure(Some(target), &[], false).map_err(|e| e.to_string())
                }),
            );
        }
        for features in THEME_FEATURES {
            handles.push(
                scope.spawn(move || verify_pure(None, features, true).map_err(|e| e.to_string())),
            );
        }
        for (target, backend, backend_features) in TARGETS {
            for test in [false, true] {
                handles.push(scope.spawn(move || {
                    verify(target, backend, backend_features, test).map_err(|e| e.to_string())
                }));
            }
        }
        handles.push(scope.spawn(|| verify_component_features().map_err(|e| e.to_string())));

        for handle in handles {
            handle.join().expect("graph verification thread panicked")?;
        }
        Ok::<_, String>(())
    })?;

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

fn verify_component_features() -> Result {
    const COMPONENTS: &str = "gpui-material-components";
    const MATERIAL: &str = "gpui-material";
    const EXTRAS: &str = "gpui-material-extras";

    let inspect_components_features =
        |package: &str, features: Option<&str>| -> Result<Option<Vec<String>>> {
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
                "{p} {f}",
            ];
            if let Some(features) = features {
                args.extend(["--features", features]);
            }
            let tree = output("cargo", &args)?;
            for line in tree.lines() {
                let line = line.trim_end_matches(" (*)").trim_end();
                let mut fields = line.split_whitespace();
                if fields.next() == Some(COMPONENTS) {
                    let mut active = fields
                        .nth(2)
                        .map(|list| {
                            list.split(',')
                                .filter(|item| !item.is_empty())
                                .map(str::to_owned)
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    active.sort();
                    return Ok(Some(active));
                }
            }
            Ok(None)
        };

    type FeatureCase = (
        &'static str,
        Option<&'static str>,
        Option<&'static [&'static str]>,
    );
    let cases: [FeatureCase; 13] = [
        (COMPONENTS, None, Some(&[])),
        (COMPONENTS, Some("actions"), Some(&["actions"])),
        (COMPONENTS, Some("selection"), Some(&["selection"])),
        (COMPONENTS, Some("inputs"), Some(&["inputs"])),
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
        (EXTRAS, Some("carousel"), None),
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
    ];

    std::thread::scope(|scope| {
        let handles: Vec<_> = cases
            .into_iter()
            .map(|(package, features, expected)| {
                scope.spawn(move || -> std::result::Result<(), String> {
                    let actual =
                        inspect_components_features(package, features).map_err(|e| e.to_string())?;
                    let expected = expected.map(|list| {
                        let mut sorted: Vec<String> =
                            list.iter().map(|s| (*s).to_owned()).collect();
                        sorted.sort();
                        sorted
                    });
                    if actual != expected {
                        return Err(format!(
                            "{package} ({features:?}): expected {COMPONENTS} features {expected:?}, found {actual:?}"
                        ));
                    }
                    Ok(())
                })
            })
            .collect();
        for handle in handles {
            handle.join().expect("component feature thread panicked")?;
        }
        Ok::<_, String>(())
    })?;
    Ok(())
}

fn verify_pure(target: Option<&str>, features: &[&str], theme_only: bool) -> Result {
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
        "{p} {f}",
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
    let mut found_theme = false;
    let mut found_mcu = false;
    let mut found_serde = false;
    for line in tree.lines() {
        let mut fields = line.split_whitespace();
        let package = fields.next().unwrap_or_default();
        if package.starts_with("gpui") && package != THEME {
            return Err(format!("{label}: unexpected dependency in the pure graph: {line}").into());
        }
        found_theme |= package == THEME;
        found_serde |= package == "serde";
        if package == MCU {
            found_mcu = true;
            if !line.trim_end_matches(" (*)").trim_end().ends_with("v0.1.3") {
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

fn verify(target: &str, backend: &str, backend_features: &[&str], test: bool) -> Result {
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
    let (metadata_json, tree) = std::thread::scope(|scope| {
        let metadata = scope.spawn(|| output("cargo", &metadata_args).map_err(|e| e.to_string()));
        let tree = output("cargo", &tree_args).map_err(|e| e.to_string());
        let metadata = metadata.join().expect("metadata thread panicked")?;
        Ok::<_, String>((metadata, tree?))
    })?;
    let metadata: Metadata = serde_json::from_str(&metadata_json)?;
    let packages: BTreeMap<_, _> = metadata
        .packages
        .iter()
        .map(|p| (p.id.as_str(), p))
        .collect();
    if metadata.workspace_default_members.iter().any(|id| {
        let package = packages[id.as_str()];
        package.name == GALLERY || package.name.starts_with(PROBE_PREFIX)
    }) {
        return Err(
            "Gallery and standalone probes must stay outside the default workspace members".into(),
        );
    }
    for line in tree.lines() {
        let name = line.split_whitespace().next().unwrap_or_default();
        if name.starts_with("gpui-fast")
            || matches!(name, "gpui" | "gpui-component" | "gpui-kit")
            || name.starts_with(PROBE_PREFIX)
        {
            return Err(format!("{target}: unexpected Gallery dependency {name}").into());
        }
    }
    for (name, version, features) in [
        ("gpui-base", "0.10.0", &[][..]),
        ("gpui-pre", "0.3.12", &[][..]),
        (
            "gpui-pre-platform",
            "0.3.12",
            &["font-kit", "x11", "wayland", "runtime_shaders"][..],
        ),
        (backend, "0.3.12", backend_features),
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
