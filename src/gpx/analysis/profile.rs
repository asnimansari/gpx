use chrono::{DateTime, Utc};

/// A sample on an elevation profile: distance versus elevation.
#[derive(Debug, Clone, PartialEq)]
pub struct ProfilePoint {
    /// Cumulative horizontal distance from the path start, in meters.
    pub distance_m: f64,
    /// Elevation at this distance, in meters.
    pub value: f64,
}

/// A sample on a speed profile: timestamp versus speed.
#[derive(Debug, Clone, PartialEq)]
pub struct SpeedProfilePoint {
    /// Timestamp at the start of the leg.
    pub time: DateTime<Utc>,
    /// Speed over the leg, in m/s.
    pub speed_mps: f64,
}
