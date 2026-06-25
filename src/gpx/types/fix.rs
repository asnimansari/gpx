/// Type of GPS fix (`fixType` in GPX 1.1).
///
/// Indicates the dimensionality and source of the GPS position fix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fix {
    /// GPS had no fix.
    None,
    /// 2D fix (latitude and longitude only).
    TwoD,
    /// 3D fix (latitude, longitude, and elevation).
    ThreeD,
    /// Differential GPS fix.
    Dgps,
    /// Military GPS signal used.
    Pps,
}
