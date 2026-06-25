use super::Waypoint;

/// A planned path described by an ordered sequence of route points.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Route {
    pub name: Option<String>,
    pub points: Vec<Waypoint>,
}
