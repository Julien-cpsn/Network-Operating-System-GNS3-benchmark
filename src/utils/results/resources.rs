//! Aggregates the router resource logs of an experiment's runs into one summary per router.
//!
//! Expected layout (logs are looked up in each run folder):
//! ```text
//! <experiment>/
//!   run_1/experiment.log      <- contains the "Experiment start" / "Experiment end" lines
//!   run_1/<router>.log        <- one resource log per router
//!   run_2/...
//!   resources/<router>.csv    <- written here, one summary per router
//! ```
//!
//! A router log is recognized by its content, not by its name: any `*.log` of a run folder
//! (other than `experiment.log`) that contains resource readings is a router log, and the
//! router name is the file name without extension.
//!
//! Method: every run is aligned on its own experiment start, resampled onto a common 1 s
//! grid by linear interpolation (never across a gap, never beyond the first/last reading),
//! then the runs are reduced point by point (median, mean, min, max).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use chrono::DateTime;
use tracing::{info, warn};
use crate::utils::results::flent::summary::SUMMARY_FOLDER_NAME;
use crate::utils::results::utils::natural_key;

const TARGET: &str = "results";


const RUN_DIR_PREFIX: &str = "run_";
const EXPERIMENT_LOG_NAME: &str = "experiment.log";

/// Sampling interval of the resource monitor (the `sleep 1` of the router loop).
const GRID_STEP_S: f64 = 1.0;
/// Data kept before the start and after the end of the experiment (same as the plot script).
const MARGIN_S: f64 = 10.0;
/// Two readings further apart than this are not interpolated across.
const MAX_GAP_S: f64 = 2.5 * GRID_STEP_S;

/// One reading of a router's resource log; `t` is in seconds since the experiment start.
#[derive(Debug, Clone, PartialEq)]
struct Sample {
    t: f64,
    mem_used_mib: f64,
    /// `None` when the reported value is out of the 0..=100 range (broken reading).
    cpu_pct: Option<f64>,
}

/// A router's resources during one run.
struct RouterRun {
    duration_s: f64,
    samples: Vec<Sample>,
}

#[derive(Debug, Clone, PartialEq)]
struct Stats {
    n: usize,
    median: f64,
    mean: f64,
    min: f64,
    max: f64,
}

impl Stats {
    fn of(values: &[f64]) -> Option<Stats> {
        if values.is_empty() {
            return None;
        }
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.total_cmp(b));
        let mid = sorted.len() / 2;
        let median = if sorted.len() % 2 == 1 { sorted[mid] } else { (sorted[mid - 1] + sorted[mid]) / 2.0 };
        Some(Stats {
            n: sorted.len(),
            median,
            mean: sorted.iter().sum::<f64>() / sorted.len() as f64,
            min: sorted[0],
            max: sorted[sorted.len() - 1],
        })
    }
}

#[derive(Debug)]
struct Row {
    elapsed_s: f64,
    in_experiment: bool,
    mem: Option<Stats>,
    cpu: Option<Stats>,
}

