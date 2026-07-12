use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::calibration::FaceShape;
use crate::filter::{OneEuro, OneEuroParameters};
use crate::weights::Weights;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FaceFilterParameters {
    pub enable: bool,
    #[serde(flatten)]
    pub one_euro: OneEuroParameters,
}

impl Default for FaceFilterParameters {
    fn default() -> Self {
        Self {
            enable: true,
            one_euro: OneEuroParameters::new(0.5, 3.0),
        }
    }
}

/// Temporal smoothing stage for face weights.
///
/// A generic per-channel One-Euro filter: every face blendshape is smoothed with
/// the same [`FaceFilterParameters`]. Unlike the eyes, the face has no fusion, so
/// this is a straightforward channel-wise low-pass.
///
/// It is meant to sit *before* the face calibrator, so the calibrator's
/// auto-capture sees denoised values — a single noise spike can otherwise poison
/// a captured peak.
pub struct FaceFilter {
    params: FaceFilterParameters,
    channels: Vec<OneEuro>,
    t_prev: Option<Instant>,
    weights: Weights<FaceShape>,
}

impl FaceFilter {
    pub fn new() -> Self {
        Self::with_parameters(FaceFilterParameters::default())
    }

    pub fn with_parameters(params: FaceFilterParameters) -> Self {
        Self {
            params,
            channels: vec![OneEuro::new(params.one_euro); FaceShape::count()],
            t_prev: None,
            weights: Weights::new(),
        }
    }

    pub fn parameters(&self) -> FaceFilterParameters {
        self.params
    }

    pub fn set_parameters(&mut self, params: FaceFilterParameters) {
        self.params = params;
        for channel in &mut self.channels {
            channel.parameters = params.one_euro;
        }
    }

    pub fn filter(&mut self, raw: &Weights<FaceShape>) -> &Weights<FaceShape> {
        let now = Instant::now();
        let dt = self
            .t_prev
            .map_or(0.0, |prev| now.duration_since(prev).as_secs_f32());
        self.t_prev = Some(now);

        self.weights.clear();

        for (shape, value) in raw.iter() {
            let value = if self.params.enable {
                self.channels[shape as usize].filter(value, dt)
            } else {
                // Pass through, but seed the channel (dt = 0) so re-enabling doesn't jump.
                self.channels[shape as usize].filter(value, 0.0)
            };
            self.weights.set(shape, value);
        }

        &self.weights
    }
}

impl Default for FaceFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameters_deserialize_flat() {
        let params: FaceFilterParameters =
            toml::from_str("enable = true\nmin_cutoff = 0.5\nbeta = 3.0\n").unwrap();

        assert!(params.enable);
        assert_eq!(params.one_euro.min_cutoff, 0.5);
        assert_eq!(params.one_euro.beta, 3.0);
    }
}
