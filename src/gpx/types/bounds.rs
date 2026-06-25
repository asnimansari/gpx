use serde::Deserialize;

/// Geographic bounds (`boundsType` in GPX 1.1).
///
/// Defines the rectangular extent of a GPX file using two latitude/longitude pairs.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Bounds {
    /// Minimum latitude of the bounding box (decimal degrees, WGS84).
    #[serde(rename = "@minlat")]
    pub minlat: f64,
    /// Minimum longitude of the bounding box (decimal degrees, WGS84).
    #[serde(rename = "@minlon")]
    pub minlon: f64,
    /// Maximum latitude of the bounding box (decimal degrees, WGS84).
    #[serde(rename = "@maxlat")]
    pub maxlat: f64,
    /// Maximum longitude of the bounding box (decimal degrees, WGS84).
    #[serde(rename = "@maxlon")]
    pub maxlon: f64,
}
