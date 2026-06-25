use chrono::{DateTime, Utc};
use serde::Deserialize;

/// A geographic point (`ptType` in GPX 1.1).
///
/// A latitude/longitude pair with optional elevation and time. Available for use
/// by extension schemas and other GPX-related structures.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Point {
    /// Latitude in decimal degrees relative to the WGS84 datum (-90.0 to 90.0).
    #[serde(rename = "@lat")]
    pub lat: f64,
    /// Longitude in decimal degrees relative to the WGS84 datum (-180.0 to less than 180.0).
    #[serde(rename = "@lon")]
    pub lon: f64,
    /// Elevation above sea level, in meters.
    #[serde(default)]
    pub ele: Option<f64>,
    /// Date and time associated with the point (ISO 8601).
    #[serde(default)]
    pub time: Option<DateTime<Utc>>,
}
