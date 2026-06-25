use std::fs;
use std::time::Duration;

use chrono::{TimeZone, Utc};
use gpx_rs::{Gpx, Waypoint, WaypointPath};

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

#[test]
fn haversine_distance_matches_geo_helper() {
    let a = waypoint(47.608013, -122.335167, None, None);
    let b = waypoint(47.609013, -122.335167, None, None);
    let path = WaypointPath::from_slice(&[a, b]);

    let distance = path.total_distance();
    assert!(distance > 110.0 && distance < 112.0);
}

#[test]
fn duration_and_average_speed_with_timestamps() {
    let points = vec![
        waypoint(0.0, 0.0, None, Some(t(10, 0, 0))),
        waypoint(0.0, 0.001, None, Some(t(10, 0, 10))),
        waypoint(0.0, 0.002, None, Some(t(10, 0, 20))),
    ];
    let path = WaypointPath::from_slice(&points);

    assert_eq!(path.duration(), Some(Duration::from_secs(20)));

    let avg = path.average_speed().expect("average speed");
    assert!((avg - path.total_distance() / 20.0).abs() < 1e-6);
}

#[test]
fn moving_duration_excludes_stopped_leg() {
    let points = vec![
        waypoint(0.0, 0.0, None, Some(t(10, 0, 0))),
        waypoint(0.0, 0.001, None, Some(t(10, 0, 10))),
        waypoint(0.0, 0.001, None, Some(t(10, 1, 50))),
        waypoint(0.0, 0.002, None, Some(t(10, 2, 0))),
    ];
    let path = WaypointPath::from_slice(&points);

    assert_eq!(path.moving_duration(), Some(Duration::from_secs(20)));

    let moving_avg = path.average_moving_speed().expect("moving average speed");
    assert!((moving_avg - path.total_distance() / 20.0).abs() < 1e-6);
}

#[test]
fn speed_profile_and_extrema() {
    let points = vec![
        waypoint(0.0, 0.0, None, Some(t(10, 0, 0))),
        waypoint(0.0, 0.001, None, Some(t(10, 0, 10))),
        waypoint(0.0, 0.003, None, Some(t(10, 0, 20))),
    ];
    let path = WaypointPath::from_slice(&points);

    let speeds = path.speeds();
    assert_eq!(speeds.len(), 2);
    assert!(speeds[0].unwrap() < speeds[1].unwrap());

    assert_eq!(path.min_speed(), speeds[0]);
    assert_eq!(path.max_speed(), speeds[1]);

    let profile = path.speed_profile();
    assert_eq!(profile.len(), 2);
    assert_eq!(profile[0].time, t(10, 0, 0));
    assert_eq!(profile[0].speed_mps, speeds[0].unwrap());
    assert_eq!(profile[1].time, t(10, 0, 10));
}

#[test]
fn elevation_metrics_match_ele_chained_semantics() {
    let points = vec![
        waypoint(0.0, 0.0, Some(100.0), None),
        waypoint(0.0, 0.001, Some(103.0), None),
        waypoint(0.0, 0.002, Some(100.5), None),
        waypoint(0.0, 0.003, Some(90.0), None),
    ];
    let path = WaypointPath::from_slice(&points);

    assert_eq!(path.average_elevation(), Some(98.375));
    assert_eq!(path.max_elevation(), Some(103.0));
    assert_eq!(path.min_elevation(), Some(90.0));
    assert_eq!(path.elevation_difference(), Some(13.0));
    assert_eq!(path.diff_elevation(), Some(13.0));
    assert_eq!(path.total_ascent(), Some(3.0));
    assert_eq!(path.total_descent(), Some(13.0));

    let profile = path.elevation_profile();
    assert_eq!(profile.len(), 4);
    assert_eq!(profile[0].distance_m, 0.0);
    assert_eq!(profile[0].value, 100.0);
    assert_eq!(profile.last().unwrap().value, 90.0);
}

#[test]
fn ascent_counts_gain_across_points_missing_elevation() {
    let points = vec![
        waypoint(0.0, 0.0, Some(100.0), None),
        waypoint(0.0, 0.001, None, None),
        waypoint(0.0, 0.002, Some(110.0), None),
    ];
    let path = WaypointPath::from_slice(&points);

    assert_eq!(path.total_ascent(), Some(10.0));
    assert_eq!(path.total_descent(), Some(0.0));
}

#[test]
fn missing_time_prevents_speed_and_duration_metrics() {
    let xml = fs::read_to_string("tests/fixtures/sample.gpx").expect("fixture should exist");
    let gpx = Gpx::parse(&xml).expect("fixture GPX should parse");
    let path = WaypointPath::from(&gpx.tracks[0].segments[0]);

    assert!(path.total_distance() > 0.0);
    assert_eq!(path.average_elevation(), Some(4.73));
    assert!(path.duration().is_none());
    assert!(path.moving_duration().is_none());
    assert!(path.average_speed().is_none());
    assert!(path.speeds().iter().all(|speed| speed.is_none()));
}

#[test]
fn from_track_returns_one_path_per_segment() {
    let xml = fs::read_to_string("tests/fixtures/sample.gpx").expect("fixture should exist");
    let gpx = Gpx::parse(&xml).expect("fixture GPX should parse");

    let paths = WaypointPath::from_track(&gpx.tracks[0]);
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].len(), 2);
}

#[test]
fn route_conversion_builds_connected_path() {
    let xml = fs::read_to_string("tests/fixtures/sample.gpx").expect("fixture should exist");
    let gpx = Gpx::parse(&xml).expect("fixture GPX should parse");
    let path = WaypointPath::from(&gpx.routes[0]);

    assert_eq!(path.len(), 2);
    assert!(path.total_distance() > 0.0);
}
