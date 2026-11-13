mod docs;
mod graph;
mod phase;
mod timings;

use std::{
    env, fs, io,
    path::PathBuf,
    process::{Command, ExitCode},
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const USAGE: &str = "Usage:
  cargo xtask check [<base-commit>]
  cargo xtask graph
  cargo xtask timings <new-report-directory>";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("xtask: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result {
    env::set_current_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".."))?;
    let args = env::args().skip(1).collect::<Vec<_>>();
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] | ["help" | "--help" | "-h"] => println!("{USAGE}"),
        ["check"] => check(None)?,
        ["check", base] => check(Some(base))?,
        ["graph"] => graph::check()?,
        ["timings", path] => timings::measure(path.as_ref())?,
        _ => return Err(USAGE.into()),
    }
    Ok(())
}

fn check(base: Option<&str>) -> Result {
    phase::check(&fs::read_to_string("status.toml")?, base)?;
    docs::check()?;
    graph::check()
}

fn output(program: &str, args: &[&str]) -> Result<String> {
    let result = Command::new(program).args(args).output()?;
    if !result.status.success() {
        return Err(io::Error::other(format!(
            "{program} {} failed ({}): {}",
            args.join(" "),
            result.status,
            String::from_utf8_lossy(&result.stderr).trim()
        ))
        .into());
    }
    Ok(String::from_utf8(result.stdout)?)
}
