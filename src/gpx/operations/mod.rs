//! Pure GPX edit and merge operations.
//!
//! Each function clones the input and returns a new [`Gpx`] without mutating the original.

use std::fmt;
use std::time::Duration;

use chrono::{DateTime, Utc};

use crate::gpx::geo::distance_between;
use crate::gpx::types::{Gpx, TrackSegment, Waypoint};

const EARTH_RADIUS_M: f64 = 6_378_137.0;

/// Error returned when operation arguments are invalid.
#[derive(Debug, Clone, PartialEq)]
pub enum OperationError {
    /// Neither `time_gap` nor `distance_gap` was provided to [`split`].
    MissingSplitGap,
    /// [`simplify`] was called with a non-positive tolerance.
    InvalidTolerance { tolerance: f64 },
    /// [`smooth`] was called with an invalid window size.
    InvalidSmoothWindow { window: usize },
}

impl fmt::Display for OperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSplitGap => {
                write!(f, "At least one of time_gap or distance_gap must be given")
            }
            Self::InvalidTolerance { tolerance } => {
                write!(f, "tolerance must be positive, got {tolerance}")
            }
            Self::InvalidSmoothWindow { window } => {
                write!(
                    f,
                    "window must be an odd integer of at least 3, got {window}"
                )
            }
        }
    }
}

impl std::error::Error for OperationError {}

/// Fields that can be stripped from GPX metadata.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StripMetadataFields {
    /// Remove the entire metadata element.
    pub strip_all: bool,
    pub name: bool,
    pub desc: bool,
    pub author: bool,
    pub copyright: bool,
    pub time: bool,
    pub keywords: bool,
    pub links: bool,
}

impl StripMetadataFields {
    /// Remove the entire metadata element.
    pub fn all() -> Self {
        Self {
            strip_all: true,
            ..Self::default()
        }
    }
}

/// Options for [`smooth`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SmoothOptions {
    pub window: usize,
    pub coordinates: bool,
    pub elevations: bool,
}

impl Default for SmoothOptions {
    fn default() -> Self {
        Self {
            window: 5,
            coordinates: true,
            elevations: true,
        }
    }
}

/// Return a new GPX with each point list filtered by `predicate`.
///
/// Routes, track segments, and tracks that end up empty are dropped.
pub fn filter_points<F>(gpx: &Gpx, predicate: F) -> Gpx
where
    F: Fn(&Waypoint) -> bool,
{
    let mut out = gpx.clone();
    out.waypoints = gpx
        .waypoints
        .iter()
        .filter(|w| predicate(w))
        .cloned()
        .collect();

    out.routes = gpx
        .routes
        .iter()
        .filter_map(|route| {
            let points: Vec<Waypoint> = route
                .points
                .iter()
                .filter(|p| predicate(p))
                .cloned()
                .collect();
            if points.is_empty() {
                None
            } else {
                let mut new_route = route.clone();
                new_route.points = points;
                Some(new_route)
            }
        })
        .collect();

    out.tracks = gpx
        .tracks
        .iter()
        .filter_map(|track| {
            let segments: Vec<TrackSegment> = track
                .segments
                .iter()
                .filter_map(|segment| {
                    let points: Vec<Waypoint> = segment
                        .points
                        .iter()
                        .filter(|p| predicate(p))
                        .cloned()
                        .collect();
                    if points.is_empty() {
                        None
                    } else {
                        let mut new_segment = segment.clone();
                        new_segment.points = points;
                        Some(new_segment)
                    }
                })
                .collect();
            if segments.is_empty() {
                None
            } else {
                let mut new_track = track.clone();
                new_track.segments = segments;
                Some(new_track)
            }
        })
        .collect();

    out
}

/// Crop a GPX to a geographic bounding box.
///
/// Bounds that are `None` are not enforced.
pub fn crop(
    gpx: &Gpx,
    min_lat: Option<f64>,
    max_lat: Option<f64>,
    min_lon: Option<f64>,
    max_lon: Option<f64>,
) -> Gpx {
    filter_points(gpx, |point| {
        if let Some(min_lat) = min_lat {
            if point.lat < min_lat {
                return false;
            }
        }
        if let Some(max_lat) = max_lat {
            if point.lat > max_lat {
                return false;
            }
        }
        if let Some(min_lon) = min_lon {
            if point.lon < min_lon {
                return false;
            }
        }
        if let Some(max_lon) = max_lon {
            if point.lon > max_lon {
                return false;
            }
        }
        true
    })
}

