#![allow(non_camel_case_types)]

use alloc::format;

extern crate sphere_sys;

use sphere_sys::read;
use sphere_sys::write;
use sphere_sys::Application_Connect;

pub fn open_application_socket(component_id: &str) -> Result<ApplicationSocket, &'static str> {
    let null_ending = format!("{}\n\0", component_id);

    let fd = unsafe {
        let ptr = null_ending.as_ptr();
        Application_Connect(ptr as *const sphere_sys::std::os::raw::c_char)
    };

    if fd == -1 {
        Err("Error opening application socket")
    } else {
        Ok(ApplicationSocket { fd: fd })
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ApplicationSocket {
    fd: sphere_sys::std::os::raw::c_int,
}

impl ApplicationSocket {
    pub fn write(&self, data: &[u8]) -> isize {
        unsafe {
            write(
                self.fd,
                data.as_ptr() as *const sphere_sys::std::os::raw::c_void,
                data.len(),
            )
        }
    }

    pub fn read(&self, buffer: &mut [u8]) -> isize {
        let read_count = unsafe { read(self.fd, buffer.as_mut_ptr() as *mut _, buffer.len()) };
        read_count
    }
}
