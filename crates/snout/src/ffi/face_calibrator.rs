use crate::{
    calibration::{Bounds, FaceShape, ManualFaceCalibrator},
    ffi::error::{clear_last_error, set_null_pointer_error},
    weights::Weights,
};

/// Create a new face calibrator.
#[unsafe(no_mangle)]
pub extern "C" fn snout_face_calibrator_new() -> *mut ManualFaceCalibrator {
    clear_last_error();

    Box::into_raw(Box::new(ManualFaceCalibrator::new()))
}

/// Get the calibration bounds for a face shape.
#[unsafe(no_mangle)]
pub extern "C" fn snout_face_calibrator_bounds(
    calibrator: *const ManualFaceCalibrator,
    shape: FaceShape,
) -> Bounds {
    clear_last_error();

    if calibrator.is_null() {
        set_null_pointer_error();
        return Bounds::new();
    }

    let calibrator = unsafe { &*calibrator };

    calibrator.bounds(shape)
}

/// Set the calibration bounds for a face shape.
#[unsafe(no_mangle)]
pub extern "C" fn snout_face_calibrator_set_bounds(
    calibrator: *mut ManualFaceCalibrator,
    shape: FaceShape,
    bounds: Bounds,
) {
    clear_last_error();

    if calibrator.is_null() {
        set_null_pointer_error();
        return;
    }

    let calibrator = unsafe { &mut *calibrator };

    calibrator.set_bounds(shape, bounds);
}

/// Start auto calibration of the upper bounds for a particular shape.
#[unsafe(no_mangle)]
pub extern "C" fn snout_face_calibrator_start_upper_calibration(
    calibrator: *mut ManualFaceCalibrator,
    shape: FaceShape,
    frames: usize,
) {
    clear_last_error();

    if calibrator.is_null() {
        set_null_pointer_error();
        return;
    }

    let calibrator = unsafe { &mut *calibrator };

    calibrator.start_upper_calibration(shape, frames);
}

/// Start auto calibration of the lower bounds.
#[unsafe(no_mangle)]
pub extern "C" fn snout_face_calibrator_start_lower_calibration(
    calibrator: *mut ManualFaceCalibrator,
    frames: usize,
) {
    clear_last_error();

    if calibrator.is_null() {
        set_null_pointer_error();
        return;
    }

    let calibrator = unsafe { &mut *calibrator };

    calibrator.start_lower_calibration(frames);
}

/// Calibrate raw face weights.
///
/// Returns a pointer to calibrated `Weights<FaceShape>`, or null if an error occurred.
///
/// The returned pointer is valid until the next call to [`snout_face_calibrator_calibrate`]
/// or [`snout_face_calibrator_free`].
#[unsafe(no_mangle)]
pub extern "C" fn snout_face_calibrator_calibrate(
    calibrator: *mut ManualFaceCalibrator,
    weights: *const Weights<FaceShape>,
) -> *const Weights<FaceShape> {
    clear_last_error();

    if calibrator.is_null() || weights.is_null() {
        set_null_pointer_error();
        return std::ptr::null();
    }

    let calibrator = unsafe { &mut *calibrator };
    let weights = unsafe { &*weights };

    calibrator.calibrate(weights)
}

///
/// Does nothing if the pointer is null.
#[unsafe(no_mangle)]
pub extern "C" fn snout_face_calibrator_free(calibrator: *mut ManualFaceCalibrator) {
    clear_last_error();

    if calibrator.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(calibrator));
    }
}
