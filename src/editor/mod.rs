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
    sample_rx: Receiver<f32>,
    graph: Box<dyn AudioUnit>,
    spectrum: Arc<Mutex<Vec<f32>>>,
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

    fn tick(&mut self) {
        for sample in self.sample_rx.try_iter() {
            self.graph.tick(&[sample], &mut []);
        }
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
    let spectrum = Arc::new(Mutex::new(Vec::new()));
    WebViewEditor::new_with_webview(
        PluginGui {
            sample_rx: rx,
            graph: build_fft_graph(spectrum.clone()),
            spectrum,
        },
        state,
        config,
        |builder| builder.with_devtools(true),
    )
}

impl EditorHandler for PluginGui {
    // SUPER IMPORTANT
    fn on_frame(&mut self, cx: &mut nih_plug_webview::Context) {
        // process (take FFT) of everything that's come from the DSP thread since the last frame
        self.tick();
        // grab spectrum
        let spectrum = &*self.spectrum.lock().unwrap();
        // do any expensive processing such as interpolation
        // we want to give our GUI as little work as possible
        todo!();
        // send to GUI
        let message = Message::DrawData(DrawData::Spectrum(spectrum.clone()));
        cx.send_message(json!(message).to_string());
    }

    fn on_message(&mut self, _: &mut nih_plug_webview::Context, _: String) {}

    fn on_params_changed(&mut self, _: &mut nih_plug_webview::Context) {}
}

const WINDOW_LENGTH: usize = 1024;

fn build_fft_graph(spectrum: Arc<Mutex<Vec<f32>>>) -> Box<dyn AudioUnit> {
    let fft_processor = resynth::<U1, U0, _>(WINDOW_LENGTH, move |fft| {
        let mut temp_spectrum = vec![0.0; fft.bins()];

        #[allow(clippy::needless_range_loop)]
        for i in 0..fft.bins() {
            let current_bin = fft.at(0, i);
            let normalization = WINDOW_LENGTH as f32;

            temp_spectrum[i] = current_bin.norm() / normalization;
        }
        *spectrum.lock().unwrap() = temp_spectrum;
    });
    Box::new(fft_processor)
}
