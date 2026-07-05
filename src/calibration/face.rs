use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::calibration::Bounds;
use crate::weights::{Shape, Weights};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum FaceShape {
    CheekPuffLeft,
    CheekPuffRight,
    CheekSuckLeft,
    CheekSuckRight,
    JawOpen,
    JawForward,
    JawLeft,
    JawRight,
    NoseSneerLeft,
    NoseSneerRight,
    MouthFunnel,
    MouthPucker,
    MouthLeft,
    MouthRight,
    MouthRollUpper,
    MouthRollLower,
    MouthShrugUpper,
    MouthShrugLower,
    MouthClose,
    MouthSmileLeft,
    MouthSmileRight,
    MouthFrownLeft,
    MouthFrownRight,
    MouthDimpleLeft,
    MouthDimpleRight,
    MouthUpperUpLeft,
    MouthUpperUpRight,
    MouthLowerDownLeft,
    MouthLowerDownRight,
    MouthPressLeft,
    MouthPressRight,
    MouthStretchLeft,
    MouthStretchRight,
    TongueOut,
    TongueUp,
    TongueDown,
    TongueLeft,
    TongueRight,
    TongueRoll,
    TongueBendDown,
    TongueCurlUp,
    TongueSquish,
    TongueFlat,
    TongueTwistLeft,
    TongueTwistRight,
}

impl FaceShape {
    pub const fn count() -> usize {
        const {
            assert!(Self::TongueTwistRight as usize + 1 == 45);
        }

        Self::TongueTwistRight as usize + 1
    }

    pub(crate) fn to_babble(self) -> &'static str {
        match self {
            FaceShape::CheekPuffLeft => "/cheekPuffLeft",
            FaceShape::CheekPuffRight => "/cheekPuffRight",
            FaceShape::CheekSuckLeft => "/cheekSuckLeft",
            FaceShape::CheekSuckRight => "/cheekSuckRight",
            FaceShape::JawOpen => "/jawOpen",
            FaceShape::JawForward => "/jawForward",
            FaceShape::JawLeft => "/jawLeft",
            FaceShape::JawRight => "/jawRight",
            FaceShape::NoseSneerLeft => "/noseSneerLeft",
            FaceShape::NoseSneerRight => "/noseSneerRight",
            FaceShape::MouthFunnel => "/mouthFunnel",
            FaceShape::MouthPucker => "/mouthPucker",
            FaceShape::MouthLeft => "/mouthLeft",
            FaceShape::MouthRight => "/mouthRight",
            FaceShape::MouthRollUpper => "/mouthRollUpper",
            FaceShape::MouthRollLower => "/mouthRollLower",
            FaceShape::MouthShrugUpper => "/mouthShrugUpper",
            FaceShape::MouthShrugLower => "/mouthShrugLower",
            FaceShape::MouthClose => "/mouthClose",
            FaceShape::MouthSmileLeft => "/mouthSmileLeft",
            FaceShape::MouthSmileRight => "/mouthSmileRight",
            FaceShape::MouthFrownLeft => "/mouthFrownLeft",
            FaceShape::MouthFrownRight => "/mouthFrownRight",
            FaceShape::MouthDimpleLeft => "/mouthDimpleLeft",
            FaceShape::MouthDimpleRight => "/mouthDimpleRight",
            FaceShape::MouthUpperUpLeft => "/mouthUpperUpLeft",
            FaceShape::MouthUpperUpRight => "/mouthUpperUpRight",
            FaceShape::MouthLowerDownLeft => "/mouthLowerDownLeft",
            FaceShape::MouthLowerDownRight => "/mouthLowerDownRight",
            FaceShape::MouthPressLeft => "/mouthPressLeft",
            FaceShape::MouthPressRight => "/mouthPressRight",
            FaceShape::MouthStretchLeft => "/mouthStretchLeft",
            FaceShape::MouthStretchRight => "/mouthStretchRight",
            FaceShape::TongueOut => "/tongueOut",
            FaceShape::TongueUp => "/tongueUp",
            FaceShape::TongueDown => "/tongueDown",
            FaceShape::TongueLeft => "/tongueLeft",
            FaceShape::TongueRight => "/tongueRight",
            FaceShape::TongueRoll => "/tongueRoll",
            FaceShape::TongueBendDown => "/tongueBendDown",
            FaceShape::TongueCurlUp => "/tongueCurlUp",
            FaceShape::TongueSquish => "/tongueSquish",
            FaceShape::TongueFlat => "/tongueFlat",
            FaceShape::TongueTwistLeft => "/tongueTwistLeft",
            FaceShape::TongueTwistRight => "/tongueTwistRight",
        }
    }
}

