use std::ops::{Index, IndexMut};
use std::time::Duration;

use crate::gpx::geo::distance_between;
use crate::gpx::types::{Bounds, Route, Track, TrackSegment, Waypoint};

use super::{ProfilePoint, SpeedProfilePoint, WaypointPath};

fn bounds_from_points(points: &[Waypoint]) -> Option<Bounds> {
    if points.is_empty() {
        return None;
    }
    Some(Bounds {
        minlat: points.iter().map(|p| p.lat).fold(f64::INFINITY, f64::min),
        minlon: points.iter().map(|p| p.lon).fold(f64::INFINITY, f64::min),
        maxlat: points.iter().map(|p| p.lat).fold(f64::NEG_INFINITY, f64::max),
        maxlon: points.iter().map(|p| p.lon).fold(f64::NEG_INFINITY, f64::max),
    })
}

fn track_elevation_profile(segments: &[TrackSegment]) -> Vec<ProfilePoint> {
    let mut distance_m = 0.0;
    let mut profile = Vec::new();

    for segment in segments {
        let points_with_ele: Vec<&Waypoint> =
            segment.points.iter().filter(|p| p.ele.is_some()).collect();
        if points_with_ele.is_empty() {
            continue;
        }

        if profile.is_empty() {
            profile.push(ProfilePoint {
                distance_m,
                value: points_with_ele[0].ele.unwrap(),
            });
        }

        for i in 1..points_with_ele.len() {
            distance_m += distance_between(points_with_ele[i - 1], points_with_ele[i]);
            profile.push(ProfilePoint {
                distance_m,
                value: points_with_ele[i].ele.unwrap(),
            });
        }
    }

    profile
}

macro_rules! point_container_stats {
    ($type:ty, $field:ident) => {
        impl $type {
            /// Minimum and maximum latitude/longitude covering all points.
            pub fn bounds(&self) -> Option<Bounds> {
                bounds_from_points(&self.$field)
            }

            /// Total horizontal distance along the path, in meters.
            pub fn total_distance(&self) -> f64 {
                WaypointPath::from(self).total_distance()
            }

            /// Alias for [`total_distance`](Self::total_distance).
            pub fn distance(&self) -> f64 {
                self.total_distance()
            }

            /// Elapsed time from the first to the last timestamped point.
            pub fn duration(&self) -> Option<Duration> {
                WaypointPath::from(self).duration()
            }

            /// Alias for [`duration`](Self::duration).
            pub fn total_duration(&self) -> Option<Duration> {
                self.duration()
            }

            /// Time spent moving (legs above the moving-speed threshold).
            pub fn moving_duration(&self) -> Option<Duration> {
                WaypointPath::from(self).moving_duration()
            }

            /// Average speed over the full duration, in m/s.
            pub fn average_speed(&self) -> Option<f64> {
                WaypointPath::from(self).average_speed()
            }

            /// Alias for [`average_speed`](Self::average_speed).
            pub fn speed(&self) -> Option<f64> {
                self.average_speed()
            }

            /// Average speed over moving time only, in m/s.
            pub fn average_moving_speed(&self) -> Option<f64> {
                WaypointPath::from(self).average_moving_speed()
            }

            /// Maximum leg speed, in m/s.
            pub fn max_speed(&self) -> Option<f64> {
                WaypointPath::from(self).max_speed()
            }

            /// Minimum leg speed, in m/s.
            pub fn min_speed(&self) -> Option<f64> {
                WaypointPath::from(self).min_speed()
            }

            /// Timestamp versus speed at the start of each timed leg.
            pub fn speed_profile(&self) -> Vec<SpeedProfilePoint> {
                WaypointPath::from(self).speed_profile()
            }

            /// Mean elevation over points that have elevation data.
            pub fn average_elevation(&self) -> Option<f64> {
                WaypointPath::from(self).average_elevation()
            }

            /// Alias for [`average_elevation`](Self::average_elevation).
            pub fn elevation(&self) -> Option<f64> {
                self.average_elevation()
            }

            /// Highest elevation among points with elevation data.
            pub fn max_elevation(&self) -> Option<f64> {
                WaypointPath::from(self).max_elevation()
            }

            /// Lowest elevation among points with elevation data.
            pub fn min_elevation(&self) -> Option<f64> {
                WaypointPath::from(self).min_elevation()
            }

            /// Elevation range from lowest to highest point with elevation data.
            pub fn elevation_difference(&self) -> Option<f64> {
                WaypointPath::from(self).elevation_difference()
            }

            /// Alias for [`elevation_difference`](Self::elevation_difference).
            pub fn diff_elevation(&self) -> Option<f64> {
                self.elevation_difference()
            }

            /// Total uphill elevation gain between consecutive elevation-known points.
            pub fn total_ascent(&self) -> Option<f64> {
                WaypointPath::from(self).total_ascent()
            }

            /// Total downhill elevation loss between consecutive elevation-known points.
            pub fn total_descent(&self) -> Option<f64> {
                WaypointPath::from(self).total_descent()
            }

            /// Distance versus elevation for points with elevation data.
            pub fn elevation_profile(&self) -> Vec<ProfilePoint> {
                WaypointPath::from(self).elevation_profile()
            }
        }
    };
}

