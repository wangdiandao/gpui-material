#!/usr/bin/env python3
"""Release verification, crates.io publishing, and GitHub Release orchestration."""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
import re
import subprocess
import sys
import tarfile
import tempfile
import time
import tomllib
import urllib.error
import urllib.parse
import urllib.request
import zipfile
from pathlib import Path
from typing import Any

USER_AGENT = "gpui-material-release-workflow/1.0 (+https://github.com/wangdiandao/gpui-material)"

CRATES: list[tuple[str, str]] = [
    ("gpui-material-theme", "crates/theme"),
    ("gpui-material-foundation", "crates/foundation"),
    ("gpui-material-components", "crates/components"),
    ("gpui-material-extras", "crates/extras"),
    ("gpui-material", "crates/material"),
    ("gpui-material-design-symbols", "crates/design-symbols"),
]

INTERNAL_WORKSPACE_DEPS: list[str] = [
    "gpui-material-theme",
    "gpui-material-foundation",
    "gpui-material-components",
    "gpui-material-extras",
]

SEMVER_RE = re.compile(r"^[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?$")
COMPARE_URL_RE = re.compile(r"https://github\.com/[^/\s]+/[^/\s]+/compare/[^\s)\]`]+")


def fail(message: str) -> None:
    print(f"error: {message}", file=sys.stderr)
    raise SystemExit(1)


def run(
    cmd: list[str],
    *,
    cwd: Path | None = None,
    env: dict[str, str] | None = None,
    capture: bool = False,
) -> subprocess.CompletedProcess[str]:
    print("+", " ".join(cmd), flush=True)
    result = subprocess.run(
        cmd,
        cwd=str(cwd) if cwd else None,
        env=env,
        text=True,
        capture_output=capture,
    )
    if result.returncode != 0:
        if capture:
            if result.stdout:
                print(result.stdout, file=sys.stderr)
            if result.stderr:
                print(result.stderr, file=sys.stderr)
        fail(f"command exited with status {result.returncode}: {' '.join(cmd)}")
    return result


def http_get(
    url: str,
    *,
    headers: dict[str, str] | None = None,
    timeout: int = 30,
    allow_404: bool = False,
) -> tuple[int, bytes]:
    req_headers = {"User-Agent": USER_AGENT}
    if headers:
        req_headers.update(headers)
    req = urllib.request.Request(url, headers=req_headers)
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return resp.status, resp.read()
    except urllib.error.HTTPError as err:
        if allow_404 and err.code == 404:
            return 404, b""
        body = err.read().decode("utf-8", errors="replace")
        fail(f"GET {url} failed with HTTP {err.code}: {body[:300]}")
    except urllib.error.URLError as err:
        fail(f"GET {url} failed: {err}")
    raise AssertionError("unreachable")


def write_github_output(path: str | None, outputs: dict[str, str]) -> None:
    if not path:
        return
    with open(path, "a", encoding="utf-8") as fh:
        for key, value in outputs.items():
            if "\n" in value:
                fail(f"multiline output not supported for {key}")
            fh.write(f"{key}={value}\n")


def append_summary(path: str | None, markdown: str) -> None:
    if not path:
        return
    with open(path, "a", encoding="utf-8") as fh:
        fh.write(markdown)
        if not markdown.endswith("\n"):
            fh.write("\n")


def load_toml(path: Path) -> dict[str, Any]:
    with open(path, "rb") as fh:
        return tomllib.load(fh)


def extract_changelog_section(changelog_text: str, version: str) -> tuple[str, str]:
    header_re = re.compile(
        rf"^## {re.escape(version)} - (\d{{4}}-\d{{2}}-\d{{2}})\s*$",
        re.MULTILINE,
    )
    matches = list(header_re.finditer(changelog_text))
    if len(matches) != 1:
        fail(
            f"expected exactly one '## {version} - YYYY-MM-DD' heading in CHANGELOG.md, "
            f"found {len(matches)}"
        )

    match = matches[0]
    release_date = match.group(1)
    start = match.end()
    next_heading = re.search(r"^## ", changelog_text[start:], re.MULTILINE)
    end = start + next_heading.start() if next_heading else len(changelog_text)
    body = changelog_text[start:end].strip()
    if not body:
        fail(f"CHANGELOG.md section for {version} is empty")

    compare_matches = COMPARE_URL_RE.findall(body)
    expected_suffix = f"...v{version}"
    matching_urls = [u for u in compare_matches if u.endswith(expected_suffix)]
    if len(matching_urls) != 1:
        fail(
            f"CHANGELOG.md section for {version} must contain exactly one compare URL "
            f"ending with '{expected_suffix}', found {matching_urls}"
        )

    return release_date, body + "\n"


