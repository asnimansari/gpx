# gpx-rs

A Rust library and CLI for reading, validating, editing, and converting GPX files (GPX 1.1).

## Installation

Install the CLI from crates.io:

```bash
cargo install gpx-rs
```

This installs the `gpx` binary. Build from source instead:

```bash
cargo install --path .
```

Or build locally without installing:

```bash
cargo build --release
# binary: target/release/gpx
```

## CLI

The command-line tool is named `gpx`.

```bash
gpx <COMMAND> [OPTIONS]
```

### `validate`

Validate a GPX file against the GPX 1.1 schema.

```bash
gpx validate <INPUT_FILE> [--strict] [--json]
```

| Flag | Description |
|------|-------------|
| `--strict` | Treat warnings as failures (non-zero exit code) |
| `--json` | Output the validation report as JSON |

**Examples**

```bash
gpx validate track.gpx
gpx validate track.gpx --strict
gpx validate track.gpx --json
```

On success, prints a summary with creator, waypoint/route/track counts, and any schema issues. Exit code `0` when valid (and no warnings if `--strict`); `1` otherwise.

---

### `info`

Show information and statistics about a GPX file (distance, duration, elevation, bounds).

```bash
gpx info <INPUT_FILE> [--json] [--strict]
```

| Flag | Description |
|------|-------------|
| `--json` | Output statistics as JSON |
| `--strict` | Validate against the GPX 1.1 schema before gathering info |

**Examples**

```bash
gpx info run.gpx
gpx info run.gpx --json
gpx info run.gpx --strict
```

---

### `edit`

Apply one or more transformations to a GPX file and write the result.

```bash
gpx edit <INPUT_FILE> -o <OUTPUT_FILE> [OPTIONS] [--strict]
```

| Flag | Description |
|------|-------------|
| `-o`, `--output-file` | Output file path (required) |
| `--strict` | Validate input against GPX 1.1 before editing |

**Crop** (bounding box filter; omit a bound to leave that side open):

| Flag | Description |
|------|-------------|
| `--min-lat <DEG>` | Minimum latitude |
| `--max-lat <DEG>` | Maximum latitude |
| `--min-lon <DEG>` | Minimum longitude |
| `--max-lon <DEG>` | Maximum longitude |

**Time trim** (ISO 8601 / RFC 3339 datetimes):

| Flag | Description |
|------|-------------|
| `--start <DATETIME>` | Drop points before this time |
| `--end <DATETIME>` | Drop points after this time |

**Split** (break tracks/routes at gaps):

| Flag | Description |
|------|-------------|
| `--split-time-gap <SECS>` | Split when consecutive points exceed this time gap |
| `--split-distance-gap <METERS>` | Split when consecutive points exceed this distance |

**Geometry**

| Flag | Description |
|------|-------------|
| `--simplify <TOLERANCE>` | Douglas–Peucker simplification tolerance (meters) |
| `--smooth <WINDOW>` | Moving-average smoothing window size (point count) |
| `--shift-time <SECS>` | Shift all timestamps by this many seconds |
| `--reverse` | Reverse all routes and tracks |
| `--reverse-routes` | Reverse routes only |
| `--reverse-tracks` | Reverse tracks only |

**Metadata stripping**

| Flag | Description |
|------|-------------|
| `--strip-all-metadata` | Remove all metadata fields |
| `--strip-name` | Remove name fields |
| `--strip-desc` | Remove description fields |
| `--strip-author` | Remove author |
| `--strip-copyright` | Remove copyright |
| `--strip-time` | Remove timestamps |
| `--strip-keywords` | Remove keywords |
| `--strip-links` | Remove links |
| `--strip-extensions` | Remove all extension elements |

**Precision**

| Flag | Description |
|------|-------------|
| `--precision <DIGITS>` | Decimal places for latitude/longitude |
| `--elevation-precision <DIGITS>` | Decimal places for elevation |

**Examples**

```bash
# Reverse track point order
gpx edit input.gpx -o reversed.gpx --reverse-tracks

# Crop to a bounding box and simplify
gpx edit input.gpx -o cropped.gpx --min-lat 42.4 --max-lat 42.5 --simplify 5.0

# Trim to a time range and strip extensions
gpx edit input.gpx -o trimmed.gpx --start 2024-01-01T08:00:00Z --end 2024-01-01T12:00:00Z --strip-extensions

# Reduce coordinate precision
gpx edit input.gpx -o rounded.gpx --precision 5 --elevation-precision 1
```

