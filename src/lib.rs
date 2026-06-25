//! GPX manipulation library.

pub mod gpx;

pub use gpx::{Gpx, ParseError, Route, Track, TrackSegment, Waypoint};

/// Parse a GPX document from XML text.
pub fn parse(data: &str) -> Result<Gpx, ParseError> {
    gpx::parse(data)
}
