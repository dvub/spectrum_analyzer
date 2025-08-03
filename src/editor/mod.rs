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
    ipc::{DrawData, DrawRequest, Message},
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

    spectrum: Arc<Mutex<Vec<f32>>>,
    spectrum_monitors: Vec<Monitor>,

    fps: f32,
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
    fn set_frame_rate(&mut self, frame_rate: f32) {
        for mon in self.spectrum_monitors.iter_mut() {
            mon.set_frame_rate(frame_rate);
        }
        self.fps = frame_rate;
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

    let spectrum_monitors = vec![Monitor::new(DEFAULT_MODE); NUM_MONITORS];

    let spectrum = Arc::new(Mutex::new(vec![0.0; NUM_MONITORS]));

    let mut graph = build_fft_graph(spectrum.clone());
    // TODO: does this actually... matter?
    graph.set_sample_rate(sample_rate as f64);

    WebViewEditor::new_with_webview(
        // TODO: refactor
        PluginGui {
            sample_rx: rx,
            graph,
            spectrum_monitors,

            spectrum,
            fps: 0.0,
        },
        state,
        config,
        |builder| builder.with_devtools(true),
    )
}

impl EditorHandler for PluginGui {
    fn on_frame(&mut self, _: &mut nih_plug_webview::Context) {}

    fn on_message(&mut self, cx: &mut nih_plug_webview::Context, message: String) {
        let message: Message = serde_json::from_str(&message).expect("Error reading message");
        match message {
            Message::Init => {}
            Message::Resize { width, height } => {
                // TODO: handle the bool that's returned from this?
                cx.resize_window(width, height);
            }
            Message::DrawRequest(draw_request) => match draw_request {
                DrawRequest::Spectrum(frame_rate) => {
                    if frame_rate != self.fps {
                        self.set_frame_rate(frame_rate);
                    }
                    // process (take FFT) of everything that's come from the DSP thread since the last frame
                    self.tick();

                    // TODO: refactor
                    let spectrum = &*self.spectrum.lock().unwrap();
                    let processed: Vec<_> = self
                        .spectrum_monitors
                        .iter_mut()
                        .enumerate()
                        .map(|(i, x)| {
                            x.tick(spectrum[i]);
                            x.level()
                        })
                        .collect();

                    // send to GUI
                    let message = Message::DrawData(DrawData::Spectrum(processed));
                    cx.send_message(json!(message).to_string());
                }
            },
            Message::DrawData(_) => todo!(),
        }
    }

    fn on_params_changed(&mut self, _: &mut nih_plug_webview::Context) {}
}

fn build_fft_graph(spectrum: Arc<Mutex<Vec<f32>>>) -> Box<dyn AudioUnit> {
    let fft_processor = resynth::<U1, U0, _>(WINDOW_LENGTH, move |fft| {
        let mut spectrum = spectrum.lock().unwrap();

        // TODO: !!! go back to using intermediate vec here.
        // THEN, in on_frame(), only tick each monitor once, which allows those to have a somewhat consistent framerate.

        #[allow(clippy::needless_range_loop)]
        for i in 0..fft.bins() {
            let current_bin = fft.at(0, i);
            let normalization = WINDOW_LENGTH as f32;

            let value = current_bin.norm() / normalization;
            spectrum[i] = value;
        }
    });

    Box::new(fft_processor)
}
