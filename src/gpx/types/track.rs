use super::Waypoint;

/// A segment of a track, made up of track points.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TrackSegment {
    pub points: Vec<Waypoint>,
}

/// A path made of one or more track segments.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Track {
    pub name: Option<String>,
    pub segments: Vec<TrackSegment>,
}
