#![allow(non_camel_case_types)]

extern crate sphere_sys;

use sphere_sys::std::os::raw::c_char;
use sphere_sys::std::os::raw::c_void;

use sphere_sys::curl_easy_getinfo;
use sphere_sys::curl_easy_init;
use sphere_sys::curl_easy_perform;
use sphere_sys::curl_easy_setopt;
use sphere_sys::curl_global_init;
use sphere_sys::curl_slist_append;
use sphere_sys::curl_write_callback;
use sphere_sys::CURLcode_CURLE_OK;
use sphere_sys::CURLoption_CURLOPT_CAINFO;
use sphere_sys::CURLoption_CURLOPT_CUSTOMREQUEST;
use sphere_sys::CURLoption_CURLOPT_HTTPHEADER;
use sphere_sys::CURLoption_CURLOPT_POSTFIELDS;
use sphere_sys::CURLoption_CURLOPT_SSL_CTX_FUNCTION;
use sphere_sys::CURLoption_CURLOPT_SSL_VERIFYHOST;
use sphere_sys::CURLoption_CURLOPT_URL;
use sphere_sys::CURLoption_CURLOPT_VERBOSE;
use sphere_sys::CURLoption_CURLOPT_WRITEDATA;
use sphere_sys::CURLoption_CURLOPT_WRITEFUNCTION;
use sphere_sys::DeviceAuth_SslCtxFunc;
use sphere_sys::CURL;
use sphere_sys::CURLINFO_CURLINFO_RESPONSE_CODE;
use sphere_sys::CURL_GLOBAL_ALL;

use crate::storage::get_absolute_path_in_image_package;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use core::ptr::*;

pub fn curl_init() -> Result<&'static str, &'static str> {
    unsafe {
        if curl_global_init(CURL_GLOBAL_ALL as i32) != CURLcode_CURLE_OK {
            Err("Unable to initialize curl")
        } else {
            Ok("Ok")
        }
    }
}

pub trait CurlWriteCallback<'a> = FnMut(&[u8], bool) -> () + 'a;

pub struct Curl {
    handle: *mut CURL,
}

struct Context<'a> {
    curl: &'a Curl,
    write_callback: Option<Box<dyn CurlWriteCallback<'a> + 'a>>,
}

// TODO call `curl_easy_cleanup(self.handle);` when dropped
impl Curl {
    pub fn get_request_as_string(
        &self,
        url: &str,
        ca_file: &str,
        authenticated: bool,
    ) -> Result<String, u32> {
        unsafe extern "C" fn write_callback_c(
            buffer: *mut c_char,
            _size: usize,
            nitems: usize,
            outstream: *mut c_void,
        ) -> usize {
            let data = core::slice::from_raw_parts::<u8>(buffer as *const _, nitems);

            let ctx_ptr = outstream as *mut Context;
            let ctx = &mut *ctx_ptr;

            let callback = &mut ctx.write_callback;
            if let Some(cb) = callback {
                (cb)(data, false);
            }
            nitems
        }

        let null_ending_ca_file =
            format!("{}\0", get_absolute_path_in_image_package(ca_file).unwrap());

        let mut response = String::new();

        let write_callback = |data: &[u8], _done: bool| {
            response.push_str(core::str::from_utf8(data).unwrap());
        };

        unsafe {
            curl_easy_setopt(
                self.handle,
                CURLoption_CURLOPT_CAINFO,
                null_ending_ca_file.as_ptr(),
            );

            curl_easy_setopt(
                self.handle,
                CURLoption_CURLOPT_WRITEFUNCTION,
                write_callback_c as *const curl_write_callback,
            );
        }

        unsafe extern "C" fn auth_callback_c(
            _curl: *mut CURL,
            sslctx: *mut c_void,
            _user_ctx: *mut c_void,
        ) -> u32 {
            DeviceAuth_SslCtxFunc(sslctx);
            sphere_sys::CURLcode_CURLE_OK
        };

        if authenticated {
            unsafe {
                curl_easy_setopt(
                    self.handle,
                    CURLoption_CURLOPT_SSL_CTX_FUNCTION,
                    auth_callback_c as *const c_void,
                );

                // WARNING! TURNING OFF HOSTNAME VERIFICATION HERE FOR NOW!!
                curl_easy_setopt(self.handle, CURLoption_CURLOPT_SSL_VERIFYHOST, 0u32);
            }
        }

        let ctx = Context {
            curl: self,
            write_callback: Some(Box::new(write_callback)),
        };
        let ctx = Box::new(ctx);
        let ctx = Box::into_raw(ctx);

        unsafe {
            curl_easy_setopt(self.handle, CURLoption_CURLOPT_WRITEDATA, ctx);
        }

        let null_ending = format!("{}\0", url);
        let curl_result: u32 = 0;
        unsafe {
            curl_easy_setopt(self.handle, CURLoption_CURLOPT_URL, null_ending.as_ptr());
            let _curl_result = curl_easy_perform(self.handle);
            curl_easy_setopt(self.handle, CURLoption_CURLOPT_WRITEDATA, null::<c_void>());
        }

        unsafe {
            curl_easy_setopt(self.handle, CURLoption_CURLOPT_WRITEDATA, null::<c_void>());
        }

        if curl_result == CURLcode_CURLE_OK {
            let mut response_code: u32 = 0;
            unsafe {
                curl_easy_getinfo(self.handle, CURLINFO_CURLINFO_RESPONSE_CODE, &response_code);
            }

            if response_code >= 200 && response_code <= 299 {
                Ok(response)
            } else {
                Err(response_code)
            }
        } else {
            Err(9999)
        }
    }

