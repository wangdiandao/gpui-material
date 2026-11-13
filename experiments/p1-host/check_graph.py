"""Verify the pinned package identities and active platform/test features."""

import json
from pathlib import Path
import subprocess
import sys


MANIFEST = Path(__file__).with_name("Cargo.toml")
PINNED = {
    "gpui-base": "0.9.1",
    "gpui-pre": "0.3.12",
    "gpui-pre-platform": "0.3.12",
}
PLATFORMS = {
    "x86_64-pc-windows-msvc": ("gpui-pre-windows", set()),
    "aarch64-apple-darwin": ("gpui-pre-macos", {"font-kit", "runtime_shaders"}),
    "x86_64-unknown-linux-gnu": ("gpui-pre-linux", {"x11", "wayland"}),
}


def check(condition, detail):
    if not condition:
        raise SystemExit(detail)


def verify(cargo, target, test_support):
    flags = ["--features", "test-support"] if test_support else []
    result = subprocess.check_output(
        [
            *cargo,
            "metadata",
            "--locked",
            "--format-version",
            "1",
            "--manifest-path",
            str(MANIFEST),
            "--filter-platform",
            target,
            *flags,
        ],
        encoding="utf-8",
    )
    metadata = json.loads(result)
    packages = {p["id"]: p for p in metadata["packages"]}
    nodes = metadata["resolve"]["nodes"]
    backend, backend_features = PLATFORMS[target]
    pinned = {**PINNED, backend: "0.3.12"}

    # Metadata may list optional dependencies mentioned by a weak feature.
    # The normal/build tree checks which packages the probe actually uses.
    tree = subprocess.check_output(
        [
            *cargo, "tree", "--locked", "--manifest-path", str(MANIFEST),
            "--target", target, "--edges", "normal,build", "--prefix", "none",
            "--format", "{p}", *flags,
        ],
        encoding="utf-8",
    )
    unexpected = {
        line.split()[0]
        for line in tree.splitlines()
        if line.startswith(("gpui-fast", "gpui-component"))
    }
    check(not unexpected, f"{target}: unexpected backends or styled crates: {unexpected}")
    for name, version in pinned.items():
        matches = [node for node in nodes if packages[node["id"]]["name"] == name]
        check(len(matches) == 1, f"{target}: expected one {name}, got {matches}")
        node = matches[0]
        pkg = packages[node["id"]]
        check(
            pkg["version"] == version
            and pkg["source"] == "registry+https://github.com/rust-lang/crates.io-index",
            f"{target}: unexpected package identity {pkg['id']}",
        )
        features = set(node["features"])
        if name in PINNED:
            check(
                ("test-support" in features) == test_support,
                f"{target}: wrong test-support state for {name}: {features}",
            )
        if name == backend:
            check(
                backend_features <= features,
                f"{target}: missing backend features: {backend_features - features}",
            )
        print(f"{target} {'test' if test_support else 'native'} {name}={version}: "
              f"{', '.join(sorted(features))}", flush=True)


if __name__ == "__main__":
    cargo = ["cargo", *sys.argv[1:]]
    for target in PLATFORMS:
        for test_support in (False, True):
            verify(cargo, target, test_support)
