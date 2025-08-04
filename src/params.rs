use nih_plug::prelude::*;
use nih_plug_webview::WebViewState;
use std::sync::Arc;

#[derive(Params)]
pub struct PluginParams {
    #[persist = "webview_state"]
    pub state: Arc<WebViewState>,
}

impl Default for PluginParams {
    fn default() -> Self {
        Self {
            state: Arc::new(WebViewState::new(600.0, 600.0)),
        }
    }
}
