mod embedded;
mod ipc;
mod monitor;

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

#[cfg(not(debug_assertions))]
use crate::editor::embedded::embedded_editor;
use crate::editor::{
    ipc::{DrawData, Message},
    monitor::{Meter, Monitor},
};

use crossbeam_channel::Receiver;
use fundsp::hacker::AudioUnit;
use nih_plug::editor::Editor;
use nih_plug_webview::{EditorHandler, WebViewConfig, WebViewEditor, WebViewSource, WebViewState};

use fundsp::hacker32::*;
use serde_json::json;

pub struct PluginGui {
    sample_rx: Receiver<f32>,

    // spectrum stuff
    graph: Box<dyn AudioUnit>,

    spectrum_monitors: Arc<Mutex<Vec<Monitor>>>,
}

impl PluginGui {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        state: &Arc<WebViewState>,
        rx: Receiver<f32>,
        sample_rate: f32,
    ) -> Option<Box<dyn Editor>> {
        #[cfg(debug_assertions)]
        let editor = dev_editor(state, rx, sample_rate);

        #[cfg(not(debug_assertions))]
        let editor = embedded_editor(state, rx, sample_rate);

        Some(Box::new(editor))
    }

    fn tick(&mut self) {
        for sample in self.sample_rx.try_iter() {
            self.graph.tick(&[sample], &mut []);
        }
    }
}

const WINDOW_LENGTH: usize = 1024;
const MONITOR_LEN: usize = (WINDOW_LENGTH / 2) + 1;

fn dev_editor(state: &Arc<WebViewState>, rx: Receiver<f32>, sample_rate: f32) -> WebViewEditor {
    let config = WebViewConfig {
        title: "Convolution".to_string(),
        source: WebViewSource::URL(String::from("http://localhost:3000")),
        workdir: PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/target/webview-workdir"
        )),
    };

    let spectrum_monitors = Arc::new(Mutex::new(vec![
        Monitor::new(Meter::Peak, 1.5, 60.0);
        MONITOR_LEN
    ]));

    let mut graph = build_fft_graph(spectrum_monitors.clone());

    graph.set_sample_rate(sample_rate as f64);

    WebViewEditor::new_with_webview(
        PluginGui {
            sample_rx: rx,
            graph,
            spectrum_monitors,
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

        // TODO: jesus christ what is this
        let processed: Vec<_> = self
            .spectrum_monitors
            .lock()
            .unwrap()
            .iter()
            .map(|x| x.level())
            .collect();

        // send to GUI
        let message = Message::DrawData(DrawData::Spectrum(processed));
        cx.send_message(json!(message).to_string());
    }

    fn on_message(&mut self, _: &mut nih_plug_webview::Context, _: String) {}

    fn on_params_changed(&mut self, _: &mut nih_plug_webview::Context) {}
}

fn build_fft_graph(spectrum_monitors: Arc<Mutex<Vec<Monitor>>>) -> Box<dyn AudioUnit> {
    let fft_processor = resynth::<U1, U0, _>(WINDOW_LENGTH, move |fft| {
        let mut monitors = spectrum_monitors.lock().unwrap();

        #[allow(clippy::needless_range_loop)]
        for i in 0..fft.bins() {
            let current_bin = fft.at(0, i);
            let normalization = WINDOW_LENGTH as f32;

            let value = current_bin.norm() / normalization;
            monitors[i].tick(value);
        }
    });

    Box::new(fft_processor)
}
