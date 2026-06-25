//! GPX manipulation library.

pub mod gpx;

pub use gpx::{
    convert_file, crop, detect_format, filter_points, gather, merge, merge_with_creator,
    print_human, read_geojson, read_gpx, read_kml, reduce_precision, reverse, shift_time,
    simplify, smooth, smooth_with_options, split, strip_extensions, strip_metadata, to_string,
    trim, validate_file, validate_str, write_file, write_geojson, write_kml, AnalysisOptions,
    Bounds, BoundsInfo, ConvertError, Copyright, ElevationInfo, Email, Extensions, Fix, Gpx,
    GpxInfo, GPXTPTX_NS_V1, GPXTPTX_NS_V2, GPXX_NS_V3, GPPXPX_NS_V1, InvalidGpxError, Link,
    Metadata, OperationError, ParseError, Person, Point, PointSegment, PowerExtension,
    ProfilePoint, Route, RouteInfo, Severity, SmoothOptions, SpeedProfilePoint,
    StripMetadataFields, Track, TrackExtension, TrackInfo, TrackPointExtension, TrackSegment, ValidationIssue, ValidationResult,
    Waypoint, WaypointPath,
};

/// Parse a GPX document from XML text.
pub fn parse(data: &str) -> Result<Gpx, ParseError> {
    gpx::parse(data)
}

/// Parse a GPX document from a file path.
pub fn parse_file(path: impl AsRef<std::path::Path>) -> Result<Gpx, ParseError> {
    Gpx::parse_file(path)
}