/// Trim a GPX to a date/time range.
///
/// Points without a timestamp are kept.
pub fn trim(
    gpx: &Gpx,
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
) -> Gpx {
    filter_points(gpx, |point| {
        let Some(time) = point.time else {
            return true;
        };
        if let Some(start) = start {
            if time < start {
                return false;
            }
        }
        if let Some(end) = end {
            if time > end {
                return false;
            }
        }
        true
    })
}

/// Reverse the routes and/or tracks of a GPX.
///
/// For tracks, both the order of segments and the order of points within each segment are reversed.
pub fn reverse(gpx: &Gpx, routes: bool, tracks: bool) -> Gpx {
    let mut out = gpx.clone();

    if routes {
        out.routes = gpx
            .routes
            .iter()
            .map(|route| {
                let mut new_route = route.clone();
                new_route.points.reverse();
                new_route
            })
            .collect();
    }

    if tracks {
        out.tracks = gpx
            .tracks
            .iter()
            .map(|track| {
                let mut new_track = track.clone();
                new_track.segments = track
                    .segments
                    .iter()
                    .rev()
                    .map(|segment| {
                        let mut new_segment = segment.clone();
                        new_segment.points.reverse();
                        new_segment
                    })
                    .collect();
                new_track
            })
            .collect();
    }

    out
}

/// Strip metadata fields from a GPX.
///
/// When `fields.strip_all` is true, or no individual field flag is set, the entire metadata
/// element is removed. Otherwise only the selected fields are cleared.
pub fn strip_metadata(gpx: &Gpx, fields: StripMetadataFields) -> Gpx {
    let strip_entire = fields.strip_all
        || !fields.name
            && !fields.desc
            && !fields.author
            && !fields.copyright
            && !fields.time
            && !fields.keywords
            && !fields.links;

    let mut out = gpx.clone();
    if strip_entire {
        out.metadata = None;
        return out;
    }

    let Some(mut metadata) = gpx.metadata.clone() else {
        return out;
    };

    if fields.name {
        metadata.name = None;
    }
    if fields.desc {
        metadata.desc = None;
    }
    if fields.author {
        metadata.author = None;
    }
    if fields.copyright {
        metadata.copyright = None;
    }
    if fields.time {
        metadata.time = None;
    }
    if fields.keywords {
        metadata.keywords = None;
    }
    if fields.links {
        metadata.links.clear();
    }

    out.metadata = Some(metadata);
    out
}

/// Reduce the precision of coordinates and/or elevations.
pub fn reduce_precision(
    gpx: &Gpx,
    coordinate_precision: Option<u32>,
    elevation_precision: Option<u32>,
) -> Gpx {
    map_points(gpx, |point| {
        if coordinate_precision.is_none() && elevation_precision.is_none() {
            return point.clone();
        }

        let mut new_point = point.clone();
        if let Some(precision) = coordinate_precision {
            new_point.lat = round_to(point.lat, precision);
            new_point.lon = round_to(point.lon, precision);
        }
        if let Some(precision) = elevation_precision {
            if let Some(ele) = point.ele {
                new_point.ele = Some(round_to(ele, precision));
            }
        }
        new_point
    })
}

/// Split track segments at time and/or distance gaps.
///
/// Waypoints and routes are left unchanged.
pub fn split(
    gpx: &Gpx,
    time_gap: Option<Duration>,
    distance_gap: Option<f64>,
) -> Result<Gpx, OperationError> {
    if time_gap.is_none() && distance_gap.is_none() {
        return Err(OperationError::MissingSplitGap);
    }

    let time_gap = time_gap.map(chrono::Duration::from_std).transpose().ok().flatten();

    let mut out = gpx.clone();
    out.tracks = gpx
        .tracks
        .iter()
        .map(|track| {
            let mut new_track = track.clone();
            new_track.segments = track
                .segments
                .iter()
                .flat_map(|segment| split_segment(segment, time_gap, distance_gap))
                .collect();
            new_track
        })
        .collect();
    Ok(out)
}

/// Simplify tracks and routes with the Ramer-Douglas-Peucker algorithm.
///
/// Waypoints are left unchanged.
pub fn simplify(gpx: &Gpx, tolerance: f64) -> Result<Gpx, OperationError> {
    if tolerance <= 0.0 {
        return Err(OperationError::InvalidTolerance { tolerance });
    }

    let mut out = gpx.clone();
    out.routes = gpx
        .routes
        .iter()
        .map(|route| {
            let mut new_route = route.clone();
            new_route.points = ramer_douglas_peucker(&route.points, tolerance);
            new_route
        })
        .collect();
    out.tracks = gpx
        .tracks
        .iter()
        .map(|track| {
            let mut new_track = track.clone();
            new_track.segments = track
                .segments
                .iter()
                .map(|segment| {
                    let mut new_segment = segment.clone();
                    new_segment.points = ramer_douglas_peucker(&segment.points, tolerance);
                    new_segment
                })
                .collect();
            new_track
        })
        .collect();
    Ok(out)
}

