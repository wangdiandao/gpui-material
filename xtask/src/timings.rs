use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
    process::Command,
    time::Instant,
};

use crate::{Result, output};

pub(crate) fn measure(report: &Path) -> Result {
    let host = output("rustc", &["-vV"])?;
    let target = host
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .ok_or("rustc did not report a host")?;
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

    for command in ["check", "build"] {
        let target_dir = report.join(format!("{command}-target"));
        for cache in ["cold", "warm"] {
            let started = Instant::now();
            let status = Command::new("cargo")
                .args([
                    command,
                    "--locked",
                    "--timings",
                    "-p",
                    "gpui-material-gallery",
                    "--target",
                    target,
                    "--target-dir",
                ])
                .arg(&target_dir)
                .status()?;
            if !status.success() {
                return Err(io::Error::other(format!(
                    "cargo {command} ({cache}) failed: {status}"
                ))
                .into());
            }
            let seconds = started.elapsed().as_secs_f64();
            fs::copy(
                target_dir.join("cargo-timings/cargo-timing.html"),
                report.join(format!("{command}-{cache}.html")),
            )?;
            writeln!(summary, "| `cargo {command}` | {cache} | {seconds:.3} |")?;
            println!("{command} {cache}: {seconds:.3} s");
        }
    }
    println!("Report saved to {}", report.display());
    Ok(())
}
