use std::collections::HashMap;

use serde::Serialize;

use crate::gpx::types::{Gpx, Route, Track};

/// Summary information about a GPX document for the `gpx info` command.
#[derive(Debug, Clone, Serialize)]
pub struct GpxInfo {
    pub creator: Option<String>,
    pub waypoints: usize,
    pub routes: usize,
    pub tracks: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track_statistics: Option<Vec<TrackInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_statistics: Option<Vec<RouteInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounds: Option<BoundsInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_distance_m: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_duration_s: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrackInfo {
    pub index: usize,
    pub name: Option<String>,
    pub segments: usize,
    pub points: usize,
    pub distance_m: f64,
    pub duration_s: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_speed_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_speed_kmh: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation: Option<ElevationInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RouteInfo {
    pub index: usize,
    pub name: Option<String>,
    pub points: usize,
    pub distance_m: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation: Option<ElevationInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ElevationInfo {
    pub min_m: f64,
    pub max_m: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_m: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_ascent_m: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_descent_m: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BoundsInfo {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
}

/// Gather structured information and statistics from a parsed GPX document.
pub fn gather(gpx: &Gpx) -> GpxInfo {
    let mut info = GpxInfo {
        creator: gpx.creator.clone(),
        waypoints: gpx.waypoints.len(),
        routes: gpx.routes.len(),
        tracks: gpx.tracks.len(),
        metadata: None,
        track_statistics: None,
        route_statistics: None,
        bounds: None,
        total_distance_m: None,
        total_duration_s: None,
    };

    if let Some(metadata) = &gpx.metadata {
        let mut meta = HashMap::new();
        if let Some(name) = &metadata.name {
            meta.insert("name".to_string(), name.clone());
        }
        if let Some(desc) = &metadata.desc {
            meta.insert("description".to_string(), desc.clone());
        }
        if let Some(author) = &metadata.author {
            if let Some(name) = &author.name {
                meta.insert("author".to_string(), name.clone());
            }
        }
        if let Some(time) = metadata.time {
            meta.insert("time".to_string(), time.to_rfc3339());
        }
        if let Some(keywords) = &metadata.keywords {
            meta.insert("keywords".to_string(), keywords.clone());
        }
        if !meta.is_empty() {
            info.metadata = Some(meta);
        }
    }

    if !gpx.tracks.is_empty() {
        let track_stats: Vec<TrackInfo> = gpx
            .tracks
            .iter()
            .enumerate()
            .map(|(index, track)| gather_track_info(track, index))
            .collect();
        info.total_distance_m = Some(track_stats.iter().map(|t| t.distance_m).sum());
        let total_duration: f64 = track_stats.iter().map(|t| t.duration_s).sum();
        if total_duration > 0.0 {
            info.total_duration_s = Some(total_duration);
        }
        info.track_statistics = Some(track_stats);
    }

    if !gpx.routes.is_empty() {
        info.route_statistics = Some(
            gpx.routes
                .iter()
                .enumerate()
                .map(|(index, route)| gather_route_info(route, index))
                .collect(),
        );
    }

    if !gpx.waypoints.is_empty() || !gpx.routes.is_empty() || !gpx.tracks.is_empty() {
        info.bounds = calculate_bounds(gpx);
    }

    info
}

fn gather_track_info(track: &Track, index: usize) -> TrackInfo {
    let points: usize = track.segments.iter().map(|s| s.points.len()).sum();
    let duration_s = track
        .total_duration()
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
    let distance_m = track.total_distance();
    let avg_speed_ms = track.average_speed();
    let avg_speed_kmh = avg_speed_ms.map(|s| s * 3.6);

    TrackInfo {
        index,
        name: track.name.clone(),
        segments: track.segments.len(),
        points,
        distance_m,
        duration_s,
        avg_speed_ms,
        avg_speed_kmh,
        elevation: elevation_info(
            track.min_elevation(),
            track.max_elevation(),
            track.average_elevation(),
            track.total_ascent(),
            track.total_descent(),
        ),
    }
}

fn gather_route_info(route: &Route, index: usize) -> RouteInfo {
    RouteInfo {
        index,
        name: route.name.clone(),
        points: route.points.len(),
        distance_m: route.total_distance(),
        elevation: elevation_info(
            route.min_elevation(),
            route.max_elevation(),
            route.average_elevation(),
            route.total_ascent(),
            route.total_descent(),
        ),
    }
}

fn elevation_info(
    min: Option<f64>,
    max: Option<f64>,
    avg: Option<f64>,
    ascent: Option<f64>,
    descent: Option<f64>,
) -> Option<ElevationInfo> {
    Some(ElevationInfo {
        min_m: min?,
        max_m: max?,
        avg_m: avg,
        total_ascent_m: ascent,
        total_descent_m: descent,
    })
}

fn calculate_bounds(gpx: &Gpx) -> Option<BoundsInfo> {
    let mut min_lat = f64::INFINITY;
    let mut max_lat = f64::NEG_INFINITY;
    let mut min_lon = f64::INFINITY;
    let mut max_lon = f64::NEG_INFINITY;
    let mut has_points = false;

    for point in gpx.waypoints.iter() {
        has_points = true;
        min_lat = min_lat.min(point.lat);
        max_lat = max_lat.max(point.lat);
        min_lon = min_lon.min(point.lon);
        max_lon = max_lon.max(point.lon);
    }
    for route in &gpx.routes {
        for point in &route.points {
            has_points = true;
            min_lat = min_lat.min(point.lat);
            max_lat = max_lat.max(point.lat);
            min_lon = min_lon.min(point.lon);
            max_lon = max_lon.max(point.lon);
        }
    }
    for track in &gpx.tracks {
        if let Some(bounds) = track.bounds() {
            has_points = true;
            min_lat = min_lat.min(bounds.minlat);
            max_lat = max_lat.max(bounds.maxlat);
            min_lon = min_lon.min(bounds.minlon);
            max_lon = max_lon.max(bounds.maxlon);
        }
    }

    if has_points {
        Some(BoundsInfo {
            min_lat,
            max_lat,
            min_lon,
            max_lon,
        })
    } else {
        None
    }
}

pub fn print_human(path: &std::path::Path, info: &GpxInfo) {
    use std::io::Write;

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    let _ = writeln!(out, "GPX File: {}", path.display());
    let _ = writeln!(
        out,
        "Creator: {}",
        info.creator.as_deref().unwrap_or("(unknown)")
    );
    let _ = writeln!(out);

    if let Some(metadata) = &info.metadata {
        let _ = writeln!(out, "Metadata:");
        if let Some(name) = metadata.get("name") {
            let _ = writeln!(out, "  Name: {name}");
        }
        if let Some(desc) = metadata.get("description") {
            let _ = writeln!(out, "  Description: {desc}");
        }
        if let Some(author) = metadata.get("author") {
            let _ = writeln!(out, "  Author: {author}");
        }
        if let Some(time) = metadata.get("time") {
            let _ = writeln!(out, "  Time: {time}");
        }
        if let Some(keywords) = metadata.get("keywords") {
            let _ = writeln!(out, "  Keywords: {keywords}");
        }
        let _ = writeln!(out);
    }

    let _ = writeln!(out, "Contents:");
    let _ = writeln!(out, "  Waypoints: {}", info.waypoints);
    let _ = writeln!(out, "  Routes: {}", info.routes);
    let _ = writeln!(out, "  Tracks: {}", info.tracks);
    let _ = writeln!(out);

    if let Some(tracks) = &info.track_statistics {
        let _ = writeln!(out, "Track Statistics:");
        for track in tracks {
            let default_name = format!("Track {}", track.index + 1);
            let name = track.name.as_deref().unwrap_or(&default_name);
            let _ = writeln!(out, "  {name}:");
            let _ = writeln!(out, "    Segments: {}", track.segments);
            let _ = writeln!(out, "    Points: {}", track.points);
            let _ = writeln!(
                out,
                "    Distance: {:.2} m ({:.2} km)",
                track.distance_m,
                track.distance_m / 1000.0
            );
            if track.duration_s > 0.0 {
                let _ = writeln!(out, "    Duration: {}", format_duration(track.duration_s));
            }
            if let Some(kmh) = track.avg_speed_kmh {
                let _ = writeln!(out, "    Avg Speed: {kmh:.2} km/h");
            }
            if let Some(ele) = &track.elevation {
                print_elevation(&mut out, ele, true);
            }
        }
        let _ = writeln!(out);
    }

    if let Some(routes) = &info.route_statistics {
        let _ = writeln!(out, "Route Statistics:");
        for route in routes {
            let default_name = format!("Route {}", route.index + 1);
            let name = route.name.as_deref().unwrap_or(&default_name);
            let _ = writeln!(out, "  {name}:");
            let _ = writeln!(out, "    Points: {}", route.points);
            let _ = writeln!(
                out,
                "    Distance: {:.2} m ({:.2} km)",
                route.distance_m,
                route.distance_m / 1000.0
            );
            if let Some(ele) = &route.elevation {
                print_elevation(&mut out, ele, false);
            }
        }
        let _ = writeln!(out);
    }

    if let Some(bounds) = &info.bounds {
        let _ = writeln!(out, "Bounds:");
        let _ = writeln!(
            out,
            "  Latitude: {:.6} to {:.6}",
            bounds.min_lat, bounds.max_lat
        );
        let _ = writeln!(
            out,
            "  Longitude: {:.6} to {:.6}",
            bounds.min_lon, bounds.max_lon
        );
    }

    if let (Some(distance), Some(duration)) = (info.total_distance_m, info.total_duration_s) {
        let _ = writeln!(out);
        let _ = writeln!(out, "Overall:");
        let _ = writeln!(
            out,
            "  Total Distance: {distance:.2} m ({:.2} km)",
            distance / 1000.0
        );
        if duration > 0.0 {
            let _ = writeln!(
                out,
                "  Total Duration: {}",
                format_duration(duration)
            );
        }
    }
}

fn print_elevation(out: &mut impl std::io::Write, ele: &ElevationInfo, include_avg: bool) {
    if include_avg {
        if let Some(avg) = ele.avg_m {
            let _ = writeln!(
                out,
                "    Elevation: {:.1}m - {:.1}m (avg: {:.1}m)",
                ele.min_m, ele.max_m, avg
            );
        }
    } else {
        let _ = writeln!(
            out,
            "    Elevation: {:.1}m - {:.1}m",
            ele.min_m, ele.max_m
        );
    }
    if let (Some(ascent), Some(descent)) = (ele.total_ascent_m, ele.total_descent_m) {
        let _ = writeln!(
            out,
            "    Ascent/Descent: +{ascent:.1}m / -{:.1}m",
            descent.abs()
        );
    }
}

fn format_duration(secs: f64) -> String {
    let total = secs.round() as u64;
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let seconds = total % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}