    pub fn post_request_as_string(
        &self,
        url: &str,
        post_data: &str,
        content_type: &str,
        ca_file: &str,
        authenticated: bool,
    ) -> Result<String, u32> {
        unsafe extern "C" fn write_callback_c(
            buffer: *mut c_char,
            _size: usize,
            nitems: usize,
            outstream: *mut c_void,
        ) -> usize {
            let data = core::slice::from_raw_parts::<u8>(buffer as *const _, nitems);

            let ctx_ptr = outstream as *mut Context;
            let ctx = &mut *ctx_ptr;

            let callback = &mut ctx.write_callback;
            if let Some(cb) = callback {
                (cb)(data, false);
            }
            nitems
        }

        let null_ending_payload_ptr = format!("{}\0", post_data);
        unsafe {
            let headers = curl_slist_append(
                core::ptr::null_mut(),
                format!("Accept: {}\0", content_type).as_mut_ptr() as *mut _,
            );
            let headers = curl_slist_append(
                headers,
                format!("Content-Type: {}\0", content_type).as_mut_ptr() as *mut _,
            );
            let headers =
                curl_slist_append(headers, format!("Charset: utf-8\0").as_mut_ptr() as *mut _);

            curl_easy_setopt(
                self.handle,
                CURLoption_CURLOPT_CUSTOMREQUEST,
                "POST\0".as_ptr(),
            );

            curl_easy_setopt(self.handle, CURLoption_CURLOPT_HTTPHEADER, headers);

            curl_easy_setopt(
                self.handle,
                CURLoption_CURLOPT_POSTFIELDS,
                null_ending_payload_ptr,
            );
        };

        let null_ending_ca_file =
            format!("{}\0", get_absolute_path_in_image_package(ca_file).unwrap());

        let mut response = String::new();

        let write_callback = |data: &[u8], _done: bool| {
            response.push_str(core::str::from_utf8(data).unwrap());
        };

        unsafe {
            curl_easy_setopt(
                self.handle,
                CURLoption_CURLOPT_CAINFO,
                null_ending_ca_file.as_ptr(),
            );

            curl_easy_setopt(
                self.handle,
                CURLoption_CURLOPT_WRITEFUNCTION,
                write_callback_c as *const curl_write_callback,
            );
        }

        unsafe extern "C" fn auth_callback_c(
            _curl: *mut CURL,
            sslctx: *mut c_void,
            _user_ctx: *mut c_void,
        ) -> u32 {
            DeviceAuth_SslCtxFunc(sslctx);
            sphere_sys::CURLcode_CURLE_OK
        };

        if authenticated {
            unsafe {
                curl_easy_setopt(
                    self.handle,
                    CURLoption_CURLOPT_SSL_CTX_FUNCTION,
                    auth_callback_c as *const c_void,
                );

                // WARNING! TURNING OFF HOSTNAME VERIFICATION HERE FOR NOW!!
                curl_easy_setopt(self.handle, CURLoption_CURLOPT_SSL_VERIFYHOST, 0u32);
            }
        }

        let ctx = Context {
            curl: self,
            write_callback: Some(Box::new(write_callback)),
        };
        let ctx = Box::new(ctx);
        let ctx = Box::into_raw(ctx);

        unsafe {
            curl_easy_setopt(self.handle, CURLoption_CURLOPT_WRITEDATA, ctx);
        }

        let null_ending = format!("{}\0", url);
        let curl_result: u32 = 0;
        unsafe {
            curl_easy_setopt(self.handle, CURLoption_CURLOPT_URL, null_ending.as_ptr());
            let _curl_result = curl_easy_perform(self.handle);
            curl_easy_setopt(self.handle, CURLoption_CURLOPT_WRITEDATA, null::<c_void>());
        }

        unsafe {
            curl_easy_setopt(self.handle, CURLoption_CURLOPT_WRITEDATA, null::<c_void>());
        }

        if curl_result == CURLcode_CURLE_OK {
            let mut response_code: u32 = 0;
            unsafe {
                curl_easy_getinfo(self.handle, CURLINFO_CURLINFO_RESPONSE_CODE, &response_code);
            }

            if response_code >= 200 && response_code <= 299 {
                Ok(response)
            } else {
                Err(response_code)
            }
        } else {
            Err(9999)
        }
    }

