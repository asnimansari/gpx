use super::{Extensions, Link, Waypoint};

/// A track segment (`trksegType` in GPX 1.1).
///
/// A logically connected span of track points. Use separate segments when GPS
/// reception was lost or the receiver was turned off.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TrackSegment {
    /// Ordered track points (`<trkpt>`) in this segment.
    pub points: Vec<Waypoint>,
    /// Custom extension elements from namespaces outside the GPX schema.
    pub extensions: Option<Extensions>,
}

/// A track (`trkType` in GPX 1.1).
///
/// An ordered path made of one or more track segments describing movement over time.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Track {
    /// GPS device name for the track.
    pub name: Option<String>,
    /// GPS device comment for the track.
    pub cmt: Option<String>,
    /// User-defined description of the track.
    pub desc: Option<String>,
    /// Source of the track data (hardware, software, or database).
    pub src: Option<String>,
    /// Links to external resources related to this track.
    pub links: Vec<Link>,
    /// GPS track number or identifier.
    pub number: Option<u64>,
    /// Classification or category of the track (GPX `<type>` element).
    pub track_type: Option<String>,
    /// Custom extension elements from namespaces outside the GPX schema.
    pub extensions: Option<Extensions>,
    /// Track segments that make up this path.
    pub segments: Vec<TrackSegment>,
}
