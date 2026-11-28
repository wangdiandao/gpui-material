use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    fmt::Write as _,
    fs, io,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use crate::{
    Result, output,
    timings::{self, BuildTimings, TempDirGuard},
};

pub(crate) const SCHEMA_VERSION: u32 = 1;
const PERF_JSON_PREFIX: &str = "PERF_BENCH_JSON:";
const DEFAULT_WASM_TOOLCHAIN: &str = "nightly-2026-10-31";
const WASM_TARGET: &str = "wasm32-unknown-unknown";

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

const USAGE: &str = "Usage:
  cargo xtask baseline capture <output-json> [--perf] [--timings] [--wasm] [--wasm-dir <dir>]
  cargo xtask baseline compare <base-json> <head-json> [--threshold <percent>]

Capture flags:
  --perf             Run release perf_bench and record frame metrics (default when no flag is given)
  --timings          Measure cold/warm Gallery check and build times using temporary target directories
  --wasm             Build gpui-material-gallery-web (wasm-release) into a temp dir and measure sizes
  --wasm-dir <dir>   Measure existing wasm-bindgen output directory without rebuilding

Compare flags:
  --threshold <pct>  Exit non-zero if any compared metric regresses by more than <pct>%";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct BaselineReport {
    pub(crate) schema_version: u32,
    pub(crate) metadata: BaselineMetadata,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) perf: Option<PerfBaseline>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) timings: Option<BuildTimings>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) wasm: Option<WasmBaseline>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct BaselineMetadata {
    pub(crate) commit: String,
    pub(crate) tree_state: String,
    pub(crate) host: String,
    pub(crate) rustc: String,
    pub(crate) cargo: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct PerfBaseline {
    pub(crate) viewport_width: f32,
    pub(crate) viewport_height: f32,
    pub(crate) warmup_frames: usize,
    pub(crate) sample_frames: usize,
    pub(crate) benchmarks: Vec<PerfBenchmark>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct PerfBenchmark {
    pub(crate) name: String,
    pub(crate) layer: String,
    pub(crate) p50_us: f64,
    pub(crate) p95_us: f64,
    pub(crate) p99_us: f64,
    pub(crate) mean_us: f64,
    pub(crate) quads: usize,
    pub(crate) scheduled_frames: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct WasmBaseline {
    pub(crate) profile: String,
    pub(crate) wasm_bytes: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) gzip_wasm_bytes: Option<u64>,
    pub(crate) js_bytes: u64,
    pub(crate) total_bytes: u64,
    #[serde(default)]
    pub(crate) files: BTreeMap<String, u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CaptureCommand {
    Help,
    Run(CaptureOptions),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CaptureOptions {
    output: PathBuf,
    perf: bool,
    timings: bool,
    wasm: bool,
    wasm_dir: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq)]
enum CompareCommand {
    Help,
    Run(CompareOptions),
}

#[derive(Clone, Debug, PartialEq)]
struct CompareOptions {
    base: PathBuf,
    head: PathBuf,
    threshold_pct: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
struct Regression {
    section: String,
    metric: String,
    base: String,
    head: String,
    pct_change: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct ComparisonSummary {
    markdown: String,
    regressions: Vec<Regression>,
}

#[derive(Deserialize)]
struct CargoMetadata {
    target_directory: PathBuf,
}

pub(crate) fn run(args: &[&str]) -> Result {
    match args {
        [] | ["help" | "--help" | "-h"] => {
            println!("{USAGE}");
            Ok(())
        }
        ["capture", rest @ ..] => match parse_capture_args(rest)? {
            CaptureCommand::Help => {
                println!("{USAGE}");
                Ok(())
            }
            CaptureCommand::Run(options) => capture(&options),
        },
        ["compare", rest @ ..] => match parse_compare_args(rest)? {
            CompareCommand::Help => {
                println!("{USAGE}");
                Ok(())
            }
            CompareCommand::Run(options) => compare(&options),
        },
        _ => Err(USAGE.into()),
    }
}

fn parse_capture_args(args: &[&str]) -> Result<CaptureCommand> {
    let mut output_path: Option<PathBuf> = None;
    let mut perf = false;
    let mut timings = false;
    let mut wasm = false;
    let mut wasm_dir: Option<PathBuf> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--help" | "-h" | "help" => return Ok(CaptureCommand::Help),
            "--perf" => perf = true,
            "--timings" => timings = true,
            "--wasm" => wasm = true,
            "--wasm-dir" => {
                let value = args
                    .get(i + 1)
                    .ok_or("missing directory after --wasm-dir")?;
                if wasm_dir.replace(PathBuf::from(value)).is_some() {
                    return Err("--wasm-dir specified more than once".into());
                }
                i += 1;
            }
            flag if flag.starts_with("--wasm-dir=") => {
                let value = flag.trim_start_matches("--wasm-dir=");
                if value.is_empty() {
                    return Err("missing directory after --wasm-dir=".into());
                }
                if wasm_dir.replace(PathBuf::from(value)).is_some() {
                    return Err("--wasm-dir specified more than once".into());
                }
            }
            flag if flag.starts_with('-') => {
                return Err(format!("unknown capture flag `{flag}`\n\n{USAGE}").into());
            }
            value => {
                if output_path.replace(PathBuf::from(value)).is_some() {
                    return Err(format!("unexpected extra argument `{value}`\n\n{USAGE}").into());
                }
            }
        }
        i += 1;
    }

    if wasm && wasm_dir.is_some() {
        return Err("cannot combine --wasm and --wasm-dir".into());
    }

    let output = output_path.ok_or_else(|| format!("missing <output-json>\n\n{USAGE}"))?;
    if !perf && !timings && !wasm && wasm_dir.is_none() {
        perf = true;
    }

    Ok(CaptureCommand::Run(CaptureOptions {
        output,
        perf,
        timings,
        wasm,
        wasm_dir,
    }))
}

fn parse_compare_args(args: &[&str]) -> Result<CompareCommand> {
    let mut positionals: Vec<PathBuf> = Vec::new();
    let mut threshold_pct: Option<f64> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--help" | "-h" | "help" => return Ok(CompareCommand::Help),
            "--threshold" => {
                let raw = args
                    .get(i + 1)
                    .ok_or("missing percentage value after --threshold")?;
                let parsed = parse_threshold(raw)?;
                if threshold_pct.replace(parsed).is_some() {
                    return Err("--threshold specified more than once".into());
                }
                i += 1;
            }
            flag if flag.starts_with("--threshold=") => {
                let raw = flag.trim_start_matches("--threshold=");
                let parsed = parse_threshold(raw)?;
                if threshold_pct.replace(parsed).is_some() {
                    return Err("--threshold specified more than once".into());
                }
            }
            flag if flag.starts_with('-') => {
                return Err(format!("unknown compare flag `{flag}`\n\n{USAGE}").into());
            }
            value => positionals.push(PathBuf::from(value)),
        }
        i += 1;
    }

    if positionals.len() != 2 {
        return Err(format!("compare expects <base-json> <head-json>\n\n{USAGE}").into());
    }

    let head = positionals.pop().unwrap();
    let base = positionals.pop().unwrap();
    Ok(CompareCommand::Run(CompareOptions {
        base,
        head,
        threshold_pct,
    }))
}

fn parse_threshold(raw: &str) -> Result<f64> {
    let value: f64 = raw.parse().map_err(|_| {
        format!("invalid --threshold value `{raw}`: expected a non-negative number")
    })?;
    if !value.is_finite() || value < 0.0 {
        return Err(format!(
            "invalid --threshold value `{raw}`: expected a finite non-negative percentage"
        )
        .into());
    }
    Ok(value)
}

fn unique_temp_dir(label: &str) -> Result<TempDirGuard> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let seq = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = env::temp_dir().join(format!(
        "gpui-material-baseline-{label}-{}-{nanos}-{seq}",
        std::process::id()
    ));
    fs::create_dir_all(&dir)?;
    Ok(TempDirGuard(dir))
}

