use super::{Extensions, Link, Waypoint};

/// A route (`rteType` in GPX 1.1).
///
/// An ordered list of waypoints representing turn points leading to a destination.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Route {
    /// GPS device name for the route.
    pub name: Option<String>,
    /// GPS device comment for the route.
    pub cmt: Option<String>,
    /// User-defined description of the route.
    pub desc: Option<String>,
    /// Source of the route data (hardware, software, or database).
    pub src: Option<String>,
    /// Links to external resources related to this route.
    pub links: Vec<Link>,
    /// GPS route number or identifier.
    pub number: Option<u64>,
    /// Classification or category of the route (GPX `<type>` element).
    pub route_type: Option<String>,
    /// Custom extension elements from namespaces outside the GPX schema.
    pub extensions: Option<Extensions>,
    /// Ordered route points (`<rtept>`) that define the path.
    pub points: Vec<Waypoint>,
}
