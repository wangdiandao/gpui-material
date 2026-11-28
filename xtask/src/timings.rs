use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
    time::{Instant, SystemTime},
};

use serde::{Deserialize, Serialize};

use crate::{Result, output};

const WARM_LEAF_SOURCE: &str = "apps/gallery/src/main.rs";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct BuildTimings {
    pub(crate) target: String,
    pub(crate) check_cold_s: f64,
    pub(crate) check_warm_s: f64,
    pub(crate) build_cold_s: f64,
    pub(crate) build_warm_s: f64,
}

pub(crate) struct TempDirGuard(pub(crate) PathBuf);

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn touch_leaf_source(path: &Path) -> Result {
    let file = OpenOptions::new().write(true).open(path)?;
    file.set_modified(SystemTime::now())?;
    Ok(())
}

pub(crate) fn host_target() -> Result<(String, String)> {
    let host = output("rustc", &["-vV"])?;
    let target = host
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .ok_or("rustc did not report a host")?
        .to_string();
    Ok((host, target))
}

pub(crate) fn measure_workspace_timings(
    target_root: &Path,
    html_report_dir: Option<&Path>,
) -> Result<BuildTimings> {
    let (_, target) = host_target()?;
    let mut check_cold_s = 0.0;
    let mut check_warm_s = 0.0;
    let mut build_cold_s = 0.0;
    let mut build_warm_s = 0.0;

    for command in ["check", "build"] {
        let target_dir = TempDirGuard(target_root.join(format!("{command}-target")));
        for cache in ["cold", "warm"] {
            if cache == "warm" {
                touch_leaf_source(Path::new(WARM_LEAF_SOURCE))?;
            }
            let started = Instant::now();
            let status = Command::new("cargo")
                .args([
                    command,
                    "--locked",
                    "--timings",
                    "-p",
                    "gpui-material-gallery",
                    "--target",
                    &target,
                    "--target-dir",
                ])
                .arg(&target_dir.0)
                .status()?;
            if !status.success() {
                return Err(io::Error::other(format!(
                    "cargo {command} ({cache}) failed: {status}"
                ))
                .into());
            }
            let seconds = started.elapsed().as_secs_f64();
            if let Some(report) = html_report_dir {
                fs::copy(
                    target_dir.0.join("cargo-timings/cargo-timing.html"),
                    report.join(format!("{command}-{cache}.html")),
                )?;
            }
            match (command, cache) {
                ("check", "cold") => check_cold_s = seconds,
                ("check", "warm") => check_warm_s = seconds,
                ("build", "cold") => build_cold_s = seconds,
                ("build", "warm") => build_warm_s = seconds,
                _ => unreachable!(),
            }
            println!("{command} {cache}: {seconds:.3} s");
        }
    }

    Ok(BuildTimings {
        target,
        check_cold_s,
        check_warm_s,
        build_cold_s,
        build_warm_s,
    })
}

pub(crate) fn measure(report: &Path) -> Result {
    let (host, target) = host_target()?;
    if let Some(parent) = report
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(report)?;
    let report = report.canonicalize()?;
    let mut summary = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(report.join("summary.md"))?;
    let commit = output("git", &["rev-parse", "HEAD"])?;
    let cargo = output("cargo", &["-V"])?;
    let changes = output("git", &["status", "--porcelain", "--untracked-files=no"])?;
    let tree = if changes.is_empty() {
        "clean"
    } else {
        "modified"
    };
    writeln!(
        summary,
        "# Gallery build timings\n\nCommit `{}` ({tree}), target `{target}`.\n\n```text\n{}\n{}```\n\n| Cargo command | Target cache | Seconds |\n| --- | --- | ---: |",
        commit.trim(),
        cargo.trim(),
        host,
    )?;

    let timings = measure_workspace_timings(&report, Some(&report))?;
    writeln!(
        summary,
        "| `cargo check` | cold | {:.3} |",
        timings.check_cold_s
    )?;
    writeln!(
        summary,
        "| `cargo check` | warm | {:.3} |",
        timings.check_warm_s
    )?;
    writeln!(
        summary,
        "| `cargo build` | cold | {:.3} |",
        timings.build_cold_s
    )?;
    writeln!(
        summary,
        "| `cargo build` | warm | {:.3} |",
        timings.build_warm_s
    )?;
    println!("Report saved to {}", report.display());
    Ok(())
}
