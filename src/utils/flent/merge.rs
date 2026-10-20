//! Merges the runs of an experiment into one Flent result file per test.
//!
//! Output layout (next to `runs_summary.csv`, one folder per test):
//! ```text
//! <experiment>/
//!   runs_summary.csv
//!   <test>/merged-<name of the first run's file>.flent.gz
//!   run_1/<test>/<name>.flent.gz
//!   run_2/<test>/<name>.flent.gz
//! ```
//!
//! Every data series is reduced point by point across the runs (mean or median), so the
//! merged file looks like a single run and can be plotted with `flent -i <file> -p ...`.
//! Raw values (used by some plots) are pooled from all runs, with their timestamps shifted
//! so that every run starts at the `T0` of the first one.
//!
//! A `.flent.gz` is gzipped JSON. The documents are handled as `serde_json::Value`, so any
//! field this module does not know about is carried over from the first run untouched.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Read};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use chrono::NaiveDateTime;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde_json::{json, Map, Value};
use tracing::{info, warn};
use crate::utils::flent::summary::{experiment_dir, SUMMARY_FOLDER_NAME};

const TARGET: &str = "results";


/// Maximum difference (in seconds) tolerated between the time axes of two runs.
const TIME_AXIS_TOLERANCE: f64 = 1e-6;

/// How the runs are reduced at each point of the time axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MergeStat {
    #[allow(unused)]
    Mean,
    /// Robust to a single run having a spike or collapse at a given instant.
    #[default]
    Median,
}

impl MergeStat {
    fn as_str(self) -> &'static str {
        match self {
            MergeStat::Mean => "Mean",
            MergeStat::Median => "Median",
        }
    }
}

/// Writes one merged `.flent.gz` per test, in `<experiment>/<test>/`, and returns the paths.
/// Tests with fewer than two runs are skipped. Files are rewritten on every call, since runs
/// may have been added in the meantime.
///
/// `result_paths_per_run_per_test`: run -> test -> paths of the `.flent.gz` files. Each file is
/// treated as one observation; in practice there is one per run and per test.
pub fn write_merged_results(result_paths_per_run_per_test: &HashMap<String, HashMap<String, Vec<PathBuf>>>, stat: MergeStat) -> anyhow::Result<Vec<PathBuf>> {
    // test -> files, in run order (run_2 before run_10)
    let mut runs: Vec<&String> = result_paths_per_run_per_test.keys().collect();
    runs.sort_by_key(|run| natural_key(run));
    let mut per_test: BTreeMap<&str, Vec<&PathBuf>> = BTreeMap::new();
    for run in runs {
        for (test, paths) in &result_paths_per_run_per_test[run] {
            per_test.entry(test).or_default().extend(paths);
        }
    }

    let Some(first_path) = per_test.values().flatten().next() else {
        return Ok(Vec::new());
    };
    let experiment_dir = experiment_dir(first_path)?.to_path_buf();

    let mut written = Vec::new();
    for (test, paths) in per_test {
        if paths.len() < 2 {
            info!(target: TARGET, "Test '{test}': {} run, nothing to merge", paths.len());
            continue;
        }
        let paths: Vec<PathBuf> = paths.into_iter().cloned().collect();
        let mut merged = merge_runs(&paths, stat).with_context(|| format!("cannot merge the runs of test '{test}'"))?;

        let first_name = paths[0].file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        merged["metadata"]["DATA_FILENAME"] = Value::String(first_name.clone());

        let output_path = experiment_dir.join(SUMMARY_FOLDER_NAME).join(sanitize_dir_name(test)).join(&first_name);
        write_gzipped_json(&output_path, &merged)?;

        info!(target: TARGET, "Merged {} runs of test '{test}' ({}) into {}", paths.len(), stat.as_str(), output_path.file_name().unwrap().to_string_lossy());
        written.push(output_path);
    }
    Ok(written)
}