fn collect_metadata() -> Result<BaselineMetadata> {
    let (_, host) = timings::host_target()?;
    let commit = output("git", &["rev-parse", "HEAD"])?.trim().to_string();
    let changes = output("git", &["status", "--porcelain", "--untracked-files=no"])?;
    let tree_state = if changes.is_empty() {
        "clean".to_string()
    } else {
        "modified".to_string()
    };
    let rustc = output("rustc", &["-V"])?.trim().to_string();
    let cargo = output("cargo", &["-V"])?.trim().to_string();
    Ok(BaselineMetadata {
        commit,
        tree_state,
        host,
        rustc,
        cargo,
    })
}

fn capture(options: &CaptureOptions) -> Result {
    let metadata = collect_metadata()?;
    let perf = if options.perf {
        println!("Capturing release perf_bench metrics...");
        Some(capture_perf()?)
    } else {
        None
    };
    let timings = if options.timings {
        println!("Capturing Gallery cold/warm build timings...");
        Some(capture_timings()?)
    } else {
        None
    };
    let wasm = if options.wasm {
        println!("Capturing gpui-material-gallery-web WASM bundle size...");
        Some(capture_wasm()?)
    } else if let Some(dir) = &options.wasm_dir {
        println!("Measuring existing WASM artifacts in {}...", dir.display());
        Some(collect_wasm_metrics(dir, "existing")?)
    } else {
        None
    };

    let report = BaselineReport {
        schema_version: SCHEMA_VERSION,
        metadata,
        perf,
        timings,
        wasm,
    };

    if let Some(parent) = options
        .output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&options.output, format!("{json}\n"))?;
    println!("Baseline saved to {}", options.output.display());
    Ok(())
}

fn capture_perf() -> Result<PerfBaseline> {
    let stdout = output(
        "cargo",
        &[
            "test",
            "--locked",
            "--release",
            "-p",
            "gpui-material-gallery",
            "--features",
            "test-support",
            "--lib",
            "perf_bench",
            "--",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ],
    )?;
    parse_perf_bench_output(&stdout)
}

fn capture_timings() -> Result<BuildTimings> {
    let temp = unique_temp_dir("timings")?;
    timings::measure_workspace_timings(&temp.0, None)
}

