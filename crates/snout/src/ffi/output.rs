use std::ffi::c_char;

use crate::{
    calibration::{EyeShape, FaceShape},
    ffi::error::{clear_last_error, set_last_error, set_null_pointer_error, set_utf8_error},
    output::{BabbleEmitter, EtvrEmitter, OscTransport},
    track::output::Output,
    weights::Weights,
};

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
