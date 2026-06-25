use std::fs;

use chrono::TimeZone;
use gpx_rs::{Gpx, ParseError};

#[test]
fn parses_fixture_file() {
    let xml = fs::read_to_string("tests/fixtures/sample.gpx").expect("fixture should exist");
    let gpx = Gpx::parse(&xml).expect("fixture GPX should parse");

    assert_eq!(gpx.version.as_deref(), Some("1.1"));
    assert_eq!(gpx.waypoints.len(), 1);
    assert_eq!(gpx.routes.len(), 1);
    assert_eq!(gpx.tracks.len(), 1);
}

#[test]
fn parses_waypoint_fields_from_fixture() {
    let xml = fs::read_to_string("tests/fixtures/sample.gpx").expect("fixture should exist");
    let gpx = Gpx::parse(&xml).expect("fixture GPX should parse");

    let waypoint = &gpx.waypoints[0];
    assert_eq!(waypoint.lat, 47.608013);
    assert_eq!(waypoint.lon, -122.335167);
    assert_eq!(waypoint.ele, Some(4.46));
    assert_eq!(
        waypoint.time,
        Some(chrono::Utc.with_ymd_and_hms(2002, 5, 30, 9, 0, 10).unwrap())
    );
    assert_eq!(waypoint.name.as_deref(), Some("Waypoint 1"));
}

#[test]
fn parses_route_and_track_from_fixture() {
    let xml = fs::read_to_string("tests/fixtures/sample.gpx").expect("fixture should exist");
    let gpx = Gpx::parse(&xml).expect("fixture GPX should parse");

    let route = &gpx.routes[0];
    assert_eq!(route.name.as_deref(), Some("Example Route"));
    assert_eq!(route.points.len(), 2);
    assert_eq!(route.points[1].ele, Some(5.0));

    let track = &gpx.tracks[0];
    assert_eq!(track.name.as_deref(), Some("Example Track"));
    assert_eq!(track.segments.len(), 1);
    assert_eq!(track.segments[0].points.len(), 2);
}

#[test]
fn public_parse_matches_gpx_parse() {
    let xml = fs::read_to_string("tests/fixtures/sample.gpx").expect("fixture should exist");

    let from_fn = gpx_rs::parse(&xml).expect("parse should succeed");
    let from_method = Gpx::parse(&xml).expect("Gpx::parse should succeed");

    assert_eq!(from_fn, from_method);
}

#[test]
fn rejects_invalid_latitude() {
    let xml = r#"<gpx version="1.1" xmlns="http://www.topografix.com/GPX/1/1">
  <wpt lat="north" lon="-122.335167"/>
</gpx>"#;

    let err = Gpx::parse(xml).unwrap_err();
    assert!(matches!(
        err,
        ParseError::De(_) | ParseError::InvalidAttribute {
            element: "wpt",
            attribute: "lat",
            ..
        }
    ));
}
