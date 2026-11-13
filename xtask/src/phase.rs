use std::{collections::BTreeMap, path::Path};

use serde::Deserialize;

use crate::{Result, output};

#[derive(Deserialize)]
struct Status {
    schema_version: u8,
    current_phase: String,
    allowed_paths: Vec<String>,
    blockers: Vec<toml::Value>,
    frozen_commits: BTreeMap<String, String>,
    phases: BTreeMap<String, Phase>,
}

#[derive(Deserialize)]
struct Phase {
    status: Stage,
    frozen_on: Option<toml::value::Datetime>,
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

#[derive(Deserialize)]
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
        Ok(())
    }

    fn check_frozen(&self, name: &str, phase: &Phase) -> Result {
        let accepted = self
            .frozen_commits
            .get(name)
            .ok_or_else(|| format!("{name} has no accepted commit"))?;
        if phase.frozen_on.is_none()
            || phase.evidence.is_empty()
            || phase.merged_deliverables.is_empty()
        {
            return Err(format!("{name} is missing freeze evidence").into());
        }
        check_commit(accepted, "HEAD")?;
        for path in &phase.evidence {
            if !is_repo_path(path) || !Path::new(path).is_file() {
                return Err(format!("{name} evidence file missing: {path}").into());
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
            return require_paths(
                paths,
                &[
                    "status.toml".into(),
                    "docs/index.md".into(),
                    "docs/phases/index.md".into(),
                    format!("docs/phases/P0{index}.md"),
                ],
            );
        }
        require_paths(paths, &[])
    }
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
}
