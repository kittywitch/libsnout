use std::f32::consts::PI;

use serde::{Deserialize, Serialize};

/// Tuning for a [`OneEuro`] filter.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct OneEuroParameters {
    /// Cutoff frequency at rest, in Hz. Lower means smoother and laggier.
    pub min_cutoff: f32,
    /// How much the cutoff rises with speed. Higher means snappier fast motion.
    pub beta: f32,
}

impl OneEuroParameters {
    pub const fn new(min_cutoff: f32, beta: f32) -> Self {
        Self { min_cutoff, beta }
    }
}

/// A single-channel One-Euro filter.
///
/// An adaptive low-pass whose cutoff frequency rises with the signal's speed.
///
/// It smooths heavily when the value is nearly still (killing sensor jitter) and
/// lightly when it moves fast (preserving quick motion like a saccade). See
/// [`OneEuroParameters`] for the two knobs.
#[derive(Clone, Copy, Debug)]
pub struct OneEuro {
    pub parameters: OneEuroParameters,
    /// Cutoff for the internal speed estimate, in Hz.
    d_cutoff: f32,
    x_prev: f32,
    dx_prev: f32,
    initialized: bool,
}

impl OneEuro {
    pub fn new(parameters: OneEuroParameters) -> Self {
        Self {
            parameters,
            d_cutoff: 1.0,
            x_prev: 0.0,
            dx_prev: 0.0,
            initialized: false,
        }
    }

    /// Feed a new sample and return the smoothed value. `dt` is the time in
    /// seconds since the previous sample; the first sample (or any non-positive
    /// `dt`) passes through untouched to seed the filter.
    pub fn filter(&mut self, x: f32, dt: f32) -> f32 {
        if !self.initialized || dt <= 0.0 {
            self.x_prev = x;
            self.dx_prev = 0.0;
            self.initialized = true;
            return x;
        }

        let a_d = smoothing_factor(dt, self.d_cutoff);
        let dx = (x - self.x_prev) / dt;
        let dx_hat = exponential_smoothing(a_d, dx, self.dx_prev);

        let cutoff = self.parameters.min_cutoff + self.parameters.beta * dx_hat.abs();
        let a = smoothing_factor(dt, cutoff);
        let x_hat = exponential_smoothing(a, x, self.x_prev);

        self.x_prev = x_hat;
        self.dx_prev = dx_hat;
        x_hat
    }

    /// Hold the last smoothed value without taking a new sample.
    ///
    /// Used to coast through a blink: the value is frozen and the speed estimate
    /// is cleared, so filtering resumes cleanly afterward instead of lurching
    /// from a stale velocity.
    pub fn hold(&mut self) -> f32 {
        self.dx_prev = 0.0;
        self.x_prev
    }
}

fn smoothing_factor(dt: f32, cutoff: f32) -> f32 {
    let r = 2.0 * PI * cutoff * dt;
    r / (r + 1.0)
}

fn exponential_smoothing(a: f32, x: f32, prev: f32) -> f32 {
    a * x + (1.0 - a) * prev
}
