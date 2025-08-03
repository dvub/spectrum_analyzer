mod embedded;
mod ipc;

mod spectrum_analyzer;

use std::{path::PathBuf, sync::Arc};

#[cfg(not(debug_assertions))]
use crate::editor::embedded::embedded_editor;
use crate::editor::{
    ipc::{DrawData, DrawRequest, Message},
    spectrum_analyzer::SpectrumAnalyzerHelper,
};

use crossbeam_channel::Receiver;

use nih_plug::editor::Editor;
use nih_plug_webview::{EditorHandler, WebViewConfig, WebViewEditor, WebViewSource, WebViewState};

use serde_json::json;

pub struct PluginGui {
    sample_rx: Receiver<f32>,
    spectrum_analyzer: SpectrumAnalyzerHelper,
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
            self.spectrum_analyzer.tick(sample);
        }
    }
}

fn dev_editor(state: &Arc<WebViewState>, rx: Receiver<f32>, sample_rate: f32) -> WebViewEditor {
    let config = WebViewConfig {
        title: "Convolution".to_string(),
        source: WebViewSource::URL(String::from("http://localhost:3000")),
        workdir: PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/target/webview-workdir"
        )),
    };

    WebViewEditor::new_with_webview(
        // TODO: refactor
        PluginGui {
            sample_rx: rx,
            spectrum_analyzer: SpectrumAnalyzerHelper::new(sample_rate),
        },
        state,
        config,
        |builder| builder.with_devtools(true),
    )
}

impl EditorHandler for PluginGui {
    fn on_frame(&mut self, _: &mut nih_plug_webview::Context) {
        self.tick();
    }

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
                    if frame_rate != self.spectrum_analyzer.fps {
                        self.spectrum_analyzer.set_monitor_fps(frame_rate);
                    }

                    let processed = self.spectrum_analyzer.get_drawing_coordinates();
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
