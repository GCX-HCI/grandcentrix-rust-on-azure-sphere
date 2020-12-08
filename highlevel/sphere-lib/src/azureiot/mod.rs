#![allow(non_camel_case_types)]
use alloc::boxed::Box;
use alloc::format;
use core::cell::RefCell;
use core::ptr;

extern crate sphere_sys;

use sphere_sys::std::os::raw::c_char;
use sphere_sys::std::os::raw::c_int;
use sphere_sys::std::os::raw::c_uchar;
use sphere_sys::std::os::raw::c_void;

use sphere_sys::iothub_security_init;
use sphere_sys::malloc;
use sphere_sys::IoTHubDeviceClient_LL_CreateFromConnectionString;
use sphere_sys::IoTHubDeviceClient_LL_CreateFromDeviceAuth;
use sphere_sys::IoTHubDeviceClient_LL_CreateWithAzureSphereDeviceAuthProvisioning;
use sphere_sys::IoTHubDeviceClient_LL_Destroy;
use sphere_sys::IoTHubDeviceClient_LL_DoWork;
use sphere_sys::IoTHubDeviceClient_LL_SendEventAsync;
use sphere_sys::IoTHubDeviceClient_LL_SetConnectionStatusCallback;
use sphere_sys::IoTHubDeviceClient_LL_SetDeviceMethodCallback;
use sphere_sys::IoTHubDeviceClient_LL_SetDeviceTwinCallback;
use sphere_sys::IoTHubDeviceClient_LL_SetOption;
use sphere_sys::IoTHubMessage_CreateFromString;
use sphere_sys::IoTHubMessage_Destroy;
use sphere_sys::IoTHub_Init;
use sphere_sys::AZURE_SPHERE_PROV_RESULT_AZURE_SPHERE_PROV_RESULT_DEVICEAUTH_NOT_READY;
use sphere_sys::AZURE_SPHERE_PROV_RESULT_AZURE_SPHERE_PROV_RESULT_OK;
use sphere_sys::AZURE_SPHERE_PROV_RETURN_VALUE;
use sphere_sys::DEVICE_TWIN_UPDATE_STATE;
use sphere_sys::IOTHUB_CLIENT_CONNECTION_STATUS;
use sphere_sys::IOTHUB_CLIENT_CONNECTION_STATUS_REASON;
use sphere_sys::IOTHUB_CLIENT_EVENT_CONFIRMATION_CALLBACK;
use sphere_sys::IOTHUB_CLIENT_RESULT_TAG_IOTHUB_CLIENT_OK;
use sphere_sys::IOTHUB_DEVICE_CLIENT_LL_HANDLE;
use sphere_sys::IOTHUB_MESSAGE_HANDLE;
use sphere_sys::IOTHUB_SECURITY_TYPE_TAG_IOTHUB_SECURITY_TYPE_X509;

use crate::util::usleep;

pub struct AzureProvisioning<'s> {
    provisioning_result: RefCell<AZURE_SPHERE_PROV_RETURN_VALUE>,
    provisioning_handle: RefCell<IOTHUB_DEVICE_CLIENT_LL_HANDLE>,
    scope_id: &'s str,
    pub authenticated: RefCell<bool>,
    status_callback: RefCell<Option<Box<dyn StatusCallback + 's>>>,
    method_callback: RefCell<Option<Box<dyn DeviceMethodCallback + 's>>>,
}

pub trait StatusCallback = Fn(u32, u32, &AzureProvisioning) -> ();

pub trait DeviceTwinCallback = Fn(u32, &[u8]) -> ();

// method, payload -> result, result_payload - seems the result_payload needs to be a zero terminated string?
pub trait DeviceMethodCallback = FnMut(&str, &[u8]) -> (i32, alloc::string::String);

struct StatusContext<'s, 'x: 's> {
    provisioning: &'s AzureProvisioning<'x>,
}

impl<'s> AzureProvisioning<'s> {
    pub fn set_connection_status_callback<F>(&self, f: F)
    where
        F: StatusCallback,
        F: 's,
    {
        *self.status_callback.borrow_mut() = Some(Box::new(f));
        self.set_connection_status_callback_internal();
    }

