use crate::{
    calibration::{EyeShape, FaceShape},
    ffi::error::set_null_pointer_error,
    weights::Weights,
};

/// The number of face shapes.
#[unsafe(no_mangle)]
pub static SNOUT_FACE_SHAPE_COUNT: usize = FaceShape::count();

/// The number of eye shapes.
#[unsafe(no_mangle)]
pub static SNOUT_EYE_SHAPE_COUNT: usize = EyeShape::count();

/// Gets the weight for a given shape.
///
/// Returns `true` if the weight was found, `false` otherwise.
/// If out is not null, it will be set to the value.
#[unsafe(no_mangle)]
pub extern "C" fn snout_face_weights_get(
    weights: *mut Weights<FaceShape>,
    shape: FaceShape,
    out: *mut f32,
) -> bool {
    if weights.is_null() {
        set_null_pointer_error();
        return false;
    }

    let weights = unsafe { &*weights };
    let value = weights.get(shape);

    if let Some(value) = value {
        if !out.is_null() {
            unsafe { *out = value };
        }

        true
    } else {
        false
    }
}

/// Gets the weight for a given shape.
///
/// Returns `true` if the weight was found, `false` otherwise.
/// If out is not null, it will be set to the value.
#[unsafe(no_mangle)]
pub extern "C" fn snout_eye_weights_get(
    weights: *mut Weights<EyeShape>,
    shape: EyeShape,
    out: *mut f32,
) -> bool {
    if weights.is_null() {
        set_null_pointer_error();
        return false;
    }

    let weights = unsafe { &*weights };
    let value = weights.get(shape);

    if let Some(value) = value {
        if !out.is_null() {
            unsafe { *out = value };
        }

        true
    } else {
        false
    }
}
