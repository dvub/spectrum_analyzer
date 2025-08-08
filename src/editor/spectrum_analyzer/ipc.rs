use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::editor::spectrum_analyzer::monitor::MonitorMode;

#[derive(Deserialize, Serialize, TS, Debug)]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
#[ts(export)]
pub enum SpectrumAnalyzerConfigUpdate {
    Fps(f32),
    MonitorMode(MonitorMode),
    DecaySpeed(f32),
    Interpolate(bool), /*
                       TODO !! add these config options
                       interpolate: bool,
                       slope: f32,
                       frequency_range: (f32, f32),
                       magnitude_range: (f32, f32),
                       */
}