Multiple flags can be combined; transformations are applied in a fixed order (crop → trim → split → simplify → smooth → shift time → reverse → strip metadata → strip extensions → reduce precision).

---

### `merge`

Merge multiple GPX files into one.

```bash
gpx merge <INPUT_FILES>... -o <OUTPUT_FILE> [--strict]
```

| Flag | Description |
|------|-------------|
| `-o`, `--output-file` | Output file path (required) |
| `--strict` | Validate each input against GPX 1.1 before merging |

**Examples**

```bash
gpx merge part1.gpx part2.gpx part3.gpx -o combined.gpx
gpx merge *.gpx -o all.gpx --strict
```

Waypoints, routes, and tracks from all inputs are concatenated into a single document.

---

### `convert`

Convert between GPX, GeoJSON, and KML. Format is detected from the file extension (`.gpx`, `.geojson`, `.json`, `.kml`).

```bash
gpx convert <INPUT_FILE> -o <OUTPUT_FILE> [--strict]
```

| Flag | Description |
|------|-------------|
| `-o`, `--output-file` | Output file path (required) |
| `--strict` | Validate GPX input against GPX 1.1 before converting |

**Examples**

```bash
gpx convert track.gpx -o track.geojson
gpx convert route.geojson -o route.gpx
gpx convert track.gpx -o map.kml
gpx convert map.kml -o track.gpx
```

---

## Library

The library crate is published as `gpx_rs` on crates.io.

```toml
[dependencies]
gpx-rs = "0.1"
```

### Parse a GPX file

```rust
use gpx_rs::{parse, parse_file, Gpx};

// From a file
let gpx = parse_file("track.gpx")?;

// From XML text
let gpx = Gpx::parse(&xml)?;
let gpx = parse(&xml)?; // same as Gpx::parse
```

### Path statistics

Use `WaypointPath` for distance, duration, speed, and elevation on a connected sequence of points:

```rust
use gpx_rs::{parse_file, WaypointPath};

let gpx = parse_file("track.gpx")?;

// From a track segment, route, or slice of waypoints
let path = WaypointPath::from(&gpx.tracks[0].segments[0]);
// let path = WaypointPath::from(&gpx.routes[0]);
// let path = WaypointPath::from_slice(&gpx.waypoints);

println!("distance: {:.1} m", path.total_distance());
println!("duration: {:?}", path.duration());           // needs timestamps
println!("avg speed: {:.2} m/s", path.average_speed().unwrap_or(0.0));
println!("ascent: {:.1} m", path.total_ascent());
println!("descent: {:.1} m", path.total_descent());
```

Statistics are also available directly on `Route`, `TrackSegment`, and `Track`:

```rust
let route = &gpx.routes[0];
println!("route distance: {:.1} m", route.total_distance());
```

### Strava / Garmin extensions

```rust
use gpx_rs::parse_file;

let gpx = parse_file("run.gpx")?;
let point = &gpx.tracks[0].segments[0].points[0];

println!("hr: {:?}", point.heart_rate());
println!("cadence: {:?}", point.cadence());
println!("power: {:?} W", point.power_watts());
```

### Validate, edit, and write

```rust
use gpx_rs::{parse_file, simplify, to_string, validate_file, write_file};

let gpx = parse_file("track.gpx")?;

// Schema validation
let result = validate_file("track.gpx")?;
if !result.is_valid() {
    for issue in &result.issues {
        eprintln!("{issue}");
    }
}

// Transform and write
let simplified = simplify(&gpx, 5.0)?;
write_file(&simplified, "out.gpx", true)?;

// Or serialize to a string
let xml = to_string(&simplified, true);
```

### Convert formats

```rust
use gpx_rs::convert_file;

convert_file("track.gpx", "track.geojson", None, None)?;
convert_file("route.geojson", "route.gpx", None, None)?;
```

See [AGENTS.md](AGENTS.md) for full API details, statistics semantics, and extension support.

## Development

```bash
cargo build
cargo test
cargo clippy
cargo fmt
```

Run the CLI during development:

```bash
cargo run -- info tests/fixtures/sample.gpx
cargo run -- validate tests/fixtures/sample.gpx
```
