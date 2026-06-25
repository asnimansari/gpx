use chrono::{DateTime, Utc};

use super::{Extensions, Fix, Link};

/// A waypoint, route point, or track point (`wptType` in GPX 1.1).
///
/// Represents a point of interest, named feature, route vertex, or recorded
/// track position. Used for `<wpt>`, `<rtept>`, and `<trkpt>` elements.
#[derive(Debug, Clone, PartialEq)]
pub struct Waypoint {
    /// Latitude in decimal degrees relative to the WGS84 datum (-90.0 to 90.0).
    pub lat: f64,
    /// Longitude in decimal degrees relative to the WGS84 datum (-180.0 to less than 180.0).
    pub lon: f64,
    /// Elevation above sea level, in meters.
    pub ele: Option<f64>,
    /// Date and time when the point was recorded or created (ISO 8601).
    pub time: Option<DateTime<Utc>>,
    /// Magnetic variation (declination) at the point, in decimal degrees true (not magnetic).
    pub magvar: Option<f64>,
    /// Height of the geoid (mean sea level) above the WGS84 ellipsoid, in meters.
    pub geoidheight: Option<f64>,
    /// GPS device name for the waypoint.
    pub name: Option<String>,
    /// GPS device comment for the waypoint.
    pub cmt: Option<String>,
    /// User-defined description of the waypoint.
    pub desc: Option<String>,
    /// Source of the waypoint data (hardware, software, or database).
    pub src: Option<String>,
    /// Links to external resources related to this waypoint.
    pub links: Vec<Link>,
    /// GPS symbol name used to display the waypoint on a map.
    pub sym: Option<String>,
    /// Classification or category of the waypoint (GPX `<type>` element).
    pub waypoint_type: Option<String>,
    /// Type of GPS fix used to establish the position.
    pub fix: Option<Fix>,
    /// Number of satellites used to compute the GPS fix.
    pub sat: Option<u64>,
    /// Horizontal dilution of precision.
    pub hdop: Option<f64>,
    /// Vertical dilution of precision.
    pub vdop: Option<f64>,
    /// Position (3D) dilution of precision.
    pub pdop: Option<f64>,
    /// Age of differential GPS correction data, in seconds.
    pub ageofdgpsdata: Option<f64>,
    /// Identifier of the DGPS station used (0–1023).
    pub dgpsid: Option<u16>,
    /// Custom extension elements from namespaces outside the GPX schema.
    pub extensions: Option<Extensions>,
}

impl Waypoint {
    pub(crate) fn new(lat: f64, lon: f64) -> Self {
        Self {
            lat,
            lon,
            ele: None,
            time: None,
            magvar: None,
            geoidheight: None,
            name: None,
            cmt: None,
            desc: None,
            src: None,
            links: Vec::new(),
            sym: None,
            waypoint_type: None,
            fix: None,
            sat: None,
            hdop: None,
            vdop: None,
            pdop: None,
            ageofdgpsdata: None,
            dgpsid: None,
            extensions: None,
        }
    }
}
