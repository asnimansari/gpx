//! GPX 1.1 data types.
//!
//! These structs mirror the [GPX 1.1 schema](https://www.topografix.com/GPX/1/1/).

mod bounds;
mod copyright;
mod email;
mod extensions;
mod fix;
mod gpx;
mod link;
mod metadata;
mod person;
mod point;
mod point_segment;
mod route;
mod track;
mod waypoint;

pub use bounds::Bounds;
pub use copyright::Copyright;
pub use email::Email;
pub use extensions::Extensions;
pub use fix::Fix;
pub use gpx::Gpx;
pub use link::Link;
pub use metadata::Metadata;
pub use person::Person;
pub use point::Point;
pub use point_segment::PointSegment;
pub use route::Route;
pub use track::{Track, TrackSegment};
pub use waypoint::Waypoint;
