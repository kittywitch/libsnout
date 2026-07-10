use std::path::PathBuf;

use crate::calibration::FaceShape;

#[derive(Clone, Debug, PartialEq)]
#[repr(C)]
pub enum ControlEvent {
    Face { event: FaceEvent },
    Eye { event: EyeEvent },
}

#[derive(Clone, Debug, PartialEq)]
#[repr(C)]
pub enum FaceEvent {
    CalibrateLower {
        frames: u32,
    },
    CalibrateUpper {
        shape: FaceShape,
        frames: u32,
    },
    SetBounds {
        shape: FaceShape,
        lower: f32,
        upper: f32,
    },
    Capture {
        path: PathBuf,
    },
}

#[derive(Clone, Debug, PartialEq)]
#[repr(C)]
pub enum EyeEvent {
    // CalibrateCenter { frames: u32 },
    Capture { side: Side, path: PathBuf },
}

#[derive(Clone, Debug, PartialEq)]
#[repr(C)]
pub enum Side {
    Left,
    Right,
}
