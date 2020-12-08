#![allow(non_camel_case_types)]
use alloc::format;

extern crate sphere_sys;
use sphere_sys::Log_Debug;

pub fn log(message: &str) {
    let null_ending = format!("{}\n\0", message);

    unsafe {
        let ptr = null_ending.as_ptr();
        Log_Debug(ptr as *const i8);
    }
}
