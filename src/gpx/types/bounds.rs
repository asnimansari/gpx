/// Geographic bounds (`boundsType` in GPX 1.1).
///
/// Defines the rectangular extent of a GPX file using two latitude/longitude pairs.
#[derive(Debug, Clone, PartialEq)]
pub struct Bounds {
    /// Minimum latitude of the bounding box (decimal degrees, WGS84).
    pub minlat: f64,
    /// Minimum longitude of the bounding box (decimal degrees, WGS84).
    pub minlon: f64,
    /// Maximum latitude of the bounding box (decimal degrees, WGS84).
    pub maxlat: f64,
    /// Maximum longitude of the bounding box (decimal degrees, WGS84).
    pub maxlon: f64,
}
