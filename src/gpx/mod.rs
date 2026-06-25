mod error;
mod parse;
pub mod types;

pub use error::ParseError;
pub use types::{Route, Track, TrackSegment, Waypoint};

/// The root element of a GPX document.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Gpx {
    pub version: Option<String>,
    pub waypoints: Vec<Waypoint>,
    pub routes: Vec<Route>,
    pub tracks: Vec<Track>,
}

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
