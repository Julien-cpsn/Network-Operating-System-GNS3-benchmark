use std::collections::{BTreeMap, HashMap};
use std::{fmt, fs};
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use anyhow::{bail, Context};
use tracing::{debug, info, warn};


const TARGET: &str = "summary";

pub const SUMMARY_FOLDER_NAME: &str = "summary";

/// Name of the summary file, written at the root of the experiment folder.
pub const SUMMARY_FILE_NAME: &str = "runs_summary.csv";

/// Column whose presence delimits the load phase (RRUL active). Outside this phase
/// (5 s before and 5 s after), the values describe the idle network.
const ACTIVE_PHASE_COLUMN: &str = "TCP totals";

/// Below this number of runs the standard deviation is too poorly estimated: the
/// recommendation is provisional.
const RELIABLE_ESTIMATE_MIN_RUNS: usize = 5;

// `VoIP::delay` is deliberately absent: it depends on the clock offset between machines.
const METRICS: &[MetricSpec] = &[
    MetricSpec { name: "tcp_download_mbps", column: "TCP download sum", stat: RunStat::Mean, drives_planning: true },
    MetricSpec { name: "tcp_upload_mbps", column: "TCP upload sum", stat: RunStat::Mean, drives_planning: true },
    MetricSpec { name: "tcp_total_mbps", column: "TCP totals", stat: RunStat::Mean, drives_planning: true },
    MetricSpec { name: "icmp_latency_mean_ms", column: "Ping (ms) ICMP", stat: RunStat::Mean, drives_planning: true },
    MetricSpec { name: "icmp_latency_p95_ms", column: "Ping (ms) ICMP", stat: RunStat::P95, drives_planning: false },
    MetricSpec { name: "voip_rtt_mean_ms", column: "VoIP::rtt", stat: RunStat::Mean, drives_planning: true },
    MetricSpec { name: "voip_induced_delay_mean_ms", column: "VoIP::induced_delay", stat: RunStat::Mean, drives_planning: true },
    MetricSpec { name: "voip_jitter_mean_ms", column: "VoIP::jitter", stat: RunStat::Mean, drives_planning: false },
    MetricSpec { name: "voip_loss_mean", column: "VoIP::loss", stat: RunStat::Mean, drives_planning: false },
];

struct MetricSpec {
    /// Metric name in the summary file.
    name: &'static str,
    /// Column name in the Flent CSV.
    column: &'static str,
    /// How a run is reduced to a single value for this column.
    stat: RunStat,
    /// Contributes to the recommended number of runs. Tail metrics (p95) and metrics
    /// close to zero (loss, jitter) have a coefficient of variation that would skew the
    /// recommendation; they are still summarized.
    drives_planning: bool,
}


#[derive(Clone, Copy)]
enum RunStat {
    Mean,
    P95,
}

#[derive(Debug, Clone)]
pub struct PlanningConfig {
    /// Target half-width of the 95% confidence interval, relative to the mean
    /// (0.05 = ±5%).
    pub relative_precision: f64,
    /// Minimum recommended number of runs.
    pub min_runs: usize,
    /// Beyond this, the target precision is considered out of reach.
    pub max_runs: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunsNeeded {
    /// Total number of runs (already completed runs included) to reach the target precision.
    Enough(usize),
    /// More than `max_runs` runs would be needed.
    TooMany,
    /// Cannot be computed (fewer than 2 runs, or a zero mean).
    Unknown,
}


impl Default for PlanningConfig {
    fn default() -> Self {
        Self { relative_precision: 0.05, min_runs: 3, max_runs: 200 }
    }
}


#[derive(Debug, Clone)]
pub struct Summary {
    pub n: usize,
    pub mean: f64,
    pub median: f64,
    pub std_dev: Option<f64>,
    pub cv_pct: Option<f64>,
    pub ci95_half_width: Option<f64>,
    pub min: f64,
    pub max: f64,
}

impl Display for RunsNeeded {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RunsNeeded::Enough(n) => write!(f, "{n}"),
            RunsNeeded::TooMany => write!(f, ">max"),
            RunsNeeded::Unknown => Ok(()),
        }
    }
}

