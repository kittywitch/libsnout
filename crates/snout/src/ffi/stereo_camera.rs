use crate::{
    capture::{Frame, StereoCamera, discovery::CameraSource},
    ffi::error::{clear_last_error, set_last_error, set_null_pointer_error},
};

/// Open a stereo camera using the specified left and right camera sources.
///
/// Returns a pointer to the stereo camera, or null if the camera could not be opened.
/// Check [`snout_last_error`] for details.
#[unsafe(no_mangle)]
pub extern "C" fn snout_stereo_camera_open(
    left: *const CameraSource,
    right: *const CameraSource,
) -> *mut StereoCamera {
    clear_last_error();

    if left.is_null() || right.is_null() {
        set_null_pointer_error();
        return std::ptr::null_mut();
    }

    let left = unsafe { &*left };
    let right = unsafe { &*right };

    match StereoCamera::open(left, right) {
        Ok(camera) => Box::into_raw(Box::new(camera)),
        Err(e) => {
            set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// Open a stereo camera using a single side-by-side source.
///
/// Returns a pointer to the stereo camera, or null if the camera could not be opened.
/// Check [`snout_last_error`] for details.
#[unsafe(no_mangle)]
pub extern "C" fn snout_stereo_camera_open_sbs(source: *const CameraSource) -> *mut StereoCamera {
    clear_last_error();

    if source.is_null() {
        set_null_pointer_error();

        return std::ptr::null_mut();
    }

    let source = unsafe { &*source };

    match StereoCamera::open_sbs(source) {
        Ok(camera) => Box::into_raw(Box::new(camera)),
        Err(e) => {
            set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// Open a stereo camera and duplicate the source.
///
/// This will open a [`StereoCamera`] with the same source duplicated to both cameras.
///
/// Returns a pointer to the stereo camera, or null if the camera could not be opened.
/// Check [`snout_last_error`] for details.
#[unsafe(no_mangle)]
pub extern "C" fn snout_stereo_camera_open_duplicate(
    source: *const CameraSource,
) -> *mut StereoCamera {
    clear_last_error();

    if source.is_null() {
        set_null_pointer_error();

        return std::ptr::null_mut();
    }

    let source = unsafe { &*source };

    match StereoCamera::open_duplicate(source) {
        Ok(camera) => Box::into_raw(Box::new(camera)),
        Err(e) => {
            set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// Open a stereo camera from two sources.
///
/// - both sources set to the *same* camera -> side-by-side
/// - both sources set to *different* cameras -> dual
/// - exactly one source set -> duplicate (single-eye tracking)
/// - neither source set -> error
///
/// Returns a pointer to the stereo camera, or null if the camera could not be opened.
/// Check [`snout_last_error`] for details.
#[unsafe(no_mangle)]
pub extern "C" fn snout_stereo_camera_from_sources(
    left: *const CameraSource,
    right: *const CameraSource,
) -> *mut StereoCamera {
    clear_last_error();

    let left = if left.is_null() {
        None
    } else {
        Some(unsafe { &*left })
    };

    let right = if right.is_null() {
        None
    } else {
        Some(unsafe { &*right })
    };

    match StereoCamera::from_sources(left, right) {
        Ok(camera) => Box::into_raw(Box::new(camera)),
        Err(e) => {
            set_last_error(e);
            std::ptr::null_mut()
        }
    }
}

/// Free the stereo camera acquired by [`snout_stereo_camera_open`] or [`snout_stereo_camera_open_sbs`].
#[unsafe(no_mangle)]
pub extern "C" fn snout_stereo_camera_free(camera: *mut StereoCamera) {
    clear_last_error();

    if camera.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(camera));
    }
}

/// Represents a pair of stereo camera frames.
#[repr(C)]
pub struct SnoutStereoCameraFrames {
    pub left: *const Frame,
    pub right: *const Frame,
}

/// Returns the stereo camera frames from the camera.
///
/// The returned [`SnoutStereoCameraFrames`] struct contains pointers to [`Frame`] instances.
/// The frames are valid until the [`snout_stereo_camera_free`] or [`snout_stereo_camera_get_frames`] function is called.
///
/// If an error occurs, the frames will be null and the error will be set.
#[unsafe(no_mangle)]
pub extern "C" fn snout_stereo_camera_get_frames(
    camera: *mut StereoCamera,
) -> SnoutStereoCameraFrames {
    clear_last_error();

    if camera.is_null() {
        set_null_pointer_error();
        return SnoutStereoCameraFrames {
            left: std::ptr::null(),
            right: std::ptr::null(),
        };
    }

    let camera = unsafe { &mut *camera };
    match camera.get_frames() {
        Ok((left, right)) => SnoutStereoCameraFrames {
            left: left as *const Frame,
            right: right as *const Frame,
        },
        Err(e) => {
            set_last_error(e);
            SnoutStereoCameraFrames {
                left: std::ptr::null(),
                right: std::ptr::null(),
            }
        }
    }
}
