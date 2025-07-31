mod embedded;
mod ipc;

use std::{path::PathBuf, sync::Arc};

#[cfg(not(debug_assertions))]
use crate::editor::embedded::embedded_editor;
use crate::editor::ipc::{DrawData, Message};

use crossbeam_channel::{Receiver, Sender};
use fundsp::hacker::AudioUnit;
use nih_plug::editor::Editor;
use nih_plug_webview::{EditorHandler, WebViewConfig, WebViewEditor, WebViewSource, WebViewState};
use serde_json::json;

use fundsp::hacker32::*;

pub struct PluginGui {
    rx: Receiver<f32>,
    ffter: Box<dyn AudioUnit>,
}

impl PluginGui {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(state: &Arc<WebViewState>, rx: Receiver<f32>) -> Option<Box<dyn Editor>> {
        #[cfg(debug_assertions)]
        let editor = dev_editor(state, rx);

        #[cfg(not(debug_assertions))]
        let editor = embedded_editor(state);

        Some(Box::new(editor))
    }
}
fn dev_editor(state: &Arc<WebViewState>, rx: Receiver<f32>) -> WebViewEditor {
    let config = WebViewConfig {
        title: "Convolution".to_string(),
        source: WebViewSource::URL(String::from("http://localhost:3000")),
        workdir: PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/target/webview-workdir"
        )),
    };

    WebViewEditor::new_with_webview(
        PluginGui {
            rx,
            ffter: fft_graph(),
        },
        state,
        config,
        |builder| builder.with_devtools(true),
    )
}
impl EditorHandler for PluginGui {
    fn on_frame(&mut self, cx: &mut nih_plug_webview::Context) {
        for s in self.rx.try_iter() {
            self.ffter.tick(&[s], &mut []);
        }
    }

    fn on_message(&mut self, _: &mut nih_plug_webview::Context, _: String) {}

    fn on_params_changed(&mut self, _: &mut nih_plug_webview::Context) {}
}

fn fft_graph() -> Box<dyn AudioUnit> {
    let graph = pass();
    Box::new(graph)
}
