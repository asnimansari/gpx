use chrono::{DateTime, Utc};

/// A geographic point with optional metadata.
///
/// Used for GPX `<wpt>`, `<rtept>`, and `<trkpt>` elements.
#[derive(Debug, Clone, PartialEq)]
pub struct Waypoint {
    pub lat: f64,
    pub lon: f64,
    pub ele: Option<f64>,
    pub time: Option<DateTime<Utc>>,
    pub name: Option<String>,
}
