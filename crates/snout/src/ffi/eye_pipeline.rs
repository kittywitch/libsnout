use std::ffi::c_char;

use crate::{
    calibration::EyeShape,
    capture::Frame,
    ffi::error::{clear_last_error, set_last_error, set_null_pointer_error, set_utf8_error},
    pipeline::EyePipeline,
    weights::Weights,
};

/// Create a new eye pipeline.
#[unsafe(no_mangle)]
pub extern "C" fn snout_eye_pipeline_new() -> *mut EyePipeline {
    clear_last_error();

    let pipeline = EyePipeline::new();
    Box::into_raw(Box::new(pipeline))
}

/// Set the model for the eye pipeline from the given path.
///
/// Returns true if the model was loaded successfully, false otherwise.
/// Check [`snout_last_error`] for details.
///
/// If path is null, the model will be unloaded.
#[unsafe(no_mangle)]
pub extern "C" fn snout_eye_pipeline_set_model(
    pipeline: *mut EyePipeline,
    path: *const c_char,
) -> bool {
    clear_last_error();

    let path = if path.is_null() {
        None
    } else {
        let path = unsafe { std::ffi::CStr::from_ptr(path) };

        Some(match path.to_str() {
            Ok(s) => s,
            Err(e) => {
                set_utf8_error(e);
                return false;
            }
        })
    };

    if pipeline.is_null() {
        set_null_pointer_error();
        return false;
    }

    let pipeline = unsafe { &mut *pipeline };

    match pipeline.set_model(path) {
        Ok(()) => true,
        Err(e) => {
            set_last_error(e);
            false
        }
    }
}

/// Run the eye pipeline on a pair of stereo frames.
///
/// Returns a pointer to a `Weights<EyeShape>`, or null if the pipeline
/// was not ready yet or an error occurred.
///
/// The returned pointer is valid until the next call to [`snout_eye_pipeline_run`]
/// or [`snout_eye_pipeline_free`].
///
/// Check [`snout_last_error`] to determine which.
/// It will be `SnoutError_Ok` if the pipeline was not ready yet.
#[unsafe(no_mangle)]
pub extern "C" fn snout_eye_pipeline_run(
    pipeline: *mut EyePipeline,
    left: *const Frame,
    right: *const Frame,
) -> *const Weights<EyeShape> {
    clear_last_error();

    if pipeline.is_null() || left.is_null() || right.is_null() {
        set_null_pointer_error();
        return std::ptr::null();
    }

    let pipeline = unsafe { &mut *pipeline };
    let left = unsafe { &*left };
    let right = unsafe { &*right };

    match pipeline.run(left, right) {
        Ok(Some(weights)) => weights,
        Ok(None) => std::ptr::null(),
        Err(e) => {
            set_last_error(e);
            std::ptr::null()
        }
    }
}

/// Free the eye pipeline.
#[unsafe(no_mangle)]
pub extern "C" fn snout_eye_pipeline_free(pipeline: *mut EyePipeline) {
    clear_last_error();

    if pipeline.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(pipeline));
    }
}
