use crate::{
    capture::{Frame, MonoCamera, discovery::CameraSource},
    ffi::error::{clear_last_error, set_last_error, set_null_pointer_error},
};

/// Open a mono camera using the given source.
///
/// Returns null if the camera could not be opened.
/// Check [`snout_last_error`] for details.
#[unsafe(no_mangle)]
pub extern "C" fn snout_mono_camera_open(source: *const CameraSource) -> *mut MonoCamera {
    clear_last_error();

    if source.is_null() {
        set_null_pointer_error();
        return std::ptr::null_mut();
    }

    let source = unsafe { &*source };

    match MonoCamera::open(source) {
        Ok(camera) => Box::into_raw(Box::new(camera)),
        Err(e) => {
            set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// Get the next frame from the mono camera.
///
/// Returns null if the frame could not be retrieved.
/// Check [`snout_last_error`] for details.
///
/// The returned pointer is valid until the next call to [`snout_mono_camera_get_frame`] or [`snout_mono_camera_free`].
#[unsafe(no_mangle)]
pub extern "C" fn snout_mono_camera_get_frame(camera: *mut MonoCamera) -> *const Frame {
    clear_last_error();

    if camera.is_null() {
        set_null_pointer_error();
        return std::ptr::null();
    }

    let camera = unsafe { &mut *camera };

    match camera.get_frame() {
        Ok(frame) => frame as *const Frame,
        Err(e) => {
            set_last_error(e);
            std::ptr::null()
        }
    }
}

/// Free the mono camera acquired by [`snout_mono_camera_open`].
#[unsafe(no_mangle)]
pub extern "C" fn snout_mono_camera_free(camera: *mut MonoCamera) {
    clear_last_error();

    if camera.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(camera as *mut MonoCamera));
    }
}
