use std::sync::Arc;

use nih_plug::prelude::*;
use nih_plug_webview::WebViewState;

#[derive(Params)]
pub struct SpectrumAnalyzerParams {
    #[persist = "webview_state"]
    pub state: Arc<WebViewState>,
}

impl Default for SpectrumAnalyzerParams {
    fn default() -> Self {
        Self {
            state: Arc::new(WebViewState::new(600.0, 600.0)),
        }
    }
}
