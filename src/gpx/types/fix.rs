use serde::Deserialize;

/// Type of GPS fix (`fixType` in GPX 1.1).
///
/// Indicates the dimensionality and source of the GPS position fix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Fix {
    /// GPS had no fix.
    #[serde(rename = "none")]
    None,
    /// 2D fix (latitude and longitude only).
    #[serde(rename = "2d")]
    TwoD,
    /// 3D fix (latitude, longitude, and elevation).
    #[serde(rename = "3d")]
    ThreeD,
    /// Differential GPS fix.
    #[serde(rename = "dgps")]
    Dgps,
    /// Military GPS signal used.
    #[serde(rename = "pps")]
    Pps,
}
