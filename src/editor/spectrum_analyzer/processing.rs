use nih_plug::util::gain_to_db_fast;
use std::f32::consts::PI;

use crate::editor::spectrum_analyzer::{config::SpectrumAnalyzerConfig, WINDOW_LENGTH};

// https://gist.github.com/ollpu/231ebbf3717afec50fb09108aea6ad2f
// TODO: optimize this function

pub fn process_spectrum(
    input: &[f32],
    sample_rate: f32,
    config: &SpectrumAnalyzerConfig,
) -> Vec<f32> {
    // TODO: make radius configurable?
    let radius = 10;
    let slope = config.slope;
    let min_freq = config.frequency_range.0;
    let max_freq = config.frequency_range.1;
    // NOTE: is WINDOW_LENGTH a correct length for the interpolated output?
    let length = if config.interpolate {
        WINDOW_LENGTH
    } else {
        input.len()
    };
    let mut output = vec![0.0; length];

    for (i, res) in output.iter_mut().enumerate() {
        // i in [0, N[
        // x normalized to [0, 1[
        let x = i as f32 / length as f32;
        // We want to map x to frequency in range [min, max[, log scale
        // Parameters k, b. f = k*b^x

        let b = max_freq / min_freq;
        let f = min_freq * b.powf(x);

        let w = f / sample_rate * length as f32;
        // Closest FFT bin
        let p = (w as isize).clamp(0, (length / 2) as isize);

        // slope implementation

        if !config.interpolate {
            // TODO: FIX!
            let current_bin_linear = input[p as usize];
            let sloped_bin_db = gain_to_db_fast(current_bin_linear);
            *res = sloped_bin_db;
            continue;
        }
        let slope_factor_linear = calculate_slope_factor(x, slope, sample_rate);

        // Lanczos interpolation

        // To interpolate in dB space:
        let mut result = 0.;
        for iw in p - radius..=p + radius + 1 {
            if iw < 0 || iw > (length / 2) as isize {
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
    output
}
// TODO: !! 90% sure this doesn't work
pub fn calculate_slope_factor(normalized_frequency: f32, slope: f32, sample_rate: f32) -> f32 {
    let half_nyquist = sample_rate / 2.0;

    let magnitude_slope_divisor = half_nyquist.log2().powf(slope) / slope;

    let freq = normalized_frequency * half_nyquist;
    (freq + 1.).log2().powf(slope) / magnitude_slope_divisor
}

pub fn normalize(value: f32, min: f32, max: f32) -> f32 {
    (value - min) / (max - min)
}
