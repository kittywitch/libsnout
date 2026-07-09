use serde::{Deserialize, Serialize};

use crate::calibration::Bounds;
use crate::weights::{Shape, Weights};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum EyeShape {
    LeftEyePitch,
    LeftEyeYaw,
    LeftEyeLid,
    LeftEyeWiden,
    LeftEyeBrow,
    LeftEyeSquint,
    RightEyePitch,
    RightEyeYaw,
    RightEyeLid,
    RightEyeWiden,
    RightEyeBrow,
    RightEyeSquint,
}

impl From<EyeShape> for usize {
    fn from(value: EyeShape) -> Self {
        value as usize
    }
}

impl From<usize> for EyeShape {
    fn from(value: usize) -> Self {
        assert!(value < Self::count());

        unsafe { std::mem::transmute(value as u8) }
    }
}

impl Shape for EyeShape {
    fn count() -> usize {
        const {
            assert!(Self::RightEyeSquint as usize + 1 == 12);
        }

        Self::RightEyeSquint as usize + 1
    }
}

impl EyeShape {
    pub const fn count() -> usize {
        const {
            assert!(Self::RightEyeSquint as usize + 1 == 12);
        }

        Self::RightEyeSquint as usize + 1
    }

    pub fn from_model_name(name: &str) -> Option<Self> {
        match name {
            "rightEyeY" => Some(Self::RightEyePitch),
            "rightEyeX" => Some(Self::RightEyeYaw),
            "rightEyeLid" => Some(Self::RightEyeLid),
            "rightEyeWiden" => Some(Self::RightEyeWiden),
            "rightEyeBrow" => Some(Self::RightEyeBrow),
            "rightEyeSquint" => Some(Self::RightEyeSquint),

            "leftEyeY" => Some(Self::LeftEyePitch),
            "leftEyeX" => Some(Self::LeftEyeYaw),
            "leftEyeLid" => Some(Self::LeftEyeLid),
            "leftEyeWiden" => Some(Self::LeftEyeWiden),
            "leftEyeBrow" => Some(Self::LeftEyeBrow),
            "leftEyeSquint" => Some(Self::LeftEyeSquint),
            _ => None,
        }
    }

    pub(crate) fn to_etvr(self) -> Option<&'static str> {
        match self {
            Self::LeftEyePitch => Some("/avatar/parameters/v2/EyeLeftY"),
            Self::LeftEyeYaw => Some("/avatar/parameters/v2/EyeLeftX"),
            Self::LeftEyeLid => Some("/avatar/parameters/v2/EyeLidLeft"),

            Self::RightEyePitch => Some("/avatar/parameters/v2/EyeRightY"),
            Self::RightEyeYaw => Some("/avatar/parameters/v2/EyeRightX"),
            Self::RightEyeLid => Some("/avatar/parameters/v2/EyeLidRight"),
            _ => None,
        }
    }

    pub(crate) fn to_babble(self) -> &'static str {
        match self {
            Self::LeftEyePitch => "/leftEyeY",
            Self::LeftEyeYaw => "/leftEyeX",
            Self::LeftEyeLid => "/leftEyeLid",
            Self::LeftEyeWiden => "/leftEyeWiden",
            Self::LeftEyeBrow => "/leftEyeBrow",
            Self::LeftEyeSquint => "/leftEyeSquint",

            Self::RightEyePitch => "/rightEyeY",
            Self::RightEyeYaw => "/rightEyeX",
            Self::RightEyeLid => "/rightEyeLid",
            Self::RightEyeWiden => "/rightEyeWiden",
            Self::RightEyeBrow => "/rightEyeBrow",
            Self::RightEyeSquint => "/rightEyeSquint",
        }
    }

    pub(crate) fn to_etvr_value(self, value: f32) -> f32 {
        if self == Self::LeftEyeLid || self == Self::RightEyeLid {
            1. - value
        } else {
            value
        }
    }
}

