use std::{collections::BTreeMap, path::Path};

use serde::Deserialize;

use crate::{Result, output};

#[derive(Deserialize)]
struct Status {
    schema_version: u8,
    current_phase: String,
    allowed_paths: Vec<String>,
    blockers: Vec<toml::Value>,
    #[serde(default)]
    repairs: Vec<toml::Value>,
    frozen_commits: BTreeMap<String, String>,
    #[serde(default)]
    evidence_locations: BTreeMap<String, String>,
    phases: BTreeMap<String, Phase>,
}

#[derive(Deserialize, PartialEq)]
struct Phase {
    status: Stage,
    frozen_on: Option<toml::value::Datetime>,
    #[serde(default)]
    entry_conditions: Vec<String>,
    #[serde(default)]
    evidence: Vec<String>,
    #[serde(default)]
    merged_deliverables: Vec<Deliverable>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Stage {
    Planned,
    Active,
    Review,
    Blocked,
    Frozen,
}

#[derive(Deserialize, PartialEq)]
struct Deliverable {
    task: String,
    issue: u64,
    pull_request: u64,
    commit: String,
}

pub(crate) fn check(document: &str, base: Option<&str>) -> Result {
    let status: Status = toml::from_str(document)?;
    status.validate()?;
    if let Some(base) = base {
        let base = output("git", &["rev-parse", "--verify", "--end-of-options", base])?;
        let base = base.trim();
        let base_document = output("git", &["show", &format!("{base}:status.toml")])?;
        let paths = output(
            "git",
            &[
                "diff",
                "--name-only",
                "--no-renames",
                "-z",
                base,
                "HEAD",
                "--",
            ],
        )?;
        let previous: Status = toml::from_str(&base_document)?;
        status.check_paths(&previous, paths.split_terminator('\0'))?;
    }
    println!(
        "Stage {} ({:?}) verified",
        status.current_phase,
        status.current()?.status
    );
    Ok(())
}

impl Status {
    fn current(&self) -> Result<&Phase> {
        self.phases
            .get(&self.current_phase)
            .ok_or_else(|| format!("unknown current phase {}", self.current_phase).into())
    }

    fn validate(&self) -> Result {
        if self.schema_version != 1 || self.phases.len() != 10 {
            return Err("status.toml must use schema 1 and record P0 through P9".into());
        }
        let current = self.current()?;
        if current.status == Stage::Planned {
            return Err("the current phase must have started".into());
        }
        let has_scope = !self.allowed_paths.is_empty();
        let has_blockers = !self.blockers.is_empty();
        if has_scope == (current.status == Stage::Frozen)
            || has_blockers != (current.status == Stage::Blocked)
            || self.allowed_paths.iter().any(|path| !is_repo_path(path))
        {
            return Err("allowed_paths or blockers do not match the current phase".into());
        }

        let mut frozen = 0;
        for index in 0..10 {
            let name = format!("P{index}");
            let phase = self
                .phases
                .get(&name)
                .ok_or_else(|| format!("missing {name}"))?;
            let valid = match name.cmp(&self.current_phase) {
                std::cmp::Ordering::Less => phase.status == Stage::Frozen,
                std::cmp::Ordering::Equal => true,
                std::cmp::Ordering::Greater => phase.status == Stage::Planned,
            };
            if !valid {
                return Err(format!("{name} is out of stage order").into());
            }
            if phase.status == Stage::Frozen {
                frozen += 1;
                self.check_frozen(&name, phase)?;
            }
        }
        if self.frozen_commits.len() != frozen {
            return Err("frozen_commits must record exactly the frozen phases".into());
        }
        for path in self.evidence_locations.keys() {
            if !self
                .phases
                .values()
                .any(|phase| phase.status == Stage::Frozen && phase.evidence.contains(path))
            {
                return Err(format!("unknown historical evidence path: {path}").into());
            }
        }
        Ok(())
    }

    fn check_frozen(&self, name: &str, phase: &Phase) -> Result {
        let accepted = self
            .frozen_commits
            .get(name)
            .ok_or_else(|| format!("{name} has no accepted commit"))?;
        if phase.frozen_on.is_none() || phase.merged_deliverables.is_empty() {
            return Err(format!("{name} needs a freeze date and merged deliverables").into());
        }
        check_commit(accepted, "HEAD")?;
        for path in &phase.evidence {
            let location = self.evidence_locations.get(path).unwrap_or(path);
            if !is_repo_path(path) || !is_repo_path(location) || !Path::new(location).is_file() {
                return Err(format!("{name} evidence file missing: {location}").into());
            }
            output("git", &["cat-file", "-e", &format!("{accepted}:{path}")])?;
        }
        for delivery in &phase.merged_deliverables {
            if !delivery.task.starts_with(&format!("{name}-"))
                || delivery.issue == 0
                || delivery.pull_request == 0
            {
                return Err(format!("{name} has an invalid deliverable: {}", delivery.task).into());
            }
            check_commit(&delivery.commit, accepted)?;
        }
        Ok(())
    }

