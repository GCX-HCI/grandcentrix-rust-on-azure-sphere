#![allow(non_camel_case_types)]
use alloc::format;
use alloc::string::String;

extern crate sphere_sys;
use sphere_sys::free;
use sphere_sys::Storage_GetAbsolutePathInImagePackage;

pub fn get_absolute_path_in_image_package(path: &str) -> Result<String, &str> {
    let null_terminated = format!("{}\0", path);

    unsafe {
        let ptr = null_terminated.as_ptr();
        let absolute_path_c: *mut sphere_sys::std::os::raw::c_char =
            Storage_GetAbsolutePathInImagePackage(ptr as *const sphere_sys::std::os::raw::c_char);

        if absolute_path_c.is_null() {
            Err("Unable to get absolute path")
        } else {
            let mut absolute_path = String::new();
            let mut absolute_path_c_ptr = absolute_path_c;
            while *absolute_path_c_ptr != 0 {
                absolute_path.push(*absolute_path_c_ptr as u8 as char);
                absolute_path_c_ptr = absolute_path_c_ptr.offset(1);
            }

            free(absolute_path_c as *mut _);
            Ok(absolute_path)
        }
    }
}

// for now no support for other functions from storage.h since it's not needed right now
