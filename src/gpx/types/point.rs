use chrono::{DateTime, Utc};

/// A geographic point (`ptType` in GPX 1.1).
///
/// A latitude/longitude pair with optional elevation and time. Available for use
/// by extension schemas and other GPX-related structures.
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    /// Latitude in decimal degrees relative to the WGS84 datum (-90.0 to 90.0).
    pub lat: f64,
    /// Longitude in decimal degrees relative to the WGS84 datum (-180.0 to less than 180.0).
    pub lon: f64,
    /// Elevation above sea level, in meters.
    pub ele: Option<f64>,
    /// Date and time associated with the point (ISO 8601).
    pub time: Option<DateTime<Utc>>,
}
