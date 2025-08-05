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
}