/// The neutral gaze position ("straight ahead") for one eye, in the model's raw
/// [0,1] output space. The default 0.5 trusts the model's own center as-is.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct EyeCenter {
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for EyeCenter {
    fn default() -> Self {
        Self {
            yaw: 0.5,
            pitch: 0.5,
        }
    }
}

/// Spatial fusion of raw per-eye gaze estimates into a coherent binocular gaze.
pub struct EyeFusion {
    bounds: Vec<Bounds>,
    left_center: EyeCenter,
    right_center: EyeCenter,
    weights: Weights<EyeShape>,
}

impl EyeFusion {
    pub fn new() -> Self {
        let mut bounds = vec![Bounds::new_01(); EyeShape::count()];

        // Only the gaze axes (pitch/yaw) are centered [-1,1]
        bounds[EyeShape::LeftEyePitch as usize] = Bounds::new_11();
        bounds[EyeShape::LeftEyeYaw as usize] = Bounds::new_11();
        bounds[EyeShape::RightEyePitch as usize] = Bounds::new_11();
        bounds[EyeShape::RightEyeYaw as usize] = Bounds::new_11();

        Self {
            bounds,
            left_center: EyeCenter::default(),
            right_center: EyeCenter::default(),
            weights: Weights::new(),
        }
    }

    pub fn bounds(&self, shape: EyeShape) -> Bounds {
        self.bounds[shape as usize]
    }

    pub fn set_bounds(&mut self, shape: EyeShape, bounds: Bounds) {
        self.bounds[shape as usize] = bounds;
    }

    pub fn left_center(&self) -> EyeCenter {
        self.left_center
    }

    pub fn set_left_center(&mut self, center: EyeCenter) {
        self.left_center = center;
    }

    pub fn right_center(&self) -> EyeCenter {
        self.right_center
    }

    pub fn set_right_center(&mut self, center: EyeCenter) {
        self.right_center = center;
    }

    pub fn calibrate(&mut self, raw: &Weights<EyeShape>) -> &Weights<EyeShape> {
        self.weights.clear();
        self.fuse(raw);
        &self.weights
    }