fn capture_wasm() -> Result<WasmBaseline> {
    let wasm_bindgen = env::var("WASM_BINDGEN").unwrap_or_else(|_| "wasm-bindgen".to_string());
    output(&wasm_bindgen, &["--version"])?;

    let raw_toolchain = env::var("RUSTUP_TOOLCHAIN")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_WASM_TOOLCHAIN.to_string());
    let toolchain = if raw_toolchain.starts_with('+') {
        raw_toolchain
    } else {
        format!("+{raw_toolchain}")
    };

    let metadata_json = output(
        "cargo",
        &[
            &toolchain,
            "metadata",
            "--locked",
            "--no-deps",
            "--format-version=1",
        ],
    )?;
    let metadata: CargoMetadata = serde_json::from_str(&metadata_json)?;

    let status = Command::new("cargo")
        .args([
            &toolchain,
            "rustc",
            "--locked",
            "-p",
            "gpui-material-gallery-web",
            "--target",
            WASM_TARGET,
            "--profile",
            "wasm-release",
            "--",
            "-C",
            "link-arg=-zstack-size=8388608",
        ])
        .status()?;
    if !status.success() {
        return Err(io::Error::other(format!(
            "cargo {toolchain} rustc gpui-material-gallery-web failed: {status}"
        ))
        .into());
    }

    let built_wasm = metadata
        .target_directory
        .join(WASM_TARGET)
        .join("wasm-release")
        .join("gpui_material_gallery_web.wasm");
    let out_dir = unique_temp_dir("wasm")?;
    let status = Command::new(&wasm_bindgen)
        .arg(&built_wasm)
        .args([
            "--target",
            "web",
            "--no-typescript",
            "--out-name",
            "material-gallery",
            "--out-dir",
        ])
        .arg(&out_dir.0)
        .status()?;
    if !status.success() {
        return Err(io::Error::other(format!("{wasm_bindgen} failed: {status}")).into());
    }

    collect_wasm_metrics(&out_dir.0, "wasm-release")
}

fn parse_perf_bench_output(stdout: &str) -> Result<PerfBaseline> {
    for line in stdout.lines() {
        if let Some(payload) = line.trim().strip_prefix(PERF_JSON_PREFIX) {
            let parsed: PerfBaseline = serde_json::from_str(payload.trim())?;
            if parsed.benchmarks.is_empty() {
                return Err("PERF_BENCH_JSON payload contained zero benchmarks".into());
            }
            return Ok(parsed);
        }
    }

    parse_perf_bench_table(stdout)
}

fn parse_perf_bench_table(stdout: &str) -> Result<PerfBaseline> {
    let mut viewport_width = 1040.0;
    let mut viewport_height = 744.0;
    let mut warmup_frames = 20;
    let mut sample_frames = 100;
    let mut benchmarks = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if let Some((w, h, warmup, sample)) = parse_perf_header(trimmed) {
            viewport_width = w;
            viewport_height = h;
            warmup_frames = warmup;
            sample_frames = sample;
            continue;
        }
        if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
            continue;
        }
        let cells: Vec<&str> = trimmed
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        if cells.len() != 8 {
            continue;
        }
        if cells[0].eq_ignore_ascii_case("Benchmark")
            || cells[0].chars().all(|c| c == '-' || c == ':')
        {
            continue;
        }
        let Ok(p50_us) = cells[2].parse::<f64>() else {
            continue;
        };
        let Ok(p95_us) = cells[3].parse::<f64>() else {
            continue;
        };
        let Ok(p99_us) = cells[4].parse::<f64>() else {
            continue;
        };
        let Ok(mean_us) = cells[5].parse::<f64>() else {
            continue;
        };
        let Ok(quads) = cells[6].parse::<usize>() else {
            continue;
        };
        let Ok(scheduled_frames) = cells[7].parse::<usize>() else {
            continue;
        };
        benchmarks.push(PerfBenchmark {
            name: cells[0].to_string(),
            layer: cells[1].to_string(),
            p50_us,
            p95_us,
            p99_us,
            mean_us,
            quads,
            scheduled_frames,
        });
    }

    if benchmarks.is_empty() {
        return Err(
            "failed to parse perf_bench output: found neither PERF_BENCH_JSON line nor benchmark table"
                .into(),
        );
    }

    Ok(PerfBaseline {
        viewport_width,
        viewport_height,
        warmup_frames,
        sample_frames,
        benchmarks,
    })
}

fn parse_perf_header(line: &str) -> Option<(f32, f32, usize, usize)> {
    let start = line.find('(')?;
    let end = line.rfind(')')?;
    if end <= start {
        return None;
    }
    let inside = &line[start + 1..end];
    let (dims, frames) = inside.split_once(',')?;
    let (w, h) = dims.trim().split_once('x')?;
    let (warmup_part, sample_part) = frames.split_once('+')?;
    let width = w.trim().parse::<f32>().ok()?;
    let height = h.trim().parse::<f32>().ok()?;
    let warmup = warmup_part
        .split_whitespace()
        .next()?
        .parse::<usize>()
        .ok()?;
    let sample = sample_part
        .split_whitespace()
        .next()?
        .parse::<usize>()
        .ok()?;
    Some((width, height, warmup, sample))
}

fn collect_wasm_metrics(dir: &Path, profile: &str) -> Result<WasmBaseline> {
    if !dir.is_dir() {
        return Err(format!("WASM artifact directory does not exist: {}", dir.display()).into());
    }
    let mut files = BTreeMap::new();
    walk_files(dir, dir, &mut files)?;

    let mut wasm_bytes = 0u64;
    let mut found_wasm = false;
    let mut gzip_bytes = 0u64;
    let mut found_gzip = false;
    let mut js_bytes = 0u64;

    for (rel_path, size) in &files {
        if rel_path.ends_with(".wasm") {
            wasm_bytes += *size;
            found_wasm = true;
        } else if rel_path.ends_with(".wasm.gz") {
            gzip_bytes += *size;
            found_gzip = true;
        } else if rel_path.ends_with(".js") || rel_path.ends_with(".mjs") {
            js_bytes += *size;
        }
    }

    if !found_wasm {
        return Err(format!("no .wasm artifact found under {}", dir.display()).into());
    }

    Ok(WasmBaseline {
        profile: profile.to_string(),
        wasm_bytes,
        gzip_wasm_bytes: found_gzip.then_some(gzip_bytes),
        js_bytes,
        total_bytes: wasm_bytes + js_bytes,
        files,
    })
}

