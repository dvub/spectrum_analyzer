use serde::{Deserialize, Serialize};
use ts_rs::TS;

// NOTE: im not exactly sure why, but if we use
// #[ts(export, rename_all = ...)]
// instead of serde, things do not work

// unfortunately this prevents a lot of this code from looking cleaner
#[derive(Serialize, Deserialize, TS, Debug)]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
#[ts(export)]
pub enum Message {
    Init,
    Resize { width: f64, height: f64 },
}
