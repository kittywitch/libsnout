use std::{cell::RefCell, ffi::c_char};

use crate::{
    capture::{CameraError, processing::PreprocessError},
    config::ConfigError,
    output::TransportError,
    pipeline::PipelineError,
    track::TrackerError,
};

/// Represents an error that occurred during a Snout operation.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum SnoutError {
    /// The operation completed successfully.
    Ok,
    /// An null pointer was passed to a function that requires a valid pointer.
    NullPointer,
    /// The input string was not valid UTF-8.
    InvalidUtf8,
    /// The camera failed to open due to an invalid format.
    CameraInvalidFormat,
    /// An invalid frame was received from the camera.
    ///
    /// This might mean the camera was disconnected, or could be a transient error.
    CameraInvalidFrame,
    /// No usable camera sources were configured.
    CameraInvalidSources,
    /// An internal error occurred during camera operations.
    CameraInternal,
    /// An internal error occurred during preprocessing.
    PreprocessInternal,
    /// The pipeline failed to load.
    PipelineLoad,
    /// The pipeline failed during inference.
    PipelineInference,
    /// The tracker failed to load the model.
    TrackerModel,
    /// The tracker failed to open the camera.
    TrackerOpen,
    /// An internal error occurred during tracking.
    TrackerInternal,
    /// Failed to bind the transport socket.
    TransportBind,
    /// Failed to resolve the transport destination address.
    TransportResolve,
    /// The config file could not be found.
    ConfigFileNotFound,
    /// The config file could not be parsed.
    ConfigInvalidConfig,
}

impl From<TransportError> for SnoutError {
    fn from(error: TransportError) -> Self {
        match error {
            TransportError::Bind => SnoutError::TransportBind,
            TransportError::Resolve => SnoutError::TransportResolve,
        }
    }
}

impl From<TrackerError> for SnoutError {
    fn from(error: TrackerError) -> Self {
        match error {
            TrackerError::Model(_) => SnoutError::TrackerModel,
            TrackerError::Open(_) => SnoutError::TrackerOpen,
            TrackerError::Internal(_) => SnoutError::TrackerInternal,
        }
    }
}

impl From<CameraError> for SnoutError {
    fn from(error: CameraError) -> Self {
        match error {
            CameraError::InvalidFormat(_) => SnoutError::CameraInvalidFormat,
            CameraError::InvalidFrame(_) => SnoutError::CameraInvalidFrame,
            CameraError::InvalidSources => SnoutError::CameraInvalidSources,
            CameraError::Internal(_) => SnoutError::CameraInternal,
        }
    }
}

impl From<PreprocessError> for SnoutError {
    fn from(error: PreprocessError) -> Self {
        match error {
            PreprocessError::Internal(_) => SnoutError::PreprocessInternal,
        }
    }
}

impl From<PipelineError> for SnoutError {
    fn from(error: PipelineError) -> Self {
        match error {
            PipelineError::Load(_) => SnoutError::PipelineLoad,
            PipelineError::Inference(_) => SnoutError::PipelineInference,
        }
    }
}

impl From<ConfigError> for SnoutError {
    fn from(error: ConfigError) -> Self {
        match error {
            ConfigError::FileNotFound => SnoutError::ConfigFileNotFound,
            ConfigError::InvalidConfig(_) => SnoutError::ConfigInvalidConfig,
        }
    }
}

struct LastError {
    code: SnoutError,
    message: String,
}

thread_local! {
    static LAST_ERROR: RefCell<LastError> = const { RefCell::new(LastError { code: SnoutError::Ok, message: String::new() }) };
}

pub(crate) fn set_null_pointer_error() {
    LAST_ERROR.with_borrow_mut(|last_error| {
        last_error.code = SnoutError::NullPointer;
        last_error.message = "a required argument is null".to_string();
    });
}

pub(crate) fn set_utf8_error(e: std::str::Utf8Error) {
    LAST_ERROR.with_borrow_mut(|last_error| {
        last_error.code = SnoutError::InvalidUtf8;
        last_error.message = e.to_string();
    });
}

pub(crate) fn set_last_error(e: impl Into<SnoutError> + std::error::Error) {
    LAST_ERROR.with_borrow_mut(|last_error| {
        last_error.message = e.to_string();
        last_error.code = e.into();
    });
}

pub(crate) fn clear_last_error() {
    LAST_ERROR.with_borrow_mut(|last_error| {
        last_error.code = SnoutError::Ok;
        last_error.message.clear();
    });
}

/// Get the last error that occurred.
///
/// Returns the last error code on this thread.
#[unsafe(no_mangle)]
pub extern "C" fn snout_last_error() -> SnoutError {
    LAST_ERROR.with_borrow(|e| e.code)
}

/// Copies the error message from the last fallible call into `buffer`.
///
/// The message is null-terminated.
/// Returns the length of the message not including the null terminator.
///
/// If `buffer` is null or `max_len` is 0, returns the length of the message.
///
/// This will return the error message for this thread.
#[unsafe(no_mangle)]
pub extern "C" fn snout_last_error_message(buffer: *mut c_char, max_len: usize) -> usize {
    LAST_ERROR.with_borrow(|last_error| {
        if buffer.is_null() || max_len == 0 {
            return last_error.message.len();
        }

        let copy_len = std::cmp::min(last_error.message.len(), max_len - 1);

        unsafe {
            std::ptr::copy_nonoverlapping(last_error.message.as_ptr(), buffer as *mut u8, copy_len);
            *buffer.add(copy_len) = 0;
        }

        copy_len
    })
}
