#![allow(non_camel_case_types)]

extern crate sphere_sys;

pub fn sleep(seconds: u32) {
    unsafe {
        sphere_sys::sleep(seconds);
    }
}

pub fn usleep(microseconds: u32) {
    unsafe {
        sphere_sys::usleep(microseconds);
    }
}
