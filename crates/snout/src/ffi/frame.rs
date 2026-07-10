use crate::{
    capture::Frame,
    ffi::error::{clear_last_error, set_null_pointer_error},
};

/// Get the width of the frame.
#[unsafe(no_mangle)]
pub extern "C" fn snout_frame_width(frame: *const Frame) -> usize {
    clear_last_error();

    if frame.is_null() {
        set_null_pointer_error();
        return 0;
    }

    let frame = unsafe { &*frame };

    frame.width()
}

/// Get the height of the frame.
#[unsafe(no_mangle)]
pub extern "C" fn snout_frame_height(frame: *const Frame) -> usize {
    clear_last_error();

    if frame.is_null() {
        set_null_pointer_error();
        return 0;
    }

    let frame = unsafe { &*frame };
    frame.height()
}

/// Get the data of the frame.
///
/// This will not take ownership of the data.
/// The data length is [`snout_frame_width`] * [`snout_frame_height`].
#[unsafe(no_mangle)]
pub extern "C" fn snout_frame_data(frame: *const Frame) -> *const u8 {
    clear_last_error();

    if frame.is_null() {
        set_null_pointer_error();
        return std::ptr::null();
    }

    let frame = unsafe { &*frame };

    frame.as_slice().as_ptr()
}