def validate_manifests(root: Path, version: str) -> tuple[str, dict[str, Any]]:
    root_manifest = load_toml(root / "Cargo.toml")
    ws_pkg = root_manifest.get("workspace", {}).get("package", {})
    ws_version = ws_pkg.get("version")
    if ws_version != version:
        fail(f"Cargo.toml [workspace.package].version is {ws_version!r}, expected {version!r}")

    msrv = ws_pkg.get("rust-version")
    if not isinstance(msrv, str) or not msrv.strip():
        fail("Cargo.toml [workspace.package].rust-version must be a non-empty string")

    ws_deps = root_manifest.get("workspace", {}).get("dependencies", {})
    expected_pin = f"={version}"
    for dep_name in INTERNAL_WORKSPACE_DEPS:
        dep_spec = ws_deps.get(dep_name)
        dep_version = dep_spec.get("version") if isinstance(dep_spec, dict) else dep_spec
        if dep_version != expected_pin:
            fail(
                f"Cargo.toml [workspace.dependencies].{dep_name}.version is {dep_version!r}, "
                f"expected {expected_pin!r}"
            )

    for crate_name, rel_dir in CRATES:
        manifest_path = root / rel_dir / "Cargo.toml"
        pkg = load_toml(manifest_path).get("package", {})
        if pkg.get("name") != crate_name:
            fail(f"{manifest_path} package.name is {pkg.get('name')!r}, expected {crate_name!r}")
        if pkg.get("version") != {"workspace": True}:
            fail(f"{manifest_path} must inherit version.workspace = true")
        if pkg.get("publish") != ["crates-io"]:
            fail(f"{manifest_path} must declare publish = [\"crates-io\"]")

    return msrv, root_manifest


def cmd_metadata(args: argparse.Namespace) -> None:
    version = args.version
    if not SEMVER_RE.match(version):
        fail(f"invalid version {version!r}; expected semver without leading 'v' (e.g. 0.2.0)")

    root = Path.cwd()
    msrv, _ = validate_manifests(root, version)
    changelog = (root / "CHANGELOG.md").read_text(encoding="utf-8")
    release_date, notes = extract_changelog_section(changelog, version)
    compare_url = [
        u for u in COMPARE_URL_RE.findall(notes) if u.endswith(f"...v{version}")
    ][0]

    notes_path = Path(args.notes_out)
    notes_path.parent.mkdir(parents=True, exist_ok=True)
    notes_path.write_text(notes, encoding="utf-8", newline="\n")

    tag = f"v{version}"
    print(f"Validated version={version} tag={tag} msrv={msrv} date={release_date}")
    print(f"Compare URL: {compare_url}")
    write_github_output(
        args.github_output,
        {
            "version": version,
            "tag": tag,
            "msrv": msrv,
            "release_date": release_date,
            "compare_url": compare_url,
        },
    )
    append_summary(
        args.summary_out,
        (
            f"## Release preflight (`{tag}`)\n\n"
            f"- **Version**: `{version}` ({release_date})\n"
            f"- **Workspace MSRV**: `{msrv}`\n"
            f"- **Compare URL**: {compare_url}\n\n"
            f"<details>\n<summary>Extracted release notes</summary>\n\n"
            f"{notes}\n"
            f"</details>\n"
        ),
    )