/// Writes `runs_summary.csv` at the root of the experiment and logs the recommended number
/// of runs for each test. Returns the path of the file, or `None` if there are no results.
/// The file is recomputed on every call, since runs may have been added in the meantime.
///
/// `result_paths_per_run_per_test`: run -> test -> paths of the `.flent.gz` files
/// (the neighboring `.flent.csv` must already exist). Each file is treated as one
/// observation; in practice there is one per run and per test.
pub fn write_runs_summary(result_paths_per_run_per_test: &HashMap<String, HashMap<String, Vec<PathBuf>>>, config: &PlanningConfig) -> anyhow::Result<Option<PathBuf>> {
    let Some(first_path) = result_paths_per_run_per_test
        .values()
        .flat_map(|per_test| per_test.values())
        .flatten()
        .next()
    else {
        return Ok(None);
    };
    let experiment_dir = experiment_dir(first_path)?;
    let summary_dir = experiment_dir.join(SUMMARY_FOLDER_NAME);

    if !summary_dir.exists() {
        fs::create_dir(&summary_dir)?;
    }

    // test -> metric -> one value per run
    let mut values: BTreeMap<&str, BTreeMap<&'static str, Vec<f64>>> = BTreeMap::new();
    // test -> number of usable runs
    let mut runs_per_test: BTreeMap<&str, usize> = BTreeMap::new();

    let mut runs: Vec<&String> = result_paths_per_run_per_test.keys().collect();

    runs.sort();
    for run in runs {
        for (test, paths) in &result_paths_per_run_per_test[run] {
            for path in paths {
                let csv_path = path.with_extension("csv");
                let run_metrics = extract_run_metrics(&csv_path).with_context(|| format!("run \"{run}\", test \"{test}\""))?;

                if run_metrics.is_empty() {
                    warn!(target: TARGET, "No usable metric in {}", csv_path.display());
                    continue;
                }

                *runs_per_test.entry(test).or_default() += 1;

                for (metric, value) in run_metrics {
                    values.entry(test).or_default().entry(metric).or_default().push(value);
                }
            }
        }
    }

    let summary_path = summary_dir.join(SUMMARY_FILE_NAME);
    let mut writer = csv::Writer::from_path(&summary_path).with_context(|| format!("cannot create {}", summary_path.display()))?;

    writer.write_record([
        "test", "metric", "n_runs", "mean", "median", "std_dev", "cv_pct", "ci95_half_width",
        "ci95_half_width_pct", "min", "max", "runs_needed", "drives_planning",
    ])?;

    for (test, per_metric) in &values {
        let mut recommended = 0usize;
        let mut unreachable_metrics: Vec<&str> = Vec::new();
        let mut unknown = false;

        for spec in METRICS {
            let Some(samples) = per_metric.get(spec.name) else { continue };
            let summary = summarize(samples);
            let needed = runs_needed(&summary, config);

            if spec.drives_planning {
                match needed {
                    RunsNeeded::Enough(n) => recommended = recommended.max(n),
                    RunsNeeded::TooMany => unreachable_metrics.push(spec.name),
                    RunsNeeded::Unknown => unknown = true,
                }
            }

            let half_width_pct = match (summary.ci95_half_width, summary.mean) {
                (Some(h), m) if m.abs() > f64::EPSILON => Some(h / m.abs() * 100.0),
                _ => None,
            };

            writer.write_record([
                test.to_string(),
                spec.name.to_string(),
                summary.n.to_string(),
                fmt_f(summary.mean),
                fmt_f(summary.median),
                fmt_opt(summary.std_dev),
                fmt_opt(summary.cv_pct),
                fmt_opt(summary.ci95_half_width),
                fmt_opt(half_width_pct),
                fmt_f(summary.min),
                fmt_f(summary.max),
                needed.to_string(),
                spec.drives_planning.to_string(),
            ])?;
        }

        let done = runs_per_test.get(test).copied().unwrap_or(0);
        log_recommendation(test, done, recommended, &unreachable_metrics, unknown, config);
    }

    writer.flush()?;

    debug!(target: TARGET, "\tWrote runs summary");
    Ok(Some(summary_path))
}

fn log_recommendation(test: &str, done: usize, recommended: usize, unreachable_metrics: &[&str], unknown: bool, config: &PlanningConfig) {
    if done < 2 || (recommended == 0 && unreachable_metrics.is_empty()) {
        warn!(target: TARGET, "\t\tTest \"{test}\": {done} run(s), at least 2 are needed to estimate the variability (3 minimum recommended)");
        return;
    }
    if !unreachable_metrics.is_empty() {
        warn!(
            target: TARGET,
            "\t\tTest \"{test}\": more than {} runs would be needed for ±{:.0}% on {}; the configuration is too unstable for this precision",
            config.max_runs, config.relative_precision * 100.0, unreachable_metrics.join(", ")
        );
    }
    if recommended > 0 {
        let missing = recommended.saturating_sub(done);
        let provisional = if done < RELIABLE_ESTIMATE_MIN_RUNS {
            " (provisional: fewer than 5 runs, the variability is still poorly estimated)"
        } else {
            ""
        };
        if missing == 0 {
            info!(target: TARGET, "\t\tTest \"{test}\": {done} runs done, {recommended} needed for ±{:.0}% (95% CI): enough{provisional}", config.relative_precision * 100.0);
        } else {
            info!(target: TARGET, "\t\tTest \"{test}\": {done} runs done, {recommended} needed for ±{:.0}% (95% CI): {missing} more run(s){provisional}", config.relative_precision * 100.0);
        }
    }
    if unknown {
        warn!(target: TARGET, "\t\tTest \"{test}\": some planning metrics have a zero mean, runs needed not computable for them");
    }
}