/// Smooth track and route coordinates and elevations with a centered moving average.
///
/// Waypoints are left unchanged.
pub fn smooth(gpx: &Gpx, window: usize) -> Result<Gpx, OperationError> {
    smooth_with_options(gpx, SmoothOptions {
        window,
        ..SmoothOptions::default()
    })
}

/// Smooth tracks and routes with custom moving-average options.
pub fn smooth_with_options(gpx: &Gpx, options: SmoothOptions) -> Result<Gpx, OperationError> {
    let SmoothOptions {
        window,
        coordinates,
        elevations,
    } = options;

    if window < 3 || window % 2 == 0 {
        return Err(OperationError::InvalidSmoothWindow { window });
    }

    let half = window / 2;

    let mut out = gpx.clone();
    out.routes = gpx
        .routes
        .iter()
        .map(|route| {
            let mut new_route = route.clone();
            new_route.points = smooth_points(&route.points, half, coordinates, elevations);
            new_route
        })
        .collect();
    out.tracks = gpx
        .tracks
        .iter()
        .map(|track| {
            let mut new_track = track.clone();
            new_track.segments = track
                .segments
                .iter()
                .map(|segment| {
                    let mut new_segment = segment.clone();
                    new_segment.points =
                        smooth_points(&segment.points, half, coordinates, elevations);
                    new_segment
                })
                .collect();
            new_track
        })
        .collect();
    Ok(out)
}

/// Shift all point timestamps by `delta`.
///
/// Points without a timestamp and metadata time are left unchanged.
pub fn shift_time(gpx: &Gpx, delta: Duration) -> Gpx {
    let delta = chrono::Duration::from_std(delta).unwrap_or_default();
    map_points(gpx, |point| {
        let mut new_point = point.clone();
        if let Some(time) = point.time {
            new_point.time = Some(time + delta);
        }
        new_point
    })
}

/// Strip all extensions from a GPX.
pub fn strip_extensions(gpx: &Gpx) -> Gpx {
    let mut out = map_points(gpx, |point| {
        let mut new_point = point.clone();
        new_point.extensions = None;
        new_point
    });

    if let Some(metadata) = out.metadata.as_mut() {
        metadata.extensions = None;
    }

    for route in &mut out.routes {
        route.extensions = None;
    }

    for track in &mut out.tracks {
        track.extensions = None;
        for segment in &mut track.segments {
            segment.extensions = None;
        }
    }

    out.extensions = None;
    out
}

/// Merge multiple GPX instances into one.
///
/// Waypoints, routes, and tracks are concatenated in order.
pub fn merge(gpxs: &[Gpx]) -> Gpx {
    merge_with_creator(gpxs, None)
}

/// Merge multiple GPX instances into one, optionally setting the creator.
pub fn merge_with_creator(gpxs: &[Gpx], creator: Option<String>) -> Gpx {
    let mut merged = Gpx {
        waypoints: Vec::new(),
        routes: Vec::new(),
        tracks: Vec::new(),
        ..Default::default()
    };

    for gpx in gpxs {
        merged.waypoints.extend(gpx.waypoints.iter().cloned());
        merged.routes.extend(gpx.routes.iter().cloned());
        merged.tracks.extend(gpx.tracks.iter().cloned());
    }

    if let Some(creator) = creator {
        merged.creator = Some(creator);
    }

    merged
}

fn map_points<F>(gpx: &Gpx, transform: F) -> Gpx
where
    F: Fn(&Waypoint) -> Waypoint,
{
    let mut out = gpx.clone();
    out.waypoints = gpx.waypoints.iter().map(&transform).collect();
    out.routes = gpx
        .routes
        .iter()
        .map(|route| {
            let mut new_route = route.clone();
            new_route.points = route.points.iter().map(&transform).collect();
            new_route
        })
        .collect();
    out.tracks = gpx
        .tracks
        .iter()
        .map(|track| {
            let mut new_track = track.clone();
            new_track.segments = track
                .segments
                .iter()
                .map(|segment| {
                    let mut new_segment = segment.clone();
                    new_segment.points = segment.points.iter().map(&transform).collect();
                    new_segment
                })
                .collect();
            new_track
        })
        .collect();
    out
}

