use serde::Deserialize;

use super::{Extensions, Link, Waypoint};

/// A track segment (`trksegType` in GPX 1.1).
///
/// A logically connected span of track points. Use separate segments when GPS
/// reception was lost or the receiver was turned off.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct TrackSegment {
    /// Ordered track points (`<trkpt>`) in this segment.
    #[serde(rename = "trkpt", default)]
    pub points: Vec<Waypoint>,
    /// Custom extension elements from namespaces outside the GPX schema.
    #[serde(default)]
    pub extensions: Option<Extensions>,
}

/// A track (`trkType` in GPX 1.1).
///
/// An ordered path made of one or more track segments describing movement over time.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct Track {
    /// GPS device name for the track.
    #[serde(default)]
    pub name: Option<String>,
    /// GPS device comment for the track.
    #[serde(default)]
    pub cmt: Option<String>,
    /// User-defined description of the track.
    #[serde(default)]
    pub desc: Option<String>,
    /// Source of the track data (hardware, software, or database).
    #[serde(default)]
    pub src: Option<String>,
    /// Links to external resources related to this track.
    #[serde(rename = "link", default)]
    pub links: Vec<Link>,
    /// GPS track number or identifier.
    #[serde(default)]
    pub number: Option<u64>,
    /// Classification or category of the track (GPX `<type>` element).
    #[serde(rename = "type", default)]
    pub track_type: Option<String>,
    /// Custom extension elements from namespaces outside the GPX schema.
    #[serde(default)]
    pub extensions: Option<Extensions>,
    /// Track segments that make up this path.
    #[serde(rename = "trkseg", default)]
    pub segments: Vec<TrackSegment>,
}