    /// Fuse the two per-eye gaze estimates into a coherent binocular gaze.
    ///
    /// Gaze is decomposed into *version* (the shared, conjugate direction both
    /// eyes point) and *vergence* (how far the eyes converge toward the nose).
    ///
    /// Version is a precision-weighted average that trusts the more-open eye, so
    /// winks and blinks fall out without a special case. Vergence is trusted only
    /// while both eyes are open and is never allowed to diverge.
    ///
    /// The stage is stateless and always finite: when neither eye is trustworthy
    /// (a blink) it degrades to a plain mean rather than dividing by zero.
    /// Holding gaze through the blink is a temporal concern handled downstream.
    fn fuse(&mut self, raw: &Weights<EyeShape>) {
        // Center each raw gaze channel to [-1,1] around its per-eye center (see
        // `center`); a missing channel defaults to its center, i.e. straight ahead.
        let left_center = self.left_center;
        let right_center = self.right_center;

        let left_pitch = center(
            raw.get(EyeShape::LeftEyePitch).unwrap_or(left_center.pitch),
            left_center.pitch,
        );
        let left_yaw = center(
            raw.get(EyeShape::LeftEyeYaw).unwrap_or(left_center.yaw),
            left_center.yaw,
        );

        let right_pitch = center(
            raw.get(EyeShape::RightEyePitch)
                .unwrap_or(right_center.pitch),
            right_center.pitch,
        );
        let right_yaw = center(
            raw.get(EyeShape::RightEyeYaw).unwrap_or(right_center.yaw),
            right_center.yaw,
        );

        // Openness (1 = fully open) doubles as a confidence proxy: a closing eye
        // occludes its own iris, so its gaze estimate degrades.
        let left_open = 1. - raw.get(EyeShape::LeftEyeLid).unwrap_or(0.);
        let right_open = 1. - raw.get(EyeShape::RightEyeLid).unwrap_or(0.);

        let left_weight = precision(left_open);
        let right_weight = precision(right_open);
        let weight_sum = left_weight + right_weight;

        // Version: the shared conjugate gaze. Precision-weighted while at least one
        // eye is trustworthy, falling back to a plain mean during a blink so the
        // output stays finite (never 0/0).
        let (version_pitch, version_yaw) = if weight_sum > CONFIDENCE_EPSILON {
            (
                (left_pitch * left_weight + right_pitch * right_weight) / weight_sum,
                (left_yaw * left_weight + right_yaw * right_weight) / weight_sum,
            )
        } else {
            (
                (left_pitch + right_pitch) * 0.5,
                (left_yaw + right_yaw) * 0.5,
            )
        };

        // Eyes are yoked: share the version and permit only convergence, never
        // divergence. Vergence is observable only with both eyes open, so fade it
        // out as either one closes.
        let vergence_confidence = left_weight.min(right_weight);
        let vergence = ((right_yaw - left_yaw) * 0.5 * vergence_confidence).max(0.);
        let left_eye_yaw = version_yaw - vergence;
        let right_eye_yaw = version_yaw + vergence;

        // Pitch is fully conjugate: both eyes share the version.
        let pitch = version_pitch.clamp(-1., 1.);

        let left_lid = self.bounds[EyeShape::LeftEyeLid as usize].remap(left_open);
        let right_lid = self.bounds[EyeShape::RightEyeLid as usize].remap(right_open);

        self.weights.set(EyeShape::LeftEyePitch, pitch);
        self.weights
            .set(EyeShape::LeftEyeYaw, left_eye_yaw.clamp(-1., 1.));
        self.weights.set(EyeShape::LeftEyeLid, left_lid);

        self.weights.set(EyeShape::RightEyePitch, pitch);
        self.weights
            .set(EyeShape::RightEyeYaw, right_eye_yaw.clamp(-1., 1.));
        self.weights.set(EyeShape::RightEyeLid, right_lid);

        // Expression channels are remapped through their bounds when present.
        for shape in [
            EyeShape::LeftEyeWiden,
            EyeShape::RightEyeWiden,
            EyeShape::LeftEyeBrow,
            EyeShape::RightEyeBrow,
            EyeShape::LeftEyeSquint,
            EyeShape::RightEyeSquint,
        ] {
            if let Some(value) = raw.get(shape) {
                self.weights
                    .set(shape, self.bounds[shape as usize].remap(value));
            }
        }
    }
}

/// Center a raw gaze channel from [0,1] onto [-1,1], where `c` is the raw value
/// that should read as 0 ("straight ahead").
///
/// The endpoints always map to the extremes (raw 0 -> -1, raw 1 -> +1), so
/// moving `c` slides where center sits *without* clipping the range. With the
/// default c = 0.5 this is exactly `raw * 2 - 1`.
fn center(raw: f32, c: f32) -> f32 {
    // Keep the center away from the extremes so the denominator stays positive.
    let c = c.clamp(0.05, 0.95);
    (raw - c) / ((1. - 2. * c) * raw + c)
}

/// Combined openness below which neither eye is trusted and the fused gaze falls
/// back to a plain average instead of a precision-weighted one.
const CONFIDENCE_EPSILON: f32 = 1e-4;

/// Openness at/below which an eye contributes no confidence.
const OPEN_MIN: f32 = 0.15;

/// Openness at/above which an eye is fully trusted.
const OPEN_FULL: f32 = 0.5;

/// Map eye openness to a fusion weight (precision). Deliberately nonlinear: a
/// mostly-open eye still tracks well, so confidence stays high until the lid is
/// nearly shut, then falls off smoothly.
fn precision(open: f32) -> f32 {
    smoothstep(OPEN_MIN, OPEN_FULL, open)
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0., 1.);
    t * t * (3. - 2. * t)
}