fn split_segment(
    segment: &TrackSegment,
    time_gap: Option<chrono::Duration>,
    distance_gap: Option<f64>,
) -> Vec<TrackSegment> {
    if segment.points.is_empty() {
        return vec![segment.clone()];
    }

    let mut parts: Vec<Vec<Waypoint>> = vec![Vec::new()];
    for point in &segment.points {
        if let Some(last) = parts.last().and_then(|part| part.last()) {
            if is_gap(last, point, time_gap, distance_gap) {
                parts.push(Vec::new());
            }
        }
        parts.last_mut().expect("parts is non-empty").push(point.clone());
    }

    parts
        .into_iter()
        .map(|points| {
            let mut new_segment = segment.clone();
            new_segment.points = points;
            new_segment
        })
        .collect()
}

fn is_gap(
    prev: &Waypoint,
    point: &Waypoint,
    time_gap: Option<chrono::Duration>,
    distance_gap: Option<f64>,
) -> bool {
    if let Some(time_gap) = time_gap {
        if let (Some(prev_time), Some(point_time)) = (prev.time, point.time) {
            if point_time - prev_time > time_gap {
                return true;
            }
        }
    }

    if let Some(distance_gap) = distance_gap {
        if distance_between(prev, point) > distance_gap {
            return true;
        }
    }

    false
}

fn perpendicular_distance(point: &Waypoint, start: &Waypoint, end: &Waypoint) -> f64 {
    let lat0 = start.lat.to_radians();
    let lon0 = start.lon.to_radians();
    let cos_lat0 = lat0.cos();

    let project = |p: &Waypoint| -> (f64, f64) {
        let x = (p.lon.to_radians() - lon0) * cos_lat0 * EARTH_RADIUS_M;
        let y = (p.lat.to_radians() - lat0) * EARTH_RADIUS_M;
        (x, y)
    };

    let (px, py) = project(point);
    let (ex, ey) = project(end);

    let segment_length_sq = ex * ex + ey * ey;
    if segment_length_sq == 0.0 {
        return (px * px + py * py).sqrt();
    }

    let t = ((px * ex + py * ey) / segment_length_sq).clamp(0.0, 1.0);
    let dx = px - t * ex;
    let dy = py - t * ey;
    (dx * dx + dy * dy).sqrt()
}

fn ramer_douglas_peucker(points: &[Waypoint], tolerance: f64) -> Vec<Waypoint> {
    if points.len() < 3 {
        return points.to_vec();
    }

    let mut keep = vec![false; points.len()];
    keep[0] = true;
    *keep.last_mut().expect("len >= 3") = true;

    let mut stack = vec![(0_usize, points.len() - 1)];
    while let Some((start, end)) = stack.pop() {
        let mut max_distance = 0.0;
        let mut index = start;
        for i in (start + 1)..end {
            let distance = perpendicular_distance(&points[i], &points[start], &points[end]);
            if distance > max_distance {
                max_distance = distance;
                index = i;
            }
        }
        if max_distance > tolerance {
            keep[index] = true;
            stack.push((start, index));
            stack.push((index, end));
        }
    }

    points
        .iter()
        .zip(keep)
        .filter(|&(_, kept)| kept)
        .map(|(point, _)| point.clone())
        .collect()
}

fn smooth_points(
    points: &[Waypoint],
    half: usize,
    coordinates: bool,
    elevations: bool,
) -> Vec<Waypoint> {
    let n = points.len();
    if n < 3 {
        return points.to_vec();
    }

    let mut smoothed = Vec::with_capacity(n);
    for (i, point) in points.iter().enumerate() {
        let k = half.min(i).min(n - 1 - i);
        if k == 0 {
            smoothed.push(point.clone());
            continue;
        }

        let neighbors = &points[i - k..=i + k];
        let mut new_point = point.clone();

        if coordinates {
            let lat_sum: f64 = neighbors.iter().map(|p| p.lat).sum();
            let lon_sum: f64 = neighbors.iter().map(|p| p.lon).sum();
            let count = neighbors.len() as f64;
            new_point.lat = lat_sum / count;
            new_point.lon = lon_sum / count;
        }

        if elevations && point.ele.is_some() {
            let ele_values: Vec<f64> = neighbors.iter().filter_map(|p| p.ele).collect();
            if !ele_values.is_empty() {
                let ele_sum: f64 = ele_values.iter().sum();
                new_point.ele = Some(ele_sum / ele_values.len() as f64);
            }
        }

        smoothed.push(new_point);
    }

    smoothed
}

fn round_to(value: f64, precision: u32) -> f64 {
    let factor = 10_f64.powi(precision as i32);
    (value * factor).round() / factor
}