impl From<FaceShape> for usize {
    fn from(value: FaceShape) -> Self {
        value as usize
    }
}

impl From<usize> for FaceShape {
    fn from(value: usize) -> Self {
        assert!(value < Self::count());

        unsafe { std::mem::transmute(value as u8) }
    }
}

/// Error returned when a string does not name a [`FaceShape`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ParseFaceShapeError;

impl std::fmt::Display for ParseFaceShapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("unknown face shape")
    }
}

impl std::error::Error for ParseFaceShapeError {}

impl FromStr for FaceShape {
    type Err = ParseFaceShapeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "cheekPuffLeft" => Self::CheekPuffLeft,
            "cheekPuffRight" => Self::CheekPuffRight,
            "cheekSuckLeft" => Self::CheekSuckLeft,
            "cheekSuckRight" => Self::CheekSuckRight,
            "jawOpen" => Self::JawOpen,
            "jawForward" => Self::JawForward,
            "jawLeft" => Self::JawLeft,
            "jawRight" => Self::JawRight,
            "noseSneerLeft" => Self::NoseSneerLeft,
            "noseSneerRight" => Self::NoseSneerRight,
            "mouthFunnel" => Self::MouthFunnel,
            "mouthPucker" => Self::MouthPucker,
            "mouthLeft" => Self::MouthLeft,
            "mouthRight" => Self::MouthRight,
            "mouthRollUpper" => Self::MouthRollUpper,
            "mouthRollLower" => Self::MouthRollLower,
            "mouthShrugUpper" => Self::MouthShrugUpper,
            "mouthShrugLower" => Self::MouthShrugLower,
            "mouthClose" => Self::MouthClose,
            "mouthSmileLeft" => Self::MouthSmileLeft,
            "mouthSmileRight" => Self::MouthSmileRight,
            "mouthFrownLeft" => Self::MouthFrownLeft,
            "mouthFrownRight" => Self::MouthFrownRight,
            "mouthDimpleLeft" => Self::MouthDimpleLeft,
            "mouthDimpleRight" => Self::MouthDimpleRight,
            "mouthUpperUpLeft" => Self::MouthUpperUpLeft,
            "mouthUpperUpRight" => Self::MouthUpperUpRight,
            "mouthLowerDownLeft" => Self::MouthLowerDownLeft,
            "mouthLowerDownRight" => Self::MouthLowerDownRight,
            "mouthPressLeft" => Self::MouthPressLeft,
            "mouthPressRight" => Self::MouthPressRight,
            "mouthStretchLeft" => Self::MouthStretchLeft,
            "mouthStretchRight" => Self::MouthStretchRight,
            "tongueOut" => Self::TongueOut,
            "tongueUp" => Self::TongueUp,
            "tongueDown" => Self::TongueDown,
            "tongueLeft" => Self::TongueLeft,
            "tongueRight" => Self::TongueRight,
            "tongueRoll" => Self::TongueRoll,
            "tongueBendDown" => Self::TongueBendDown,
            "tongueCurlUp" => Self::TongueCurlUp,
            "tongueSquish" => Self::TongueSquish,
            "tongueFlat" => Self::TongueFlat,
            "tongueTwistLeft" => Self::TongueTwistLeft,
            "tongueTwistRight" => Self::TongueTwistRight,
            _ => return Err(ParseFaceShapeError),
        })
    }
}

