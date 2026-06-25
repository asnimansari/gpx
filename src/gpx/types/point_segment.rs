use super::Point;

/// An ordered sequence of points (`ptsegType` in GPX 1.1).
///
/// Used to represent polygons, polylines, and similar geometries in extension schemas.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PointSegment {
    /// Ordered geographic points in the segment.
    pub points: Vec<Point>,
}
