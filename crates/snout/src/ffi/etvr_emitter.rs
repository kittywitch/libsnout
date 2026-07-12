use crate::{
    calibration::EyeShape,
    ffi::error::{clear_last_error, set_null_pointer_error},
    output::{EtvrEmitter, OscTransport},
    weights::Weights,
};

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
