mod dsp;
mod editor;
mod params;

use crossbeam_channel::{bounded, Receiver, Sender};
use fundsp::hacker32::*;
use nih_plug::prelude::*;
use params::SpectrumAnalyzerParams;
use std::sync::Arc;

use crate::{dsp::build_graph, editor::PluginGui};

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

struct SpectrumAnalyzer {
    params: Arc<SpectrumAnalyzerParams>,

    graph: BigBlockAdapter,
    buffers: Vec<Vec<f32>>,

    tx: Sender<f32>,
    rx: Receiver<f32>,
}

impl Default for SpectrumAnalyzer {
    fn default() -> Self {
        let (tx, rx) = bounded(1024);
        Self {
            params: Arc::new(SpectrumAnalyzerParams::default()),
            graph: BigBlockAdapter::new(Box::new(sink())),
            buffers: Vec::new(),
            tx,
            rx,
        }
    }
}

impl Plugin for SpectrumAnalyzer {
    const NAME: &'static str = "Spectrum Analyzer";
    const VENDOR: &'static str = "dvub";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "todo@todo.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();

    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.buffers = vec![vec![0.0; buffer_config.max_buffer_size as usize]; 2];

        let graph = build_graph(self.tx.clone());

        // TODO: refactor these steps
        self.graph = BigBlockAdapter::new(graph);
        self.graph
            .set_sample_rate(f64::from(buffer_config.sample_rate));
        self.graph.allocate();

        true
    }

    // not sure if this is needed..
    fn reset(&mut self) {
        self.graph.reset();
    }

    fn editor(&mut self, _: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        PluginGui::new(&self.params.state, self.rx.clone())
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for (i, chan) in buffer.as_slice_immutable().iter().enumerate() {
            self.buffers[i][..buffer.samples()].copy_from_slice(chan);
        }

        self.graph
            .process_big(buffer.samples(), &self.buffers, buffer.as_slice());

        ProcessStatus::Normal
    }
}

impl ClapPlugin for SpectrumAnalyzer {
    const CLAP_ID: &'static str = "com.your-domain.spectrum-analyzer";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Spectrum Analyzer Plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Analyzer,
        ClapFeature::Mastering,
        ClapFeature::Mixing,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for SpectrumAnalyzer {
    const VST3_CLASS_ID: [u8; 16] = *b"Exactly16Chars!!";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Tools,
        Vst3SubCategory::Stereo,
        Vst3SubCategory::Fx,
        Vst3SubCategory::Mastering,
    ];
}

nih_export_clap!(SpectrumAnalyzer);
nih_export_vst3!(SpectrumAnalyzer);
