use crate::{
    calibration::{EyeFusion, EyeShape},
    capture::{Frame, discovery::CameraSource, processing::FramePreprocessor},
    config::Config,
    ffi::{
        error::{clear_last_error, set_last_error, set_null_pointer_error},
        query::CAMERA_INFO,
    },
    pipeline::EyePipeline,
    track::eye::EyeTracker,
    weights::Weights,
};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SnoutEyeReport {
    /// The raw left frame.
    pub left_raw_frame: *const Frame,
    /// The raw right frame.
    pub right_raw_frame: *const Frame,
    /// The left frame after preprocessing.
    pub left_processed_frame: *const Frame,
    /// The right frame after preprocessing.
    pub right_processed_frame: *const Frame,
    /// A pointer to the weights, or null during warmup.
    pub weights: *const Weights<EyeShape>,
}

impl SnoutEyeReport {
    const fn null() -> Self {
        Self {
            left_raw_frame: std::ptr::null(),
            right_raw_frame: std::ptr::null(),
            left_processed_frame: std::ptr::null(),
            right_processed_frame: std::ptr::null(),
            weights: std::ptr::null(),
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SnoutEyeTrackerFields {
    pub left_preprocessor: *mut FramePreprocessor,
    pub right_preprocessor: *mut FramePreprocessor,
    pub fusion: *mut EyeFusion,
    pub pipeline: *mut EyePipeline,
}

impl SnoutEyeTrackerFields {
    const fn null() -> Self {
        Self {
            left_preprocessor: std::ptr::null_mut(),
            right_preprocessor: std::ptr::null_mut(),
            pipeline: std::ptr::null_mut(),
            fusion: std::ptr::null_mut(),
        }
    }
}

/// Creates a new [`EyeTracker`].
#[unsafe(no_mangle)]
pub extern "C" fn snout_eye_tracker_new() -> *mut EyeTracker {
    clear_last_error();

    let tracker = EyeTracker::new();
    Box::into_raw(Box::new(tracker))
}

/// Creates a new [`EyeTracker`] with the given configuration.
///
/// You have to make sure `snout_query_cameras` was called before calling this function, otherwise the source will be null.
///
/// Returns null if there was an error, check [`snout_last_error`] for details.
#[unsafe(no_mangle)]
pub extern "C" fn snout_eye_tracker_with_config(config: *const Config) -> *mut EyeTracker {
    clear_last_error();

    if config.is_null() {
        return std::ptr::null_mut();
    }

    let cameras = CAMERA_INFO.lock().expect("Failed to acquire lock");

    let config = unsafe { &*config };
    match EyeTracker::with_config(&cameras, config) {
        Ok(tracker) => Box::into_raw(Box::new(tracker)),
        Err(err) => {
            set_last_error(err);
            std::ptr::null_mut()
        }
    }
}

/// Drops an [`EyeTracker`] instance created by [`snout_eye_tracker_new`].
#[unsafe(no_mangle)]
pub extern "C" fn snout_eye_tracker_free(tracker: *mut EyeTracker) {
    clear_last_error();

    if tracker.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(tracker));
    }
}

/// Set the camera sources for the [`EyeTracker`] instance.
///
/// If both sources are null, the camera will be closed.
/// If left and right point to the same source, the camera will be opened in side-by-side mode.
#[unsafe(no_mangle)]
pub extern "C" fn snout_eye_tracker_set_source(
    tracker: *mut EyeTracker,
    left: *const CameraSource,
    right: *const CameraSource,
) {
    clear_last_error();

    if tracker.is_null() {
        set_null_pointer_error();
        return;
    }

    let tracker = unsafe { &mut *tracker };

    let left = if left.is_null() {
        None
    } else {
        let left = unsafe { &*left };
        Some(left.clone())
    };

    let right = if right.is_null() {
        None
    } else {
        let right = unsafe { &*right };
        Some(right.clone())
    };

    tracker.set_source(left, right);
}

/// Track eyes using the [`EyeTracker`] instance.
///
/// Returns a null report if the tracker is null or an error occurs.
/// See [`snout_last_error`] for details.
///
/// If the error is [`SnoutError_Ok`], then there was insufficient data or a transient error.
#[unsafe(no_mangle)]
pub extern "C" fn snout_eye_tracker_track(tracker: *mut EyeTracker) -> SnoutEyeReport {
    clear_last_error();

    if tracker.is_null() {
        set_null_pointer_error();
        return SnoutEyeReport::null();
    }

    let tracker = unsafe { &mut *tracker };

    match tracker.track() {
        Ok(Some(report)) => SnoutEyeReport {
            left_raw_frame: report.left_raw_frame,
            right_raw_frame: report.right_raw_frame,
            left_processed_frame: report.left_processed_frame,
            right_processed_frame: report.right_processed_frame,
            weights: report.weights as *const Weights<EyeShape>,
        },
        Ok(None) => SnoutEyeReport::null(),
        Err(e) => {
            set_last_error(e);
            SnoutEyeReport::null()
        }
    }
}

/// Returns the raw pointers to the [`EyeTracker`] fields.
///
/// This can be used for configuring the tracker.
/// Pointers are valid until [`snout_eye_tracker_free`] is called.
#[unsafe(no_mangle)]
pub extern "C" fn snout_eye_tracker_fields(tracker: *mut EyeTracker) -> SnoutEyeTrackerFields {
    clear_last_error();

    if tracker.is_null() {
        set_null_pointer_error();
        return SnoutEyeTrackerFields::null();
    }

    let tracker = unsafe { &mut *tracker };

    SnoutEyeTrackerFields {
        left_preprocessor: &mut tracker.left_preprocessor,
        right_preprocessor: &mut tracker.right_preprocessor,
        pipeline: &mut tracker.pipeline,
        fusion: &mut tracker.calibrator,
    }
}
