use std::collections::BTreeMap;

use serde::Deserialize;

use crate::{Result, output};

const GALLERY: &str = "gpui-material-gallery";
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
    let default_tree = output(
        "cargo",
        &[
            "tree",
            "--locked",
            "--edges",
            "normal,build",
            "--prefix",
            "none",
            "--format",
            "{p}",
        ],
    )?;
    if default_tree
        .lines()
        .any(|line| line.starts_with("gpui") || line.starts_with("p1-"))
    {
        return Err("the default dependency graph contains GPUI or a P1 experiment".into());
    }

    for (target, backend, backend_features) in TARGETS {
        for test in [false, true] {
            verify(target, backend, backend_features, test)?;
        }
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
    let metadata: Metadata = serde_json::from_str(&output("cargo", &metadata_args)?)?;
    let tree = output("cargo", &tree_args)?;
    let packages: BTreeMap<_, _> = metadata
        .packages
        .iter()
        .map(|p| (p.id.as_str(), p))
        .collect();
    if metadata.workspace_default_members.iter().any(|id| {
        let package = packages[id.as_str()];
        matches!(
            package.name.as_str(),
            GALLERY | "p1-host" | "p1-theme" | "p1-shapes"
        )
    }) {
        return Err(
            "Gallery and P1 experiments must stay outside the default workspace members".into(),
        );
    }
    for line in tree.lines() {
        let name = line.split_whitespace().next().unwrap_or_default();
        if name.starts_with("gpui-fast")
            || matches!(name, "gpui" | "gpui-component" | "gpui-kit")
            || name.starts_with("p1-")
        {
            return Err(format!("{target}: unexpected Gallery dependency {name}").into());
        }
    }
    for (name, version, features) in [
        ("gpui-base", "0.9.1", &[][..]),
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
    println!(
        "{target} {}: pinned GPUI/Base and {backend} verified",
        if test { "test-support" } else { "native" }
    );
    Ok(())
}
