use crossbeam_channel::Receiver;
use include_dir::include_dir;
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use nih_plug_webview::{
    wry::{
        http::{Request, Response},
        WebViewId,
    },
    WebViewConfig, WebViewEditor, WebViewSource, WebViewState,
};

use crate::editor::{build_fft_graph, monitor::Meter, PluginGui};

#[allow(dead_code)]
pub fn embedded_editor(
    state: &Arc<WebViewState>,
    rx: Receiver<f32>,
    sample_rate: f32,
) -> WebViewEditor {
    let protocol_name = "assets".to_string();

    let rel_config = WebViewConfig {
        title: "Convolution".to_string(),
        source: WebViewSource::CustomProtocol {
            protocol: protocol_name.clone(),
            url: String::new(),
        },
        // tODO: should we change this?
        workdir: PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/target/webview-workdir"
        )),
    };

    let spectrum = Arc::new(Mutex::new(Vec::new()));
    let mut graph = build_fft_graph(spectrum.clone());
    graph.set_sample_rate(sample_rate as f64);

    WebViewEditor::new_with_webview(
        PluginGui {
            sample_rx: rx,
            graph,
            spectrum,
            spectrum_monitors: vec![
                crate::editor::monitor::Monitor::new(
                    Meter::RMS,
                    0.1,
                    sample_rate
                );
                513
            ],
        },
        state,
        rel_config,
        move |builder| {
            builder
                .with_devtools(false)
                .with_custom_protocol(protocol_name.clone(), build_protocol())
        },
    )
}

// type refactoring is probably pointless?
type Res = Response<Cow<'static, [u8]>>;
type Protocol = dyn Fn(WebViewId, Request<Vec<u8>>) -> Res + 'static;

fn build_protocol() -> Box<Protocol> {
    Box::new(move |_id, req: Request<Vec<u8>>| {
        let path = req.uri().path();
        let file = if path == "/" {
            "index.html"
        } else {
            &path[1..]
        };
        // should we hardcode this or something?
        let dir = include_dir!("$CARGO_MANIFEST_DIR/gui/assets/");

        let mime_type =
            mime_guess::from_ext(Path::new(file).extension().unwrap().to_str().unwrap())
                .first_or_text_plain()
                .to_string();
        if let Some(result_file) = dir.get_file(file) {
            Response::builder()
                .header("content-type", mime_type)
                .header("Access-Control-Allow-Origin", "*")
                .body(result_file.contents().into())
                .expect("Error constructing response")
        } else {
            panic!("Web asset not found. {file:?}")
        }
    })
}
