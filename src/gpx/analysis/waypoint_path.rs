use std::time::Duration;

use crate::gpx::geo::distance_between;
use crate::gpx::types::{Route, Track, TrackSegment, Waypoint};

use super::{AnalysisOptions, ProfilePoint, SpeedProfilePoint};

/// A connected sequence of waypoints for distance, speed, and elevation analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct WaypointPath {
    points: Vec<Waypoint>,
    options: AnalysisOptions,
}

impl WaypointPath {
    /// Creates a path from an owned list of waypoints.
    pub fn new(points: Vec<Waypoint>) -> Self {
        Self {
            points,
            options: AnalysisOptions::default(),
        }
    }

    /// Creates a path from a slice of waypoints.
    pub fn from_slice(points: &[Waypoint]) -> Self {
        Self::new(points.to_vec())
    }

    /// Returns a copy of this path with custom analysis thresholds.
    pub fn with_options(mut self, options: AnalysisOptions) -> Self {
        self.options = options;
        self
    }

    /// One `WaypointPath` per track segment, preserving GPX gap semantics.
    pub fn from_track(track: &Track) -> Vec<WaypointPath> {
        track.segments.iter().map(WaypointPath::from).collect()
    }

    /// Ordered waypoints in this path.
    pub fn points(&self) -> &[Waypoint] {
        &self.points
    }

    /// Number of waypoints in this path.
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Whether this path contains no waypoints.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Total horizontal distance along the path, in meters.
    pub fn total_distance(&self) -> f64 {
        self.leg_distances().iter().sum()
    }

    /// Elapsed time from the first to the last timestamped point.
    pub fn duration(&self) -> Option<Duration> {
        let first = self.points.first()?.time?;
        let last = self.points.last()?.time?;
        duration_between(first, last)
    }

    /// Time spent moving (legs at or above the moving-speed threshold).
    pub fn moving_duration(&self) -> Option<Duration> {
        let speeds = self.leg_speeds();
        let durations = self.leg_durations();

        let mut total_ms = 0u64;
        let mut has_timed_leg = false;

        for (speed, duration) in speeds.iter().zip(durations.iter()) {
            let Some(duration) = duration else {
                continue;
            };
            has_timed_leg = true;
            if speed.is_some_and(|s| s > self.options.moving_speed_threshold_mps) {
                total_ms += duration.as_millis() as u64;
            }
        }

        if has_timed_leg {
            Some(Duration::from_millis(total_ms))
        } else {
            None
        }
    }

    /// Average speed over the full duration, in m/s.
    pub fn average_speed(&self) -> Option<f64> {
        let secs = self.duration()?.as_secs_f64();
        if secs > 0.0 {
            Some(self.total_distance() / secs)
        } else {
            None
        }
    }

    /// Average speed over moving time only, in m/s.
    pub fn average_moving_speed(&self) -> Option<f64> {
        let secs = self.moving_duration()?.as_secs_f64();
        if secs > 0.0 {
            Some(self.total_distance() / secs)
        } else {
            None
        }
    }

    /// Instantaneous speed for each leg (length = points - 1), in m/s.
    pub fn speeds(&self) -> Vec<Option<f64>> {
        self.leg_speeds()
    }

    /// Maximum leg speed, in m/s.
    pub fn max_speed(&self) -> Option<f64> {
        self.speeds().iter().filter_map(|s| *s).reduce(f64::max)
    }

    /// Minimum leg speed, in m/s.
    pub fn min_speed(&self) -> Option<f64> {
        self.speeds().iter().filter_map(|s| *s).reduce(f64::min)
    }

    /// Timestamp versus speed at the start of each timed leg.
    pub fn speed_profile(&self) -> Vec<SpeedProfilePoint> {
        self.points
            .windows(2)
            .enumerate()
            .filter_map(|(i, window)| {
                let time = window[0].time?;
                let speed = self.leg_speeds().get(i).copied()??;
                Some(SpeedProfilePoint {
                    time,
                    speed_mps: speed,
                })
            })
            .collect()
    }

    /// Per-point elevation values, when present.
    pub fn elevations(&self) -> Vec<Option<f64>> {
        self.points.iter().map(|p| p.ele).collect()
    }

    /// Mean elevation over points that have elevation data.
    pub fn average_elevation(&self) -> Option<f64> {
        let elevations: Vec<f64> = self.points.iter().filter_map(|p| p.ele).collect();
        if elevations.is_empty() {
            None
        } else {
            Some(elevations.iter().sum::<f64>() / elevations.len() as f64)
        }
    }

