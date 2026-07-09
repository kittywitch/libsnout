use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::calibration::EyeShape;
use crate::filter::{OneEuro, OneEuroParameters};
use crate::weights::Weights;

/// Non-gaze eye channels, filtered per-channel and passed through in place.
const EXPRESSION_SHAPES: [EyeShape; 6] = [
    EyeShape::LeftEyeWiden,
    EyeShape::RightEyeWiden,
    EyeShape::LeftEyeBrow,
    EyeShape::RightEyeBrow,
    EyeShape::LeftEyeSquint,
    EyeShape::RightEyeSquint,
];

/// Tuning for [`EyeFilter`].
///
/// The channels are smoothed with deliberately different responsiveness: the
/// shared gaze direction (*version*) keeps saccades crisp, eye crossing is
/// smoothed hard because it is slow and noisy, and the lids stay responsive so
/// blinks are not smeared.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct EyeFilterParameters {
    pub enable: bool,

    /// Shared gaze direction pitch and yaw,
    /// i.e. where both eyes point together (version)
    pub version: OneEuroParameters,

    /// Eye crossing (vergence).
    pub vergence: OneEuroParameters,

    /// Eyelids.
    pub lid: OneEuroParameters,

    /// Expression channels (widen / brow / squint).
    pub expression: OneEuroParameters,

    /// Coast (hold gaze steady) while the more-open eye's openness is at or below
    /// this value — i.e. during a full blink, when neither eye can see. The lids
    /// keep updating so the blink itself is still visible.
    pub coast_openness: f32,
}

impl Default for EyeFilterParameters {
    fn default() -> Self {
        Self {
            enable: true,
            // Preserve fast flicks (saccades); smooth heavily when still.
            version: OneEuroParameters::new(0.5, 3.0),
            // Crossing is slow and low-amplitude, so smooth it hard.
            vergence: OneEuroParameters::new(0.3, 0.5),
            // Blinks are fast; keep the lids crisp.
            lid: OneEuroParameters::new(2.0, 5.0),
            // Expressions: moderate.
            expression: OneEuroParameters::new(0.8, 2.0),
            coast_openness: 0.1,
        }
    }
}

/// Temporal smoothing stage for fused eye gaze.
///
/// Consumes the [`Weights`] produced by the spatial fusion stage and returns a
/// smoothed set of the same shape. Internally it re-derives the shared *version*
/// (conjugate gaze) and *vergence* (eye crossing) from the per-eye yaw, filters
/// each channel with its own responsiveness, holds gaze steady through a full
/// blink, then reconstructs the per-eye values.
///
/// It is a standalone stage on purpose: consumers that want the raw fused signal
/// (e.g. dynamic foveated rendering, which does its own filtering) simply skip
/// it, while avatar outputs run through it.
pub struct EyeFilter {
    params: EyeFilterParameters,

    pitch: OneEuro,
    yaw: OneEuro,
    vergence: OneEuro,
    left_lid: OneEuro,
    right_lid: OneEuro,
    expressions: [OneEuro; EXPRESSION_SHAPES.len()],

    t_prev: Option<Instant>,
    weights: Weights<EyeShape>,
}

impl EyeFilter {
    pub fn new() -> Self {
        Self::with_parameters(EyeFilterParameters::default())
    }

    pub fn with_parameters(params: EyeFilterParameters) -> Self {
        Self {
            params,
            pitch: OneEuro::new(params.version),
            yaw: OneEuro::new(params.version),
            vergence: OneEuro::new(params.vergence),
            left_lid: OneEuro::new(params.lid),
            right_lid: OneEuro::new(params.lid),
            expressions: std::array::from_fn(|_| OneEuro::new(params.expression)),
            t_prev: None,
            weights: Weights::new(),
        }
    }

    pub fn parameters(&self) -> EyeFilterParameters {
        self.params
    }