point_container_stats!(Route, points);
point_container_stats!(TrackSegment, points);

impl Route {
    /// Number of route points.
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Whether this route has no points.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

impl TrackSegment {
    /// Number of track points in this segment.
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Whether this segment has no points.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

impl Index<usize> for Route {
    type Output = Waypoint;

    fn index(&self, index: usize) -> &Self::Output {
        &self.points[index]
    }
}

impl IndexMut<usize> for Route {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.points[index]
    }
}

impl Index<std::ops::Range<usize>> for Route {
    type Output = [Waypoint];

    fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
        &self.points[index]
    }
}

impl<'a> IntoIterator for &'a Route {
    type Item = &'a Waypoint;
    type IntoIter = std::slice::Iter<'a, Waypoint>;

    fn into_iter(self) -> Self::IntoIter {
        self.points.iter()
    }
}

impl Index<usize> for TrackSegment {
    type Output = Waypoint;

    fn index(&self, index: usize) -> &Self::Output {
        &self.points[index]
    }
}

impl IndexMut<usize> for TrackSegment {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.points[index]
    }
}

impl Index<std::ops::Range<usize>> for TrackSegment {
    type Output = [Waypoint];

    fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
        &self.points[index]
    }
}

impl<'a> IntoIterator for &'a TrackSegment {
    type Item = &'a Waypoint;
    type IntoIter = std::slice::Iter<'a, Waypoint>;

    fn into_iter(self) -> Self::IntoIter {
        self.points.iter()
    }
}

impl Track {
    /// Number of track segments.
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// Whether this track has no segments.
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Minimum and maximum latitude/longitude across all segments.
    pub fn bounds(&self) -> Option<Bounds> {
        let mut minlat = f64::INFINITY;
        let mut minlon = f64::INFINITY;
        let mut maxlat = f64::NEG_INFINITY;
        let mut maxlon = f64::NEG_INFINITY;
        let mut has_points = false;

        for point in self.segments.iter().flat_map(|segment| segment.points.iter()) {
            has_points = true;
            minlat = minlat.min(point.lat);
            minlon = minlon.min(point.lon);
            maxlat = maxlat.max(point.lat);
            maxlon = maxlon.max(point.lon);
        }

        if has_points {
            Some(Bounds {
                minlat,
                minlon,
                maxlat,
                maxlon,
            })
        } else {
            None
        }
    }

    /// Total horizontal distance summed across segments, in meters.
    pub fn total_distance(&self) -> f64 {
        WaypointPath::from_track(self)
            .iter()
            .map(WaypointPath::total_distance)
            .sum()
    }

    /// Alias for [`total_distance`](Self::total_distance).
    pub fn distance(&self) -> f64 {
        self.total_distance()
    }

    /// Sum of per-segment durations.
    pub fn duration(&self) -> Option<Duration> {
        let durations: Vec<Duration> = WaypointPath::from_track(self)
            .iter()
            .filter_map(WaypointPath::duration)
            .collect();
        if durations.is_empty() {
            None
        } else {
            Some(durations.into_iter().sum())
        }
    }

    /// Alias for [`duration`](Self::duration).
    pub fn total_duration(&self) -> Option<Duration> {
        self.duration()
    }

