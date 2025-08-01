mod embedded;
mod ipc;

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

#[cfg(not(debug_assertions))]
use crate::editor::embedded::embedded_editor;
use crate::editor::ipc::{DrawData, Message};

use crossbeam_channel::Receiver;
use fundsp::hacker::AudioUnit;
use nih_plug::editor::Editor;
use nih_plug_webview::{EditorHandler, WebViewConfig, WebViewEditor, WebViewSource, WebViewState};

use fundsp::hacker32::*;
use serde_json::json;

pub struct PluginGui {
    rx: Receiver<f32>,
    ffter: Box<dyn AudioUnit>,
    x: Arc<Mutex<Vec<f32>>>,
}

impl PluginGui {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(state: &Arc<WebViewState>, rx: Receiver<f32>) -> Option<Box<dyn Editor>> {
        #[cfg(debug_assertions)]
        let editor = dev_editor(state, rx);

        #[cfg(not(debug_assertions))]
        let editor = embedded_editor(state, rx);

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
    let x = Arc::new(Mutex::new(Vec::new()));
    WebViewEditor::new_with_webview(
        PluginGui {
            rx,
            ffter: fft_graph(x.clone()),
            x,
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

        let x = &*self.x.lock().unwrap();
        let message = Message::DrawData(DrawData::Spectrum(x.clone()));
        cx.send_message(json!(message).to_string());
    }

    fn on_message(&mut self, _: &mut nih_plug_webview::Context, _: String) {}

    fn on_params_changed(&mut self, _: &mut nih_plug_webview::Context) {}
}

fn fft_graph(output: Arc<Mutex<Vec<f32>>>) -> Box<dyn AudioUnit> {
    let fft_processor = resynth::<U1, U0, _>(1024, move |fft| {
        let mut y = vec![0.0; fft.bins()];

        for i in 0..fft.bins() {
            let current_bin = fft.at(0, i);
            y[i] = current_bin.norm();
        }
        *output.lock().unwrap() = y;
    });

    let graph = pass() >> fft_processor;

    Box::new(graph)
}