/// `<experiment>/<run>/<test>/<file>` -> `<experiment>`.
pub fn experiment_dir(result_path: &Path) -> anyhow::Result<&Path> {
    match result_path.ancestors().nth(3) {
        Some(dir) if !dir.as_os_str().is_empty() => Ok(dir),
        _ => bail!("unexpected result path layout: {}", result_path.display()),
    }
}

/// Reduces a run to one value per metric, computed over the load phase.
/// Columns are read by name (their order changes from one export to the next).
fn extract_run_metrics(csv_path: &Path) -> anyhow::Result<BTreeMap<&'static str, f64>> {
    let mut reader = csv::Reader::from_path(csv_path).with_context(|| format!("cannot read {}", csv_path.display()))?;
    let headers = reader.headers()?.clone();
    let index_of = |name: &str| headers.iter().position(|h| h == name);

    // Without this column (a test with no RRUL), all rows are kept.
    let active_idx = index_of(ACTIVE_PHASE_COLUMN);
    let columns: Vec<(&MetricSpec, usize)> = METRICS.iter().filter_map(|m| index_of(m.column).map(|i| (m, i))).collect();

    let mut samples: Vec<Vec<f64>> = vec![Vec::new(); columns.len()];
    for record in reader.records() {
        let record = record?;

        if let Some(idx) = active_idx {
            if record.get(idx).and_then(parse_value).is_none() {
                continue;
            }
        }

        for (k, (_, idx)) in columns.iter().enumerate() {
            if let Some(v) = record.get(*idx).and_then(parse_value) {
                samples[k].push(v);
            }
        }
    }

    let mut result = BTreeMap::new();
    for ((spec, _), mut column_samples) in columns.into_iter().zip(samples) {
        if column_samples.is_empty() {
            continue;
        }
        let value = match spec.stat {
            RunStat::Mean => column_samples.iter().sum::<f64>() / column_samples.len() as f64,
            RunStat::P95 => {
                column_samples.sort_by(|a, b| a.total_cmp(b));
                percentile_sorted(&column_samples, 0.95)
            }
        };
        result.insert(spec.name, value);
    }

    Ok(result)
}

fn parse_value(raw: &str) -> Option<f64> {
    raw.trim().parse::<f64>().ok().filter(|v| v.is_finite())
}

/// Percentile by linear interpolation. `sorted` must be sorted and non-empty.
fn percentile_sorted(sorted: &[f64], q: f64) -> f64 {
    let pos = q * (sorted.len() - 1) as f64;
    let (lo, hi) = (pos.floor() as usize, pos.ceil() as usize);
    sorted[lo] + (sorted[hi] - sorted[lo]) * (pos - lo as f64)
}

/// Statistics across runs. `values` must be non-empty.
fn summarize(values: &[f64]) -> Summary {
    let n = values.len();
    let mean = values.iter().sum::<f64>() / n as f64;
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let median = percentile_sorted(&sorted, 0.5);

    let std_dev = (n >= 2).then(|| {
        (values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1) as f64).sqrt()
    });
    let cv_pct = match std_dev {
        Some(s) if mean.abs() > f64::EPSILON => Some(s / mean.abs() * 100.0),
        _ => None,
    };
    let ci95_half_width = std_dev.map(|s| t_critical_95(n - 1) * s / (n as f64).sqrt());

    Summary { n, mean, median, std_dev, cv_pct, ci95_half_width, min: sorted[0], max: sorted[n - 1] }
}

/// Smallest total number of runs `n` such that `t(n-1) * s / sqrt(n) <= precision * |mean|`,
/// where `s` is the observed standard deviation across runs. The value of `t` depends on
/// `n`, hence the iterative search rather than a closed-form formula.
fn runs_needed(summary: &Summary, config: &PlanningConfig) -> RunsNeeded {
    let (Some(s), true) = (summary.std_dev, summary.mean.abs() > f64::EPSILON) else {
        return RunsNeeded::Unknown;
    };

    let target = config.relative_precision * summary.mean.abs();
    let first = config.min_runs.max(2);

    for n in first..=config.max_runs {
        if t_critical_95(n - 1) * s / (n as f64).sqrt() <= target {
            return RunsNeeded::Enough(n);
        }
    }

    RunsNeeded::TooMany
}

/// Two-sided 95% Student's t quantile for `df` degrees of freedom.
fn t_critical_95(df: usize) -> f64 {
    const TABLE: [f64; 30] = [
        12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.160,
        2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060, 2.056,
        2.052, 2.048, 2.045, 2.042,
    ];

    match df {
        0 => f64::INFINITY,
        1..=30 => TABLE[df - 1],
        31..=60 => 2.0,
        61..=120 => 1.98,
        _ => 1.96,
    }
}

fn fmt_f(v: f64) -> String {
    format!("{v:.4}")
}

fn fmt_opt(v: Option<f64>) -> String {
    v.map(fmt_f).unwrap_or_default()
}