    /// Sum of per-segment moving durations.
    pub fn moving_duration(&self) -> Option<Duration> {
        let durations: Vec<Duration> = WaypointPath::from_track(self)
            .iter()
            .filter_map(WaypointPath::moving_duration)
            .collect();
        if durations.is_empty() {
            None
        } else {
            Some(durations.into_iter().sum())
        }
    }

    /// Average speed over the full track duration, in m/s.
    pub fn average_speed(&self) -> Option<f64> {
        let secs = self.duration()?.as_secs_f64();
        if secs > 0.0 {
            Some(self.total_distance() / secs)
        } else {
            None
        }
    }

    /// Alias for [`average_speed`](Self::average_speed).
    pub fn speed(&self) -> Option<f64> {
        self.average_speed()
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

    /// Maximum leg speed across all segments, in m/s.
    pub fn max_speed(&self) -> Option<f64> {
        WaypointPath::from_track(self)
            .iter()
            .filter_map(WaypointPath::max_speed)
            .reduce(f64::max)
    }

    /// Minimum leg speed across all segments, in m/s.
    pub fn min_speed(&self) -> Option<f64> {
        WaypointPath::from_track(self)
            .iter()
            .filter_map(WaypointPath::min_speed)
            .reduce(f64::min)
    }

    /// Concatenated per-segment speed profiles.
    pub fn speed_profile(&self) -> Vec<SpeedProfilePoint> {
        WaypointPath::from_track(self)
            .iter()
            .flat_map(WaypointPath::speed_profile)
            .collect()
    }

    /// Mean elevation over all points with elevation data.
    pub fn average_elevation(&self) -> Option<f64> {
        let elevations: Vec<f64> = self
            .segments
            .iter()
            .flat_map(|segment| segment.points.iter())
            .filter_map(|point| point.ele)
            .collect();
        if elevations.is_empty() {
            None
        } else {
            Some(elevations.iter().sum::<f64>() / elevations.len() as f64)
        }
    }

    /// Alias for [`average_elevation`](Self::average_elevation).
    pub fn elevation(&self) -> Option<f64> {
        self.average_elevation()
    }

    /// Highest elevation across all segments.
    pub fn max_elevation(&self) -> Option<f64> {
        WaypointPath::from_track(self)
            .iter()
            .filter_map(WaypointPath::max_elevation)
            .reduce(f64::max)
    }

    /// Lowest elevation across all segments.
    pub fn min_elevation(&self) -> Option<f64> {
        WaypointPath::from_track(self)
            .iter()
            .filter_map(WaypointPath::min_elevation)
            .reduce(f64::min)
    }

    /// Elevation range across the track.
    pub fn elevation_difference(&self) -> Option<f64> {
        Some(self.max_elevation()? - self.min_elevation()?)
    }

    /// Alias for [`elevation_difference`](Self::elevation_difference).
    pub fn diff_elevation(&self) -> Option<f64> {
        self.elevation_difference()
    }

    /// Total uphill gain summed across segments.
    pub fn total_ascent(&self) -> Option<f64> {
        let ascents: Vec<f64> = WaypointPath::from_track(self)
            .iter()
            .filter_map(WaypointPath::total_ascent)
            .collect();
        if ascents.is_empty() {
            None
        } else {
            Some(ascents.into_iter().sum())
        }
    }

    /// Total downhill loss summed across segments.
    pub fn total_descent(&self) -> Option<f64> {
        let descents: Vec<f64> = WaypointPath::from_track(self)
            .iter()
            .filter_map(WaypointPath::total_descent)
            .collect();
        if descents.is_empty() {
            None
        } else {
            Some(descents.into_iter().sum())
        }
    }

    /// Continuous distance-elevation profile across all segments.
    pub fn elevation_profile(&self) -> Vec<ProfilePoint> {
        track_elevation_profile(&self.segments)
    }
}

impl Index<usize> for Track {
    type Output = TrackSegment;

    fn index(&self, index: usize) -> &Self::Output {
        &self.segments[index]
    }
}

impl IndexMut<usize> for Track {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.segments[index]
    }
}

impl Index<std::ops::Range<usize>> for Track {
    type Output = [TrackSegment];

    fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
        &self.segments[index]
    }
}

impl<'a> IntoIterator for &'a Track {
    type Item = &'a TrackSegment;
    type IntoIter = std::slice::Iter<'a, TrackSegment>;

    fn into_iter(self) -> Self::IntoIter {
        self.segments.iter()
    }
}