/// Writes `<experiment_dir>/resources/<router>.csv` for every router found in the runs and
/// returns the paths. Files are rewritten on every call, since runs may have been added.
pub fn write_resources_summaries(experiment_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut per_router: BTreeMap<String, Vec<RouterRun>> = BTreeMap::new();
    let mut usable_runs = 0usize;

    for run_dir in list_run_dirs(experiment_dir)? {
        let window = read_text_lossy(&run_dir.join(EXPERIMENT_LOG_NAME)).ok().and_then(|t| experiment_window(&t));
        let Some((start, end)) = window else {
            warn!(target: TARGET, "{}: no experiment start/end found, run ignored for resources", run_dir.display());
            continue;
        };
        usable_runs += 1;

        for log_path in list_router_log_candidates(&run_dir)? {
            let text = read_text_lossy(&log_path)?;
            let samples = parse_resource_log(&text, start, end);
            if samples.is_empty() {
                //debug!(target: TARGET, "{}: no resource readings, not a router log", log_path.display());
                continue;
            }
            let router = log_path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
            per_router.entry(router).or_default().push(RouterRun { duration_s: end - start, samples });
        }
    }

    let output_dir = experiment_dir.join(SUMMARY_FOLDER_NAME);
    let mut written = Vec::new();
    for (router, runs) in &per_router {
        if runs.len() < usable_runs {
            warn!(target: TARGET, "Router '{router}': resource log found in {}/{usable_runs} runs", runs.len());
        }
        let output_path = output_dir.join(format!("{router}.csv"));
        write_summary_csv(&output_path, &summarize(runs))?;
        info!(target: TARGET, "Summarized resources of router '{router}' over {} run(s): {}", runs.len(), output_path.display());
        written.push(output_path);
    }
    if written.is_empty() {
        info!(target: TARGET, "No router resource log found in {}", experiment_dir.display());
    }
    Ok(written)
}

/// Reduces the runs of a router, point by point, on a common time grid.
fn summarize(runs: &[RouterRun]) -> Vec<Row> {
    let durations: Vec<f64> = runs.iter().map(|r| r.duration_s).collect();
    let max_duration = durations.iter().copied().fold(0.0, f64::max);
    let median_duration = Stats::of(&durations).map_or(max_duration, |s| s.median);

    let mem_series: Vec<Vec<(f64, f64)>> =
        runs.iter().map(|r| r.samples.iter().map(|s| (s.t, s.mem_used_mib)).collect()).collect();
    let cpu_series: Vec<Vec<(f64, f64)>> =
        runs.iter().map(|r| r.samples.iter().filter_map(|s| s.cpu_pct.map(|c| (s.t, c))).collect()).collect();

    let steps = ((max_duration + 2.0 * MARGIN_S) / GRID_STEP_S).round() as usize;
    let mut rows = Vec::new();
    for k in 0..=steps {
        let t = -MARGIN_S + k as f64 * GRID_STEP_S;
        let at = |series: &Vec<Vec<(f64, f64)>>| -> Option<Stats> {
            let values: Vec<f64> = series.iter().filter_map(|s| interpolate(s, t)).collect();
            Stats::of(&values)
        };
        let (mem, cpu) = (at(&mem_series), at(&cpu_series));
        if mem.is_none() && cpu.is_none() {
            continue;
        }
        rows.push(Row { elapsed_s: t, in_experiment: t >= 0.0 && t <= median_duration + 1e-9, mem, cpu });
    }
    rows
}

/// Linear interpolation of `points` (sorted by time) at time `t`. `None` before the first
/// point, after the last one, or when the two surrounding points are too far apart.
fn interpolate(points: &[(f64, f64)], t: f64) -> Option<f64> {
    let next = points.partition_point(|p| p.0 <= t);
    if next == 0 {
        return None;
    }
    let (t0, v0) = points[next - 1];
    if t0 == t {
        return Some(v0);
    }
    let &(t1, v1) = points.get(next)?;
    if t1 - t0 > MAX_GAP_S {
        return None;
    }
    Some(v0 + (v1 - v0) * (t - t0) / (t1 - t0))
}