/// Merges the result files of several runs of the same test into one result document.
pub fn merge_runs(paths: &[PathBuf], stat: MergeStat) -> anyhow::Result<Value> {
    if paths.len() < 2 {
        bail!("at least two runs are needed to merge");
    }
    let docs = paths
        .iter()
        .map(|p| load_result(p).with_context(|| format!("cannot read {}", p.display())))
        .collect::<anyhow::Result<Vec<Value>>>()?;
    let first = &docs[0];

    // Refuse to merge things that are not repeats of the same test.
    for (path, doc) in paths.iter().zip(&docs) {
        if !same_json(&doc["version"], &first["version"]) {
            bail!("{}: different file format version", path.display());
        }
        for key in ["NAME", "STEP_SIZE"] {
            if !same_json(&doc["metadata"][key], &first["metadata"][key]) {
                bail!(
                    "{}: different {key} ({} vs {}); these runs are not repeats of the same test",
                    path.display(), doc["metadata"][key], first["metadata"][key]
                );
            }
        }
    }

    // Common time axis: the shortest run. The axes must agree over that span.
    let axes = paths
        .iter()
        .zip(&docs)
        .map(|(p, d)| numbers(&d["x_values"]).with_context(|| format!("{}: invalid x_values", p.display())))
        .collect::<anyhow::Result<Vec<Vec<f64>>>>()?;
    let n = axes.iter().map(Vec::len).min().unwrap_or(0);
    if n == 0 {
        bail!("a run has no data points");
    }
    for (path, axis) in paths.iter().zip(&axes) {
        if axis.len() != n {
            warn!(target: TARGET, "{}: {} points, truncating to {n}", path.display(), axis.len());
        }
        if axis.iter().zip(&axes[0]).take(n).any(|(a, b)| (a - b).abs() > TIME_AXIS_TOLERANCE) {
            bail!("{}: time axis differs from the first run; cannot merge point by point", path.display());
        }
    }
    let x_values: Vec<Value> = axes[0][..n].iter().map(|&x| Value::from(x)).collect();

    // Series: point-wise reduction over the runs that have the series.
    let mut names: BTreeSet<&str> = BTreeSet::new();
    for doc in &docs {
        if let Some(results) = doc["results"].as_object() {
            names.extend(results.keys().map(String::as_str));
        }
    }
    let mut results = Map::new();
    for name in names {
        let series: Vec<&Vec<Value>> = docs.iter().filter_map(|d| d["results"][name].as_array()).collect();
        if series.len() != docs.len() {
            warn!(target: TARGET, "Series '{name}' is present in {}/{} runs", series.len(), docs.len());
        }
        let reduced: Vec<Value> = (0..n)
            .map(|i| {
                let mut values: Vec<f64> = series
                    .iter()
                    .filter_map(|s| s.get(i).and_then(Value::as_f64))
                    .filter(|v| v.is_finite())
                    .collect();
                reduce(&mut values, stat).map_or(Value::Null, Value::from)
            })
            .collect();
        results.insert(name.to_string(), Value::Array(reduced));
    }

    let raw_values = pool_raw_values(paths, &docs);

    // Metadata: the first run's, with a note that this is a merge.
    let mut metadata = first["metadata"].as_object().cloned().context("metadata is missing")?;
    let mut series_meta = metadata.get("SERIES_META").and_then(Value::as_object).cloned().unwrap_or_default();
    for doc in &docs[1..] {
        if let Some(other) = doc["metadata"]["SERIES_META"].as_object() {
            for (name, meta) in other {
                series_meta.entry(name.clone()).or_insert_with(|| meta.clone());
            }
        }
    }
    metadata.insert("SERIES_META".into(), Value::Object(series_meta));
    let label = format!("{} of {} runs", stat.as_str(), docs.len());

    info!(target: TARGET, "\t> {}", label);

    /*
    let title = match metadata.get("TITLE").and_then(Value::as_str) {
        Some(title) if !title.is_empty() => format!("{title},{label}"),
        _ => label,
    };

    metadata.insert("TITLE".into(), Value::String(title));
    */

    let merged_runs: Vec<Value> = paths
        .iter()
        .filter_map(|p| p.file_name())
        .map(|n| Value::String(n.to_string_lossy().into_owned()))
        .collect();
    metadata.insert("MERGED_RUNS".into(), Value::Array(merged_runs));
    metadata.insert("MERGE_STAT".into(), json!(stat.as_str()));

    Ok(json!({
        "version": first["version"],
        "metadata": metadata,
        "x_values": x_values,
        "results": results,
        "raw_values": raw_values,
    }))
}

/// Pools the raw values of all runs, shifting each run onto the first run's `T0`.
/// If a `T0` cannot be read, the raw values are left out: they would make no sense unshifted.
fn pool_raw_values(paths: &[PathBuf], docs: &[Value]) -> Map<String, Value> {
    let t0s: Option<Vec<f64>> = docs.iter().map(|d| d["metadata"]["T0"].as_str().and_then(parse_flent_time)).collect();
    let Some(t0s) = t0s else {
        warn!(target: TARGET, "Cannot read T0 in every run: raw values are not included in the merged file");
        return Map::new();
    };

    let mut pooled: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for (run, doc) in docs.iter().enumerate() {
        let shift = t0s[run] - t0s[0];
        let Some(raw) = doc["raw_values"].as_object() else { continue };
        for (name, entries) in raw {
            let Some(entries) = entries.as_array() else {
                warn!(target: TARGET, "{}: unexpected raw values for '{name}'", paths[run].display());
                continue;
            };
            let pool = pooled.entry(name.clone()).or_default();
            for entry in entries {
                let mut entry = entry.clone();
                if let Some(t) = entry.get("t").and_then(Value::as_f64) {
                    entry["t"] = Value::from(t - shift);
                }
                pool.push(entry);
            }
        }
    }
    for entries in pooled.values_mut() {
        entries.sort_by(|a, b| time_of(a).total_cmp(&time_of(b)));
    }
    pooled.into_iter().map(|(name, entries)| (name, Value::Array(entries))).collect()
}