    fn check_paths<'a>(&self, base: &Self, paths: impl Iterator<Item = &'a str>) -> Result {
        let stage = self.current()?.status;
        let previous_stage = base.current()?.status;
        if self.current_phase == base.current_phase
            && stage == Stage::Frozen
            && previous_stage == Stage::Frozen
        {
            return self.check_frozen_maintenance(base, paths);
        }
        if self.current_phase == base.current_phase && previous_stage != Stage::Frozen {
            if stage == Stage::Frozen {
                return require_paths(paths, &["status.toml".into()]);
            }
            if self.allowed_paths != base.allowed_paths {
                return Err("the PR changed the active phase's allowed_paths".into());
            }
            return require_paths(paths, &base.allowed_paths);
        }
        if previous_stage == Stage::Frozen
            && stage == Stage::Active
            && self
                .current_phase
                .as_bytes()
                .get(1)
                .zip(base.current_phase.as_bytes().get(1))
                .is_some_and(|(next, previous)| *next == previous + 1)
        {
            let index = &self.current_phase[1..];
            let phase = format!("docs/phases/P0{index}");
            return require_paths(
                paths,
                &[
                    "status.toml".into(),
                    "docs/index.md".into(),
                    "docs/index.zh.md".into(),
                    "docs/index.i18n.yaml".into(),
                    "docs/phases.md".into(),
                    "docs/phases.zh.md".into(),
                    "docs/phases.i18n.yaml".into(),
                    format!("{phase}.md"),
                    format!("{phase}.zh.md"),
                    format!("{phase}.i18n.yaml"),
                ],
            );
        }
        require_paths(paths, &[])
    }

    fn check_frozen_maintenance<'a>(
        &self,
        base: &Self,
        paths: impl Iterator<Item = &'a str>,
    ) -> Result {
        if self.schema_version != base.schema_version
            || self.frozen_commits != base.frozen_commits
            || self.phases != base.phases
            || self.repairs != base.repairs
        {
            return Err("maintenance must preserve frozen phase records".into());
        }
        let moves_evidence = self.evidence_locations != base.evidence_locations;
        let paths = paths.collect::<Vec<_>>();
        let updates_maintenance_scope = paths
            .iter()
            .any(|path| is_p1_experiment_path(path) || is_github_maintenance_path(path));
        let updates_token_index = paths.contains(&"docs/theme/token-index.md");
        let updates_contribution_guide = paths.contains(&"CONTRIBUTION.md");
        let denied = paths
            .into_iter()
            .filter(|path| {
                !is_maintenance_path(path, moves_evidence)
                    && !is_p1_experiment_path(path)
                    && !(updates_maintenance_scope && *path == "xtask/src/phase.rs")
                    && !(updates_token_index && *path == "xtask/src/tokens.rs")
                    && !(updates_contribution_guide
                        && matches!(*path, "xtask/src/phase.rs" | "xtask/src/docs/i18n.rs"))
            })
            .collect::<Vec<_>>();
        if !denied.is_empty() {
            return Err(format!(
                "paths outside frozen maintenance scope: {}",
                denied.join(", ")
            )
            .into());
        }
        Ok(())
    }
}

fn is_p1_experiment_path(path: &str) -> bool {
    [
        "experiments/p1-host/",
        "experiments/p1-shapes/",
        "experiments/p1-theme/",
    ]
    .iter()
    .any(|prefix| path.starts_with(prefix))
        && is_repo_path(path)
}

fn is_maintenance_path(path: &str, moves_evidence: bool) -> bool {
    matches!(
        path,
        "README.md"
            | "README.zh.md"
            | "README.i18n.yaml"
            | "CONTRIBUTION.md"
            | "CONTRIBUTION.zh.md"
            | "CONTRIBUTION.i18n.yaml"
            | "THIRD_PARTY_NOTICES.md"
            | "THIRD_PARTY_NOTICES.zh.md"
            | "THIRD_PARTY_NOTICES.i18n.yaml"
    ) || (path.starts_with("docs/")
        && (path.ends_with(".md") || path.ends_with(".i18n.yaml"))
        && is_repo_path(path))
        || is_github_maintenance_path(path)
        || (moves_evidence
            && matches!(
                path,
                "status.toml"
                    | "xtask/src/phase.rs"
                    | "xtask/src/main.rs"
                    | "xtask/src/docs.rs"
                    | "xtask/src/docs/i18n.rs"
            ))
}

