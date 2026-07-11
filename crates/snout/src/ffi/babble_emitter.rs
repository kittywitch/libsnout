use crate::{
    calibration::{EyeShape, FaceShape},
    ffi::error::{clear_last_error, set_null_pointer_error},
    output::{BabbleEmitter, OscTransport},
    weights::Weights,
};

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

/// Send eye weights via the Babble protocol.
#[unsafe(no_mangle)]
pub extern "C" fn snout_babble_emitter_process_eyes(
    emitter: *mut BabbleEmitter,
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
