use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{DateTime, Utc};
use clap::Args;
use serde_json::json;

use gpx_rs::{
    convert_file, crop, detect_format, gather, merge as merge_gpx, print_human, read_gpx,
    reduce_precision, reverse, shift_time, simplify, smooth, split, strip_extensions,
    strip_metadata, trim, validate_file, write_file, ConvertError, Gpx, InvalidGpxError,
    OperationError, StripMetadataFields, ValidationResult,
};

#[derive(Debug)]
pub struct CliError(String);

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for CliError {}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> Self {
        Self(err.to_string())
    }
}

impl From<gpx_rs::ParseError> for CliError {
    fn from(err: gpx_rs::ParseError) -> Self {
        Self(err.to_string())
    }
}

impl From<ConvertError> for CliError {
    fn from(err: ConvertError) -> Self {
        Self(err.to_string())
    }
}

impl From<OperationError> for CliError {
    fn from(err: OperationError) -> Self {
        Self(err.to_string())
    }
}

impl From<InvalidGpxError> for CliError {
    fn from(err: InvalidGpxError) -> Self {
        Self(err.to_string())
    }
}

#[derive(Args, Default)]
pub struct EditOptions {
    #[arg(long)]
    pub min_lat: Option<f64>,
    #[arg(long)]
    pub max_lat: Option<f64>,
    #[arg(long)]
    pub min_lon: Option<f64>,
    #[arg(long)]
    pub max_lon: Option<f64>,
    #[arg(long)]
    pub start: Option<String>,
    #[arg(long)]
    pub end: Option<String>,
    #[arg(long)]
    pub split_time_gap: Option<f64>,
    #[arg(long)]
    pub split_distance_gap: Option<f64>,
    #[arg(long)]
    pub simplify: Option<f64>,
    #[arg(long)]
    pub smooth: Option<usize>,
    #[arg(long)]
    pub shift_time: Option<f64>,
    #[arg(long)]
    pub reverse: bool,
    #[arg(long)]
    pub reverse_routes: bool,
    #[arg(long)]
    pub reverse_tracks: bool,
    #[arg(long)]
    pub strip_name: bool,
    #[arg(long)]
    pub strip_desc: bool,
    #[arg(long)]
    pub strip_author: bool,
    #[arg(long)]
    pub strip_copyright: bool,
    #[arg(long)]
    pub strip_time: bool,
    #[arg(long)]
    pub strip_keywords: bool,
    #[arg(long)]
    pub strip_links: bool,
    #[arg(long)]
    pub strip_all_metadata: bool,
    #[arg(long)]
    pub strip_extensions: bool,
    #[arg(long)]
    pub precision: Option<u32>,
    #[arg(long)]
    pub elevation_precision: Option<u32>,
}

pub fn validate(path: &Path, strict: bool, json: bool) -> Result<u8, CliError> {
    if !path.exists() {
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "file": path.display().to_string(),
                    "error": "file not found"
                }))
                .unwrap()
            );
        } else {
            return Err(CliError(format!("File not found: {}", path.display())));
        }
        return Ok(1);
    }

    let result = validate_file(path)?;
    let failed = !result.is_valid() || (strict && !result.warnings().is_empty());

    if json {
        print_validate_json(path, &result, failed);
        return Ok(if failed { 1 } else { 0 });
    }

    print_validate_text(path, &result, strict, failed)
}

fn print_validate_json(path: &Path, result: &ValidationResult, failed: bool) {
    let report = json!({
        "file": path.display().to_string(),
        "valid": result.is_valid(),
        "passed": !failed,
        "errors": result.errors().len(),
        "warnings": result.warnings().len(),
        "issues": result.issues.iter().map(|i| i.to_json_value()).collect::<Vec<_>>(),
    });
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}

fn print_validate_text(
    path: &Path,
    result: &ValidationResult,
    strict: bool,
    failed: bool,
) -> Result<u8, CliError> {
    let n_errors = result.errors().len();
    let n_warnings = result.warnings().len();

    if !result.issues.is_empty() {
        let mark = if failed { "✗" } else { "⚠" };
        println!(
            "{mark} {}: {n_errors} error{}, {n_warnings} warning{}",
            path.display(),
            if n_errors == 1 { "" } else { "s" },
            if n_warnings == 1 { "" } else { "s" },
        );
        println!();
        for issue in &result.issues {
            println!("  {issue}");
        }
        println!();
    }

    if result.is_valid() {
        let gpx = read_gpx(path).map_err(CliError::from)?;
        if failed {
            println!(
                "✗ {}: failed because warnings are present (--strict)",
                path.display()
            );
        } else {
            println!("✓ Valid GPX file: {}", path.display());
        }
        println!("  Creator: {}", gpx.creator.as_deref().unwrap_or("(unknown)"));
        println!("  Waypoints: {}", gpx.waypoints.len());
        println!("  Routes: {}", gpx.routes.len());
        println!("  Tracks: {}", gpx.tracks.len());
    } else if strict {
        let _ = strict;
    }

    Ok(if failed { 1 } else { 0 })
}

pub fn info(path: &Path, json: bool, strict: bool) -> Result<u8, CliError> {
    if !path.exists() {
        return Err(CliError(format!("File not found: {}", path.display())));
    }

    if strict {
        validate_or_raise(path)?;
    }

    let gpx = read_gpx(path)?;
    let info = gather(&gpx);

    if json {
        println!("{}", serde_json::to_string_pretty(&info).unwrap());
    } else {
        print_human(path, &info);
    }

    Ok(0)
}

