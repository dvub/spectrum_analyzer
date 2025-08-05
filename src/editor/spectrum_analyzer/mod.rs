mod config;
pub mod ipc;
pub mod monitor;
mod processing;
use monitor::Monitor;

use crossbeam_channel::Receiver;
use fundsp::hacker32::*;
use nih_plug::prelude::AtomicF32;
use std::sync::{atomic::Ordering, Arc, Mutex};

use crate::editor::spectrum_analyzer::{
    config::SpectrumAnalyzerConfig,
    processing::{normalize, process_spectrum},
};
const WINDOW_LENGTH: usize = 4096;
const NUM_MONITORS: usize = (WINDOW_LENGTH / 2) + 1;

pub struct SpectrumAnalyzerHelper {
    // NOTE: we could probably compute the FFT without fundsp
    // (but i like fundsp)
    graph: Box<dyn AudioUnit>,

    sample_rx: Receiver<f32>,

    spectrum: Arc<Mutex<Vec<f32>>>,
    spectrum_monitors: Vec<Monitor>,

    sample_rate: Arc<AtomicF32>,

    config: SpectrumAnalyzerConfig,
}

impl SpectrumAnalyzerHelper {
    pub fn new(sample_rate: Arc<AtomicF32>, sample_rx: Receiver<f32>) -> Self {
        let config = SpectrumAnalyzerConfig::default();

        let spectrum_monitors = vec![Monitor::new(config.monitor_mode); NUM_MONITORS];
        let spectrum = Arc::new(Mutex::new(vec![0.0; NUM_MONITORS]));

        let mut graph = build_fft_graph(spectrum.clone());
        graph.set_sample_rate(sample_rate.load(Ordering::Relaxed) as f64);
        Self {
            spectrum,
            spectrum_monitors,
            graph,
            sample_rate,
            sample_rx,

            config,
        }
    }
    fn tick(&mut self) {
        for sample in self.sample_rx.try_iter() {
            self.graph.tick(&[sample], &mut [])
        }
    }
    fn get_bin_levels(&mut self) -> Vec<f32> {
        let spectrum = &*self.spectrum.lock().unwrap();
        self.spectrum_monitors
            .iter_mut()
            .enumerate()
            .map(|(i, x)| {
                x.tick(spectrum[i]);
                x.level()
            })
            .collect()
    }
    fn get_drawing_coordinates(&mut self) -> Vec<(f32, f32)> {
        let sample_rate = self.sample_rate.load(Ordering::Relaxed);
        let min_mag = self.config.magnitude_range.0;
        let max_mag = self.config.magnitude_range.1;

        let linear_levels = self.get_bin_levels();
        let output = process_spectrum(&linear_levels, sample_rate, &self.config);
        output
            .iter()
            .enumerate()
            .map(|(i, magnitude)| {
                let freq_normalized = i as f32 / output.len() as f32;

                let magnitude_normalized = normalize(*magnitude, min_mag, max_mag);

                (freq_normalized, magnitude_normalized)
            })
            .collect()
    }

    pub fn set_monitor_mode(&mut self, meter: monitor::MonitorMode) {
        for mon in self.spectrum_monitors.iter_mut() {
            mon.set_mode(meter);
        }
    }

    pub fn set_monitor_fps(&mut self, frame_rate: f32) {
        for mon in self.spectrum_monitors.iter_mut() {
            mon.set_frame_rate(frame_rate);
        }
    }
    pub fn set_monitor_decay_speed(&mut self, speed: f32) {
        for mon in self.spectrum_monitors.iter_mut() {
            mon.set_decay_speed(speed);
        }
    }

    pub fn handle_draw_request(&mut self) -> Vec<(f32, f32)> {
        // QUESTION: is it cheaper to just always set the FPS, even if it hasn't changed?
        // (maybe the compiler will optimize the decay calculations or something)
        self.tick();
        self.get_drawing_coordinates()
    }
}

fn build_fft_graph(spectrum: Arc<Mutex<Vec<f32>>>) -> Box<dyn AudioUnit> {
    let fft_processor = resynth::<U1, U0, _>(WINDOW_LENGTH, move |fft| {
        let mut spectrum = spectrum.lock().unwrap();

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