    pub fn set_parameters(&mut self, params: EyeFilterParameters) {
        self.params = params;

        self.pitch.parameters = params.version;
        self.yaw.parameters = params.version;

        self.vergence.parameters = params.vergence;

        self.left_lid.parameters = params.lid;
        self.right_lid.parameters = params.lid;

        for expression in &mut self.expressions {
            expression.parameters = params.expression;
        }
    }

    /// Smooth one frame of fused eye weights.
    ///
    /// The frame is timed from the wall clock.
    pub fn filter(&mut self, fused: &Weights<EyeShape>) -> &Weights<EyeShape> {
        let now = Instant::now();
        let dt = self
            .t_prev
            .map_or(0.0, |prev| now.duration_since(prev).as_secs_f32());
        self.t_prev = Some(now);

        self.step(fused, dt);
        &self.weights
    }

    /// Core filtering step with an explicit frame time (seconds since the
    /// previous frame). `dt <= 0` seeds the filters and passes the frame through.
    fn step(&mut self, fused: &Weights<EyeShape>, dt: f32) {
        self.weights.clear();

        if !self.params.enable {
            for (shape, value) in fused.iter() {
                self.weights.set(shape, value);
            }
            return;
        }

        // Recover the shared direction and the crossing from the per-eye yaw.
        // Reconstructing `version ± vergence` at the end is exactly lossless, so
        // an unfiltered frame comes out identical to how it went in.
        let left_yaw = fused.get(EyeShape::LeftEyeYaw).unwrap_or(0.);
        let right_yaw = fused.get(EyeShape::RightEyeYaw).unwrap_or(0.);
        let left_pitch = fused.get(EyeShape::LeftEyePitch).unwrap_or(0.);
        let right_pitch = fused.get(EyeShape::RightEyePitch).unwrap_or(0.);

        let version_yaw = (left_yaw + right_yaw) * 0.5;
        let vergence = (right_yaw - left_yaw) * 0.5;
        // Pitch is shared already; average defensively in case they differ.
        let pitch_in = (left_pitch + right_pitch) * 0.5;

        // Lids double as the openness / blink signal. Missing lids are treated as
        // open so we never coast without evidence. They are always filtered so the
        // blink stays visible even while gaze is held.
        let left_open = fused.get(EyeShape::LeftEyeLid).unwrap_or(1.0);
        let right_open = fused.get(EyeShape::RightEyeLid).unwrap_or(1.0);
        let left_lid = self.left_lid.filter(left_open, dt);
        let right_lid = self.right_lid.filter(right_open, dt);

        // Coast gaze only when *both* eyes are shut; a wink still tracks off the
        // open eye (its gaze already drives the fused version).
        let openness = left_open.max(right_open);
        let (pitch, version_yaw, vergence) = if openness > self.params.coast_openness {
            (
                self.pitch.filter(pitch_in, dt),
                self.yaw.filter(version_yaw, dt),
                self.vergence.filter(vergence, dt),
            )
        } else {
            (self.pitch.hold(), self.yaw.hold(), self.vergence.hold())
        };

        let left_yaw = version_yaw - vergence;
        let right_yaw = version_yaw + vergence;

        self.weights.set(EyeShape::LeftEyePitch, pitch);
        self.weights.set(EyeShape::RightEyePitch, pitch);
        self.weights.set(EyeShape::LeftEyeYaw, left_yaw);
        self.weights.set(EyeShape::RightEyeYaw, right_yaw);
        self.weights.set(EyeShape::LeftEyeLid, left_lid);
        self.weights.set(EyeShape::RightEyeLid, right_lid);

        for (i, &shape) in EXPRESSION_SHAPES.iter().enumerate() {
            if let Some(value) = fused.get(shape) {
                self.weights
                    .set(shape, self.expressions[i].filter(value, dt));
            }
        }
    }
}

impl Default for EyeFilter {
    fn default() -> Self {
        Self::new()
    }
}
