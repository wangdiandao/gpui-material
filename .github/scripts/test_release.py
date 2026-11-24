#!/usr/bin/env python3
"""Unit tests for .github/scripts/release.py."""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest import mock


def _load_release_module():
    module_path = Path(__file__).resolve().with_name("release.py")
    spec = importlib.util.spec_from_file_location("release", module_path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


release = _load_release_module()


class PublishCratesTests(unittest.TestCase):
    def test_publish_passes_locked_and_skips_existing_versions(self) -> None:
        version = "0.3.3"
        existing_crates = {"gpui-material-theme", "gpui-material-foundation"}
        crate_payloads = {
            name: f"archive-bytes-{name}-{version}".encode("utf-8")
            for name, _ in release.CRATES
        }
        checksums = {
            name: hashlib.sha256(payload).hexdigest()
            for name, payload in crate_payloads.items()
        }

        def fake_crates_io_version(crate_name: str, ver: str):
            self.assertEqual(ver, version)
            if crate_name in existing_crates:
                return {"num": ver, "yanked": False, "checksum": checksums[crate_name]}
            return None

        def fake_wait_for_registry(crate_name: str, ver: str, timeout_s: int = 300):
            self.assertEqual(ver, version)
            return {"num": ver, "yanked": False, "checksum": checksums[crate_name]}

        def fake_http_get(url: str, *, headers=None, timeout: int = 30, allow_404: bool = False):
            for name, payload in crate_payloads.items():
                expected_url = f"https://static.crates.io/crates/{name}/{name}-{version}.crate"
                if url == expected_url:
                    return 200, payload
            raise AssertionError(f"unexpected http_get URL: {url}")

        with (
            tempfile.TemporaryDirectory() as tmp,
            mock.patch.dict(os.environ, {"CARGO_REGISTRY_TOKEN": "test-token"}, clear=False),
            mock.patch.object(release, "validate_manifests", return_value=("1.95.0", {})),
            mock.patch.object(release, "crates_io_version", side_effect=fake_crates_io_version),
            mock.patch.object(release, "wait_for_registry", side_effect=fake_wait_for_registry) as wait_mock,
            mock.patch.object(release, "http_get", side_effect=fake_http_get),
            mock.patch.object(release, "run") as run_mock,
        ):
            summary_path = Path(tmp) / "summary.md"
            args = argparse.Namespace(version=version, summary_out=str(summary_path))
            release.cmd_publish_crates(args)

            expected_published = [
                name for name, _ in release.CRATES if name not in existing_crates
            ]
            actual_cmds = [call.args[0] for call in run_mock.call_args_list]
            self.assertEqual(
                actual_cmds,
                [["cargo", "publish", "--locked", "-p", name] for name in expected_published],
            )
            self.assertEqual(wait_mock.call_count, len(release.CRATES))
            summary = summary_path.read_text(encoding="utf-8")
            for name, sha in checksums.items():
                self.assertIn(f"| `{name}` | `{version}` | `{sha}` |", summary)


class CheckCiTests(unittest.TestCase):
    def test_check_ci_filters_by_check_name_query_param(self) -> None:
        captured_urls: list[str] = []

        def fake_http_get(url: str, *, headers=None, timeout: int = 30, allow_404: bool = False):
            captured_urls.append(url)
            payload = {
                "check_runs": [
                    {
                        "id": 10,
                        "name": "CI required",
                        "status": "completed",
                        "conclusion": "success",
                        "html_url": "https://github.com/wangdiandao/gpui-material/runs/10",
                    }
                ]
            }
            return 200, json.dumps(payload).encode("utf-8")

        with mock.patch.object(release, "http_get", side_effect=fake_http_get):
            args = argparse.Namespace(
                repo="wangdiandao/gpui-material",
                sha="deadbeef",
                check_name="CI required",
                require_green=True,
                summary_out=None,
            )
            release.cmd_check_ci(args)

        self.assertEqual(len(captured_urls), 1)
        self.assertIn("check_name=CI%20required", captured_urls[0])
        self.assertIn("per_page=100", captured_urls[0])

    def test_check_ci_fails_when_require_green_and_not_success(self) -> None:
        def fake_http_get(url: str, *, headers=None, timeout: int = 30, allow_404: bool = False):
            payload = {
                "check_runs": [
                    {
                        "id": 11,
                        "name": "CI required",
                        "status": "completed",
                        "conclusion": "failure",
                    }
                ]
            }
            return 200, json.dumps(payload).encode("utf-8")

        with mock.patch.object(release, "http_get", side_effect=fake_http_get):
            args = argparse.Namespace(
                repo="wangdiandao/gpui-material",
                sha="deadbeef",
                check_name="CI required",
                require_green=True,
                summary_out=None,
            )
            with self.assertRaises(SystemExit) as ctx:
                release.cmd_check_ci(args)
            self.assertEqual(ctx.exception.code, 1)


if __name__ == "__main__":
    unittest.main()
