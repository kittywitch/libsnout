#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::unnecessary_cast)]

use std::ffi::CStr;
use std::path::Path;

use crate::ffi::error::{clear_last_error, set_utf8_error};
use crate::track::initialize_runtime;

pub mod babble_emitter;
pub mod config;
pub mod error;
pub mod etvr_emitter;
pub mod eye_fusion;
pub mod eye_pipeline;
pub mod eye_tracker;
pub mod face_calibrator;
pub mod face_pipeline;
pub mod face_tracker;
pub mod frame;
pub mod mono_camera;
pub mod osc_transport;
pub mod output;
pub mod preprocessor;
pub mod query;
pub mod stereo_camera;
pub mod weights;

/// Initialize the runtime.
///
/// If `path` is not null, it will be considered first when searching for `libonnxruntime.so`.
#[unsafe(no_mangle)]
pub extern "C" fn snout_initialize_runtime(path: *const std::ffi::c_char) {
    clear_last_error();

    let path = if path.is_null() {
        None
    } else {
        let path = unsafe { CStr::from_ptr(path) };
        let path = match path.to_str() {
            Ok(s) => s,
            Err(e) => {
                set_utf8_error(e);
                return;
            }
        };

        Some(Path::new(path))
    };

    initialize_runtime(path);
}
