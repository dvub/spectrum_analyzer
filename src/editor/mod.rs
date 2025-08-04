mod embedded;
mod ipc;
mod spectrum_analyzer;

use embedded::build_protocol;
use ipc::{DrawData, DrawRequest, Message};
use spectrum_analyzer::SpectrumAnalyzerHelper;

use crossbeam_channel::Receiver;
use nih_plug::{editor::Editor, prelude::AtomicF32};
use nih_plug_webview::{
    Context, EditorHandler, WebViewConfig, WebViewEditor, WebViewSource, WebViewState,
};
use serde_json::json;
use std::{path::PathBuf, sync::Arc};

pub struct PluginGui {
    spectrum_analyzer: SpectrumAnalyzerHelper,
}

impl PluginGui {
    // TODO: refactor more
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        state: &Arc<WebViewState>,
        rx: Receiver<f32>,
        sample_rate: Arc<AtomicF32>,
    ) -> Option<Box<dyn Editor>> {
        // SOURCE
        let protocol_name = "assets".to_string();
        let source = if cfg!(debug_assertions) {
            WebViewSource::URL(String::from("http://localhost:3000"))
        } else {
            // with --release, we use a custom protocol
            // this protocol will bundle the GUI in the plugin
            WebViewSource::CustomProtocol {
                protocol: protocol_name.clone(),
                url: String::new(),
            }
        };
        // CONFIG
        let config = WebViewConfig {
            title: "Spectrum Analyzer".to_string(),
            source,
            // TODO: should we change this?
            workdir: PathBuf::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/target/webview-workdir"
            )),
        };
        // EDITOR
        let editor_base = PluginGui {
            spectrum_analyzer: SpectrumAnalyzerHelper::new(sample_rate, rx.clone()),
        };

        Some(Box::new(WebViewEditor::new_with_webview(
            editor_base,
            state,
            config,
            move |mut builder| {
                if !cfg!(debug_assertions) {
                    builder = builder.with_custom_protocol(protocol_name.clone(), build_protocol())
                }
                // if --release, no devtools available to users
                builder = builder.with_devtools(cfg!(debug_assertions));
                builder
            },
        )))
    }

    fn handle_message(&mut self, message: Message, cx: &mut Context) {
        match message {
            Message::Init => {}
            Message::Resize { width, height } => {
                let resize_result = cx.resize_window(width, height);
                if !resize_result {
                    println!("WARNING: the window was not resized upon request");
                }
            }
            // !!
            Message::DrawRequest(draw_request) => self.handle_draw_request(draw_request, cx),
            Message::DrawData(_) => todo!(),
        }
    }

    fn handle_draw_request(&mut self, draw_request: DrawRequest, cx: &mut Context) {
        match draw_request {
            DrawRequest::Spectrum(frame_rate) => {
                let coordinates = self.spectrum_analyzer.handle_request(frame_rate);
                let message = Message::DrawData(DrawData::Spectrum(coordinates));
                cx.send_message(json!(message).to_string());
            }
        }
    }
}

impl EditorHandler for PluginGui {
    fn on_frame(&mut self, _: &mut Context) {}

    fn on_message(&mut self, cx: &mut Context, message: String) {
        let message: Message = serde_json::from_str(&message).expect("Error reading message");
        self.handle_message(message, cx);
    }

    fn on_params_changed(&mut self, _: &mut Context) {}
}
