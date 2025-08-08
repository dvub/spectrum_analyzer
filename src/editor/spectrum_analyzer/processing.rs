use nih_plug::util::{gain_to_db, gain_to_db_fast};
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

    for (index, res) in output.iter_mut().enumerate() {
        // i in [0, N[
        // x normalized to [0, 1[
        let normalized_freq = index as f32 / length as f32;
        // We want to map x to frequency in range [min, max[, log scale
        // Parameters k, b. f = k*b^x
        let b = max_freq / min_freq;
        let current_freq_log = min_freq * b.powf(normalized_freq);

        // NOTE:
        // if we skip interpolation, we get a blocky look
        // this is because of how we snap to FFT bins
        // (or at least, i think it's caused by that)

        // Closest FFT bin
        let w = current_freq_log / sample_rate * length as f32;
        let p = (w as isize).clamp(0, (length / 2) as isize);

        let slope_factor_linear = calculate_slope_factor(current_freq_log, slope, sample_rate);

        if !config.interpolate {
            // TODO: possibly refactor to reduce repitition
            let current_bin_linear = input[p as usize];
            // TODO: add config option for fast gain to db conversion
            let sloped_bin_db = gain_to_db(current_bin_linear * slope_factor_linear);
            *res = sloped_bin_db;
            continue;
        }

        // Lanczos interpolation
        // (expensive)
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
            let sloped_bin_db = gain_to_db(sloped_bin_linear);

            result += lanczos * sloped_bin_db;
        }
        *res = result;
    }
    output
}
fn calculate_slope_factor(freq: f32, slope: f32, sample_rate: f32) -> f32 {
    let half_nyquist = sample_rate / 2.0;

    let magnitude_slope_divisor = half_nyquist.log2().powf(slope) / slope;
    (freq + 1.).log2().powf(slope) / magnitude_slope_divisor
}

pub fn normalize(value: f32, min: f32, max: f32) -> f32 {
    (value - min) / (max - min)
}