    fn set_connection_status_callback_internal(&self) {
        unsafe {
            unsafe extern "C" fn connection_status_callback(
                result: IOTHUB_CLIENT_CONNECTION_STATUS,
                reason: IOTHUB_CLIENT_CONNECTION_STATUS_REASON,
                user_context_callback: *mut c_void,
            ) {
                let ctx_ptr = user_context_callback as *mut StatusContext;
                let ctx = &mut *ctx_ptr;

                *ctx.provisioning.authenticated.borrow_mut() = result == 0;

                let callback = &*ctx.provisioning.status_callback.borrow();
                if let Some(cb) = callback {
                    (cb)(result, reason, ctx.provisioning);
                }
            }

            let callback = connection_status_callback;

            let ctx = StatusContext { provisioning: self };
            // Trait object with a stable address
            let ctx = Box::new(ctx); // as Box<dyn StatusCallback>;
                                     // Raw pointer
            let ctx = Box::into_raw(ctx);

            IoTHubDeviceClient_LL_SetConnectionStatusCallback(
                *self.provisioning_handle.borrow(),
                Some(callback),
                ctx as *mut _,
            );
        }
    }

    // TODO make it work like the other two callbacks internally
    pub fn set_device_twin_callback<F>(&self, f: F)
    where
        F: DeviceTwinCallback,
        F: 's,
    {
        unsafe {
            unsafe extern "C" fn device_twin_callback<F>(
                update_state: DEVICE_TWIN_UPDATE_STATE,
                payload: *const c_uchar,
                size: usize,
                user_context_callback: *mut c_void,
            ) where
                F: DeviceTwinCallback,
            {
                let pl = core::slice::from_raw_parts(payload, size);

                let callback_ptr = user_context_callback as *mut F;
                let callback = &mut *callback_ptr;
                callback(update_state, &pl);
            }

            let callback = device_twin_callback::<F>;
            // Trait object with a stable address
            let func = Box::new(f) as Box<dyn DeviceTwinCallback>;
            // Thin pointer
            let func = Box::new(func);
            // Raw pointer
            let func = Box::into_raw(func);

            IoTHubDeviceClient_LL_SetDeviceTwinCallback(
                *self.provisioning_handle.borrow(),
                Some(callback),
                func as *mut _,
            );
        }
    }

    pub fn set_device_method_callback<F>(&self, f: F)
    where
        F: DeviceMethodCallback,
        F: 's,
    {
        *self.method_callback.borrow_mut() = Some(Box::new(f));
        self.set_device_method_callback_internal();
    }

    fn set_device_method_callback_internal(&self) {
        unsafe {
            unsafe extern "C" fn device_method_callback(
                method_name: *const c_char,
                payload: *const c_uchar,
                size: usize,
                response: *mut *mut c_uchar,
                response_size: *mut usize,
                user_context_callback: *mut c_void,
            ) -> c_int {
                let method = core::str::from_utf8(core::slice::from_raw_parts(
                    method_name as *const _,
                    count_until_zero(method_name),
                ))
                .unwrap();
                let pl = core::slice::from_raw_parts(payload, size);

                let ctx_ptr = user_context_callback as *mut StatusContext;
                let ctx = &mut *ctx_ptr;

                // method, payload -> result, result_payload
                let callback = &mut *ctx.provisioning.method_callback.borrow_mut();
                if let Some(cb) = callback {
                    let (result, result_payload) = (cb)(&method, &pl);

                    // The response payload content. This must be heap-allocated, 'free' will be called on this buffer by the Azure IoT Hub SDK.
                    // apparently the SDK wants a zero terminated string
                    let null_ending_payload = format!("{}\0", result_payload);
                    *response_size = null_ending_payload.len();
                    let response_buffer_ptr = malloc(null_ending_payload.len() as u32);
                    core::ptr::copy(
                        null_ending_payload.as_ptr(),
                        response_buffer_ptr as *mut _,
                        null_ending_payload.len(),
                    );
                    *response = response_buffer_ptr as *mut _;

                    result
                } else {
                    404
                }
            }

            let callback = device_method_callback;

            let ctx = StatusContext { provisioning: self };
            // Trait object with a stable address
            let ctx = Box::new(ctx); // as Box<dyn StatusCallback>;
                                     // Raw pointer
            let ctx = Box::into_raw(ctx);

            IoTHubDeviceClient_LL_SetDeviceMethodCallback(
                *self.provisioning_handle.borrow(),
                Some(callback),
                ctx as *mut _,
            );
        }
    }

    pub fn do_work(&self) {
        unsafe { IoTHubDeviceClient_LL_DoWork(*self.provisioning_handle.borrow()) };
    }

