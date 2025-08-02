#[derive(Clone, Copy)]
pub enum Meter {
    Peak,
    RMS,
}

#[derive(Clone)]
pub struct Monitor {
    meter: Meter,
    state: f32,
    smoothing: f32,
    timescale: f32,
}

impl Monitor {
    pub fn new(meter: Meter, timescale: f32, sample_rate: f32) -> Self {
        let smoothing = (0.5f32).powf(1.0 / (timescale * sample_rate));

        Monitor {
            meter,
            state: 0.0,
            smoothing,
            timescale,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.smoothing = (0.5f32).powf(1.0 / (self.timescale * sample_rate));
    }

    pub fn tick(&mut self, value: f32) {
        match self.meter {
            Meter::Peak => self.state = f32::max(self.state * self.smoothing, f32::abs(value)),
            Meter::RMS => {
                self.state = self.state * self.smoothing + (value.powi(2)) * (1.0 - self.smoothing);
            }
        }
    }

    pub fn level(&self) -> f32 {
        match self.meter {
            Meter::Peak => self.state,
            Meter::RMS => self.state.sqrt(),
        }
    }
}
