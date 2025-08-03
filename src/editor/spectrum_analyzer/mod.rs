mod monitor;
mod utils;

use std::{
    f32::consts::PI,
    sync::{Arc, Mutex},
};

use fundsp::hacker32::*;

use monitor::{Meter, Monitor};

use crate::editor::spectrum_analyzer::utils::ValueScaling;

pub struct SpectrumAnalyzerHelper {
    // spectrum stuff
    graph: Box<dyn AudioUnit>,
    spectrum: Arc<Mutex<Vec<f32>>>,
    spectrum_monitors: Vec<Monitor>,

    sample_rate: f32,

    pub fps: f32,

    frequency_scaling: ValueScaling,
    frequency_range: (f32, f32),
    magnitude_scaling: ValueScaling,
    magnitude_range: (f32, f32),
}

const WINDOW_LENGTH: usize = 1024;
const NUM_MONITORS: usize = (WINDOW_LENGTH / 2) + 1;

// in seconds!
const DEFAULT_PEAK_DECAY: f32 = 0.25;
const DEFAULT_MODE: Meter = Meter::Rms(DEFAULT_PEAK_DECAY);

impl SpectrumAnalyzerHelper {
    pub fn new(sample_rate: f32) -> Self {
        let spectrum_monitors = vec![Monitor::new(DEFAULT_MODE); NUM_MONITORS];
        let spectrum = Arc::new(Mutex::new(vec![0.0; NUM_MONITORS]));

        let mut graph = build_fft_graph(spectrum.clone());
        graph.set_sample_rate(sample_rate as f64);

        Self {
            spectrum,
            spectrum_monitors,
            fps: 0.0,
            graph,
            sample_rate: 44100.0,
            frequency_range: (10.0, 22_050.0),
            frequency_scaling: ValueScaling::Frequency,
            magnitude_range: (-100.0, 6.0),
            magnitude_scaling: ValueScaling::Decibels,
        }
    }
    pub fn tick(&mut self, input: f32) {
        self.graph.tick(&[input], &mut [])
    }
    pub fn set_monitor_fps(&mut self, frame_rate: f32) {
        for mon in self.spectrum_monitors.iter_mut() {
            mon.set_frame_rate(frame_rate);
        }
        self.fps = frame_rate;
    }
    pub fn get_drawing_coordinates(&mut self) -> Vec<(f32, f32)> {
        let spectrum = &*self.spectrum.lock().unwrap();
        self.spectrum_monitors
            .iter_mut()
            .enumerate()
            .map(|(i, x)| {
                x.tick(spectrum[i]);
                let linear_level = x.level();

                let half_nyquist = self.sample_rate / 2.0;

                let freq = (i as f32 / spectrum.len() as f32) * half_nyquist;

                let magnitude_normalized = self.magnitude_scaling.value_to_normalized(
                    linear_level,
                    self.magnitude_range.0,
                    self.magnitude_range.1,
                );
                let freq_normalized = self.frequency_scaling.value_to_normalized(
                    freq,
                    self.frequency_range.0,
                    self.frequency_range.1,
                );

                (freq_normalized, magnitude_normalized)
            })
            .collect()
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

// https://gist.github.com/ollpu/231ebbf3717afec50fb09108aea6ad2f
fn lanczos(output: &mut [f32], sample_rate: f32) {
    for (i, res) in output.iter_mut().enumerate() {
        // i in [0, N[
        // x normalized to [0, 1[
        let x = i as f32 / WINDOW_LENGTH as f32;
        // We want to map x to frequency in range [40, 20k[, log scale
        // Parameters k, b. f = k*b^x
        // Know: k*b^0 = 40, k*b^1 = 20k
        // Therefore k = 40, b = 500
        let f = 40. * 500f32.powf(x);
        // w in [40, 20k[ / SR * N
        let w = f / sample_rate * WINDOW_LENGTH as f32;
        // Closest FFT bin
        let p = (w as isize).clamp(0, (WINDOW_LENGTH / 2) as isize);

        // Lanczos interpolation
        let radius = 10;
        // To interpolate in linear space:
        // let mut result = Complex::new(0., 0.);
        // To interpolate in dB space:
        let mut result = 0.;
        for iw in p - radius..=p + radius + 1 {
            if iw < 0 || iw > (WINDOW_LENGTH / 2) as isize {
                continue;
            }
            let delta = w - iw as f32;
            if delta.abs() > radius as f32 {
                continue;
            }
            let lanczos = if delta == 0. {
                1.
            } else {
                radius as f32 * (PI * delta).sin() * (PI * delta / radius as f32).sin()
                    / (PI * delta).powi(2)
            };
            // Linear space
            // let value = self.complex_buf[iw as usize];
            // dB space
            let value = 20. * res.log10();
            result += lanczos * value;
        }
        // If interpolated in linear space, convert to dB now
        // *res = 20. * result.norm().log10();
        // Otherwise use result directly
        *res = result;
        if !(*res).is_finite() {
            *res = -250.;
        }
    }
}