/// Readings of a resource log, as seconds since `start`, restricted to the experiment
/// window plus a margin. Each reading is a line like
/// `2026-10-19T11:31:54.226655Z TRACE rx: 3921<TAB>3661<TAB>260<TAB>8.63268`
/// (total MiB, free MiB, used MiB, CPU %). Echoed commands, the header and any line that
/// does not hold four numbers are ignored, as are impossible readings (used > total).
fn parse_resource_log(text: &str, start: f64, end: f64) -> Vec<Sample> {
    let mut samples: Vec<Sample> = text
        .lines()
        .filter_map(|line| {
            let (head, payload) = line.split_once("TRACE rx:")?;
            let time = head.split_whitespace().find_map(parse_time)?;
            let mut fields = payload.split_whitespace().take(4).map(|f| f.parse::<f64>().ok().filter(|v| v.is_finite()));
            let (total, _free, used, cpu) = (fields.next()??, fields.next()??, fields.next()??, fields.next()??);
            if total <= 0.0 || used < 0.0 || used > total {
                return None;
            }
            Some(Sample { t: time - start, mem_used_mib: used, cpu_pct: (0.0..=100.0).contains(&cpu).then_some(cpu) })
        })
        .filter(|s| s.t >= -MARGIN_S && s.t <= end - start + MARGIN_S)
        .collect();
    samples.sort_by(|a, b| a.t.total_cmp(&b.t));
    samples
}

/// Start and end of the experiment (as epoch seconds) from `experiment.log`: the last
/// "Experiment start" and the last "Experiment end", like the plot script does.
fn experiment_window(text: &str) -> Option<(f64, f64)> {
    let last_time_of = |marker: &str| {
        text.lines()
            .filter(|line| line.contains(marker))
            .filter_map(|line| line.split_whitespace().find_map(parse_time))
            .last()
    };
    let start = last_time_of("INFO experiment: Experiment start")?;
    let end = last_time_of("INFO experiment: Experiment end")?;
    (end >= start).then_some((start, end))
}

/// RFC 3339 timestamp (`2026-10-19T11:31:54.226655Z`) as epoch seconds.
fn parse_time(token: &str) -> Option<f64> {
    DateTime::parse_from_rfc3339(token).ok().map(|dt| dt.timestamp_micros() as f64 / 1e6)
}

fn list_run_dirs(experiment_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut dirs: Vec<PathBuf> = fs::read_dir(experiment_dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_dir() && p.file_name().is_some_and(|n| n.to_string_lossy().starts_with(RUN_DIR_PREFIX)))
        .collect();
    dirs.sort_by_key(|p| natural_key(&p.file_name().unwrap_or_default().to_string_lossy()));
    Ok(dirs)
}

fn list_router_log_candidates(run_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut logs: Vec<PathBuf> = fs::read_dir(run_dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.is_file()
                && p.extension().is_some_and(|e| e == "log")
                && p.file_name().is_some_and(|n| n != EXPERIMENT_LOG_NAME)
        })
        .collect();
    logs.sort();
    Ok(logs)
}

fn write_summary_csv(path: &Path, rows: &[Row]) -> anyhow::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).with_context(|| format!("cannot create {}", dir.display()))?;
    }
    let mut writer = csv::Writer::from_path(path).with_context(|| format!("cannot create {}", path.display()))?;
    writer.write_record([
        "elapsed_s", "in_experiment",
        "n_mem", "mem_used_mib_median", "mem_used_mib_mean", "mem_used_mib_min", "mem_used_mib_max",
        "n_cpu", "cpu_pct_median", "cpu_pct_mean", "cpu_pct_min", "cpu_pct_max",
    ])?;
    let stats_fields = |s: &Option<Stats>| -> Vec<String> {
        match s {
            Some(s) => vec![
                s.n.to_string(),
                format!("{:.4}", s.median),
                format!("{:.4}", s.mean),
                format!("{:.4}", s.min),
                format!("{:.4}", s.max),
            ],
            None => vec!["0".into(), String::new(), String::new(), String::new(), String::new()],
        }
    };
    for row in rows {
        let mut record = vec![format!("{:.3}", row.elapsed_s), row.in_experiment.to_string()];
        record.extend(stats_fields(&row.mem));
        record.extend(stats_fields(&row.cpu));
        writer.write_record(&record)?;
    }
    writer.flush()?;
    Ok(())
}

/// The logs may contain bytes that are not valid UTF-8 (serial console noise).
fn read_text_lossy(path: &Path) -> anyhow::Result<String> {
    let bytes = fs::read(path).with_context(|| format!("cannot read {}", path.display()))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}