impl Shape for FaceShape {
    fn count() -> usize {
        const {
            assert!(Self::TongueTwistRight as usize + 1 == 45);
        }

        Self::TongueTwistRight as usize + 1
    }
}

pub struct ManualFaceCalibrator {
    bounds: Vec<Bounds>,
    weights: Weights<FaceShape>,
    calibration: Option<Calibration>,
}

/// Number of frames collected during a calibration pass.
/// At ~30 fps this is roughly three seconds of neutral hold.
const CALIBRATION_SAMPLES: usize = 100;

/// In-progress neutral-hold calibration
struct Calibration {
    /// Frames left to collect before finalizing.
    remaining: usize,
    /// Running sum of raw values, indexed by shape.
    sums: Vec<f32>,
    /// Number of frames each shape was observed in, indexed by shape.
    counts: Vec<u32>,
}

impl Calibration {
    fn new() -> Self {
        Self {
            remaining: CALIBRATION_SAMPLES,
            sums: vec![0.0; FaceShape::count()],
            counts: vec![0; FaceShape::count()],
        }
    }
}

impl ManualFaceCalibrator {
    pub fn new() -> Self {
        Self {
            bounds: vec![Bounds::new_01(); FaceShape::count()],
            weights: Weights::new(),
            calibration: None,
        }
    }

    pub fn bounds(&self, shape: FaceShape) -> Bounds {
        self.bounds[shape as usize]
    }

    pub fn set_bounds(&mut self, shape: FaceShape, bounds: Bounds) {
        tracing::debug!(shape = ?shape, ?bounds, "set_bounds");
        self.bounds[shape as usize] = bounds;
    }

    pub fn set_upper(&mut self, shape: FaceShape, upper: f32) {
        tracing::debug!(shape = ?shape, upper, "set_upper");
        self.bounds[shape as usize].upper = upper;
    }

    pub fn set_lower(&mut self, shape: FaceShape, lower: f32) {
        tracing::debug!(shape = ?shape, lower, "set_lower");
        self.bounds[shape as usize].lower = lower;
    }

    /// Begins a neutral-hold calibration pass.
    ///
    /// Over the next [`CALIBRATION_SAMPLES`] calls to [`Self::calibrate`],
    /// raw shape values are averaged and the resulting per-shape means become the new lower bounds.
    ///
    /// Calling this while a pass is already running restarts it from scratch.
    pub fn start_calibration(&mut self) {
        tracing::debug!(samples = CALIBRATION_SAMPLES, "start_calibration");
        self.calibration = Some(Calibration::new());
    }

    pub fn calibrate(&mut self, raw: &Weights<FaceShape>) -> &Weights<FaceShape> {
        self.weights.clear();

        for (shape, value) in raw.iter() {
            let bounds = &self.bounds[<FaceShape as Into<usize>>::into(shape)];
            self.weights.set(shape, bounds.remap(value));
        }

        self.collect_calibration(raw);

        &self.weights
    }

    fn collect_calibration(&mut self, raw: &Weights<FaceShape>) {
        let finished = if let Some(calibration) = &mut self.calibration {
            for (shape, value) in raw.iter() {
                let index = shape as usize;
                calibration.sums[index] += value;
                calibration.counts[index] += 1;
            }

            calibration.remaining -= 1;
            calibration.remaining == 0
        } else {
            false
        };

        if finished {
            let calibration = self.calibration.take().unwrap();

            for shape in FaceShape::iter() {
                let index = shape as usize;
                if calibration.counts[index] > 0 {
                    let mean = calibration.sums[index] / calibration.counts[index] as f32;
                    tracing::debug!(?shape, lower = mean, "calibrated lower bound");
                    self.bounds[index].lower = mean;
                }
            }
        }
    }
}
