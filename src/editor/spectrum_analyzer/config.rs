use crate::editor::spectrum_analyzer::monitor::MonitorMode;
const DEFAULT_FREQ_RANGE: (f32, f32) = (10.0, 20_000.0); // hz
const DEFAULT_MAGNITUDE_RANGE: (f32, f32) = (-100.0, 6.0); // db
const DEFAULT_SLOPE: f32 = 4.5; // db/oct (or at least should be)
const DEFAULT_PEAK_DECAY: f32 = 0.25; // seconds
const DEFAULT_INTERPOLATION: bool = true;
pub const DEFAULT_MONITOR_MODE: MonitorMode = MonitorMode::Rms(DEFAULT_PEAK_DECAY);

pub struct SpectrumAnalyzerConfig {
    pub interpolate: bool,
    pub slope: f32,
    pub frequency_range: (f32, f32),
    pub magnitude_range: (f32, f32),
}

impl Default for SpectrumAnalyzerConfig {
    fn default() -> Self {
        Self {
            interpolate: DEFAULT_INTERPOLATION,
            frequency_range: DEFAULT_FREQ_RANGE,
            magnitude_range: DEFAULT_MAGNITUDE_RANGE,
            slope: DEFAULT_SLOPE,
        }
    }
}
