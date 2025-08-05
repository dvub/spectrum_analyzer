use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::editor::spectrum_analyzer::ipc::SpectrumAnalyzerConfigUpdate;

#[derive(Serialize, Deserialize, TS, Debug)]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
#[ts(export)]
pub enum Message {
    Init,
    Resize { width: f64, height: f64 },
    DrawData(DrawData),
    DrawRequest(DrawRequest),
    SpectrumAnalyzerConfigUpdate(SpectrumAnalyzerConfigUpdate),
}
#[derive(Serialize, Deserialize, TS, Debug)]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
#[ts(export)]
pub enum DrawData {
    Spectrum(Vec<(f32, f32)>),
}
#[derive(Serialize, Deserialize, TS, Debug)]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
#[ts(export)]
pub enum DrawRequest {
    Spectrum,
}
