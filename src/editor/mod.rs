mod embedded;
mod ipc;

use std::{path::PathBuf, sync::Arc};

#[cfg(not(debug_assertions))]
use crate::editor::embedded::embedded_editor;

use nih_plug::editor::Editor;
use nih_plug_webview::{EditorHandler, WebViewConfig, WebViewEditor, WebViewSource, WebViewState};

pub struct PluginGui {}

impl PluginGui {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(state: &Arc<WebViewState>) -> Option<Box<dyn Editor>> {
        #[cfg(debug_assertions)]
        let editor = dev_editor(state);

        #[cfg(not(debug_assertions))]
        let editor = embedded_editor(state);

        Some(Box::new(editor))
    }
}
fn dev_editor(state: &Arc<WebViewState>) -> WebViewEditor {
    let config = WebViewConfig {
        title: "Convolution".to_string(),
        source: WebViewSource::URL(String::from("http://localhost:3000")),
        workdir: PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/target/webview-workdir"
        )),
    };

    WebViewEditor::new_with_webview(PluginGui {}, state, config, |builder| {
        builder.with_devtools(true)
    })
}
impl EditorHandler for PluginGui {
    fn on_frame(&mut self, _: &mut nih_plug_webview::Context) {}

    fn on_message(&mut self, _: &mut nih_plug_webview::Context, _: String) {}

    fn on_params_changed(&mut self, _: &mut nih_plug_webview::Context) {}
}
