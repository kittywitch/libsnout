use std::ffi::CStr;

use crate::{
    config::Config,
    ffi::error::{clear_last_error, set_last_error, set_null_pointer_error, set_utf8_error},
};

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
