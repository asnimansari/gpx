# gpx-rs

A Rust library and CLI for manipulating GPX files.

## Project layout

```
gpx-rs/
├── Cargo.toml      # Package manifest (library + binary)
├── src/
│   ├── lib.rs      # Library crate (`gpx_rs`)
│   └── main.rs     # CLI binary (`gpx-rs`)
└── README.md
```

## Library

The library crate is published as `gpx_rs`. Add it as a dependency in another Rust project:

```toml
gpx_rs = { path = "path/to/gpx-rs" }
```

## CLI

Build and run the command-line tool:

```bash
cargo build
cargo run --bin gpx-rs
```

## Development

```bash
cargo build
cargo test
cargo clippy
```
