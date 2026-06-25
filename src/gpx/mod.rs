mod error;
mod parse;
pub mod types;

pub use error::ParseError;
pub use types::{
    Bounds, Copyright, Email, Extensions, Fix, Gpx, Link, Metadata, Person, Point,
    PointSegment, Route, Track, TrackSegment, Waypoint,
};

impl Gpx {
    /// Parse a GPX document from XML text.
    pub fn parse(data: &str) -> Result<Self, ParseError> {
        parse::parse_gpx(data)
    }
}

/// Parse a GPX document from XML text.
pub fn parse(data: &str) -> Result<Gpx, ParseError> {
    parse::parse_gpx(data)
}
