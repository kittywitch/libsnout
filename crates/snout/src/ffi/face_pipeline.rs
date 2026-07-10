use std::ffi::c_char;

use crate::{
    calibration::FaceShape,
    capture::Frame,
    ffi::error::{clear_last_error, set_last_error, set_null_pointer_error, set_utf8_error},
    pipeline::FacePipeline,
    weights::Weights,
};

/// Create a new face pipeline.
///
/// Returns a pointer to the pipeline.
#[unsafe(no_mangle)]
pub extern "C" fn snout_face_pipeline_new() -> *mut FacePipeline {
    clear_last_error();

    let pipeline = FacePipeline::new();

    Box::into_raw(Box::new(pipeline))
}

/// Set the model for the face pipeline from the given path.
///
/// Returns true if the model was loaded successfully, false otherwise.
/// Check [`snout_last_error`] for details.
///
/// If path is null, the model will be unloaded.
#[unsafe(no_mangle)]
pub extern "C" fn snout_face_pipeline_set_model(
    pipeline: *mut FacePipeline,
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

/// Run the face pipeline on a frame.
///
/// Returns a pointer to a `Weights<FaceShape>`, or null if the pipeline
/// was not ready yet or an error occurred.
///
/// The returned pointer is valid until the next call to [`snout_face_pipeline_run`]
/// or [`snout_face_pipeline_free`].
///
/// Check [`snout_get_last_error`] to determine which.
/// It will be `SnoutError_Ok` if the pipeline was not ready yet.
#[unsafe(no_mangle)]
pub extern "C" fn snout_face_pipeline_run(
    pipeline: *mut FacePipeline,
    frame: *const Frame,
) -> *const Weights<FaceShape> {
    clear_last_error();

    if pipeline.is_null() || frame.is_null() {
        set_null_pointer_error();
        return std::ptr::null();
    }

    let pipeline = unsafe { &mut *pipeline };
    let frame = unsafe { &*frame };

    match pipeline.run(frame) {
        Ok(Some(weights)) => weights,
        Ok(None) => std::ptr::null(),
        Err(e) => {
            set_last_error(e);
            std::ptr::null()
        }
    }
}

/// Free the face pipeline.
#[unsafe(no_mangle)]
pub extern "C" fn snout_face_pipeline_free(pipeline: *mut FacePipeline) {
    clear_last_error();

    if pipeline.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(pipeline));
    }
}
