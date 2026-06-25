use serde::{Deserialize, Serialize};

use crate::gpx::extensions::{PowerExtension, TrackExtension, TrackPointExtension};

/// Extension elements from namespaces other than the GPX schema (`extensionsType`).
///
/// Supports typed Garmin / Strava extensions alongside opaque XML for unknown vendors.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
pub struct Extensions {
    /// Garmin / Strava track point extension (`gpxtpx:TrackPointExtension`).
    #[serde(rename = "TrackPointExtension", default, skip_serializing_if = "Option::is_none")]
    pub track_point: Option<TrackPointExtension>,
    /// Strava-style instantaneous power in watts (`<power>watts</power>`).
    #[serde(rename = "power", default, skip_serializing_if = "Option::is_none")]
    pub power: Option<u16>,
    /// Garmin power extension (`gpxpx:PowerExtension`).
    #[serde(rename = "PowerExtension", default, skip_serializing_if = "Option::is_none")]
    pub power_extension: Option<PowerExtension>,
    /// Garmin track extension (`gpxx:TrackExtension`).
    #[serde(rename = "TrackExtension", default, skip_serializing_if = "Option::is_none")]
    pub track: Option<TrackExtension>,
    /// Raw inner XML for extension elements that are not modeled above.
    ///
    /// Preserved for round-trip when reading vendor-specific or future extensions.
    #[serde(default, rename = "$value", skip_serializing_if = "String::is_empty")]
    pub inner_xml: String,
}

impl Extensions {
    /// Heart rate from the track point extension, if present.
    pub fn heart_rate(&self) -> Option<u8> {
        self.track_point.as_ref()?.hr
    }

    /// Cadence from the track point extension, if present.
    pub fn cadence(&self) -> Option<u8> {
        self.track_point.as_ref()?.cad
    }

    /// Power in watts from Strava `<power>` or Garmin `PowerExtension`, if present.
    pub fn power_watts(&self) -> Option<u16> {
        self.power
            .or_else(|| self.power_extension.as_ref()?.power_in_watts)
    }

    /// Whether any typed or raw extension content is present.
    pub fn is_empty(&self) -> bool {
        self.track_point.is_none()
            && self.power.is_none()
            && self.power_extension.is_none()
            && self.track.is_none()
            && self.inner_xml.is_empty()
    }
}
