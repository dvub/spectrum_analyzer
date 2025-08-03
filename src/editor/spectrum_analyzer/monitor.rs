//! Provides basic, configurable smoothing for FFT bins.

// code ported from FunDSP:
// https://github.com/SamiPerttu/fundsp/blob/a4f126bcbb5c6b93c4cd65662035655913e1e830/src/dynamics.rs#L343

// this is a stripped-down version of meter/monitor stuff from fundsp

const DEFAULT_FPS: f32 = 30.0;

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum Meter {
    Sample,
    Peak(f32),
    Rms(f32),
}

#[derive(Clone)]
pub struct Monitor {
    meter: Meter,
    state: f32,
    smoothing: f32,
}

impl Monitor {
    /// Create a new Monitor based on the given mode.
    pub fn new(meter: Meter) -> Self {
        let mut monitor = Self {
            meter,
            state: 0.0,
            smoothing: 0.0,
        };

        monitor.set_frame_rate(DEFAULT_FPS);
        monitor
    }

    /// Set the frame rate at which the meter is updated.
    pub fn set_frame_rate(&mut self, frame_rate: f32) {
        let timescale = match self.meter {
            Meter::Sample => {
                return;
            }
            Meter::Peak(timescale) => timescale,
            Meter::Rms(timescale) => timescale,
        };
        self.smoothing = 0.5f32.powf(1.0 / (timescale * frame_rate));
    }

    /// Process an input value.
    pub fn tick(&mut self, value: f32) {
        match self.meter {
            Meter::Sample => self.state = value,
            Meter::Peak(_) => self.state = f32::max(self.state * self.smoothing, value.abs()),
            Meter::Rms(_) => {
                self.state = self.state * self.smoothing + value.powi(2) * (1.0 - self.smoothing)
            }
        }
    }

    pub fn level(&self) -> f32 {
        match self.meter {
            Meter::Sample => self.state,
            Meter::Peak(_) => self.state,
            Meter::Rms(_) => self.state.sqrt(),
        }
    }
}
