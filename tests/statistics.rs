use std::fs;
use std::time::Duration;

use chrono::{TimeZone, Utc};
use gpx_rs::{Gpx, Route, Track, TrackSegment, Waypoint};

fn waypoint(lat: f64, lon: f64, ele: Option<f64>, time: Option<chrono::DateTime<Utc>>) -> Waypoint {
    Waypoint {
        lat,
        lon,
        ele,
        time,
        magvar: None,
        geoidheight: None,
        name: None,
        cmt: None,
        desc: None,
        src: None,
        links: vec![],
        sym: None,
        waypoint_type: None,
        fix: None,
        sat: None,
        hdop: None,
        vdop: None,
        pdop: None,
        ageofdgpsdata: None,
        dgpsid: None,
        extensions: None,
    }
}

fn t(h: u32, m: u32, s: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2024, 1, 1, h, m, s).unwrap()
}

fn route_from_points(points: Vec<Waypoint>) -> Route {
    Route {
        name: None,
        cmt: None,
        desc: None,
        src: None,
        links: vec![],
        number: None,
        route_type: None,
        extensions: None,
        points,
    }
}

fn segment_from_points(points: Vec<Waypoint>) -> TrackSegment {
    TrackSegment {
        points,
        extensions: None,
    }
}

#[test]
fn route_statistics_match_waypoint_path() {
    let points = vec![
        waypoint(0.0, 0.0, Some(100.0), Some(t(10, 0, 0))),
        waypoint(0.0, 0.001, Some(103.0), Some(t(10, 0, 10))),
        waypoint(0.0, 0.002, Some(100.5), Some(t(10, 0, 20))),
    ];
    let route = route_from_points(points);

    assert_eq!(route.len(), 3);
    assert_eq!(route[0].lat, 0.0);
    assert_eq!(route.distance(), route.total_distance());
    assert_eq!(route.duration(), Some(Duration::from_secs(20)));
    assert_eq!(route.total_duration(), route.duration());
    assert_eq!(route.total_ascent(), Some(3.0));
    assert_eq!(route.diff_elevation(), Some(3.0));
    assert_eq!(route.elevation(), route.average_elevation());
    assert_eq!(route.speed(), route.average_speed());
    assert!(route.bounds().is_some());
    assert_eq!(route.speed_profile().len(), 2);
    assert_eq!(route.elevation_profile().len(), 3);
}

#[test]
fn track_segment_supports_indexing_and_iteration() {
    let segment = segment_from_points(vec![
        waypoint(1.0, 2.0, None, None),
        waypoint(1.1, 2.1, None, None),
    ]);

    assert_eq!(segment.len(), 2);
    let collected: Vec<_> = (&segment).into_iter().collect();
    assert_eq!(collected.len(), 2);
    assert!(segment.total_distance() > 0.0);
}

#[test]
fn track_aggregates_segment_statistics() {
    let track = Track {
        name: None,
        cmt: None,
        desc: None,
        src: None,
        links: vec![],
        number: None,
        track_type: None,
        extensions: None,
        segments: vec![
            segment_from_points(vec![
                waypoint(0.0, 0.0, Some(100.0), Some(t(10, 0, 0))),
                waypoint(0.0, 0.001, Some(110.0), Some(t(10, 0, 10))),
            ]),
            segment_from_points(vec![
                waypoint(0.0, 0.002, Some(105.0), Some(t(10, 1, 0))),
                waypoint(0.0, 0.003, Some(95.0), Some(t(10, 1, 10))),
            ]),
        ],
    };

    assert_eq!(track.len(), 2);
    assert_eq!(track[0].len(), 2);
    assert!(track.total_distance() > 0.0);
    assert_eq!(track.duration(), Some(Duration::from_secs(20)));
    assert_eq!(track.total_ascent(), Some(10.0));
    assert_eq!(track.total_descent(), Some(10.0));
    assert_eq!(track.elevation_profile().len(), 3);
    assert!(track.bounds().is_some());
}

#[test]
fn gpx_metadata_accessors_and_parse_file() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<gpx version="1.1" creator="gpx-rs">
  <metadata>
    <name>Sample GPX</name>
    <desc>A sample GPX file for testing</desc>
    <bounds minlat="47.0" minlon="-123.0" maxlat="48.0" maxlon="-122.0"/>
  </metadata>
</gpx>"#;
    let gpx = Gpx::parse(xml).expect("GPX should parse");

    assert_eq!(gpx.name(), Some("Sample GPX"));
    assert_eq!(gpx.desc(), Some("A sample GPX file for testing"));
    assert!(gpx.bounds().is_some());

    let path = std::env::temp_dir().join("gpx-rs-statistics-test.gpx");
    fs::write(&path, xml).expect("write temp gpx");
    let from_file = Gpx::parse_file(&path).expect("parse_file should work");
    assert_eq!(from_file.name(), gpx.name());
}
