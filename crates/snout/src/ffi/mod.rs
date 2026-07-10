#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::unnecessary_cast)]

use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::Path;

use crate::calibration::{EyeFusion, EyeShape, FaceShape, ManualFaceCalibrator};
use crate::capture::Frame;
use crate::capture::{discovery::CameraSource, processing::FramePreprocessor};
use crate::config::Config;
use crate::ffi::error::{clear_last_error, set_last_error, set_null_pointer_error, set_utf8_error};
use crate::ffi::query::CAMERA_INFO;
use crate::output::{BabbleEmitter, EtvrEmitter, OscTransport};
use crate::pipeline::{EyePipeline, FacePipeline};
use crate::track::eye::EyeTracker;
use crate::track::face::FaceTracker;
use crate::track::initialize_runtime;
use crate::track::output::Output;
use crate::weights::Weights;

pub mod error;
pub mod eye_fusion;
pub mod eye_pipeline;
pub mod face_calibrator;
pub mod face_pipeline;
pub mod frame;
pub mod mono_camera;
pub mod preprocessor;
pub mod query;
pub mod stereo_camera;
pub mod weights;

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

/// Create a new UDP OSC transport.
///
/// `destination` is a null-terminated string like "127.0.0.1:9000".
/// Returns null if the socket could not be bound or the address could not be resolved.
/// See [`snout_last_error`] for details.
#[unsafe(no_mangle)]
pub extern "C" fn snout_osc_transport_udp(destination: *const c_char) -> *mut OscTransport {
    clear_last_error();

    if destination.is_null() {
        set_null_pointer_error();
        return std::ptr::null_mut();
    }

    let destination = unsafe { std::ffi::CStr::from_ptr(destination) };
    let destination = match destination.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_utf8_error(e);
            return std::ptr::null_mut();
        }
    };

    match OscTransport::udp(destination) {
        Ok(transport) => Box::into_raw(Box::new(transport)),
        Err(e) => {
            set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// Free an OSC transport.
#[unsafe(no_mangle)]
pub extern "C" fn snout_osc_transport_free(transport: *mut OscTransport) {
    clear_last_error();

    if transport.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(transport));
    }
}

/// Flush the OSC transport.
///
/// Check [`snout_last_error`] to see if an error occurred.
#[unsafe(no_mangle)]
pub extern "C" fn snout_osc_transport_flush(transport: *mut OscTransport) {
    clear_last_error();

    if transport.is_null() {
        set_null_pointer_error();
        return;
    }

    let transport = unsafe { &mut *transport };

    if let Err(e) = transport.flush() {
        set_last_error(e);
    }
}

/// Create a new Babble emitter.
#[unsafe(no_mangle)]
pub extern "C" fn snout_babble_emitter_new() -> *mut BabbleEmitter {
    clear_last_error();

    Box::into_raw(Box::new(BabbleEmitter::new()))
}

/// Free a Babble emitter.
#[unsafe(no_mangle)]
pub extern "C" fn snout_babble_emitter_free(emitter: *mut BabbleEmitter) {
    clear_last_error();

    if emitter.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(emitter));
    }
}

/// Send face weights via the Babble protocol.
#[unsafe(no_mangle)]
pub extern "C" fn snout_babble_emitter_process_face(
    emitter: *mut BabbleEmitter,
    weights: *const Weights<FaceShape>,
    transport: *mut OscTransport,
) {
    clear_last_error();

    if emitter.is_null() || weights.is_null() || transport.is_null() {
        set_null_pointer_error();
        return;
    }

    let emitter = unsafe { &mut *emitter };
    let weights = unsafe { &*weights };
    let transport = unsafe { &mut *transport };

    emitter.process_face(weights, transport);
}

/// Create a new ETVR emitter.
#[unsafe(no_mangle)]
pub extern "C" fn snout_etvr_emitter_new() -> *mut EtvrEmitter {
    clear_last_error();

    Box::into_raw(Box::new(EtvrEmitter::new()))
}

/// Free an ETVR emitter.
#[unsafe(no_mangle)]
pub extern "C" fn snout_etvr_emitter_free(emitter: *mut EtvrEmitter) {
    clear_last_error();

    if emitter.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(emitter));
    }
}

/// Send eye weights via the ETVR protocol.
#[unsafe(no_mangle)]
pub extern "C" fn snout_etvr_emitter_process_eyes(
    emitter: *mut EtvrEmitter,
    weights: *const Weights<EyeShape>,
    transport: *mut OscTransport,
) {
    clear_last_error();

    if emitter.is_null() || weights.is_null() || transport.is_null() {
        set_null_pointer_error();
        return;
    }

    let emitter = unsafe { &mut *emitter };
    let weights = unsafe { &*weights };
    let transport = unsafe { &mut *transport };

    emitter.process_eyes(weights, transport);
}

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
    pub pipeline: *mut EyePipeline,
    pub calibrator: *mut EyeFusion,
}

