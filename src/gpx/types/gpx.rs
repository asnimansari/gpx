use super::{Extensions, Metadata, Route, Track, Waypoint};

/// The root GPX document (`gpxType` in GPX 1.1).
///
/// Contains optional metadata followed by waypoints, routes, and tracks.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Gpx {
    /// GPX schema version. GPX 1.1 files use `"1.1"`.
    pub version: Option<String>,
    /// Name of the software or service that created the file.
    pub creator: Option<String>,
    /// Metadata describing the file, author, and geographic extent.
    pub metadata: Option<Metadata>,
    /// Standalone waypoints (`<wpt>`) in the document.
    pub waypoints: Vec<Waypoint>,
    /// Routes (`<rte>`) in the document.
    pub routes: Vec<Route>,
    /// Tracks (`<trk>`) in the document.
    pub tracks: Vec<Track>,
    /// Custom extension elements from namespaces outside the GPX schema.
    pub extensions: Option<Extensions>,
}
