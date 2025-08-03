use std::sync::{Arc, Mutex};

use crossbeam_channel::Receiver;
use fundsp::hacker32::*;

use crate::editor::monitor::{Meter, Monitor};

pub struct SpectrumAnalyzerHelper {
    // spectrum stuff
    graph: Box<dyn AudioUnit>,
    spectrum: Arc<Mutex<Vec<f32>>>,
    spectrum_monitors: Vec<Monitor>,
    fps: f32,
}

const WINDOW_LENGTH: usize = 1024;
const NUM_MONITORS: usize = (WINDOW_LENGTH / 2) + 1;

// in seconds!
const DEFAULT_PEAK_DECAY: f32 = 1.5;
const DEFAULT_MODE: Meter = Meter::Peak(DEFAULT_PEAK_DECAY);

impl SpectrumAnalyzerHelper {
    pub fn new() -> Self {
        let spectrum_monitors = vec![Monitor::new(DEFAULT_MODE); NUM_MONITORS];

        let spectrum = Arc::new(Mutex::new(vec![0.0; NUM_MONITORS]));

        let mut graph = build_fft_graph(spectrum.clone());
        // TODO: does this actually... matter?
        let sample_rate = todo!();
        graph.set_sample_rate(sample_rate as f64);

        Self {
            spectrum,
            spectrum_monitors,
            fps: 0.0,
            graph,
        }
    }

    pub fn tick(&mut self, input: f32) {
        self.graph.tick(&[input], &mut [])
    }
    pub fn set_frame_rate(&mut self, frame_rate: f32) {
        for mon in self.spectrum_monitors.iter_mut() {
            mon.set_frame_rate(frame_rate);
        }
        self.fps = frame_rate;
    }
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
