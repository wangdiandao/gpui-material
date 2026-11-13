use std::collections::BTreeMap;

use serde::Deserialize;

use crate::{Result, output};

#[derive(Clone, Deserialize)]
struct Status {
    schema_version: u8,
    current_phase: String,
    allowed_paths: Vec<String>,
    blockers: Vec<toml::Value>,
    #[serde(default)]
    repairs: Vec<toml::Table>,
    frozen_commits: BTreeMap<String, String>,
    phases: BTreeMap<String, Phase>,
}

#[derive(Clone, Deserialize, PartialEq)]
struct Phase {
    status: Stage,
    frozen_on: Option<toml::value::Datetime>,
    #[serde(default)]
    entry_conditions: Vec<String>,
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

#[derive(Clone, Deserialize, PartialEq)]
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
        let open_repairs = self
            .repairs
            .iter()
            .filter(|repair| repair.get("status").and_then(toml::Value::as_str) == Some("open"))
            .count();
        if open_repairs > 1 || (open_repairs == 1 && current.status != Stage::Blocked) {
            return Err("only one repair may be open while the current phase is blocked".into());
        }
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
            if phase.status == Stage::Frozen
                || (phase.status == Stage::Blocked && self.open_repair().is_some())
            {
                frozen += 1;
                self.check_frozen(&name, phase)?;
            }
        }
        if self.frozen_commits.len() != frozen {
            return Err("frozen_commits must record exactly the frozen phases".into());
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
        if self.open_repair().is_some() || base.open_repair().is_some() {
            return self.check_repair_paths(base, paths);
        }
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

    fn open_repair(&self) -> Option<&toml::Table> {
        self.repairs
            .iter()
            .find(|repair| repair.get("status").and_then(toml::Value::as_str) == Some("open"))
    }

    fn check_repair_paths<'a>(&self, base: &Self, paths: impl Iterator<Item = &'a str>) -> Result {
        let mut phases = base.phases.clone();
        phases.get_mut(&base.current_phase).unwrap().status = self.current()?.status;
        let mut commits = base.frozen_commits.clone();
        let mut repairs = base.repairs.clone();
        if self.current_phase != base.current_phase || self.phases != phases {
            return Err("a repair must preserve phase history".into());
        }
        if base.open_repair().is_none() && base.current()?.status != Stage::Frozen {
            return Err("a frozen-phase repair requires an existing freeze".into());
        }
        if self.repairs == repairs {
            if self.allowed_paths != base.allowed_paths
                || self.blockers != base.blockers
                || self.frozen_commits != commits
            {
                return Err("repair changes must preserve their registered scope".into());
            }
            return require_paths(paths, &base.allowed_paths);
        }

        if let Some(repair) = base.open_repair() {
            let phase = repair
                .get("phase")
                .and_then(toml::Value::as_str)
                .ok_or("repair phase must be a string")?;
            let accepted = self
                .frozen_commits
                .get(phase)
                .ok_or("missing repair acceptance commit")?;
            let prior = commits
                .get(phase)
                .ok_or("repair phase must have been frozen")?;
            check_commit(prior, accepted)?;
            if accepted == prior {
                return Err("a completed repair needs a new acceptance commit".into());
            }
            commits.insert(phase.into(), accepted.clone());
            let entry = repairs.iter_mut().find(|entry| *entry == repair).unwrap();
            entry.insert("status".into(), toml::Value::String("completed".into()));
            entry.insert("commit".into(), toml::Value::String(accepted.clone()));
        }
        if let Some(repair) = self.open_repair() {
            let phase = repair
                .get("phase")
                .and_then(toml::Value::as_str)
                .ok_or("repair phase must be a string")?;
            let issue = repair
                .get("issue")
                .and_then(toml::Value::as_integer)
                .ok_or("repair issue must be an integer")?;
            if !commits.contains_key(phase)
                || issue <= 0
                || repairs
                    .iter()
                    .any(|entry| entry.get("issue") == repair.get("issue"))
                || self.blockers.as_slice() != [toml::Value::Integer(issue)]
                || self.current()?.status != Stage::Blocked
                || !repair
                    .get("scope")
                    .and_then(toml::Value::as_str)
                    .is_some_and(|scope| !scope.trim().is_empty())
                || repair.get("commit").is_some()
                || self.allowed_paths.iter().any(|path| {
                    matches!(
                        path.as_str(),
                        "status.toml" | "xtask/" | "xtask/src/" | "xtask/src/phase.rs"
                    )
                })
            {
                return Err("invalid open repair, blocker or scope".into());
            }
            repairs.push(repair.clone());
        } else if self.current()?.status != Stage::Frozen {
            return Err("completing the last repair must restore the freeze".into());
        }
        if self.repairs != repairs || self.frozen_commits != commits {
            return Err("a repair transition must preserve unrelated acceptance records".into());
        }
        let mut allowed = vec!["status.toml".into()];
        if base.open_repair().is_none() {
            allowed.push("xtask/src/phase.rs".into());
        }
        require_paths(paths, &allowed)
    }

    fn check_frozen_maintenance<'a>(
        &self,
        base: &Self,
        paths: impl Iterator<Item = &'a str>,
    ) -> Result {
        if self.schema_version != base.schema_version
            || self.frozen_commits != base.frozen_commits
            || self.phases != base.phases
            || self.repairs.len() != base.repairs.len()
            || self
                .repairs
                .iter()
                .zip(&base.repairs)
                .any(|(current, previous)| {
                    let mut previous = previous.clone();
                    previous.remove("evidence");
                    current != &previous
                })
        {
            return Err("maintenance must preserve frozen phase records".into());
        }
        let paths = paths.collect::<Vec<_>>();
        let updates_maintenance_scope = paths
            .iter()
            .any(|path| is_p1_experiment_path(path) || is_github_maintenance_path(path));
        let updates_token_index = paths.contains(&"docs/theme/token-index.md");
        let updates_contribution_guide = paths.contains(&"CONTRIBUTION.md");
        let denied = paths
            .into_iter()
            .filter(|path| {
                !is_maintenance_path(path)
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

fn is_maintenance_path(path: &str) -> bool {
    matches!(
        path,
        "status.toml"
            | "README.md"
            | "README.zh.md"
            | "README.i18n.yaml"
            | "CONTRIBUTION.md"
            | "CONTRIBUTION.zh.md"
            | "CONTRIBUTION.i18n.yaml"
            | "THIRD_PARTY_NOTICES.md"
            | "THIRD_PARTY_NOTICES.zh.md"
            | "THIRD_PARTY_NOTICES.i18n.yaml"
    ) || (path.starts_with("docs/")
        && (path.ends_with(".md") || path.ends_with(".i18n.yaml") || path.ends_with(".png"))
        && is_repo_path(path))
        || is_github_maintenance_path(path)
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
    fn freeze_accepts_issue_and_pr_records_without_evidence_files() {
        let mut status: Status = toml::from_str(include_str!("../../status.toml")).unwrap();
        let head = output("git", &["rev-parse", "HEAD"]).unwrap();
        let head = head.trim();
        status.frozen_commits.insert("P3".into(), head.into());
        let phase = toml::from_str(&format!(
            r#"
status = "frozen"
frozen_on = 2026-11-13
merged_deliverables = [{{ task = "P3-01", issue = 1, pull_request = 2, commit = "{head}" }}]
"#
        ))
        .unwrap();
        status.check_frozen("P3", &phase).unwrap();
    }

    #[test]
    fn frozen_maintenance_allows_docs_ci_and_templates() {
        assert!(is_maintenance_path("docs/architecture.md"));
        assert!(is_maintenance_path("README.md"));
        assert!(is_maintenance_path("README.zh.md"));
        assert!(is_maintenance_path("docs/architecture.i18n.yaml"));
        assert!(is_maintenance_path("docs/phases/figure.png"));
        assert!(!is_maintenance_path("docs/architecture.yaml"));
        assert!(is_maintenance_path(".github/workflows/ci.yml"));
        assert!(is_maintenance_path(".github/pull_request_template.md"));
        assert!(is_maintenance_path(".github/ISSUE_TEMPLATE/bug_report.md"));
        assert!(is_maintenance_path("status.toml"));
        assert!(!is_maintenance_path("xtask/src/phase.rs"));
        assert!(!is_maintenance_path("docs/../README.md"));
        assert!(!is_maintenance_path(".github/workflows/release.yml"));
        assert!(!is_maintenance_path(
            ".github/ISSUE_TEMPLATE/../../README.md"
        ));
        assert!(!is_maintenance_path("apps/gallery/src/main.rs"));
    }

    #[test]
    fn frozen_maintenance_accepts_a_baseline_with_legacy_file_records() {
        let document = include_str!("../../status.toml");
        let current: Status = toml::from_str(document).unwrap();
        let previous = document
            .replace("scope =", "evidence = \"docs/archive.md\", scope =")
            .replace(
                "[phases.P0]",
                "[phases.P0]\nevidence = [\"docs/archive.md\"]",
            );
        let mut base: Status = toml::from_str(&previous).unwrap();
        let paths = [
            "status.toml",
            "docs/phases/p02/gallery-windows.png",
            "experiments/p1-host/evidence/01-build.md",
            "xtask/src/phase.rs",
        ];
        current
            .check_frozen_maintenance(&base, paths.into_iter())
            .unwrap();
        base.repairs[0].insert("issue".into(), toml::Value::Integer(999));
        assert!(
            current
                .check_frozen_maintenance(&base, paths.into_iter())
                .is_err()
        );
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
