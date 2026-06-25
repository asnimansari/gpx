use crate::gpx::types::Waypoint;

const EARTH_RADIUS_M: f64 = 6_378_137.0;

/// Great-circle distance between two WGS84 coordinates, in meters.
pub fn haversine_m(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let phi1 = lat1.to_radians();
    let phi2 = lat2.to_radians();
    let dphi = (lat2 - lat1).to_radians();
    let dlambda = (lon2 - lon1).to_radians();
    let a = (dphi / 2.0).sin().powi(2) + phi1.cos() * phi2.cos() * (dlambda / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    EARTH_RADIUS_M * c
}

/// Haversine distance between two waypoints, in meters.
pub fn distance_between(a: &Waypoint, b: &Waypoint) -> f64 {
    haversine_m(a.lat, a.lon, b.lat, b.lon)
}