    pub fn set_keep_alive_seconds(&self, seconds: u32) -> Result<&'static str, &'static str> {
        let keep_alive_option = b"keepalive\0";

        let res = unsafe {
            IoTHubDeviceClient_LL_SetOption(
                *self.provisioning_handle.borrow(),
                keep_alive_option.as_ptr() as *const i8,
                &seconds as *const _ as *const sphere_sys::std::os::raw::c_void,
            )
        };

        if res == IOTHUB_CLIENT_RESULT_TAG_IOTHUB_CLIENT_OK {
            Ok("ok")
        } else {
            Err("Unable to set keep alive timeout")
        }
    }

    pub fn send_telemetry(&self, payload: &str) -> Result<&'static str, &'static str> {
        let null_ending_payload = format!("{}\0", payload);

        let success = unsafe {
            let res;

            let message_handle: Box<IOTHUB_MESSAGE_HANDLE> = Box::new(
                IoTHubMessage_CreateFromString(null_ending_payload.as_ptr() as *const i8),
            );

            if ((*message_handle) as *const _) != ptr::null() {
                let callback: IOTHUB_CLIENT_EVENT_CONFIRMATION_CALLBACK = None;
                let send_result = IoTHubDeviceClient_LL_SendEventAsync(
                    *self.provisioning_handle.borrow(),
                    *message_handle,
                    callback,
                    ptr::null_mut(),
                );

                IoTHubMessage_Destroy(*message_handle);

                if send_result == IOTHUB_CLIENT_RESULT_TAG_IOTHUB_CLIENT_OK {
                    res = true;
                } else {
                    res = false;
                }
            } else {
                res = false;
            }

            res
        };

        if success {
            Ok("Message sent")
        } else {
            Err("Message send failed")
        }
    }

    pub fn reconnect(&self) {
        unsafe {
            IoTHubDeviceClient_LL_Destroy(*self.provisioning_handle.borrow());
            *self.authenticated.borrow_mut() = false;
            *self.provisioning_handle.borrow_mut() = core::ptr::null_mut();

            let null_ending_scope_id = format!("{}\0", self.scope_id);
            let mut handle = Box::<IOTHUB_DEVICE_CLIENT_LL_HANDLE>::new_uninit();

            crate::logging::log("azureiot: destroyed old handle");

            loop {
                let result: AZURE_SPHERE_PROV_RETURN_VALUE = {
                    let scope_id_ptr = null_ending_scope_id.as_ptr();
                    IoTHubDeviceClient_LL_CreateWithAzureSphereDeviceAuthProvisioning(
                        scope_id_ptr as *const i8,
                        10000,
                        handle.as_mut_ptr(),
                    )
                };

                crate::logging::log(&format!("azureioit: got result {}", result.result));

                if result.result
                    == AZURE_SPHERE_PROV_RESULT_AZURE_SPHERE_PROV_RESULT_DEVICEAUTH_NOT_READY
                {
                    usleep(500);
                    continue;
                }

                if result.result == AZURE_SPHERE_PROV_RESULT_AZURE_SPHERE_PROV_RESULT_OK {
                    let real_handle = *handle.assume_init();
                    *self.provisioning_handle.borrow_mut() = real_handle;
                    *self.provisioning_result.borrow_mut() = result;

                    self.set_connection_status_callback_internal();
                    self.set_device_method_callback_internal();

                    self.do_work();

                    *self.authenticated.borrow_mut() = true;

                    break;
                } else {
                    break;
                }
            }
        }
    }

    pub fn is_authenticated(&self) -> bool {
        *self.authenticated.borrow() == true
    }

    pub fn azure_create_device_auth_provisioning(
        scope_id: &'s str,
        wait_for_auth_ready: bool,
    ) -> Result<AzureProvisioning<'s>, &'static str> {
        let null_ending_scope_id = format!("{}\0", scope_id);

        let mut handle = Box::<IOTHUB_DEVICE_CLIENT_LL_HANDLE>::new_uninit();

        loop {
            let result: AZURE_SPHERE_PROV_RETURN_VALUE = unsafe {
                let scope_id_ptr = null_ending_scope_id.as_ptr();
                IoTHubDeviceClient_LL_CreateWithAzureSphereDeviceAuthProvisioning(
                    scope_id_ptr as *const i8,
                    20000,
                    handle.as_mut_ptr(),
                )
            };

            if wait_for_auth_ready
                && result.result
                    == AZURE_SPHERE_PROV_RESULT_AZURE_SPHERE_PROV_RESULT_DEVICEAUTH_NOT_READY
            {
                usleep(500);
                continue;
            }

            if result.result == AZURE_SPHERE_PROV_RESULT_AZURE_SPHERE_PROV_RESULT_OK {
                let real_handle = unsafe { *handle.assume_init() };

                break Ok(AzureProvisioning {
                    provisioning_result: RefCell::new(result),
                    provisioning_handle: RefCell::new(real_handle),
                    scope_id: scope_id,
                    authenticated: RefCell::new(true),
                    status_callback: RefCell::new(None),
                    method_callback: RefCell::new(None),
                });
            } else {
                break Err("Provisioning failed");
            }
        }
    }

