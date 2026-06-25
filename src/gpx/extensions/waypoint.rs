//! Convenience accessors for Garmin / Strava extensions on GPX elements.

use crate::gpx::types::Waypoint;

use super::{PowerExtension, TrackPointExtension};

impl Waypoint {
    /// Garmin / Strava track point extension, if present.
    pub fn track_point_extension(&self) -> Option<&TrackPointExtension> {
        self.extensions.as_ref()?.track_point.as_ref()
    }

    /// Garmin power extension, if present.
    pub fn power_extension(&self) -> Option<&PowerExtension> {
        self.extensions.as_ref()?.power_extension.as_ref()
    }

    /// Heart rate in beats per minute (`gpxtpx:hr`).
    pub fn heart_rate(&self) -> Option<u8> {
        self.extensions.as_ref()?.heart_rate()
    }

    /// Cadence in revolutions per minute (`gpxtpx:cad`).
    pub fn cadence(&self) -> Option<u8> {
        self.extensions.as_ref()?.cadence()
    }

    /// Instantaneous power in watts (`gpxpx:PowerInWatts`).
    pub fn power_watts(&self) -> Option<u16> {
        self.extensions.as_ref()?.power_watts()
    }
}
