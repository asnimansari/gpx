use serde::Deserialize;

use super::{Extensions, Metadata, Route, Track, Waypoint};

/// The root GPX document (`gpxType` in GPX 1.1).
///
/// Contains optional metadata followed by waypoints, routes, and tracks.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
#[serde(rename = "gpx")]
pub struct Gpx {
    /// GPX schema version. GPX 1.1 files use `"1.1"`.
    #[serde(rename = "@version", default)]
    pub version: Option<String>,
    /// Name of the software or service that created the file.
    #[serde(rename = "@creator", default)]
    pub creator: Option<String>,
    /// Metadata describing the file, author, and geographic extent.
    #[serde(default)]
    pub metadata: Option<Metadata>,
    /// Standalone waypoints (`<wpt>`) in the document.
    #[serde(rename = "wpt", default)]
    pub waypoints: Vec<Waypoint>,
    /// Routes (`<rte>`) in the document.
    #[serde(rename = "rte", default)]
    pub routes: Vec<Route>,
    /// Tracks (`<trk>`) in the document.
    #[serde(rename = "trk", default)]
    pub tracks: Vec<Track>,
    /// Custom extension elements from namespaces outside the GPX schema.
    #[serde(default)]
    pub extensions: Option<Extensions>,
}
