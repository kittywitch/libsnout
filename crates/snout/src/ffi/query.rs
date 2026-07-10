use std::{ffi::c_char, sync::Mutex};

use crate::{
    capture::discovery::{self, CameraInfo, CameraSource},
    ffi::error::clear_last_error,
};

// TODO: thread_local!
pub(crate) static CAMERA_INFO: Mutex<Vec<CameraInfo>> = Mutex::new(Vec::new());

/// Discover all available cameras.
///
/// Results are accessed via [`snout_camera_name`] and [`snout_camera_source`].
/// Returns the number of cameras found.
#[unsafe(no_mangle)]
pub extern "C" fn snout_query_cameras() -> usize {
    clear_last_error();

    let mut cameras = CAMERA_INFO.lock().expect("Failed to acquire lock");

    *cameras = discovery::query_cameras();
    cameras.len()
}

/// Get the human-readable name for the camera at `index`.
///
/// Copies the name into the buffer, null-terminating it.
/// The length of the name, not including the null terminator, is returned.
///
/// If buffer is null or max_len is 0 then the length of the name is returned.
#[unsafe(no_mangle)]
pub extern "C" fn snout_camera_name(index: usize, buffer: *mut c_char, max_len: usize) -> usize {
    clear_last_error();

    let cameras = CAMERA_INFO.lock().expect("Failed to acquire lock");

    let Some(info) = cameras.get(index) else {
        return 0;
    };

    if buffer.is_null() || max_len == 0 {
        return info.name.len();
    }

    let copy_len = std::cmp::min(info.name.len(), max_len - 1);

    unsafe {
        std::ptr::copy_nonoverlapping(info.name.as_ptr(), buffer as *mut u8, copy_len);
        *buffer.add(copy_len) = 0;
    }

    copy_len
}

/// Get the display name for the camera at `index`.
///
/// Copies the display name into the buffer, null-terminating it.
/// The length of the display name, not including the null terminator, is returned.
///
/// If buffer is null or max_len is 0 then the length of the display name is returned.
#[unsafe(no_mangle)]
pub extern "C" fn snout_camera_display_name(
    index: usize,
    buffer: *mut c_char,
    max_len: usize,
) -> usize {
    clear_last_error();

    let cameras = CAMERA_INFO.lock().expect("Failed to acquire lock");

    let Some(info) = cameras.get(index) else {
        return 0;
    };

    let display_name = info.display_name();

    if buffer.is_null() || max_len == 0 {
        return display_name.len();
    }

    let copy_len = std::cmp::min(display_name.len(), max_len - 1);

    unsafe {
        std::ptr::copy_nonoverlapping(display_name.as_ptr(), buffer as *mut u8, copy_len);
        *buffer.add(copy_len) = 0;
    }

    copy_len
}

/// Get the source for the camera at `index`.
///
/// Returns null if `index` is out of bounds.
/// The pointer is valid until [`snout_camera_source_free`] is called.
#[unsafe(no_mangle)]
pub extern "C" fn snout_camera_source(index: usize) -> *mut CameraSource {
    clear_last_error();

    let cameras = CAMERA_INFO.lock().expect("Failed to acquire lock");

    let Some(info) = cameras.get(index) else {
        return std::ptr::null_mut();
    };

    Box::into_raw(Box::new(info.source.clone()))
}

/// Free the camera source acquired by [`snout_camera_source`].
#[unsafe(no_mangle)]
pub extern "C" fn snout_camera_source_free(source: *mut CameraSource) {
    clear_last_error();

    if source.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(source as *mut CameraSource));
    }
}

/// Compare two camera sources for equality.
///
/// Returns `true` if the sources are equal, `false` otherwise.
/// If either source is null, returns `false`.
#[unsafe(no_mangle)]
pub extern "C" fn snout_camera_source_eq(a: *const CameraSource, b: *const CameraSource) -> bool {
    clear_last_error();

    if a.is_null() || b.is_null() {
        return false;
    }

    unsafe { (*a) == (*b) }
}
