use serde::Deserialize;

use super::{Extensions, Link, Waypoint};

/// A route (`rteType` in GPX 1.1).
///
/// An ordered list of waypoints representing turn points leading to a destination.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct Route {
    /// GPS device name for the route.
    #[serde(default)]
    pub name: Option<String>,
    /// GPS device comment for the route.
    #[serde(default)]
    pub cmt: Option<String>,
    /// User-defined description of the route.
    #[serde(default)]
    pub desc: Option<String>,
    /// Source of the route data (hardware, software, or database).
    #[serde(default)]
    pub src: Option<String>,
    /// Links to external resources related to this route.
    #[serde(rename = "link", default)]
    pub links: Vec<Link>,
    /// GPS route number or identifier.
    #[serde(default)]
    pub number: Option<u64>,
    /// Classification or category of the route (GPX `<type>` element).
    #[serde(rename = "type", default)]
    pub route_type: Option<String>,
    /// Custom extension elements from namespaces outside the GPX schema.
    #[serde(default)]
    pub extensions: Option<Extensions>,
    /// Ordered route points (`<rtept>`) that define the path.
    #[serde(rename = "rtept", default)]
    pub points: Vec<Waypoint>,
}