    /// Highest elevation among points with elevation data.
    pub fn max_elevation(&self) -> Option<f64> {
        self.points.iter().filter_map(|p| p.ele).reduce(f64::max)
    }

    /// Lowest elevation among points with elevation data.
    pub fn min_elevation(&self) -> Option<f64> {
        self.points.iter().filter_map(|p| p.ele).reduce(f64::min)
    }

    /// Elevation range from lowest to highest point with elevation data.
    pub fn elevation_difference(&self) -> Option<f64> {
        Some(self.max_elevation()? - self.min_elevation()?)
    }

    /// Alias for [`elevation_difference`](Self::elevation_difference).
    pub fn diff_elevation(&self) -> Option<f64> {
        self.elevation_difference()
    }

    /// Total uphill elevation gain between consecutive points with elevation data.
    pub fn total_ascent(&self) -> Option<f64> {
        self.elevation_gains().map(|gains| {
            let threshold = self.options.elevation_noise_threshold_m;
            gains
                .iter()
                .filter(|&&gain| gain > threshold)
                .sum()
        })
    }

    /// Total downhill elevation loss between consecutive points with elevation data.
    pub fn total_descent(&self) -> Option<f64> {
        self.elevation_gains().map(|gains| {
            let threshold = self.options.elevation_noise_threshold_m;
            gains
                .iter()
                .filter(|&&gain| gain < -threshold)
                .map(|gain| -gain)
                .sum()
        })
    }

    /// Distance versus elevation for points with elevation data.
    ///
    /// Distance is accumulated only between consecutive elevation-known points.
    pub fn elevation_profile(&self) -> Vec<ProfilePoint> {
        let points_with_ele: Vec<&Waypoint> =
            self.points.iter().filter(|point| point.ele.is_some()).collect();
        if points_with_ele.is_empty() {
            return Vec::new();
        }

        let mut distance_m = 0.0;
        let mut profile = vec![ProfilePoint {
            distance_m,
            value: points_with_ele[0].ele.unwrap(),
        }];

        for i in 1..points_with_ele.len() {
            distance_m += distance_between(points_with_ele[i - 1], points_with_ele[i]);
            profile.push(ProfilePoint {
                distance_m,
                value: points_with_ele[i].ele.unwrap(),
            });
        }

        profile
    }

    fn leg_distances(&self) -> Vec<f64> {
        self.points
            .windows(2)
            .map(|window| distance_between(&window[0], &window[1]))
            .collect()
    }

    fn leg_durations(&self) -> Vec<Option<Duration>> {
        self.points
            .windows(2)
            .map(|window| match (window[0].time, window[1].time) {
                (Some(start), Some(end)) => duration_between(start, end),
                _ => None,
            })
            .collect()
    }

    fn leg_speeds(&self) -> Vec<Option<f64>> {
        self.leg_distances()
            .iter()
            .zip(self.leg_durations())
            .map(|(distance, duration)| {
                duration.and_then(|duration| {
                    let secs = duration.as_secs_f64();
                    if secs > 0.0 {
                        Some(distance / secs)
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    fn elevation_gains(&self) -> Option<Vec<f64>> {
        let points_with_ele: Vec<&Waypoint> =
            self.points.iter().filter(|point| point.ele.is_some()).collect();
        if points_with_ele.len() < 2 {
            return None;
        }
        Some(
            points_with_ele
                .windows(2)
                .map(|window| window[1].ele.unwrap() - window[0].ele.unwrap())
                .collect(),
        )
    }
}

impl From<&TrackSegment> for WaypointPath {
    fn from(segment: &TrackSegment) -> Self {
        Self::from_slice(&segment.points)
    }
}

impl From<&Route> for WaypointPath {
    fn from(route: &Route) -> Self {
        Self::from_slice(&route.points)
    }
}

impl<'a> IntoIterator for &'a WaypointPath {
    type Item = &'a Waypoint;
    type IntoIter = std::slice::Iter<'a, Waypoint>;

    fn into_iter(self) -> Self::IntoIter {
        self.points.iter()
    }
}

fn duration_between(
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
) -> Option<Duration> {
    let millis = (end - start).num_milliseconds();
    if millis >= 0 {
        Some(Duration::from_millis(millis as u64))
    } else {
        None
    }
}