impl SnoutEyeTrackerFields {
    const fn null() -> Self {
        Self {
            left_preprocessor: std::ptr::null_mut(),
            right_preprocessor: std::ptr::null_mut(),
            pipeline: std::ptr::null_mut(),
            calibrator: std::ptr::null_mut(),
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
        calibrator: &mut tracker.calibrator,
    }
}

// ── Output ──

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SnoutOutputFields {
    pub transport: *mut OscTransport,
    pub babble: *mut BabbleEmitter,
    pub etvr: *mut EtvrEmitter,
}

impl SnoutOutputFields {
    const fn null() -> Self {
        Self {
            transport: std::ptr::null_mut(),
            babble: std::ptr::null_mut(),
            etvr: std::ptr::null_mut(),
        }
    }
}

/// Create a new output.
///
/// You need to call [`snout_output_set_destination`] to set the destination address.
/// The resulting object is owned by the caller and must be freed with [`snout_output_free`].
#[unsafe(no_mangle)]
pub extern "C" fn snout_output_new() -> *mut Output {
    clear_last_error();

    Box::into_raw(Box::new(Output::new()))
}

/// Free an output.
#[unsafe(no_mangle)]
pub extern "C" fn snout_output_free(output: *mut Output) {
    clear_last_error();

    if output.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(output));
    }
}

/// Set the destination address of the output.
///
/// `destination` is a null-terminated string like "127.0.0.1:9400".
#[unsafe(no_mangle)]
pub extern "C" fn snout_output_set_destination(output: *mut Output, destination: *const c_char) {
    clear_last_error();

    if output.is_null() || destination.is_null() {
        set_null_pointer_error();
        return;
    }

    let destination = unsafe { std::ffi::CStr::from_ptr(destination) };
    let destination = match destination.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_utf8_error(e);
            return;
        }
    };

    let output = unsafe { &mut *output };

    if let Err(e) = output.set_destination(destination) {
        set_last_error(e);
    }
}

/// Send face weights via all enabled face emitters.
#[unsafe(no_mangle)]
pub extern "C" fn snout_output_send_face(output: *mut Output, weights: *const Weights<FaceShape>) {
    clear_last_error();

    if output.is_null() || weights.is_null() {
        set_null_pointer_error();
        return;
    }

    let output = unsafe { &mut *output };
    let weights = unsafe { &*weights };

    output.send_face(weights);
}

/// Send eye weights via all enabled eye emitters.
#[unsafe(no_mangle)]
pub extern "C" fn snout_output_send_eyes(output: *mut Output, weights: *const Weights<EyeShape>) {
    clear_last_error();

    if output.is_null() || weights.is_null() {
        set_null_pointer_error();
        return;
    }

    let output = unsafe { &mut *output };
    let weights = unsafe { &*weights };

    output.send_eyes(weights);
}

/// Flush the output transport.
#[unsafe(no_mangle)]
pub extern "C" fn snout_output_flush(output: *mut Output) {
    clear_last_error();

    if output.is_null() {
        set_null_pointer_error();
        return;
    }

    let output = unsafe { &mut *output };

    if let Err(e) = output.flush() {
        set_last_error(e);
    }
}

/// Returns the raw pointers to the [`Output`] fields.
///
/// This can be used for direct access to the transport and emitters.
/// Pointers are valid until [`snout_output_free`] is called.
///
/// The transport pointer is null if no destination is set.
#[unsafe(no_mangle)]
pub extern "C" fn snout_output_fields(output: *mut Output) -> SnoutOutputFields {
    clear_last_error();

    if output.is_null() {
        set_null_pointer_error();
        return SnoutOutputFields::null();
    }

    let output = unsafe { &mut *output };

    let transport = output
        .transport
        .as_mut()
        .map(|t| t as *mut OscTransport)
        .unwrap_or(std::ptr::null_mut());

    SnoutOutputFields {
        transport,
        babble: &mut output.babble,
        etvr: &mut output.etvr,
    }
}

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

/// Load a configuration file from the given path.
///
/// Will return null if the path is null or if the file cannot be parsed.
/// Check [`snout_get_last_error`] to get the error code and message.
///
/// The returned object must be freed with [`snout_config_free`].
#[unsafe(no_mangle)]
pub extern "C" fn snout_config_load(path: *const std::ffi::c_char) -> *mut Config {
    clear_last_error();

    if path.is_null() {
        set_null_pointer_error();
        return std::ptr::null_mut();
    }

    let path = unsafe { CStr::from_ptr(path) };
    let path = match path.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_utf8_error(e);
            return std::ptr::null_mut();
        }
    };

    match crate::config::load(path) {
        Ok(config) => Box::into_raw(Box::new(config)) as *mut Config,
        Err(e) => {
            set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// Free the given config created by [`snout_config_load`].
#[unsafe(no_mangle)]
pub extern "C" fn snout_config_free(config: *mut Config) {
    clear_last_error();

    if config.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(config));
    }
}
