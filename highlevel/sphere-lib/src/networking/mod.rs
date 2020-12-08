#![allow(non_camel_case_types)]
use alloc::format;

extern crate sphere_sys;
use sphere_sys::Networking_GetInterfaceCount;
use sphere_sys::Networking_IsNetworkingReady;
use sphere_sys::Networking_SetInterfaceState;

pub fn set_interface_state(interface: &str, enable: bool) -> i32 {
    let null_ending = format!("{}\0", interface);

    let result = unsafe {
        let ptr = null_ending.as_ptr();
        Networking_SetInterfaceState(ptr as *const i8, enable)
    };

    result
}

pub fn get_interface_count() -> isize {
    unsafe { Networking_GetInterfaceCount() }
}

pub fn is_networking_ready() -> bool {
    let mut is_ready = false;

    unsafe {
        Networking_IsNetworkingReady(&mut is_ready);
    };

    is_ready
}