pub fn edit(
    input: &Path,
    output: &Path,
    strict: bool,
    options: EditOptions,
) -> Result<u8, CliError> {
    if !input.exists() {
        return Err(CliError(format!("File not found: {}", input.display())));
    }

    if strict {
        validate_or_raise(input)?;
    }

    let mut gpx = read_gpx(input)?;

    if options.min_lat.is_some()
        || options.max_lat.is_some()
        || options.min_lon.is_some()
        || options.max_lon.is_some()
    {
        gpx = crop(
            &gpx,
            options.min_lat,
            options.max_lat,
            options.min_lon,
            options.max_lon,
        );
    }

    if options.start.is_some() || options.end.is_some() {
        let start = options
            .start
            .as_deref()
            .map(parse_datetime)
            .transpose()?;
        let end = options.end.as_deref().map(parse_datetime).transpose()?;
        gpx = trim(&gpx, start, end);
    }

    if options.split_time_gap.is_some() || options.split_distance_gap.is_some() {
        let time_gap = options
            .split_time_gap
            .map(|secs| Duration::from_secs_f64(secs.max(0.0)));
        gpx = split(&gpx, time_gap, options.split_distance_gap)?;
    }

    if let Some(tolerance) = options.simplify {
        gpx = simplify(&gpx, tolerance)?;
    }

    if let Some(window) = options.smooth {
        gpx = smooth(&gpx, window)?;
    }

    if let Some(secs) = options.shift_time {
        gpx = shift_time(&gpx, Duration::from_secs_f64(secs));
    }

    if options.reverse {
        gpx = reverse(&gpx, true, true);
    } else {
        if options.reverse_routes {
            gpx = reverse(&gpx, true, false);
        }
        if options.reverse_tracks {
            gpx = reverse(&gpx, false, true);
        }
    }

    if options.strip_all_metadata
        || options.strip_name
        || options.strip_desc
        || options.strip_author
        || options.strip_copyright
        || options.strip_time
        || options.strip_keywords
        || options.strip_links
    {
        let fields = StripMetadataFields {
            strip_all: options.strip_all_metadata,
            name: options.strip_name,
            desc: options.strip_desc,
            author: options.strip_author,
            copyright: options.strip_copyright,
            time: options.strip_time,
            keywords: options.strip_keywords,
            links: options.strip_links,
        };
        gpx = strip_metadata(&gpx, fields);
    }

    if options.strip_extensions {
        gpx = strip_extensions(&gpx);
    }

    if options.precision.is_some() || options.elevation_precision.is_some() {
        gpx = reduce_precision(&gpx, options.precision, options.elevation_precision);
    }

    write_gpx_output(&gpx, output)?;
    Ok(0)
}

pub fn merge(inputs: &[PathBuf], output: &Path, strict: bool) -> Result<u8, CliError> {
    if inputs.is_empty() {
        return Err(CliError("At least one input file is required".to_string()));
    }

    let mut gpxs = Vec::with_capacity(inputs.len());
    for path in inputs {
        if !path.exists() {
            return Err(CliError(format!("File not found: {}", path.display())));
        }
        if strict {
            validate_or_raise(path)?;
        }
        gpxs.push(read_gpx(path)?);
    }

    let merged = merge_gpx(&gpxs);
    write_gpx_output(&merged, output)?;
    Ok(0)
}

pub fn convert(input: &Path, output: &Path, strict: bool) -> Result<u8, CliError> {
    if !input.exists() {
        return Err(CliError(format!("File not found: {}", input.display())));
    }

    let input_format = detect_format(input)
        .ok_or_else(|| CliError(format!("Could not detect input format for: {}", input.display())))?;

    if strict && input_format == "gpx" {
        validate_or_raise(input)?;
    }

    convert_file(input, output, None, None)?;
    Ok(0)
}

fn validate_or_raise(path: &Path) -> Result<(), CliError> {
    let result = validate_file(path)?;
    if !result.warnings().is_empty() {
        let n = result.warnings().len();
        eprintln!(
            "⚠ {}: {n} warning{}",
            path.display(),
            if n == 1 { "" } else { "s" }
        );
        for issue in result.warnings() {
            eprintln!("  {issue}");
        }
    }
    if !result.is_valid() {
        return Err(InvalidGpxError::new(result).into());
    }
    Ok(())
}

fn write_gpx_output(gpx: &Gpx, output: &Path) -> Result<(), CliError> {
    match detect_format(output) {
        Some("gpx") => write_file(gpx, output, true).map_err(CliError::from),
        Some(format) => {
            let temp = std::env::temp_dir().join(format!("gpx-rs-{}.gpx", std::process::id()));
            write_file(gpx, &temp, true)?;
            convert_file(&temp, output, Some("gpx"), Some(format))?;
            let _ = std::fs::remove_file(temp);
            Ok(())
        }
        None => write_file(gpx, output, true).map_err(CliError::from),
    }
}

fn parse_datetime(value: &str) -> Result<DateTime<Utc>, CliError> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Ok(dt.with_timezone(&Utc));
    }
    use chrono::NaiveDateTime;
    const FORMATS: &[&str] = &[
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%SZ",
    ];
    for format in FORMATS {
        if let Ok(naive) = NaiveDateTime::parse_from_str(value, format) {
            return Ok(naive.and_utc());
        }
    }
    Err(CliError(format!("invalid datetime '{value}'")))
}