fn walk_files(root: &Path, current: &Path, out: &mut BTreeMap<String, u64>) -> Result {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            walk_files(root, &path, out)?;
        } else if file_type.is_file() {
            let rel = path
                .strip_prefix(root)?
                .to_string_lossy()
                .replace('\\', "/");
            out.insert(rel, entry.metadata()?.len());
        }
    }
    Ok(())
}

fn compare(options: &CompareOptions) -> Result {
    let base: BaselineReport = serde_json::from_str(&fs::read_to_string(&options.base)?)?;
    let head: BaselineReport = serde_json::from_str(&fs::read_to_string(&options.head)?)?;
    let summary = compare_reports(&base, &head, options.threshold_pct)?;
    print!("{}", summary.markdown);

    if let Some(threshold) = options.threshold_pct
        && !summary.regressions.is_empty()
    {
        return Err(format!(
            "{} metric(s) regressed by more than +{threshold:.2}%",
            summary.regressions.len()
        )
        .into());
    }
    Ok(())
}

fn compare_reports(
    base: &BaselineReport,
    head: &BaselineReport,
    threshold_pct: Option<f64>,
) -> Result<ComparisonSummary> {
    let mut md = String::new();
    let mut regressions = Vec::new();
    let mut compared_sections = 0usize;

    let _ = writeln!(md, "# Baseline comparison\n");
    let _ = writeln!(
        md,
        "- Base: `{}` ({}, `{}`)",
        short_commit(&base.metadata.commit),
        base.metadata.tree_state,
        base.metadata.host
    );
    let _ = writeln!(
        md,
        "- Head: `{}` ({}, `{}`)",
        short_commit(&head.metadata.commit),
        head.metadata.tree_state,
        head.metadata.host
    );
    if let Some(threshold) = threshold_pct {
        let _ = writeln!(md, "- Regression threshold: `+{threshold:.2}%`");
    }

    match (&base.perf, &head.perf) {
        (Some(base_perf), Some(head_perf)) => {
            compared_sections += 1;
            append_perf_section(
                &mut md,
                base_perf,
                head_perf,
                threshold_pct,
                &mut regressions,
            );
        }
        (Some(_), None) => {
            let _ = writeln!(
                md,
                "\n_Note: `perf` present only in base baseline; skipped comparison._"
            );
        }
        (None, Some(_)) => {
            let _ = writeln!(
                md,
                "\n_Note: `perf` present only in head baseline; skipped comparison._"
            );
        }
        (None, None) => {}
    }

    match (&base.timings, &head.timings) {
        (Some(base_timings), Some(head_timings)) => {
            compared_sections += 1;
            append_timings_section(
                &mut md,
                base_timings,
                head_timings,
                threshold_pct,
                &mut regressions,
            );
        }
        (Some(_), None) => {
            let _ = writeln!(
                md,
                "\n_Note: `timings` present only in base baseline; skipped comparison._"
            );
        }
        (None, Some(_)) => {
            let _ = writeln!(
                md,
                "\n_Note: `timings` present only in head baseline; skipped comparison._"
            );
        }
        (None, None) => {}
    }

    match (&base.wasm, &head.wasm) {
        (Some(base_wasm), Some(head_wasm)) => {
            compared_sections += 1;
            append_wasm_section(
                &mut md,
                base_wasm,
                head_wasm,
                threshold_pct,
                &mut regressions,
            );
        }
        (Some(_), None) => {
            let _ = writeln!(
                md,
                "\n_Note: `wasm` present only in base baseline; skipped comparison._"
            );
        }
        (None, Some(_)) => {
            let _ = writeln!(
                md,
                "\n_Note: `wasm` present only in head baseline; skipped comparison._"
            );
        }
        (None, None) => {}
    }

    if compared_sections == 0 {
        return Err(
            "no overlapping baseline sections (`perf`, `timings`, `wasm`) to compare".into(),
        );
    }

    if let Some(threshold) = threshold_pct
        && !regressions.is_empty()
    {
        let _ = writeln!(md, "\n## Regressions exceeding +{threshold:.2}%\n");
        for reg in &regressions {
            let _ = writeln!(
                md,
                "- [{}] {}: {} -> {} ({:+.1}%)",
                reg.section, reg.metric, reg.base, reg.head, reg.pct_change
            );
        }
    }

    Ok(ComparisonSummary {
        markdown: md,
        regressions,
    })
}

