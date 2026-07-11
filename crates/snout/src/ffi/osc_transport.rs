use std::ffi::c_char;

use crate::{
    ffi::error::{clear_last_error, set_last_error, set_null_pointer_error, set_utf8_error},
    output::OscTransport,
};

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
