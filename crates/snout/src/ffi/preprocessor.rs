use crate::{
    capture::{
        Frame,
        processing::{Crop, FramePreprocessor, PreprocessConfig},
    },
    ffi::error::{clear_last_error, set_last_error, set_null_pointer_error},
};

/// Create a new frame preprocessor.
#[unsafe(no_mangle)]
pub extern "C" fn snout_frame_preprocessor_new() -> *mut FramePreprocessor {
    clear_last_error();

    Box::into_raw(Box::new(FramePreprocessor::new()))
}

/// Free the frame preprocessor created by [`snout_frame_preprocessor_new`].
#[unsafe(no_mangle)]
pub extern "C" fn snout_frame_preprocessor_free(preprocessor: *mut FramePreprocessor) {
    clear_last_error();

    if preprocessor.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(preprocessor));
    }
}

/// Get the current preprocessing configuration.
///
/// returns a copy of the current configuration.
#[unsafe(no_mangle)]
pub extern "C" fn snout_frame_preprocessor_config(
    preprocessor: *const FramePreprocessor,
) -> PreprocessConfig {
    clear_last_error();

    if preprocessor.is_null() {
        set_null_pointer_error();
        return PreprocessConfig::default();
    }

    let preprocessor = unsafe { &*preprocessor };

    *preprocessor.config()
}

/// Set the preprocessing configuration.
#[unsafe(no_mangle)]
pub extern "C" fn snout_frame_preprocessor_set_config(
    preprocessor: *mut FramePreprocessor,
    config: PreprocessConfig,
) {
    clear_last_error();

    if preprocessor.is_null() {
        set_null_pointer_error();
        return;
    }

    let preprocessor = unsafe { &mut *preprocessor };

    preprocessor.set_config(config);
}

/// Get the current preprocessing crop.
///
/// returns a copy of the current crop.
#[unsafe(no_mangle)]
pub extern "C" fn snout_frame_preprocessor_crop(preprocessor: *const FramePreprocessor) -> Crop {
    clear_last_error();

    if preprocessor.is_null() {
        set_null_pointer_error();
        return Crop::default();
    }

    let preprocessor = unsafe { &*preprocessor };

    preprocessor.crop()
}

/// Set the preprocessing crop.
#[unsafe(no_mangle)]
pub extern "C" fn snout_frame_preprocessor_set_crop(
    preprocessor: *mut FramePreprocessor,
    crop: *const Crop,
) {
    clear_last_error();

    if preprocessor.is_null() {
        set_null_pointer_error();
        return;
    }

    if crop.is_null() {
        set_null_pointer_error();
        return;
    }

    let crop = unsafe { *crop };

    let preprocessor = unsafe { &mut *preprocessor };

    preprocessor.set_crop(crop);
}

/// Process a frame using the preprocessor.
///
/// Returns a pointer to the processed frame, or null if an error occurred.
/// The returned frame is valid until the next call to [`snout_frame_preprocessor_process`]
/// or [`snout_frame_preprocessor_free`].
#[unsafe(no_mangle)]
pub extern "C" fn snout_frame_preprocessor_process(
    preprocessor: *mut FramePreprocessor,
    frame: *const Frame,
) -> *const Frame {
    clear_last_error();

    if preprocessor.is_null() || frame.is_null() {
        set_null_pointer_error();
        return std::ptr::null();
    }

    let preprocessor = unsafe { &mut *preprocessor };
    let frame = unsafe { &*frame };

    match preprocessor.process(frame) {
        Ok(result) => result as *const Frame,
        Err(e) => {
            set_last_error(e);
            std::ptr::null()
        }
    }
}