    pub fn download<'a, F>(
        &self,
        url: &str,
        ca_file: &str,
        write_callback: F,
    ) -> Result<&'static str, &'static str>
    where
        F: CurlWriteCallback<'a>,
        F: 'a,
    {
        unsafe extern "C" fn write_callback_c(
            buffer: *mut c_char,
            _size: usize,
            nitems: usize,
            outstream: *mut c_void,
        ) -> usize {
            let data = core::slice::from_raw_parts::<u8>(buffer as *const _, nitems);

            let ctx_ptr = outstream as *mut Context;
            let ctx = &mut *ctx_ptr;

            let callback = &mut ctx.write_callback;
            if let Some(cb) = callback {
                (cb)(data, false);
            }
            nitems
        }

        let null_ending_ca_file =
            format!("{}\0", get_absolute_path_in_image_package(ca_file).unwrap());

        unsafe {
            curl_easy_setopt(
                self.handle,
                CURLoption_CURLOPT_CAINFO,
                null_ending_ca_file.as_ptr(),
            );

            curl_easy_setopt(
                self.handle,
                CURLoption_CURLOPT_WRITEFUNCTION,
                write_callback_c as *const curl_write_callback,
            );
        }

        let ctx = Context {
            curl: self,
            write_callback: Some(Box::new(write_callback)),
        };
        let ctx = Box::new(ctx);
        let ctx = Box::into_raw(ctx);

        unsafe {
            curl_easy_setopt(self.handle, CURLoption_CURLOPT_WRITEDATA, ctx);
        }

        let null_ending = format!("{}\0", url);
        unsafe {
            curl_easy_setopt(self.handle, CURLoption_CURLOPT_URL, null_ending.as_ptr());
            curl_easy_perform(self.handle);
            curl_easy_setopt(self.handle, CURLoption_CURLOPT_WRITEDATA, null::<c_void>());
        }

        unsafe {
            curl_easy_setopt(self.handle, CURLoption_CURLOPT_WRITEDATA, null::<c_void>());
        }

        // call one last time to indicate it's done
        unsafe {
            let ctx = &mut *ctx;
            let callback = &mut ctx.write_callback;
            if let Some(cb) = callback {
                (cb)(&[0u8; 0], true);
            }
        }

        Ok("Ok")
    }

    pub fn new() -> Result<Curl, &'static str> {
        unsafe {
            let curl = curl_easy_init();

            if curl.is_null() {
                Err("Unable to initialize easy curl")
            } else {
                curl_easy_setopt(curl, CURLoption_CURLOPT_VERBOSE, 1u64);

                let curl_data = Curl { handle: curl };

                Ok(curl_data)
            }
        }
    }
}
