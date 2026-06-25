/// Default moving-speed threshold: 0.5 km/h expressed in m/s.
pub const DEFAULT_MOVING_SPEED_THRESHOLD_MPS: f64 = 0.5 / 3.6;

/// Thresholds used when computing movement and elevation statistics.
#[derive(Debug, Clone, PartialEq)]
pub struct AnalysisOptions {
    /// Legs with speed at or below this value (m/s) are treated as stopped.
    ///
    /// Defaults to 0.5 km/h (`0.5 / 3.6` m/s).
    pub moving_speed_threshold_mps: f64,
    /// Elevation changes smaller than this value (m) are ignored for ascent/descent.
    pub elevation_noise_threshold_m: f64,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            moving_speed_threshold_mps: DEFAULT_MOVING_SPEED_THRESHOLD_MPS,
            elevation_noise_threshold_m: 0.0,
        }
    }
}
