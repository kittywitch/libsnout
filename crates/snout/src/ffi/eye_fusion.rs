use crate::{
    calibration::{Bounds, EyeFusion, EyeShape},
    ffi::error::{clear_last_error, set_null_pointer_error},
    weights::Weights,
};

// TODO: snout_eye_fusion_left_center
// TODO: snout_eye_fusion_right_center
// TODO: snout_eye_fusion_set_left_center
// TODO: snout_eye_fusion_set_right_center

/// Create a new eye fusion.
#[unsafe(no_mangle)]
pub extern "C" fn snout_eye_fusion_new() -> *mut EyeFusion {
    clear_last_error();

    Box::into_raw(Box::new(EyeFusion::new()))
}

/// Get the calibration bounds for an eye shape.
#[unsafe(no_mangle)]
pub extern "C" fn snout_eye_fusion_bounds(fusion: *const EyeFusion, shape: EyeShape) -> Bounds {
    clear_last_error();

    if fusion.is_null() {
        set_null_pointer_error();
        return Bounds::new();
    }

    let fusion = unsafe { &*fusion };

    fusion.bounds(shape)
}

/// Set the calibration bounds for an eye shape.
#[unsafe(no_mangle)]
pub extern "C" fn snout_eye_fusion_set_bounds(
    fusion: *mut EyeFusion,
    shape: EyeShape,
    bounds: Bounds,
) {
    clear_last_error();

    if fusion.is_null() {
        set_null_pointer_error();
        return;
    }

    let fusion = unsafe { &mut *fusion };

    fusion.set_bounds(shape, bounds);
}

/// Calibrate raw eye weights.
///
/// Returns a pointer to calibrated `Weights<EyeShape>`, or null if an error occurred.
///
/// The returned pointer is valid until the next call to [`snout_eye_fusion_calibrate`]
/// or [`snout_eye_fusion_free`].
#[unsafe(no_mangle)]
pub extern "C" fn snout_eye_fusion_calibrate(
    fusion: *mut EyeFusion,
    weights: *const Weights<EyeShape>,
) -> *const Weights<EyeShape> {
    clear_last_error();

    if fusion.is_null() || weights.is_null() {
        set_null_pointer_error();
        return std::ptr::null();
    }

    let fusion = unsafe { &mut *fusion };
    let weights = unsafe { &*weights };

    fusion.calibrate(weights)
}

/// Free the eye fusion.
///
/// Does nothing if the pointer is null.
#[unsafe(no_mangle)]
pub extern "C" fn snout_eye_fusion_free(fusion: *mut EyeFusion) {
    clear_last_error();

    if fusion.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(fusion));
    }
}
