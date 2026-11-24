#!/usr/bin/env bash
set -euo pipefail

base=${1:-}
docs=false
website=false
checks=false
release_scripts=false
native=false

if [[ -z "$base" || "$base" =~ ^0+$ ]]; then
  # Manual runs or pushes with no comparison base get the full checks.
  docs=true
  website=true
  checks=true
  release_scripts=true
  native=true
else
  git diff --check "$base" HEAD

  changed_paths=$(mktemp)
  trap 'rm -f "$changed_paths"' EXIT
  # Include both sides of renames so moving files across scopes exercises both.
  git diff --name-only --no-renames -z "$base" HEAD > "$changed_paths"
  while IFS= read -r -d '' path; do
    case "$path" in
      website/*|.github/workflows/website.yml|.github/workflows/website-build.yml)
        website=true
        ;;
      apps/gallery-web/*)
        website=true
        checks=true
        ;;
      docs/*|*.md|*.i18n.yaml)
        docs=true
        website=true
        ;;
      .github/workflows/release.yml|.github/scripts/*release*.py)
        release_scripts=true
        ;;
      apps/gallery/*|crates/*|.cargo/*|Cargo.toml|Cargo.lock|rust-toolchain.toml|.github/workflows/ci.yml|.github/scripts/ci-scope.sh)
        website=true
        native=true
        ;;
      *)
        native=true
        ;;
    esac
  done < "$changed_paths"
fi

if [[ "$native" == true ]]; then
  docs=true
  checks=true
  release_scripts=true
fi
printf 'docs=%s\nwebsite=%s\nchecks=%s\nrelease_scripts=%s\nnative=%s\n' \
  "$docs" "$website" "$checks" "$release_scripts" "$native" >> "$GITHUB_OUTPUT"