fn time_of(entry: &Value) -> f64 {
    entry.get("t").and_then(Value::as_f64).unwrap_or(0.0)
}

/// Mean or median of the values; `None` if there are none. May reorder `values`.
fn reduce(values: &mut [f64], stat: MergeStat) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    Some(match stat {
        MergeStat::Mean => values.iter().sum::<f64>() / values.len() as f64,
        MergeStat::Median => {
            values.sort_by(|a, b| a.total_cmp(b));
            let mid = values.len() / 2;
            if values.len() % 2 == 1 { values[mid] } else { (values[mid - 1] + values[mid]) / 2.0 }
        }
    })
}

/// JSON equality that does not distinguish `1` from `1.0`.
fn same_json(a: &Value, b: &Value) -> bool {
    match (a.as_f64(), b.as_f64()) {
        (Some(x), Some(y)) => x == y,
        _ => a == b,
    }
}

fn numbers(value: &Value) -> Option<Vec<f64>> {
    value.as_array()?.iter().map(Value::as_f64).collect()
}

/// Flent writes `2026-10-19T11:35:00.000000Z` (UTC), or the same without the `Z` (local time,
/// older files), with or without fractional seconds. Only differences between runs are used,
/// so the zone does not matter as long as all runs use the same convention.
fn parse_flent_time(text: &str) -> Option<f64> {
    let naive = NaiveDateTime::parse_from_str(text.trim_end_matches('Z'), "%Y-%m-%dT%H:%M:%S%.f").ok()?;
    Some(naive.and_utc().timestamp_micros() as f64 / 1e6)
}

fn load_result(path: &Path) -> anyhow::Result<Value> {
    let mut file = BufReader::new(File::open(path)?);
    let mut text = String::new();
    if path.extension().is_some_and(|e| e == "gz") {
        GzDecoder::new(file).read_to_string(&mut text)?;
    } else {
        file .read_to_string(&mut text)?;
    }
    match serde_json::from_str(&text) {
        Ok(value) => Ok(value),
        // Python's json module writes NaN / Infinity, which strict JSON parsers reject.
        Err(error) => serde_json::from_str(&sanitize_non_finite(&text)).map_err(|_| error.into()),
    }
}

/// Replaces the bare tokens `NaN`, `Infinity` and `-Infinity` (outside strings) by `null`.
fn sanitize_non_finite(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let (mut in_string, mut escaped, mut i) = (false, false, 0);
    while i < bytes.len() {
        let rest = &bytes[i..];
        if in_string {
            match bytes[i] {
                _ if escaped => escaped = false,
                b'\\' => escaped = true,
                b'"' => in_string = false,
                _ => {}
            }
            out.push(bytes[i]);
            i += 1;
        } else if bytes[i] == b'"' {
            in_string = true;
            out.push(b'"');
            i += 1;
        } else if let Some(len) = [&b"-Infinity"[..], b"Infinity", b"NaN"].iter().find(|t| rest.starts_with(t)).map(|t| t.len()) {
            out.extend_from_slice(b"null");
            i += len;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    // Only ASCII tokens were replaced, so the bytes are still valid UTF-8.
    String::from_utf8(out).unwrap_or_else(|_| text.to_string())
}

fn write_gzipped_json(path: &Path, value: &Value) -> anyhow::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).with_context(|| format!("cannot create {}", dir.display()))?;
    }

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .with_context(|| format!("cannot create {}", path.display()))?;

    let mut encoder = GzEncoder::new(BufWriter::new(file), Compression::default());
    serde_json::to_writer_pretty(&mut encoder, value)?;
    encoder.finish()?.into_inner().map_err(|e| e.into_error())?;
    Ok(())
}

/// `run_2` < `run_10`: compares the text prefix, then the trailing number.
fn natural_key(name: &str) -> (String, u64) {
    let split = name.trim_end_matches(|c: char| c.is_ascii_digit()).len();
    let (prefix, digits) = name.split_at(split);
    (prefix.to_string(), digits.parse().unwrap_or(0))
}

fn sanitize_dir_name(test: &str) -> String {
    test.replace(['/', '\\', '\0'], "_")
}
