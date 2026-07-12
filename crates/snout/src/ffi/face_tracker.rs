use crate::{
    calibration::{FaceShape, ManualFaceCalibrator},
    capture::{Frame, discovery::CameraSource, processing::FramePreprocessor},
    config::Config,
    ffi::{
        error::{clear_last_error, set_last_error, set_null_pointer_error},
        query::CAMERA_INFO,
    },
    pipeline::FacePipeline,
    track::face::FaceTracker,
    weights::Weights,
};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SnoutFaceReport {
    /// The raw frame.
    pub raw_frame: *const Frame,
    /// The frame after preprocessing.
    pub processed_frame: *const Frame,
    /// A pointer to the weights.
    pub weights: *const Weights<FaceShape>,
}

impl SnoutFaceReport {
    const fn null() -> Self {
        Self {
            raw_frame: std::ptr::null(),
            processed_frame: std::ptr::null(),
            weights: std::ptr::null(),
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SnoutFaceTrackerFields {
    pub preprocessor: *mut FramePreprocessor,
    pub pipeline: *mut FacePipeline,
    pub calibrator: *mut ManualFaceCalibrator,
}

impl SnoutFaceTrackerFields {
    const fn null() -> Self {
        Self {
            preprocessor: std::ptr::null_mut(),
            pipeline: std::ptr::null_mut(),
            calibrator: std::ptr::null_mut(),
        }
    }
}

/// Creates a new [`FaceTracker`].
#[unsafe(no_mangle)]
pub extern "C" fn snout_face_tracker_new() -> *mut FaceTracker {
    clear_last_error();

    let tracker = FaceTracker::new();

    Box::into_raw(Box::new(tracker))
}

/// Creates a new [`FaceTracker`] with the given configuration.
///
/// You have to make sure `snout_query_cameras` was called before calling this function, otherwise the source will be null.
///
/// Returns null if there was an error, check [`snout_last_error`] for details.
#[unsafe(no_mangle)]
pub extern "C" fn snout_face_tracker_with_config(config: *const Config) -> *mut FaceTracker {
    clear_last_error();

    if config.is_null() {
        return std::ptr::null_mut();
    }

    let cameras = CAMERA_INFO.lock().expect("Failed to acquire lock");

    let config = unsafe { &*config };
    match FaceTracker::with_config(&cameras, config) {
        Ok(tracker) => Box::into_raw(Box::new(tracker)),
        Err(err) => {
            set_last_error(err);
            std::ptr::null_mut()
        }
    }
}

/// Drops a [`FaceTracker`] instance created by [`snout_face_tracker_new`].
#[unsafe(no_mangle)]
pub extern "C" fn snout_face_tracker_free(tracker: *mut FaceTracker) {
    clear_last_error();

    if tracker.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(tracker));
    }
}

/// Set the camera source for the [`FaceTracker`] instance.
///
/// If `source` is null, the camera will be closed.
#[unsafe(no_mangle)]
pub extern "C" fn snout_face_tracker_set_source(
    tracker: *mut FaceTracker,
    source: *const CameraSource,
) {
    clear_last_error();

    if tracker.is_null() {
        set_null_pointer_error();
        return;
    }

    let tracker = unsafe { &mut *tracker };

    let source = if source.is_null() {
        None
    } else {
        let source = unsafe { &*source };
        Some(source.clone())
    };

    tracker.set_source(source);
}

/// Track a face using the [`FaceTracker`] instance.
///
/// Returns a null report if the tracker is null or an error occurs.
/// See [`snout_last_error`] for details.
///
/// If the error is [`SnoutError_Ok`], then there was insufficient data or a transient error.
/// Call [`snout_face_tracker_track`] again to retry.
#[unsafe(no_mangle)]
pub extern "C" fn snout_face_tracker_track(tracker: *mut FaceTracker) -> SnoutFaceReport {
    clear_last_error();

    if tracker.is_null() {
        set_null_pointer_error();
        return SnoutFaceReport::null();
    }

    let tracker = unsafe { &mut *tracker };

    match tracker.track() {
        Ok(Some(report)) => SnoutFaceReport {
            raw_frame: report.raw_frame,
            processed_frame: report.processed_frame,
            weights: report.weights as *const Weights<FaceShape>,
        },
        Ok(None) => SnoutFaceReport::null(),
        Err(e) => {
            set_last_error(e);
            SnoutFaceReport::null()
        }
    }
}

/// Returns the raw pointers to the [`FaceTracker`] fields.
///
/// This can be used for configuring the tracker.
/// Pointers are valid until [`snout_face_tracker_free`] is called.
#[unsafe(no_mangle)]
pub extern "C" fn snout_face_tracker_fields(tracker: *mut FaceTracker) -> SnoutFaceTrackerFields {
    clear_last_error();

    if tracker.is_null() {
        set_null_pointer_error();
        return SnoutFaceTrackerFields::null();
    }

    let tracker = unsafe { &mut *tracker };

    SnoutFaceTrackerFields {
        preprocessor: &mut tracker.preprocessor,
        pipeline: &mut tracker.pipeline,
        calibrator: &mut tracker.calibrator,
    }
}
