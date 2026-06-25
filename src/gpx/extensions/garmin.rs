//! Garmin and Strava GPX extension schemas.
//!
//! Strava GPX exports use Garmin's Track Point Extension (`gpxtpx`) namespace for
//! heart rate, cadence, and temperature. Power data uses Garmin's Power Extension (`gpxpx`).

use serde::{Deserialize, Serialize};

/// Garmin Track Point Extension v1/v2 (`gpxtpx:TrackPointExtension`).
///
/// Namespace: `http://www.garmin.com/xmlschemas/TrackPointExtension/v1` (also v2).
/// Used by Garmin devices and Strava GPX exports.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
pub struct TrackPointExtension {
    /// Air temperature at the point, in degrees Celsius.
    #[serde(rename = "atemp", default, skip_serializing_if = "Option::is_none")]
    pub atemp: Option<f64>,
    /// Water temperature at the point, in degrees Celsius.
    #[serde(rename = "wtemp", default, skip_serializing_if = "Option::is_none")]
    pub wtemp: Option<f64>,
    /// Depth below water surface, in meters.
    #[serde(rename = "depth", default, skip_serializing_if = "Option::is_none")]
    pub depth: Option<f64>,
    /// Heart rate in beats per minute.
    #[serde(rename = "hr", default, skip_serializing_if = "Option::is_none")]
    pub hr: Option<u8>,
    /// Cadence in revolutions per minute.
    #[serde(rename = "cad", default, skip_serializing_if = "Option::is_none")]
    pub cad: Option<u8>,
}

/// Garmin Power Extension v1 (`gpxpx:PowerExtension`).
///
/// Namespace: `http://www.garmin.com/xmlschemas/PowerExtension/v1`.
/// Present in Strava GPX exports when a power meter was used.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct PowerExtension {
    /// Instantaneous power in watts.
    #[serde(
        rename = "PowerInWatts",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub power_in_watts: Option<u16>,
}

/// Garmin GPX Extensions v3 track extension (`gpxx:TrackExtension`).
///
/// Namespace: `http://www.garmin.com/xmlschemas/GpxExtensions/v3`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct TrackExtension {
    /// Display color hint for the track on a Garmin device.
    #[serde(rename = "DisplayColor", default, skip_serializing_if = "Option::is_none")]
    pub display_color: Option<String>,
}

/// `gpxtpx` namespace URI (Track Point Extension v1).
pub const GPXTPTX_NS_V1: &str = "http://www.garmin.com/xmlschemas/TrackPointExtension/v1";

/// `gpxtpx` namespace URI (Track Point Extension v2).
pub const GPXTPTX_NS_V2: &str = "http://www.garmin.com/xmlschemas/TrackPointExtension/v2";

/// `gpxpx` namespace URI (Power Extension v1).
pub const GPPXPX_NS_V1: &str = "http://www.garmin.com/xmlschemas/PowerExtension/v1";

/// `gpxx` namespace URI (GPX Extensions v3).
pub const GPXX_NS_V3: &str = "http://www.garmin.com/xmlschemas/GpxExtensions/v3";
