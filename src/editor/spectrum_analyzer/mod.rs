mod monitor;

use monitor::{Meter, Monitor};

use crossbeam_channel::Receiver;
use fundsp::hacker32::*;
use nih_plug::{
    prelude::AtomicF32,
    util::{db_to_gain, gain_to_db, gain_to_db_fast},
};
use std::{
    f32::consts::PI,
    sync::{atomic::Ordering, Arc, Mutex},
};

pub struct SpectrumAnalyzerHelper {
    // NOTE: we could probably compute the FFT without fundsp
    // (but i like fundsp)
    graph: Box<dyn AudioUnit>,

    sample_rx: Receiver<f32>,

    spectrum: Arc<Mutex<Vec<f32>>>,
    spectrum_monitors: Vec<Monitor>,

    sample_rate: Arc<AtomicF32>,

    pub fps: f32,

    frequency_range: (f32, f32),
    magnitude_range: (f32, f32),
}

const DEFAULT_FPS: f32 = 60.0;
const DEFAULT_FREQ_RANGE: (f32, f32) = (10.0, 20_000.0);
const DEFAULT_MAGNITUDE_RANGE: (f32, f32) = (-78.0, 6.0);

const WINDOW_LENGTH: usize = 4096;
const NUM_MONITORS: usize = (WINDOW_LENGTH / 2) + 1;

// in seconds!
const DEFAULT_PEAK_DECAY: f32 = 0.25;
const DEFAULT_MODE: Meter = Meter::Rms(DEFAULT_PEAK_DECAY);

impl SpectrumAnalyzerHelper {
    pub fn new(sample_rate: Arc<AtomicF32>, sample_rx: Receiver<f32>) -> Self {
        let spectrum_monitors = vec![Monitor::new(DEFAULT_MODE); NUM_MONITORS];
        let spectrum = Arc::new(Mutex::new(vec![0.0; NUM_MONITORS]));

        let mut graph = build_fft_graph(spectrum.clone());
        graph.set_sample_rate(sample_rate.load(Ordering::Relaxed) as f64);
        Self {
            spectrum,
            spectrum_monitors,
            graph,
            sample_rate,
            sample_rx,
            fps: DEFAULT_FPS,
            frequency_range: DEFAULT_FREQ_RANGE,
            magnitude_range: DEFAULT_MAGNITUDE_RANGE,
        }
    }
    fn tick(&mut self) {
        for sample in self.sample_rx.try_iter() {
            self.graph.tick(&[sample], &mut [])
        }
    }
    fn set_monitor_fps(&mut self, frame_rate: f32) {
        for mon in self.spectrum_monitors.iter_mut() {
            mon.set_frame_rate(frame_rate);
        }
        self.fps = frame_rate;
    }
    // TODO: refactor
    fn get_drawing_coordinates(&mut self) -> Vec<(f32, f32)> {
        let sample_rate = self.sample_rate.load(Ordering::Relaxed);

        let spectrum = &*self.spectrum.lock().unwrap();
        let linear_levels: Vec<_> = self
            .spectrum_monitors
            .iter_mut()
            .enumerate()
            .map(|(i, x)| {
                x.tick(spectrum[i]);
                x.level()
            })
            .collect();
        let mut output = vec![0.0; WINDOW_LENGTH];
        lanczos(
            &linear_levels,
            &mut output,
            sample_rate,
            self.frequency_range.0,
            self.frequency_range.1,
        );

        let half_nyquist = sample_rate / 2.0;

        output
            .iter()
            .enumerate()
            .map(|(i, magnitude)| {
                let freq = (i as f32 / output.len() as f32) * half_nyquist;

                let magnitude_normalized =
                    normalize(*magnitude, self.magnitude_range.0, self.magnitude_range.1);

                let freq_normalized =
                    normalize(freq, self.frequency_range.0, self.frequency_range.1);

                (freq_normalized, magnitude_normalized)
            })
            .collect()
    }

    pub fn handle_request(&mut self, frame_rate: f32) -> Vec<(f32, f32)> {
        self.tick();
        // QUESTION: is it cheaper to just always set the FPS, even if it hasn't changed?
        // (maybe the compiler will optimize the decay calculations or something)
        if frame_rate != self.fps {
            self.set_monitor_fps(frame_rate);
        }
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
// https://gist.github.com/ollpu/231ebbf3717afec50fb09108aea6ad2f
// TODO: optimize this function
// TODO: ! add more parameters and remove constants
fn lanczos(input: &[f32], output: &mut [f32], sample_rate: f32, min_freq: f32, max_freq: f32) {
    let radius = 10;
    let slope = 4.5;

    for (i, res) in output.iter_mut().enumerate() {
        // i in [0, N[
        // x normalized to [0, 1[
        let x = i as f32 / WINDOW_LENGTH as f32;
        // We want to map x to frequency in range [min, max[, log scale
        // Parameters k, b. f = k*b^x
        let k = min_freq;
        let b = max_freq / min_freq;
        let f = k * b.powf(x);

        let w = f / sample_rate * WINDOW_LENGTH as f32;
        // Closest FFT bin
        let p = (w as isize).clamp(0, (WINDOW_LENGTH / 2) as isize);

        // slope implementation
        let slope_factor_linear = calculate_slope_factor(x, slope, sample_rate);

        // Lanczos interpolation

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
            let current_bin_linear = input[iw as usize];
            let sloped_bin_linear = current_bin_linear * slope_factor_linear;
            let sloped_bin_db = gain_to_db_fast(sloped_bin_linear);

            result += lanczos * sloped_bin_db;
        }
        // If interpolated in linear space, convert to dB now
        // *res = 20. * result.norm().log10();
        // Otherwise use result directly
        *res = result;
    }
}

fn calculate_slope_factor(normalized_frequency: f32, slope: f32, sample_rate: f32) -> f32 {
    let half_nyquist = sample_rate / 2.0;

    let magnitude_slope_divisor = half_nyquist.log2().powf(slope) / slope;

    let freq = normalized_frequency * half_nyquist;
    (freq + 1.).log2().powf(slope) / magnitude_slope_divisor
}

fn normalize(value: f32, min: f32, max: f32) -> f32 {
    (value - min) / (max - min)
}
