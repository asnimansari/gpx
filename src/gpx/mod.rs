pub mod analysis;

mod error;
mod geo;
mod parse;
pub mod types;

pub use analysis::{AnalysisOptions, ProfilePoint, SpeedProfilePoint, WaypointPath};
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

    /// Parse a GPX document from a file path.
    pub fn parse_file(path: impl AsRef<std::path::Path>) -> Result<Self, ParseError> {
        let data = std::fs::read_to_string(path)?;
        Self::parse(&data)
    }

    /// Name of the GPX file (from metadata).
    pub fn name(&self) -> Option<&str> {
        self.metadata.as_ref()?.name.as_deref()
    }

    /// Description of the GPX file contents (from metadata).
    pub fn desc(&self) -> Option<&str> {
        self.metadata.as_ref()?.desc.as_deref()
    }

    /// Person or organization who created the GPX file (from metadata).
    pub fn author(&self) -> Option<&types::Person> {
        self.metadata.as_ref()?.author.as_ref()
    }

    /// Copyright and license information (from metadata).
    pub fn copyright(&self) -> Option<&types::Copyright> {
        self.metadata.as_ref()?.copyright.as_ref()
    }

    /// Links associated with the file (from metadata).
    pub fn links(&self) -> Option<&[types::Link]> {
        self.metadata.as_ref().map(|metadata| metadata.links.as_slice())
    }

    /// Creation date of the file (from metadata).
    pub fn time(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.metadata.as_ref()?.time
    }

    /// Keywords associated with the file (from metadata).
    pub fn keywords(&self) -> Option<&str> {
        self.metadata.as_ref()?.keywords.as_deref()
    }

    /// Geographic bounds of the file (from metadata).
    pub fn bounds(&self) -> Option<&types::Bounds> {
        self.metadata.as_ref()?.bounds.as_ref()
    }
}

/// Parse a GPX document from XML text.
pub fn parse(data: &str) -> Result<Gpx, ParseError> {
    parse::parse_gpx(data)
}
