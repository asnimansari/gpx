.DEFAULT_GOAL := help
BIN := gpx-rs

.PHONY: help build release check test clippy fmt fmt-check run clean ci

help:
	@echo "Targets:"
	@echo "  make build      - debug build"
	@echo "  make release    - optimized build"
	@echo "  make check      - fast compile check"
	@echo "  make test       - run all tests"
	@echo "  make clippy     - lint with warnings as errors"
	@echo "  make fmt        - format code"
	@echo "  make fmt-check  - verify formatting"
	@echo "  make run        - run the $(BIN) CLI"
	@echo "  make clean      - remove build artifacts"
	@echo "  make ci         - check, fmt-check, clippy, and test"

build:
	cargo build

release:
	cargo build --release

check:
	cargo check

test:
	cargo test

clippy:
	cargo clippy --all-targets -- -D warnings

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

run:
	cargo run --bin $(BIN)

clean:
	cargo clean

ci: check fmt-check clippy test
