# AGENTS.md

This file provides guidance for AI assistants working with the **gpx-rs** codebase.

## Project Overview

gpx-rs is a Rust library and CLI for reading, writing, validating, editing, and analyzing GPX (GPS Exchange Format) files. It targets **GPX 1.1** and exposes typed structs that mirror the schema, plus path analytics, format conversion, and schema validation.

- **Crate name**: `gpx_rs` (library), `gpx-rs` (package), `gpx` (binary)
- **Current version**: 0.1.0
- **Edition**: 2021
- **Schema reference**: [GPX 1.1](https://www.topografix.com/GPX/1/1/)

## Repository Structure

```
gpx-rs/
├── Cargo.toml              # Package manifest (library + binary)
├── AGENTS.md               # AI assistant guidance (this file)
├── README.md
├── src/
│   ├── lib.rs              # Public API re-exports
│   ├── main.rs             # CLI entry (`gpx` binary)
│   ├── cli/                # CLI commands (clap)
│   │   ├── mod.rs
│   │   └── commands.rs
│   └── gpx/
│       ├── mod.rs          # Module wiring, Gpx::parse / parse_file
│       ├── error.rs        # ParseError
│       ├── geo.rs          # Haversine distance helpers
│       ├── info.rs         # GpxInfo gathering for `gpx info`
│       ├── parse/          # XML parsing (quick-xml + serde)
│       │   └── mod.rs
│       ├── serialize/      # GPX XML writer
│       │   └── mod.rs
│       ├── validation/     # GPX 1.1 schema validation
│       │   └── mod.rs
│       ├── convert/        # GPX ↔ GeoJSON ↔ KML
│       │   └── mod.rs
│       ├── operations/     # Edit/merge transforms (crop, trim, …)
│       │   └── mod.rs
│       ├── analysis/       # Path statistics
│       │   ├── mod.rs
│       │   ├── waypoint_path.rs
│       │   ├── statistics.rs   # impl on Route, TrackSegment, Track
│       │   ├── options.rs
│       │   └── profile.rs
│       └── types/          # GPX 1.1 data types (one file per element)
│           ├── gpx.rs
│           ├── waypoint.rs
│           ├── track.rs
│           ├── route.rs
│           ├── metadata.rs
│           └── ...
└── tests/
    ├── parse_gpx.rs
    ├── waypoint_path.rs
    ├── statistics.rs
    ├── cli.rs
    └── fixtures/
        └── sample.gpx
```

## Key Dependencies

- **quick-xml**: XML deserialization and low-level XML reading in validation/convert
- **serde** / **serde_json**: GPX type mapping and GeoJSON I/O
- **chrono**: `DateTime<Utc>` for GPX `<time>`; used in analytics and edit ops
- **clap**: CLI argument parsing (binary only)

No geo crate — haversine is implemented in `src/gpx/geo.rs`.

## Development Setup

```bash
cargo build
cargo test
cargo clippy
cargo fmt
```

Build and run the CLI:

```bash
cargo build --bin gpx
cargo run --bin gpx -- info tests/fixtures/sample.gpx
```

Run a single test file:

```bash
cargo test --test waypoint_path
cargo test --test parse_gpx
cargo test --test cli
```

## Code Architecture

### Type Model

GPX elements are plain `Deserialize` structs under `src/gpx/types/`, re-exported from `gpx_rs`:

```
Gpx
├── waypoints: Vec<Waypoint>     # standalone <wpt>
├── routes: Vec<Route>           # each has points: Vec<Waypoint>
├── tracks: Vec<Track>           # each has segments: Vec<TrackSegment>
└── metadata, extensions, …

TrackSegment / Route
└── points: Vec<Waypoint>        # ordered path vertices

Waypoint
├── lat, lon: f64                # required (WGS84 decimal degrees)
├── ele: Option<f64>             # meters
├── time: Option<DateTime<Utc>>  # ISO 8601 UTC
└── name, cmt, desc, links, fix, …
```

Rust uses friendly field names (`waypoints`, `routes`, `tracks`, `points`, `segments`) rather than GPX shorthand (`wpt`, `rte`, `trk`). Access metadata via `gpx.metadata` — there are no proxy accessors on `Gpx`.

### Parsing

- Entry points: `gpx_rs::parse(&str)`, `Gpx::parse(&str)`, `Gpx::parse_file(path)` → `Result<Gpx, ParseError>`
- Primary path: `quick_xml::de::from_str` into `Gpx`
- Fallback: strip default GPX namespace and retry (`parse/mod.rs`)
- Types use `#[serde(rename = ...)]` for XML names (`@lat`, `trkpt`, `type` → `waypoint_type`, etc.)

### Serialization

GPX write is implemented in `src/gpx/serialize/mod.rs` (manual XML writer, not serde `Serialize`):

```rust
use gpx_rs::{to_string, write_file};

let xml = to_string(&gpx, true);
write_file(&gpx, "out.gpx", true)?;
```

`Extensions::inner_xml` is written through verbatim when present.

### Validation

GPX 1.1 schema validation (`src/gpx/validation/mod.rs`) uses a declarative rule table (no XSD engine). Validates raw XML before/independent of dataclass parsing:

```rust
use gpx_rs::{validate_str, validate_file, ValidationResult};

let result = validate_file("track.gpx")?;
assert!(result.is_valid());  // no errors; warnings allowed
```

`InvalidGpxError` is raised by CLI `--strict` when schema errors are found.

### Analytics (`WaypointPath`)

Core statistics live on `WaypointPath` — a connected sequence of waypoints:

```rust
use gpx_rs::{Gpx, WaypointPath};

let gpx = Gpx::parse(xml)?;
let path = WaypointPath::from(&gpx.routes[0]);

path.total_distance();       // meters (haversine, 2D)
path.duration();             // Option<Duration> — needs timestamps
path.moving_duration();      // legs above moving threshold
path.average_speed();        // m/s
path.speeds();               // per-leg speeds
path.speed_profile();        // Vec<SpeedProfilePoint>
path.total_ascent();         // meters
path.elevation_profile();    // Vec<ProfilePoint>
```

Constructors:

- `WaypointPath::from_slice(&[Waypoint])`
- `WaypointPath::from(&TrackSegment)` / `from(&Route)`
- `WaypointPath::from_track(&Track)` → `Vec<WaypointPath>` (one per segment; **do not** bridge segment gaps)

### Container Statistics (`Route`, `TrackSegment`, `Track`)

`src/gpx/analysis/statistics.rs` adds convenience methods on `Route`, `TrackSegment`, and `Track` that delegate to `WaypointPath` (or aggregate across segments for `Track`). Canonical names match [sgraaf/gpx](https://github.com/sgraaf/gpx) — no aliases (`distance`, `speed`, `elevation` proxies were removed upstream):

- `total_distance`, `total_duration`, `moving_duration`
- `average_speed`, `average_moving_speed`, `max_speed`, `min_speed`
- `average_elevation`, `max_elevation`, `min_elevation`, `diff_elevation`
- `total_ascent`, `total_descent`, `speed_profile`, `elevation_profile`, `bounds()`
- `Index` / `IntoIterator` on containers

### Operations

Pure edit functions in `src/gpx/operations/mod.rs` clone input and return a new `Gpx`:

`crop`, `trim`, `reverse`, `split`, `simplify`, `smooth`, `shift_time`, `reduce_precision`, `strip_metadata`, `strip_extensions`, `merge`, `filter_points`

Invalid arguments return `OperationError` (e.g. split without gap, non-positive simplify tolerance, invalid smooth window).

### Format Conversion

`src/gpx/convert/mod.rs` supports GPX, GeoJSON, and KML:

```rust
use gpx_rs::{detect_format, convert_file, read_geojson, write_kml};

convert_file("in.gpx", "out.geojson", None, None)?;
```

GeoJSON uses a `FeatureCollection` with `gpx_type` hints on features. KML reads/writes Placemarks (Point → waypoint, LineString → route, MultiGeometry → track segments).

### CLI

The `gpx` binary (`src/cli/`) implements:

| Command | Purpose |
|---------|---------|
| `validate <file>` | Schema validation; `--strict`, `--json` |
| `info <file>` | File summary and statistics; `--json`, `--strict` |
| `edit <file> -o <out>` | Transforms (crop, trim, reverse, simplify, …); `--strict` |
| `merge <files…> -o <out>` | Concatenate GPX files; `--strict` |
| `convert <in> -o <out>` | Format conversion; `--strict` (GPX input) |

`--strict` on non-validate commands prints warnings to stderr and aborts on schema errors before processing.

### Design Principles

1. **Parse types stay dumb** — serde structs in `types/`; logic lives in dedicated modules (`parse/`, `analysis/`, `operations/`, etc.).
2. **Option for missing data** — time/elevation analytics return `Option` when GPX data is incomplete.
3. **Segment gaps** — multi-segment tracks represent GPS dropouts; never auto-join segments for distance/speed.
4. **Minimize dependencies** — prefer small inline helpers (e.g. haversine) over new crates unless justified.
5. **Pure operations** — edit functions never mutate input; return new `Gpx` values.

## Code Style and Conventions

### Rust Idioms

- `snake_case` functions/fields, `PascalCase` types
- GPX XML names via serde `rename`; avoid Rust keywords (`waypoint_type` not `type`)
- Preserve GPX shorthand in field names: `ele`, `cmt`, `desc`, `sym`
- Doc comments on public items; module docs in `types/mod.rs`
- Prefer focused diffs — don't refactor unrelated code

### Formatting and Linting

- `cargo fmt` for formatting
- `cargo clippy` before submitting changes
- Keep public API changes reflected in `src/lib.rs` re-exports

### Naming: Rust vs GPX

| GPX XML | Rust field |
|---------|------------|
| `@lat`, `@lon` | `lat`, `lon` |
| `<ele>` | `ele: Option<f64>` |
| `<time>` | `time: Option<DateTime<Utc>>` |
| `<type>` | `waypoint_type`, `track_type`, `route_type` |
| `<trkpt>` | `TrackSegment::points` |
| `<rtept>` | `Route::points` |
| `<trkseg>` | `Track::segments` |
| `<wpt>` / `<rte>` / `<trk>` | `waypoints` / `routes` / `tracks` |

## Testing

- **Unit tests**: inline in `parse/mod.rs`, `validation/mod.rs`, `convert/mod.rs`
- **Integration tests**: `tests/parse_gpx.rs`, `tests/waypoint_path.rs`, `tests/statistics.rs`, `tests/cli.rs`
- **Fixture**: `tests/fixtures/sample.gpx` — has `ele` on track points but **no** `time` (distance/elevation work; speed/duration return `None`)

When adding analytics, use synthetic waypoints with known lat/lon/time/ele in tests. CLI tests use `assert_cmd` against the `gpx` binary.

## Common Tasks

### Adding a New GPX Type

1. Create `src/gpx/types/<name>.rs` with `Deserialize` struct and serde renames
2. Register in `src/gpx/types/mod.rs`
3. Re-export from `src/gpx/mod.rs` and `src/lib.rs`
4. Add parse coverage in `tests/parse_gpx.rs` or fixture XML
5. Update `serialize/mod.rs` and `validation/mod.rs` SCHEMA if the type appears in XML output/validation

### Adding a Path Statistic

1. Implement on `WaypointPath` in `src/gpx/analysis/waypoint_path.rs`
2. Add delegation in `src/gpx/analysis/statistics.rs` for `Route` / `TrackSegment` / `Track` if appropriate
3. Document units (meters, m/s, UTC timestamps)
4. Add tests in `tests/waypoint_path.rs`

### Adding an Edit Operation

1. Implement as a pure function in `src/gpx/operations/mod.rs`
2. Wire CLI flag in `src/cli/commands.rs` (`EditOptions` + `edit()` handler)
3. Re-export from `src/gpx/mod.rs` and `src/lib.rs`
4. Add unit test in `operations/mod.rs` or CLI test in `tests/cli.rs`

### Adding a CLI Command

1. Add subcommand to `src/cli/mod.rs` (`Command` enum)
2. Implement handler in `src/cli/commands.rs`
3. Add integration test in `tests/cli.rs`

## Statistics Semantics

Align with common GPX tooling (and sgraaf/gpx reference behavior):

| Metric | Behavior |
|--------|----------|
| Distance | Haversine on lat/lon; earth radius `6_378_137` m |
| Duration | First point `time` → last point `time` |
| Moving duration | Sum leg durations where speed **>** `0.5 / 3.6` m/s (0.5 km/h) |
| `elevation_difference` / `diff_elevation` | `max_elevation - min_elevation` |
| Ascent / descent | Gains between consecutive **ele-known** points (skip gaps) |
| Elevation profile | Distance accumulated only between consecutive ele-known points |
| Speed profile | `(DateTime<Utc>, speed_mps)` at leg start |

Configurable via `AnalysisOptions` (`with_options`).

Track-level stats sum per-segment durations/distances; elevation profile accumulates distance continuously across segments.

## Important Notes

- **GPX 1.1 only** — no GPX 1.0 or vendor extension schemas beyond opaque `Extensions`
- **Coordinates**: WGS84 decimal degrees
- **Elevations**: meters above sea level
- **Timestamps**: UTC, ISO 8601
- **Types are `Deserialize` only** — GPX output uses the dedicated `serialize/` writer, not serde `Serialize`
- **CLI and library share logic** — CLI calls library functions; keep business logic in `gpx/`, not `cli/`

## Reference

For comparable Python GPX API and semantics, see [sgraaf/gpx](https://github.com/sgraaf/gpx). gpx-rs uses `WaypointPath` for core analytics and delegates container methods from `Route` / `TrackSegment` / `Track`. Validation, operations, and CLI design follow the same project.
