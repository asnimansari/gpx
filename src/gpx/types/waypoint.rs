use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::{Extensions, Fix, Link};

/// A waypoint, route point, or track point (`wptType` in GPX 1.1).
///
/// Represents a point of interest, named feature, route vertex, or recorded
/// track position. Used for `<wpt>`, `<rtept>`, and `<trkpt>` elements.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Waypoint {
    /// Latitude in decimal degrees relative to the WGS84 datum (-90.0 to 90.0).
    #[serde(rename = "@lat")]
    pub lat: f64,
    /// Longitude in decimal degrees relative to the WGS84 datum (-180.0 to less than 180.0).
    #[serde(rename = "@lon")]
    pub lon: f64,
    /// Elevation above sea level, in meters.
    #[serde(default)]
    pub ele: Option<f64>,
    /// Date and time when the point was recorded or created (ISO 8601).
    #[serde(default)]
    pub time: Option<DateTime<Utc>>,
    /// Magnetic variation (declination) at the point, in decimal degrees true (not magnetic).
    #[serde(default)]
    pub magvar: Option<f64>,
    /// Height of the geoid (mean sea level) above the WGS84 ellipsoid, in meters.
    #[serde(default)]
    pub geoidheight: Option<f64>,
    /// GPS device name for the waypoint.
    #[serde(default)]
    pub name: Option<String>,
    /// GPS device comment for the waypoint.
    #[serde(default)]
    pub cmt: Option<String>,
    /// User-defined description of the waypoint.
    #[serde(default)]
    pub desc: Option<String>,
    /// Source of the waypoint data (hardware, software, or database).
    #[serde(default)]
    pub src: Option<String>,
    /// Links to external resources related to this waypoint.
    #[serde(rename = "link", default)]
    pub links: Vec<Link>,
    /// GPS symbol name used to display the waypoint on a map.
    #[serde(default)]
    pub sym: Option<String>,
    /// Classification or category of the waypoint (GPX `<type>` element).
    #[serde(rename = "type", default)]
    pub waypoint_type: Option<String>,
    /// Type of GPS fix used to establish the position.
    #[serde(default)]
    pub fix: Option<Fix>,
    /// Number of satellites used to compute the GPS fix.
    #[serde(default)]
    pub sat: Option<u64>,
    /// Horizontal dilution of precision.
    #[serde(default)]
    pub hdop: Option<f64>,
    /// Vertical dilution of precision.
    #[serde(default)]
    pub vdop: Option<f64>,
    /// Position (3D) dilution of precision.
    #[serde(default)]
    pub pdop: Option<f64>,
    /// Age of differential GPS correction data, in seconds.
    #[serde(default)]
    pub ageofdgpsdata: Option<f64>,
    /// Identifier of the DGPS station used (0–1023).
    #[serde(default)]
    pub dgpsid: Option<u16>,
    /// Custom extension elements from namespaces outside the GPX schema.
    #[serde(default)]
    pub extensions: Option<Extensions>,
}
