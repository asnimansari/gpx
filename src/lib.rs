//! GPX manipulation library.

pub mod gpx;

pub use gpx::{
    AnalysisOptions, Bounds, Copyright, Email, Extensions, Fix, Gpx, Link, Metadata, ParseError,
    Person, Point, PointSegment, ProfilePoint, Route, SpeedProfilePoint, Track, TrackSegment,
    Waypoint, WaypointPath,
};

/// Parse a GPX document from XML text.
pub fn parse(data: &str) -> Result<Gpx, ParseError> {
    gpx::parse(data)
}