    // RECONNECT doesn't work in this case! Need to be added
    pub fn azure_create_from_device_auth(
        uri: &str,
        device_id: &str,
    ) -> Result<AzureProvisioning<'s>, &'static str> {
        let null_ending_uri = format!("{}\0", uri);
        let null_ending_device_id = format!("{}\0", device_id);

        unsafe {
            iothub_security_init(IOTHUB_SECURITY_TYPE_TAG_IOTHUB_SECURITY_TYPE_X509);
        };

        let result: IOTHUB_DEVICE_CLIENT_LL_HANDLE = unsafe {
            let uri_ptr = null_ending_uri.as_ptr();
            let device_id_ptr = null_ending_device_id.as_ptr();
            IoTHubDeviceClient_LL_CreateFromDeviceAuth(
                uri_ptr as *const i8,
                device_id_ptr as *const i8,
                Some(sphere_sys::MQTT_Protocol),
            )
        };

        if !result.is_null() {
            unsafe {
                let device_id_option: u32 = 1;
                let device_id_option_ptr = (&device_id_option) as *const u32;

                if IoTHubDeviceClient_LL_SetOption(
                    result,
                    format!("{}\0", "SetDeviceId").as_ptr() as *const i8,
                    device_id_option_ptr as *const _,
                ) != IOTHUB_CLIENT_RESULT_TAG_IOTHUB_CLIENT_OK
                {
                    return Err("Connect failed in SetDeviceId");
                }
            };

            let real_handle = result;

            let result = AZURE_SPHERE_PROV_RETURN_VALUE {
                result: AZURE_SPHERE_PROV_RESULT_AZURE_SPHERE_PROV_RESULT_OK,
                prov_device_error: 0,
                iothub_client_error: 0,
            };

            Ok(AzureProvisioning {
                provisioning_result: RefCell::new(result),
                provisioning_handle: RefCell::new(real_handle),
                scope_id: "",
                authenticated: RefCell::new(true),
                status_callback: RefCell::new(None),
                method_callback: RefCell::new(None),
            })
        } else {
            Err("Connect failed")
        }
    }

    // RECONNECT doesn't work in this case! Need to be added
    pub fn azure_create_from_connection_string(
        connection_string: &str,
    ) -> Result<AzureProvisioning<'s>, &'static str> {
        let null_ending_connection_string = format!("{}\0", connection_string);

        let result: IOTHUB_DEVICE_CLIENT_LL_HANDLE = unsafe {
            let connection_string_ptr = null_ending_connection_string.as_ptr();
            IoTHubDeviceClient_LL_CreateFromConnectionString(
                connection_string_ptr as *const i8,
                Some(sphere_sys::MQTT_Protocol),
            )
        };

        if !result.is_null() {
            let real_handle = result;

            let result = AZURE_SPHERE_PROV_RETURN_VALUE {
                result: AZURE_SPHERE_PROV_RESULT_AZURE_SPHERE_PROV_RESULT_OK,
                prov_device_error: 0,
                iothub_client_error: 0,
            };

            Ok(AzureProvisioning {
                provisioning_result: RefCell::new(result),
                provisioning_handle: RefCell::new(real_handle),
                scope_id: "",
                authenticated: RefCell::new(true),
                status_callback: RefCell::new(None),
                method_callback: RefCell::new(None),
            })
        } else {
            Err("Connect failed")
        }
    }

    pub fn init() -> i32 {
        unsafe { IoTHub_Init() }
    }

    pub fn set_option(&self, option: &str, data: &str) -> u32 {
        let null_ending_option_name = format!("{}\0", option);
        let null_ending_certs = format!("{}\0", data);
        unsafe {
            IoTHubDeviceClient_LL_SetOption(
                *self.provisioning_handle.borrow(),
                null_ending_option_name.as_ptr() as *const _,
                null_ending_certs.as_ptr() as *const _,
            )
        }
    }
}

unsafe fn count_until_zero(ptr: *const i8) -> usize {
    let mut count: isize = 0;
    loop {
        if *(ptr.offset(count)) == 0 {
            break count as usize;
        }

        count += 1;
    }
}
