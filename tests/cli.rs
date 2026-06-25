use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn validate_passes_fixture() {
    cargo_bin_cmd!("gpx")
        .args(["validate", "tests/fixtures/sample.gpx"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Valid GPX file"));
}

#[test]
fn validate_json_output() {
    cargo_bin_cmd!("gpx")
        .args(["validate", "--json", "tests/fixtures/sample.gpx"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"valid\": true"));
}

#[test]
fn info_prints_contents() {
    cargo_bin_cmd!("gpx")
        .args(["info", "tests/fixtures/sample.gpx"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Example Track"));
}

#[test]
fn convert_gpx_to_geojson() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("out.geojson");

    cargo_bin_cmd!("gpx")
        .args([
            "convert",
            "tests/fixtures/sample.gpx",
            "-o",
            output.to_str().unwrap(),
        ])
        .assert()
        .success();

    let content = std::fs::read_to_string(output).unwrap();
    assert!(content.contains("FeatureCollection"));
}

#[test]
fn edit_reverse_tracks() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("reversed.gpx");

    cargo_bin_cmd!("gpx")
        .args([
            "edit",
            "tests/fixtures/sample.gpx",
            "-o",
            output.to_str().unwrap(),
            "--reverse-tracks",
        ])
        .assert()
        .success();

    let gpx = gpx_rs::Gpx::parse_file(&output).unwrap();
    let first = gpx.tracks[0].segments[0].points[0].lat;
    let last = gpx
        .tracks[0]
        .segments[0]
        .points
        .last()
        .unwrap()
        .lat;
    assert!((first - 47.609123).abs() < 1e-6);
    assert!((last - 47.608013).abs() < 1e-6);
}

#[test]
fn merge_two_files() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("merged.gpx");

    cargo_bin_cmd!("gpx")
        .args([
            "merge",
            "tests/fixtures/sample.gpx",
            "tests/fixtures/sample.gpx",
            "-o",
            output.to_str().unwrap(),
        ])
        .assert()
        .success();

    let gpx = gpx_rs::Gpx::parse_file(&output).unwrap();
    assert_eq!(gpx.waypoints.len(), 2);
    assert_eq!(gpx.tracks.len(), 2);
}
