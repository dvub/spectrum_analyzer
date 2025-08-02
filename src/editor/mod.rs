mod embedded;
mod ipc;
mod monitor;

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Instant,
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

    last_call: Instant,
    last_fps: f32,
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

    // this actually sucks
    fn update_fps(&mut self) {
        let last_call = self.last_call;
        let current = Instant::now();

        let diff = current - last_call;
        let diff_ms = diff.as_millis();

        self.last_call = current;

        if diff_ms == 0 {
            return;
        }
        // TODO: is there a precision issue?
        let fps = (1000 / diff_ms) as f32;

        if fps != self.last_fps {
            for monitor in &mut *self.spectrum_monitors.lock().unwrap() {
                monitor.set_frame_rate(fps);
            }
        }
        self.last_fps = fps;
    }
}

const WINDOW_LENGTH: usize = 1024;
const NUM_MONITORS: usize = (WINDOW_LENGTH / 2) + 1;

// in seconds!
const DEFAULT_PEAK_DECAY: f32 = 1.5;
const DEFAULT_MODE: Meter = Meter::Peak(DEFAULT_PEAK_DECAY);

fn dev_editor(state: &Arc<WebViewState>, rx: Receiver<f32>, sample_rate: f32) -> WebViewEditor {
    let config = WebViewConfig {
        title: "Convolution".to_string(),
        source: WebViewSource::URL(String::from("http://localhost:3000")),
        workdir: PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/target/webview-workdir"
        )),
    };

    let spectrum_monitors = Arc::new(Mutex::new(vec![Monitor::new(DEFAULT_MODE); NUM_MONITORS]));

    let mut graph = build_fft_graph(spectrum_monitors.clone());
    // TODO: does this actually... matter?
    graph.set_sample_rate(sample_rate as f64);

    WebViewEditor::new_with_webview(
        // TODO: refactor
        PluginGui {
            sample_rx: rx,
            graph,
            spectrum_monitors,
            last_call: Instant::now(),
            last_fps: 0.0,
        },
        state,
        config,
        |builder| builder.with_devtools(true),
    )
}

impl EditorHandler for PluginGui {
    fn on_frame(&mut self, cx: &mut nih_plug_webview::Context) {
        self.update_fps();
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

        // TODO: !!! go back to using intermediate vec here.
        // THEN, in on_frame(), only tick each monitor once, which allows those to have a somewhat consistent framerate.

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