fn is_github_maintenance_path(path: &str) -> bool {
    matches!(
        path,
        ".github/workflows/ci.yml" | ".github/pull_request_template.md"
    ) || (path.starts_with(".github/ISSUE_TEMPLATE/")
        && path.ends_with(".md")
        && is_repo_path(path))
}

fn check_commit(commit: &str, descendant: &str) -> Result {
    if commit.len() != 40 || !commit.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("expected a full Git SHA: {commit}").into());
    }
    output("git", &["merge-base", "--is-ancestor", commit, descendant])?;
    Ok(())
}

fn is_repo_path(path: &str) -> bool {
    !path.contains(['\\', ':'])
        && path
            .trim_end_matches('/')
            .split('/')
            .all(|part| !matches!(part, "" | "." | ".."))
}

fn require_paths<'a>(paths: impl Iterator<Item = &'a str>, allowed: &[String]) -> Result {
    let denied = paths
        .filter(|path| {
            !allowed.iter().any(|entry| {
                if entry.ends_with('/') {
                    path.starts_with(entry)
                } else {
                    *path == entry
                }
            })
        })
        .collect::<Vec<_>>();
    if denied.is_empty() {
        Ok(())
    } else {
        Err(format!("paths outside the base phase scope: {}", denied.join(", ")).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_distinguishes_files_and_directories() {
        let allowed = ["README.md".into(), "apps/gallery/".into()];
        assert!(
            require_paths(
                ["README.md", "apps/gallery/src/main.rs"].into_iter(),
                &allowed
            )
            .is_ok()
        );
        assert!(
            require_paths(
                ["README.md.bak", "apps/gallery-backup/src/main.rs"].into_iter(),
                &allowed
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_paths_outside_the_checkout() {
        for path in [
            "",
            "/",
            "/docs/",
            "docs/../Cargo.toml",
            "C:/docs",
            "docs\\index.md",
        ] {
            assert!(!is_repo_path(path), "{path}");
        }
        assert!(is_repo_path("docs/phases/"));
    }

    #[test]
    fn frozen_maintenance_allows_docs_ci_and_templates() {
        assert!(is_maintenance_path("docs/architecture.md", false));
        assert!(is_maintenance_path("README.md", false));
        assert!(is_maintenance_path("README.zh.md", false));
        assert!(is_maintenance_path("docs/architecture.i18n.yaml", false));
        assert!(!is_maintenance_path("docs/architecture.yaml", false));
        assert!(is_maintenance_path(".github/workflows/ci.yml", false));
        assert!(is_maintenance_path(
            ".github/pull_request_template.md",
            false
        ));
        assert!(is_maintenance_path(
            ".github/ISSUE_TEMPLATE/bug_report.md",
            false
        ));
        assert!(is_maintenance_path("status.toml", true));
        assert!(is_maintenance_path("xtask/src/phase.rs", true));
        assert!(!is_maintenance_path("status.toml", false));
        assert!(!is_maintenance_path("xtask/src/phase.rs", false));
        assert!(!is_maintenance_path("docs/../README.md", true));
        assert!(!is_maintenance_path(".github/workflows/release.yml", true));
        assert!(!is_maintenance_path(
            ".github/ISSUE_TEMPLATE/../../README.md",
            true
        ));
        assert!(!is_maintenance_path("apps/gallery/src/main.rs", true));
    }

    #[test]
    fn frozen_ci_maintenance_preserves_phase_scope() {
        let base: Status = toml::from_str(include_str!("../../status.toml")).unwrap();
        base.check_frozen_maintenance(
            &base,
            [".github/workflows/ci.yml", "xtask/src/phase.rs"].into_iter(),
        )
        .unwrap();
        assert!(
            base.check_frozen_maintenance(&base, ["xtask/src/phase.rs"].into_iter())
                .is_err()
        );
        assert!(
            base.check_frozen_maintenance(
                &base,
                [".github/workflows/ci.yml", "apps/gallery/src/main.rs"].into_iter(),
            )
            .is_err()
        );
    }

    #[test]
    fn frozen_token_index_maintenance_preserves_token_scope() {
        let base: Status = toml::from_str(include_str!("../../status.toml")).unwrap();
        let index_and_generator = ["docs/theme/token-index.md", "xtask/src/tokens.rs"];
        base.check_frozen_maintenance(&base, index_and_generator.into_iter())
            .unwrap();
        assert!(
            base.check_frozen_maintenance(&base, ["xtask/src/tokens.rs"].into_iter())
                .is_err()
        );
        for path in ["tokens/material.json", "crates/theme/src/generated.rs"] {
            assert!(
                base.check_frozen_maintenance(
                    &base,
                    index_and_generator.into_iter().chain([path]),
                )
                .is_err()
            );
        }
    }
}