def cmd_check_ci(args: argparse.Namespace) -> None:
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    headers = {"Accept": "application/vnd.github+json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"

    encoded_check_name = urllib.parse.quote(args.check_name, safe="")
    url = (
        f"https://api.github.com/repos/{args.repo}/commits/{args.sha}/check-runs"
        f"?check_name={encoded_check_name}&per_page=100"
    )
    _, raw = http_get(url, headers=headers)
    payload = json.loads(raw.decode("utf-8"))
    runs = [
        r for r in payload.get("check_runs", []) if r.get("name") == args.check_name
    ]
    runs.sort(key=lambda r: r.get("id", 0))
    latest = runs[-1] if runs else None

    if latest and latest.get("status") == "completed" and latest.get("conclusion") == "success":
        html_url = latest.get("html_url", "")
        print(f"Check '{args.check_name}' passed on {args.sha}: {html_url}")
        append_summary(
            args.summary_out,
            f"- **Main CI (`{args.check_name}`)**: passed ({html_url})\n",
        )
        return

    status_desc = (
        f"status={latest.get('status')} conclusion={latest.get('conclusion')}"
        if latest
        else "not found"
    )
    message = f"Check '{args.check_name}' on {args.sha} is not green ({status_desc})"
    if args.require_green:
        fail(message)
    print(f"warning: {message} (allowed in dry-run mode)")
    append_summary(
        args.summary_out,
        f"- **Main CI (`{args.check_name}`)**: warning — {status_desc} (dry-run only)\n",
    )


def format_dep_table(spec: Any) -> str:
    if isinstance(spec, str):
        return json.dumps(spec)
    if isinstance(spec, dict):
        parts: list[str] = []
        for k, v in spec.items():
            if k == "path":
                continue
            if isinstance(v, bool):
                parts.append(f"{k} = {'true' if v else 'false'}")
            elif isinstance(v, str):
                parts.append(f"{k} = {json.dumps(v)}")
            elif isinstance(v, list):
                items = ", ".join(json.dumps(x) for x in v)
                parts.append(f"{k} = [{items}]")
        return "{ " + ", ".join(parts) + " }"
    fail(f"unsupported dependency spec: {spec!r}")
    raise AssertionError("unreachable")


def cargo_target_dir(root: Path, toolchain: str | None) -> Path:
    cmd = ["cargo"]
    if toolchain:
        cmd.append(f"+{toolchain}")
    cmd.extend(["metadata", "--format-version", "1", "--no-deps"])
    res = run(cmd, cwd=root, capture=True)
    return Path(json.loads(res.stdout)["target_directory"])


def cmd_verify_packages(args: argparse.Namespace) -> None:
    root = Path.cwd()
    version = args.version
    msrv, root_manifest = validate_manifests(root, version)
    toolchain = args.toolchain or msrv
    cargo_prefix = ["cargo", f"+{toolchain}"]

    target_dir = cargo_target_dir(root, toolchain)
    package_dir = target_dir / "package"

    package_cmd = [*cargo_prefix, "package", "--locked", "--no-verify"]
    for crate_name, _ in CRATES:
        package_cmd.extend(["-p", crate_name])
    run(package_cmd, cwd=root)

    crate_archives: dict[str, Path] = {}
    for crate_name, _ in CRATES:
        archive_path = package_dir / f"{crate_name}-{version}.crate"
        if not archive_path.is_file():
            fail(f"expected package archive not found: {archive_path}")

        prefix = f"{crate_name}-{version}/"
        with tarfile.open(archive_path, "r:gz") as tf:
            names = set(tf.getnames())
        required_files = [f"{prefix}Cargo.toml", f"{prefix}LICENSE"]
        if crate_name == "gpui-material":
            required_files.append(f"{prefix}README.md")
        for req in required_files:
            if req not in names:
                fail(f"{archive_path.name} is missing required file {req}")
        crate_archives[crate_name] = archive_path

    material_manifest = load_toml(root / "crates/material/Cargo.toml")
    facade_features = sorted(
        k for k in material_manifest.get("features", {}).keys() if k != "default"
    )
    ws_deps = root_manifest["workspace"]["dependencies"]
    gpui_spec = format_dep_table(ws_deps["gpui"])
    gpui_base_spec = format_dep_table(ws_deps["gpui-base"])
    theme_manifest = load_toml(root / "crates/theme/Cargo.toml")
    serde_json_raw = theme_manifest["dev-dependencies"]["serde_json"]
    serde_json_ver = (
        serde_json_raw["version"] if isinstance(serde_json_raw, dict) else serde_json_raw
    )

    with tempfile.TemporaryDirectory(prefix="gpui-material-verify-") as tmp_str:
        tmp = Path(tmp_str)
        unpacked = tmp / "unpacked"
        unpacked.mkdir()

        unpacked_dirs: dict[str, Path] = {}
        for crate_name, archive_path in crate_archives.items():
            with tarfile.open(archive_path, "r:gz") as tf:
                tf.extractall(unpacked, filter="data")
            unpacked_dirs[crate_name] = (unpacked / f"{crate_name}-{version}").resolve()

        facade_patch_block = "\n".join(
            [
                "[patch.crates-io]",
                *(
                    f'{name} = {{ path = {json.dumps(unpacked_dirs[name].as_posix())} }}'
                    for name, _ in CRATES
                    if name != "gpui-material-design-symbols"
                ),
            ]
        )
        symbols_patch_block = "\n".join(
            [
                "[patch.crates-io]",
                f'gpui-material-design-symbols = {{ path = {json.dumps(unpacked_dirs["gpui-material-design-symbols"].as_posix())} }}',
            ]
        )

        # 1. Downstream facade consumer (default + all features)
        facade_dir = tmp / "facade-consumer"
        (facade_dir / "src").mkdir(parents=True)
        all_facade_list = ", ".join(
            json.dumps(f"gpui-material/{feat}") for feat in facade_features
        )
        (facade_dir / "Cargo.toml").write_text(
            f"""[package]
name = "facade-consumer"
version = "0.0.0"
edition = "2024"
publish = false

[features]
default = []
all-facade = [{all_facade_list}, "dep:serde_json"]

[dependencies]
gpui = {gpui_spec}
gpui-base = {gpui_base_spec}
gpui-material = "={version}"
serde_json = {{ version = {json.dumps(serde_json_ver)}, optional = true }}

{facade_patch_block}
""",
            encoding="utf-8",
            newline="\n",
        )
        (facade_dir / "src/lib.rs").write_text(
            """pub fn smoke_default(window: &gpui::Window, cx: &gpui::App) -> gpui_base::Theme {
    let _ = gpui_material::MaterialTheme::baseline(gpui_material::theme::Appearance::Light);
    let _ = gpui_material::Button::new("ok", "OK");
    gpui_material::current_base_theme(window, cx)
}

#[cfg(feature = "all-facade")]
pub fn smoke_all_features() {
    let theme = gpui_material::MaterialTheme::from_seed(
        gpui_material::theme::Argb::from_u32(0xFF6750A4),
        gpui_material::theme::Appearance::Light,
        0.0,
    );
    let _ = serde_json::to_string(&theme);
    let _ = std::any::TypeId::of::<gpui_material::theme::DynamicVariant>();
    let _ = std::any::TypeId::of::<gpui_material::Carousel>();
    let _ = std::any::TypeId::of::<gpui_material::ButtonGroup>();
    let _ = std::any::TypeId::of::<gpui_material::SearchBar>();
    let _ = std::any::TypeId::of::<gpui_material::Toolbar>();
    let _ = std::any::TypeId::of::<gpui_material::date_time::DatePicker>();
    let _ = std::any::TypeId::of::<gpui_material::window_chrome::TitleBar>();
}
""",
            encoding="utf-8",
            newline="\n",
        )

        # 2. Downstream symbols consumer (without and with embed-assets)
        symbols_dir = tmp / "symbols-consumer"
        (symbols_dir / "src").mkdir(parents=True)
        (symbols_dir / "Cargo.toml").write_text(
            f"""[package]
name = "symbols-consumer"
version = "0.0.0"
edition = "2024"
publish = false

[features]
default = []
embed-assets = ["gpui-material-design-symbols/embed-assets"]

[dependencies]
gpui-material-design-symbols = {{ version = "={version}", default-features = false }}

{symbols_patch_block}
""",
            encoding="utf-8",
            newline="\n",
        )
        (symbols_dir / "src/lib.rs").write_text(
            """use gpui_material_design_symbols::{Symbol, SymbolAxes, SymbolFont, SymbolStyle};

pub fn smoke_symbols(bytes: &[u8]) {
    let _ = Symbol::Favorite.codepoint();
    let _ = SymbolStyle::Outlined;
    let _ = SymbolAxes::default().filled(true);
    let _ = SymbolFont::from_bytes(bytes.to_vec());
}

#[cfg(feature = "embed-assets")]
pub fn smoke_embedded() {
    let _ = SymbolFont::embedded(SymbolStyle::Outlined);
}
""",
            encoding="utf-8",
            newline="\n",
        )

        # Reuse the active target directory so CI rust-cache and local dev builds share compiled dependencies.
        verify_env = os.environ.copy()
        verify_env.setdefault("CARGO_TARGET_DIR", str(target_dir))
        if args.offline:
            offline_flags = ["--offline"]
        else:
            offline_flags = []

        run([*cargo_prefix, "check", *offline_flags], cwd=facade_dir, env=verify_env)
        run(
            [*cargo_prefix, "check", *offline_flags, "--features", "all-facade"],
            cwd=facade_dir,
            env=verify_env,
        )
        run([*cargo_prefix, "check", *offline_flags], cwd=symbols_dir, env=verify_env)
        run(
            [*cargo_prefix, "check", *offline_flags, "--features", "embed-assets"],
            cwd=symbols_dir,
            env=verify_env,
        )

    append_summary(
        args.summary_out,
        (
            f"- **Packaged & verified crates (MSRV `{toolchain}`)**: "
            f"all 6 `.crate` archives verified via downstream `facade-consumer` "
            f"(default + {len(facade_features)} optional features) and `symbols-consumer` "
            f"(default + `embed-assets`).\n"
        ),
    )


def sparse_index_url(crate_name: str) -> str:
    lower = crate_name.lower()
    if len(lower) == 1:
        prefix = "1"
    elif len(lower) == 2:
        prefix = "2"
    elif len(lower) == 3:
        prefix = f"3/{lower[0]}"
    else:
        prefix = f"{lower[:2]}/{lower[2:4]}"
    return f"https://index.crates.io/{prefix}/{lower}"


def crates_io_version(crate_name: str, version: str) -> dict[str, Any] | None:
    status, raw = http_get(
        f"https://crates.io/api/v1/crates/{crate_name}/{version}",
        allow_404=True,
    )
    if status == 404:
        return None
    return json.loads(raw.decode("utf-8"))["version"]


def sparse_index_has_version(crate_name: str, version: str) -> bool:
    status, raw = http_get(sparse_index_url(crate_name), allow_404=True)
    if status == 404:
        return False
    for line in raw.decode("utf-8").splitlines():
        line = line.strip()
        if not line:
            continue
        entry = json.loads(line)
        if entry.get("vers") == version:
            if entry.get("yanked"):
                fail(f"{crate_name} {version} is yanked in crates.io sparse index")
            return True
    return False


def wait_for_registry(crate_name: str, version: str, timeout_s: int = 300) -> dict[str, Any]:
    deadline = time.monotonic() + timeout_s
    while time.monotonic() < deadline:
        meta = crates_io_version(crate_name, version)
        if meta is not None and sparse_index_has_version(crate_name, version):
            if meta.get("yanked"):
                fail(f"{crate_name} {version} is yanked on crates.io")
            return meta
        print(f"Waiting for {crate_name} {version} to appear in crates.io sparse index...", flush=True)
        time.sleep(5)
    fail(f"timed out waiting for {crate_name} {version} on crates.io sparse index")
    raise AssertionError("unreachable")


def cmd_publish_crates(args: argparse.Namespace) -> None:
    root = Path.cwd()
    version = args.version
    validate_manifests(root, version)

    token = os.environ.get("CARGO_REGISTRY_TOKEN", "")
    if not token:
        fail("CARGO_REGISTRY_TOKEN is not set while publishing crates")

    results: list[tuple[str, str]] = []

    for crate_name, _ in CRATES:
        existing = crates_io_version(crate_name, version)
        if existing is not None:
            print(
                f"{crate_name} {version} already exists on crates.io; "
                "skipping upload and verifying registry metadata.",
                flush=True,
            )
        else:
            run(["cargo", "publish", "--locked", "-p", crate_name], cwd=root)
        meta = wait_for_registry(crate_name, version)

        expected_sha = meta["checksum"]
        _, crate_bytes = http_get(
            f"https://static.crates.io/crates/{crate_name}/{crate_name}-{version}.crate",
            timeout=60,
        )
        actual_sha = hashlib.sha256(crate_bytes).hexdigest()
        if actual_sha != expected_sha:
            fail(
                f"checksum mismatch for {crate_name} {version}: "
                f"downloaded {actual_sha}, registry reported {expected_sha}"
            )
        results.append((crate_name, actual_sha))
        print(f"Verified {crate_name} {version} sha256={actual_sha}", flush=True)

    rows = "\n".join(
        f"| `{name}` | `{version}` | `{sha}` |" for name, sha in results
    )
    append_summary(
        args.summary_out,
        (
            "## crates.io publication\n\n"
            "| Crate | Version | SHA-256 |\n"
            "| --- | --- | --- |\n"
            f"{rows}\n"
        ),
    )


def gh_api_json(endpoint: str, extra_args: list[str] | None = None) -> tuple[int, dict[str, Any]]:
    cmd = ["gh", "api", endpoint]
    if extra_args:
        cmd.extend(extra_args)
    res = subprocess.run(cmd, text=True, capture_output=True)
    if res.returncode != 0:
        return res.returncode, {"stderr": res.stderr.strip(), "stdout": res.stdout.strip()}
    return 0, json.loads(res.stdout) if res.stdout.strip() else {}


def ensure_annotated_tag(repo: str, tag: str, version: str, target_sha: str) -> None:
    rc, ref_data = gh_api_json(f"repos/{repo}/git/ref/tags/{tag}")
    if rc == 0:
        obj = ref_data.get("object", {})
        obj_type = obj.get("type")
        obj_sha = obj.get("sha")
        if obj_type == "tag":
            rc2, tag_obj = gh_api_json(f"repos/{repo}/git/tags/{obj_sha}")
            if rc2 != 0:
                fail(f"failed to resolve existing tag object {tag}: {tag_obj}")
            resolved_sha = tag_obj.get("object", {}).get("sha")
        else:
            resolved_sha = obj_sha
        if resolved_sha != target_sha:
            fail(
                f"tag {tag} already exists and points to {resolved_sha}, "
                f"not expected commit {target_sha}"
            )
        print(f"Tag {tag} already points to {target_sha}; reusing existing tag.")
        return

    rc_tag, tag_obj = gh_api_json(
        f"repos/{repo}/git/tags",
        [
            "-X",
            "POST",
            "-f",
            f"tag={tag}",
            "-f",
            f"message=GPUI Material {version}",
            "-f",
            f"object={target_sha}",
            "-f",
            "type=commit",
        ],
    )
    if rc_tag != 0:
        fail(f"failed to create annotated tag object {tag}: {tag_obj}")
    tag_sha = tag_obj["sha"]
    rc_ref, ref_res = gh_api_json(
        f"repos/{repo}/git/refs",
        ["-X", "POST", "-f", f"ref=refs/tags/{tag}", "-f", f"sha={tag_sha}"],
    )
    if rc_ref != 0:
        fail(f"failed to create ref refs/tags/{tag}: {ref_res}")
    print(f"Created annotated tag {tag} ({tag_sha}) -> {target_sha}")


def verify_source_archives(repo: str, version: str) -> list[tuple[str, int, int, str]]:
    tag = f"v{version}"
    verified: list[tuple[str, int, int, str]] = []
    for kind in ("tar.gz", "zip"):
        url = f"https://github.com/{repo}/archive/refs/tags/{tag}.{kind}"
        data: bytes | None = None
        last_err: Exception | None = None
        for attempt in range(1, 6):
            try:
                _, data = http_get(url, timeout=60)
                break
            except SystemExit:
                if attempt == 5:
                    raise
                time.sleep(3)
            except Exception as err:  # pragma: no cover
                last_err = err
                time.sleep(3)
        if data is None:
            fail(f"failed to download {url}: {last_err}")

        sha = hashlib.sha256(data).hexdigest()
        cargo_entry = f"gpui-material-{version}/Cargo.toml"
        if kind == "tar.gz":
            with tarfile.open(fileobj=io.BytesIO(data), mode="r:gz") as tf:
                member = tf.extractfile(cargo_entry)
                if member is None:
                    fail(f"{tag}.{kind} is missing {cargo_entry}")
                cargo_text = member.read().decode("utf-8")
                count = len(tf.getnames())
        else:
            with zipfile.ZipFile(io.BytesIO(data)) as zf:
                cargo_text = zf.read(cargo_entry).decode("utf-8")
                count = len(zf.namelist())

        if f'version = "{version}"' not in cargo_text:
            fail(f"{tag}.{kind} {cargo_entry} does not declare version = \"{version}\"")
        verified.append((kind, len(data), count, sha))
        print(f"Verified archive {tag}.{kind}: bytes={len(data)} entries={count} sha256={sha}")
    return verified


def cmd_publish_github_release(args: argparse.Namespace) -> None:
    repo = args.repo
    version = args.version
    tag = f"v{version}"

    ensure_annotated_tag(repo, tag, version, args.sha)

    view = subprocess.run(
        ["gh", "release", "view", tag, "--repo", repo, "--json", "url,isDraft"],
        text=True,
        capture_output=True,
    )
    if view.returncode == 0:
        info = json.loads(view.stdout)
        if info.get("isDraft"):
            fail(f"GitHub Release {tag} already exists as a draft; resolve manually")
        release_url = info["url"]
        print(f"GitHub Release {tag} already exists: {release_url}")
    else:
        created = run(
            [
                "gh",
                "release",
                "create",
                tag,
                "--repo",
                repo,
                "--verify-tag",
                "--title",
                tag,
                "--notes-file",
                args.notes,
            ],
            capture=True,
        )
        release_url = created.stdout.strip().splitlines()[-1]
        print(f"Published GitHub Release {tag}: {release_url}")

    archives = verify_source_archives(repo, version)
    cmp_status, _ = http_get(args.compare_url, timeout=30)
    if cmp_status != 200:
        fail(f"compare URL returned HTTP {cmp_status}: {args.compare_url}")
    print(f"Verified compare URL (HTTP {cmp_status}): {args.compare_url}")

    write_github_output(args.github_output, {"release_url": release_url})
    archive_rows = "\n".join(
        f"| `{tag}.{kind}` | {size:,} | {entries} | `{sha}` |"
        for kind, size, entries, sha in archives
    )
    append_summary(
        args.summary_out,
        (
            "## GitHub Release\n\n"
            f"- **Release**: {release_url}\n"
            f"- **Commit**: `{args.sha}`\n"
            f"- **Compare**: {args.compare_url} (HTTP {cmp_status})\n\n"
            "| Archive | Bytes | Entries | SHA-256 |\n"
            "| --- | ---: | ---: | --- |\n"
            f"{archive_rows}\n"
        ),
    )


def main() -> None:
    parser = argparse.ArgumentParser(description="GPUI Material release helper")
    sub = parser.add_subparsers(dest="command", required=True)

    p_meta = sub.add_parser("metadata", help="Validate manifests and extract changelog notes")
    p_meta.add_argument("--version", required=True)
    p_meta.add_argument("--notes-out", required=True)
    p_meta.add_argument("--github-output")
    p_meta.add_argument("--summary-out")
    p_meta.set_defaults(func=cmd_metadata)

    p_ci = sub.add_parser("check-ci", help="Verify CI required status on target commit")
    p_ci.add_argument("--repo", required=True)
    p_ci.add_argument("--sha", required=True)
    p_ci.add_argument("--check-name", default="CI required")
    p_ci.add_argument("--require-green", action="store_true")
    p_ci.add_argument("--summary-out")
    p_ci.set_defaults(func=cmd_check_ci)

    p_verify = sub.add_parser(
        "verify-packages", help="Package all six crates and verify from downstream consumers"
    )
    p_verify.add_argument("--version", required=True)
    p_verify.add_argument("--toolchain", help="Rustup toolchain override (defaults to workspace MSRV)")
    p_verify.add_argument("--offline", action="store_true")
    p_verify.add_argument("--summary-out")
    p_verify.set_defaults(func=cmd_verify_packages)

    p_pub = sub.add_parser("publish-crates", help="Publish all six crates in dependency order")
    p_pub.add_argument("--version", required=True)
    p_pub.add_argument("--summary-out")
    p_pub.set_defaults(func=cmd_publish_crates)

    p_rel = sub.add_parser(
        "publish-github-release", help="Create annotated tag, GitHub Release, and verify archives"
    )
    p_rel.add_argument("--repo", required=True)
    p_rel.add_argument("--version", required=True)
    p_rel.add_argument("--sha", required=True)
    p_rel.add_argument("--notes", required=True)
    p_rel.add_argument("--compare-url", required=True)
    p_rel.add_argument("--github-output")
    p_rel.add_argument("--summary-out")
    p_rel.set_defaults(func=cmd_publish_github_release)

    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