fn append_perf_section(
    md: &mut String,
    base: &PerfBaseline,
    head: &PerfBaseline,
    threshold_pct: Option<f64>,
    regressions: &mut Vec<Regression>,
) {
    let base_map: BTreeMap<&str, &PerfBenchmark> = base
        .benchmarks
        .iter()
        .map(|bench| (bench.name.as_str(), bench))
        .collect();
    let head_names: BTreeSet<&str> = head
        .benchmarks
        .iter()
        .map(|bench| bench.name.as_str())
        .collect();

    let _ = writeln!(md, "\n## Frame performance (`perf_bench`)\n");
    let _ = writeln!(md, "| Benchmark | Metric | Base | Head | Delta | % |");
    let _ = writeln!(md, "| --- | --- | ---: | ---: | ---: | ---: |");

    let mut added = Vec::new();
    for h in &head.benchmarks {
        let Some(b) = base_map.get(h.name.as_str()) else {
            added.push(h.name.as_str());
            continue;
        };
        for (label, bv, hv) in [
            ("p50 (us)", b.p50_us, h.p50_us),
            ("p95 (us)", b.p95_us, h.p95_us),
            ("p99 (us)", b.p99_us, h.p99_us),
            ("mean (us)", b.mean_us, h.mean_us),
        ] {
            let base_str = format!("{bv:.1}");
            let head_str = format!("{hv:.1}");
            let delta = normalize_zero(hv - bv);
            let delta_str = format!("{delta:+.1}");
            let pct = percent_change(bv, hv);
            let _ = writeln!(
                md,
                "| `{}` | {label} | {base_str} | {head_str} | {delta_str} | {} |",
                h.name,
                format_pct(pct)
            );
            record_regression(
                regressions,
                threshold_pct,
                "perf",
                format!("{} {label}", h.name),
                base_str,
                head_str,
                pct,
            );
        }

        let b_quads = b.quads;
        let h_quads = h.quads;
        let delta_quads = h_quads as i128 - b_quads as i128;
        let pct_quads = percent_change(b_quads as f64, h_quads as f64);
        let base_str = b_quads.to_string();
        let head_str = h_quads.to_string();
        let _ = writeln!(
            md,
            "| `{}` | quads | {base_str} | {head_str} | {delta_quads:+} | {} |",
            h.name,
            format_pct(pct_quads)
        );
        record_regression(
            regressions,
            threshold_pct,
            "perf",
            format!("{} quads", h.name),
            base_str,
            head_str,
            pct_quads,
        );
    }

    let removed: Vec<&str> = base
        .benchmarks
        .iter()
        .map(|b| b.name.as_str())
        .filter(|name| !head_names.contains(name))
        .collect();
    if !added.is_empty() {
        let _ = writeln!(md, "\n_Added benchmarks (head only): {}_", added.join(", "));
    }
    if !removed.is_empty() {
        let _ = writeln!(
            md,
            "\n_Removed benchmarks (base only): {}_",
            removed.join(", ")
        );
    }
}

fn append_timings_section(
    md: &mut String,
    base: &BuildTimings,
    head: &BuildTimings,
    threshold_pct: Option<f64>,
    regressions: &mut Vec<Regression>,
) {
    let _ = writeln!(md, "\n## Build timings (`gpui-material-gallery`)\n");
    let _ = writeln!(md, "| Metric | Base | Head | Delta | % |");
    let _ = writeln!(md, "| --- | ---: | ---: | ---: | ---: |");

    for (label, bv, hv) in [
        ("`cargo check` (cold)", base.check_cold_s, head.check_cold_s),
        ("`cargo check` (warm)", base.check_warm_s, head.check_warm_s),
        ("`cargo build` (cold)", base.build_cold_s, head.build_cold_s),
        ("`cargo build` (warm)", base.build_warm_s, head.build_warm_s),
    ] {
        let base_str = format!("{bv:.3} s");
        let head_str = format!("{hv:.3} s");
        let delta = normalize_zero(hv - bv);
        let delta_str = format!("{delta:+.3} s");
        let pct = percent_change(bv, hv);
        let _ = writeln!(
            md,
            "| {label} | {base_str} | {head_str} | {delta_str} | {} |",
            format_pct(pct)
        );
        record_regression(
            regressions,
            threshold_pct,
            "timings",
            label.to_string(),
            base_str,
            head_str,
            pct,
        );
    }
}

fn append_wasm_section(
    md: &mut String,
    base: &WasmBaseline,
    head: &WasmBaseline,
    threshold_pct: Option<f64>,
    regressions: &mut Vec<Regression>,
) {
    let _ = writeln!(md, "\n## WASM bundle size (`gpui-material-gallery-web`)\n");
    let _ = writeln!(md, "| Metric | Base | Head | Delta | % |");
    let _ = writeln!(md, "| --- | ---: | ---: | ---: | ---: |");

    let mut rows: Vec<(&str, u64, u64)> = vec![("`.wasm` (raw)", base.wasm_bytes, head.wasm_bytes)];
    if let (Some(b_gz), Some(h_gz)) = (base.gzip_wasm_bytes, head.gzip_wasm_bytes) {
        rows.push(("`.wasm` (gzip)", b_gz, h_gz));
    }
    rows.push(("`.js` glue", base.js_bytes, head.js_bytes));
    rows.push(("Total bundle", base.total_bytes, head.total_bytes));

    for (label, bv, hv) in rows {
        let base_str = format_bytes(bv);
        let head_str = format_bytes(hv);
        let delta_str = format_signed_bytes(hv as i128 - bv as i128);
        let pct = percent_change(bv as f64, hv as f64);
        let _ = writeln!(
            md,
            "| {label} | {base_str} | {head_str} | {delta_str} | {} |",
            format_pct(pct)
        );
        record_regression(
            regressions,
            threshold_pct,
            "wasm",
            label.to_string(),
            base_str,
            head_str,
            pct,
        );
    }
}

fn record_regression(
    regressions: &mut Vec<Regression>,
    threshold_pct: Option<f64>,
    section: &str,
    metric: String,
    base: String,
    head: String,
    pct: Option<f64>,
) {
    if let (Some(limit), Some(pct_change)) = (threshold_pct, pct)
        && pct_change > limit
    {
        regressions.push(Regression {
            section: section.to_string(),
            metric,
            base,
            head,
            pct_change,
        });
    }
}

fn percent_change(base: f64, head: f64) -> Option<f64> {
    if base > 0.0 {
        Some(normalize_zero((head - base) / base * 100.0))
    } else if head == 0.0 {
        Some(0.0)
    } else {
        None
    }
}

