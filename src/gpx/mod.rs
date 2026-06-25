pub mod analysis;
pub mod convert;
pub mod extensions;
pub mod info;
pub mod operations;
pub mod serialize;
pub mod validation;

mod error;
mod geo;
mod parse;
pub mod types;

pub use analysis::{AnalysisOptions, ProfilePoint, SpeedProfilePoint, WaypointPath};
pub use extensions::{
    PowerExtension, TrackExtension, TrackPointExtension, GPXTPTX_NS_V1, GPXTPTX_NS_V2, GPXX_NS_V3,
    GPPXPX_NS_V1,
};
pub use convert::{
    convert_file, detect_format, read_geojson, read_gpx, read_kml, write_geojson, write_kml,
    ConvertError,
};
pub use error::ParseError;
pub use info::{gather, print_human, BoundsInfo, ElevationInfo, GpxInfo, RouteInfo, TrackInfo};
pub use operations::{
    crop, filter_points, merge, merge_with_creator, reduce_precision, reverse, shift_time,
    simplify, smooth, smooth_with_options, split, strip_extensions, strip_metadata, trim,
    OperationError, SmoothOptions, StripMetadataFields,
};
pub use serialize::{to_string, write_file};
pub use validation::{
    validate_file, validate_str, InvalidGpxError, Severity, ValidationIssue, ValidationResult,
};
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
}

/// Parse a GPX document from XML text.
pub fn parse(data: &str) -> Result<Gpx, ParseError> {
    parse::parse_gpx(data)
}