fn format_pct(pct: Option<f64>) -> String {
    match pct {
        Some(value) => format!("{:+.1}%", normalize_zero(value)),
        None => "n/a".to_string(),
    }
}

fn normalize_zero(value: f64) -> f64 {
    if value.abs() < 1e-12 { 0.0 } else { value }
}

fn short_commit(commit: &str) -> &str {
    commit.get(..12).unwrap_or(commit)
}

fn format_grouped_u64(value: u64) -> String {
    let digits = value.to_string();
    let len = digits.len();
    let mut out = String::with_capacity(len + len / 3);
    for (idx, ch) in digits.chars().enumerate() {
        if idx > 0 && (len - idx).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

fn format_bytes(bytes: u64) -> String {
    format!("{} B", format_grouped_u64(bytes))
}

fn format_signed_bytes(delta: i128) -> String {
    let sign = if delta < 0 { '-' } else { '+' };
    let mag = delta.unsigned_abs() as u64;
    format!("{sign}{} B", format_grouped_u64(mag))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_metadata(commit: &str) -> BaselineMetadata {
        BaselineMetadata {
            commit: commit.to_string(),
            tree_state: "clean".to_string(),
            host: "x86_64-pc-windows-msvc".to_string(),
            rustc: "rustc 1.95.0".to_string(),
            cargo: "cargo 1.95.0".to_string(),
        }
    }

    fn sample_perf(p50: f64, quads: usize) -> PerfBaseline {
        PerfBaseline {
            viewport_width: 1040.0,
            viewport_height: 744.0,
            warmup_frames: 20,
            sample_frames: 100,
            benchmarks: vec![
                PerfBenchmark {
                    name: "engine/empty_window".to_string(),
                    layer: "Engine".to_string(),
                    p50_us: p50,
                    p95_us: p50 * 1.25,
                    p99_us: p50 * 1.5,
                    mean_us: p50 * 1.125,
                    quads,
                    scheduled_frames: 0,
                },
                PerfBenchmark {
                    name: "component/button_grid_64_idle".to_string(),
                    layer: "Component".to_string(),
                    p50_us: 200.0,
                    p95_us: 220.0,
                    p99_us: 240.0,
                    mean_us: 205.0,
                    quads: 128,
                    scheduled_frames: 0,
                },
            ],
        }
    }

    fn sample_timings(check_cold: f64, build_cold: f64) -> BuildTimings {
        BuildTimings {
            target: "x86_64-pc-windows-msvc".to_string(),
            check_cold_s: check_cold,
            check_warm_s: 2.0,
            build_cold_s: build_cold,
            build_warm_s: 4.0,
        }
    }

    fn sample_wasm(wasm_bytes: u64, gzip_wasm_bytes: Option<u64>, js_bytes: u64) -> WasmBaseline {
        let mut files = BTreeMap::new();
        files.insert("material-gallery_bg.wasm".to_string(), wasm_bytes);
        files.insert("material-gallery.js".to_string(), js_bytes);
        if let Some(gz) = gzip_wasm_bytes {
            files.insert("material-gallery_bg.wasm.gz".to_string(), gz);
        }
        WasmBaseline {
            profile: "wasm-release".to_string(),
            wasm_bytes,
            gzip_wasm_bytes,
            js_bytes,
            total_bytes: wasm_bytes + js_bytes,
            files,
        }
    }

    #[test]
    fn parse_capture_and_compare_cli_arguments() {
        assert_eq!(
            parse_capture_args(&["--help"]).unwrap(),
            CaptureCommand::Help
        );
        assert_eq!(
            parse_capture_args(&["out/base.json"]).unwrap(),
            CaptureCommand::Run(CaptureOptions {
                output: PathBuf::from("out/base.json"),
                perf: true,
                timings: false,
                wasm: false,
                wasm_dir: None,
            })
        );
        assert_eq!(
            parse_capture_args(&["out/all.json", "--perf", "--timings", "--wasm"]).unwrap(),
            CaptureCommand::Run(CaptureOptions {
                output: PathBuf::from("out/all.json"),
                perf: true,
                timings: true,
                wasm: true,
                wasm_dir: None,
            })
        );
        assert_eq!(
            parse_capture_args(&["out/wasm.json", "--wasm-dir", "tmp/wasm"]).unwrap(),
            CaptureCommand::Run(CaptureOptions {
                output: PathBuf::from("out/wasm.json"),
                perf: false,
                timings: false,
                wasm: false,
                wasm_dir: Some(PathBuf::from("tmp/wasm")),
            })
        );
        assert!(parse_capture_args(&["out/bad.json", "--wasm", "--wasm-dir", "tmp"]).is_err());
        assert!(parse_capture_args(&["--perf"]).is_err());
        assert!(parse_capture_args(&["a.json", "--bogus"]).is_err());

        assert_eq!(parse_compare_args(&["-h"]).unwrap(), CompareCommand::Help);
        assert_eq!(
            parse_compare_args(&["base.json", "head.json", "--threshold", "7.5"]).unwrap(),
            CompareCommand::Run(CompareOptions {
                base: PathBuf::from("base.json"),
                head: PathBuf::from("head.json"),
                threshold_pct: Some(7.5),
            })
        );
        assert_eq!(
            parse_compare_args(&["base.json", "head.json", "--threshold=0"]).unwrap(),
            CompareCommand::Run(CompareOptions {
                base: PathBuf::from("base.json"),
                head: PathBuf::from("head.json"),
                threshold_pct: Some(0.0),
            })
        );
        assert!(parse_compare_args(&["base.json"]).is_err());
        assert!(parse_compare_args(&["base.json", "head.json", "--threshold", "-1"]).is_err());
        assert!(parse_compare_args(&["base.json", "head.json", "--threshold", "NaN"]).is_err());
    }

    #[test]
    fn parse_perf_bench_prefers_json_line_and_falls_back_to_table() {
        let json_output = r#"
running 1 test
=== GPUI Material Release Performance Baseline (1040x744, 20 warmup + 100 frames) ===
| Benchmark                            | Layer     |   p50 us |   p95 us |   p99 us |  mean us | quads | frames |
| ------------------------------------ | --------- | -------- | -------- | -------- | -------- | ----- | ------ |
| engine/empty_window                  | Engine    |     42.1 |     50.2 |     55.3 |     43.4 |     1 |      0 |
PERF_BENCH_JSON:{"viewport_width":1040,"viewport_height":744,"warmup_frames":20,"sample_frames":100,"benchmarks":[{"name":"engine/empty_window","layer":"Engine","p50_us":42.125,"p95_us":50.25,"p99_us":55.375,"mean_us":43.4,"quads":1,"scheduled_frames":0}]}
test tests::perf_bench::release_performance_baseline ... ok
"#;
        let parsed = parse_perf_bench_output(json_output).unwrap();
        assert_eq!(parsed.viewport_width, 1040.0);
        assert_eq!(parsed.viewport_height, 744.0);
        assert_eq!(parsed.warmup_frames, 20);
        assert_eq!(parsed.sample_frames, 100);
        assert_eq!(parsed.benchmarks.len(), 1);
        assert_eq!(parsed.benchmarks[0].name, "engine/empty_window");
        assert!((parsed.benchmarks[0].p50_us - 42.125).abs() < 1e-9);
        assert_eq!(parsed.benchmarks[0].quads, 1);

        let table_only = r#"
=== GPUI Material Release Performance Baseline (1280x720, 10 warmup + 50 frames) ===
| Benchmark                            | Layer     |   p50 us |   p95 us |   p99 us |  mean us | quads | frames |
| ------------------------------------ | --------- | -------- | -------- | -------- | -------- | ----- | ------ |
| engine/empty_window                  | Engine    |     42.1 |     50.2 |     55.3 |     43.4 |     1 |      0 |
| component/animated_visible           | Component |    310.5 |    350.0 |    390.2 |    318.7 |    42 |     99 |
"#;
        let parsed_table = parse_perf_bench_output(table_only).unwrap();
        assert_eq!(parsed_table.viewport_width, 1280.0);
        assert_eq!(parsed_table.viewport_height, 720.0);
        assert_eq!(parsed_table.warmup_frames, 10);
        assert_eq!(parsed_table.sample_frames, 50);
        assert_eq!(parsed_table.benchmarks.len(), 2);
        assert_eq!(
            parsed_table.benchmarks[1].name,
            "component/animated_visible"
        );
        assert_eq!(parsed_table.benchmarks[1].scheduled_frames, 99);

        assert!(parse_perf_bench_output("unrelated test output").is_err());
    }

    #[test]
    fn collect_wasm_metrics_measures_directory_artifacts() {
        let dir = unique_temp_dir("test-wasm").unwrap();
        fs::write(dir.0.join("material-gallery_bg.wasm"), vec![0u8; 4096]).unwrap();
        fs::write(dir.0.join("material-gallery.js"), vec![b'x'; 512]).unwrap();
        fs::create_dir_all(dir.0.join("snippets")).unwrap();
        fs::write(dir.0.join("snippets/helper.mjs"), vec![b'y'; 128]).unwrap();

        let metrics = collect_wasm_metrics(&dir.0, "wasm-release").unwrap();
        assert_eq!(metrics.profile, "wasm-release");
        assert_eq!(metrics.wasm_bytes, 4096);
        assert_eq!(metrics.gzip_wasm_bytes, None);
        assert_eq!(metrics.js_bytes, 640);
        assert_eq!(metrics.total_bytes, 4736);
        assert_eq!(metrics.files.get("snippets/helper.mjs"), Some(&128));

        fs::write(dir.0.join("material-gallery_bg.wasm.gz"), vec![1u8; 1024]).unwrap();
        let metrics_with_gz = collect_wasm_metrics(&dir.0, "wasm-release").unwrap();
        assert_eq!(metrics_with_gz.gzip_wasm_bytes, Some(1024));
        assert_eq!(metrics_with_gz.total_bytes, 4736);

        let empty_dir = unique_temp_dir("test-wasm-empty").unwrap();
        fs::write(empty_dir.0.join("only.js"), b"export {}").unwrap();
        assert!(collect_wasm_metrics(&empty_dir.0, "wasm-release").is_err());
    }

    #[test]
    fn baseline_report_json_roundtrip() {
        let report = BaselineReport {
            schema_version: SCHEMA_VERSION,
            metadata: sample_metadata("abcdef1234567890"),
            perf: Some(sample_perf(100.0, 2)),
            timings: Some(sample_timings(42.5, 68.4)),
            wasm: Some(sample_wasm(4_194_304, Some(1_200_000), 32_768)),
        };

        let serialized = serde_json::to_string_pretty(&report).unwrap();
        let deserialized: BaselineReport = serde_json::from_str(&serialized).unwrap();
        assert_eq!(report, deserialized);

        let partial = BaselineReport {
            schema_version: SCHEMA_VERSION,
            metadata: sample_metadata("1234567890abcdef"),
            perf: Some(sample_perf(80.0, 1)),
            timings: None,
            wasm: None,
        };
        let partial_json = serde_json::to_string(&partial).unwrap();
        assert!(!partial_json.contains("\"timings\""));
        assert!(!partial_json.contains("\"wasm\""));
        let partial_back: BaselineReport = serde_json::from_str(&partial_json).unwrap();
        assert_eq!(partial, partial_back);
    }

    #[test]
    fn compare_reports_renders_markdown_and_detects_regressions() {
        let base = BaselineReport {
            schema_version: SCHEMA_VERSION,
            metadata: sample_metadata("111111111111aaaa"),
            perf: Some(sample_perf(100.0, 2)),
            timings: Some(sample_timings(40.0, 60.0)),
            wasm: Some(sample_wasm(1_000_000, Some(300_000), 20_000)),
        };
        let head = BaselineReport {
            schema_version: SCHEMA_VERSION,
            metadata: sample_metadata("222222222222bbbb"),
            perf: Some(sample_perf(115.0, 2)),
            timings: Some(sample_timings(38.0, 69.0)),
            wasm: Some(sample_wasm(1_050_000, Some(345_000), 20_000)),
        };

        let within = compare_reports(&base, &head, Some(20.0)).unwrap();
        assert!(within.regressions.is_empty());
        assert!(
            within
                .markdown
                .contains("## Frame performance (`perf_bench`)")
        );
        assert!(
            within
                .markdown
                .contains("## Build timings (`gpui-material-gallery`)")
        );
        assert!(
            within
                .markdown
                .contains("## WASM bundle size (`gpui-material-gallery-web`)")
        );
        assert!(
            within
                .markdown
                .contains("| `engine/empty_window` | p50 (us) | 100.0 | 115.0 | +15.0 | +15.0% |")
        );
        assert!(
            within
                .markdown
                .contains("| `cargo check` (cold) | 40.000 s | 38.000 s | -2.000 s | -5.0% |")
        );
        assert!(
            within
                .markdown
                .contains("| `.wasm` (raw) | 1,000,000 B | 1,050,000 B | +50,000 B | +5.0% |")
        );
        assert!(
            within
                .markdown
                .contains("| `.wasm` (gzip) | 300,000 B | 345,000 B | +45,000 B | +15.0% |")
        );

        let exceeded = compare_reports(&base, &head, Some(10.0)).unwrap();
        assert!(!exceeded.regressions.is_empty());
        assert!(
            exceeded
                .markdown
                .contains("## Regressions exceeding +10.00%")
        );
        assert!(
            exceeded
                .regressions
                .iter()
                .any(|r| r.section == "perf" && r.metric == "engine/empty_window p50 (us)")
        );
        assert!(
            exceeded
                .regressions
                .iter()
                .any(|r| r.section == "timings" && r.metric == "`cargo build` (cold)")
        );
        assert!(
            exceeded
                .regressions
                .iter()
                .any(|r| r.section == "wasm" && r.metric == "`.wasm` (gzip)")
        );
    }

    #[test]
    fn compare_reports_handles_disjoint_and_zero_base_metrics() {
        let perf_only = BaselineReport {
            schema_version: SCHEMA_VERSION,
            metadata: sample_metadata("aaaa"),
            perf: Some(sample_perf(100.0, 0)),
            timings: None,
            wasm: None,
        };
        let wasm_only = BaselineReport {
            schema_version: SCHEMA_VERSION,
            metadata: sample_metadata("bbbb"),
            perf: None,
            timings: None,
            wasm: Some(sample_wasm(1000, None, 100)),
        };
        assert!(compare_reports(&perf_only, &wasm_only, None).is_err());

        let perf_head = BaselineReport {
            schema_version: SCHEMA_VERSION,
            metadata: sample_metadata("cccc"),
            perf: Some(sample_perf(100.0, 3)),
            timings: Some(sample_timings(10.0, 20.0)),
            wasm: None,
        };
        let summary = compare_reports(&perf_only, &perf_head, Some(5.0)).unwrap();
        assert!(
            summary
                .markdown
                .contains("| `engine/empty_window` | quads | 0 | 3 | +3 | n/a |")
        );
        assert!(
            summary
                .markdown
                .contains("`timings` present only in head baseline; skipped comparison.")
        );
        assert!(summary.regressions.is_empty());
    }

    #[test]
    fn compare_command_reads_json_files_and_enforces_threshold() {
        let dir = unique_temp_dir("test-compare").unwrap();
        let base_path = dir.0.join("base.json");
        let head_path = dir.0.join("head.json");

        let base = BaselineReport {
            schema_version: SCHEMA_VERSION,
            metadata: sample_metadata("111111111111"),
            perf: Some(sample_perf(100.0, 2)),
            timings: None,
            wasm: Some(sample_wasm(100_000, None, 10_000)),
        };
        let head = BaselineReport {
            schema_version: SCHEMA_VERSION,
            metadata: sample_metadata("222222222222"),
            perf: Some(sample_perf(125.0, 2)),
            timings: None,
            wasm: Some(sample_wasm(102_000, None, 10_000)),
        };

        fs::write(&base_path, serde_json::to_string_pretty(&base).unwrap()).unwrap();
        fs::write(&head_path, serde_json::to_string_pretty(&head).unwrap()).unwrap();

        compare(&CompareOptions {
            base: base_path.clone(),
            head: head_path.clone(),
            threshold_pct: Some(30.0),
        })
        .unwrap();

        let err = compare(&CompareOptions {
            base: base_path,
            head: head_path,
            threshold_pct: Some(10.0),
        })
        .unwrap_err();
        assert!(err.to_string().contains("regressed by more than +10.00%"));
    }
}
