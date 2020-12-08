use std::env;
use std::fs;
use std::path::PathBuf;
use std::str;

extern crate bindgen;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out_dir.join("wrapper.h"), WRAPPER_H).expect("Error writing file");

    let wrapper_h_str = out_dir
        .join("wrapper.h")
        .as_path()
        .to_str()
        .unwrap()
        .replace("\\", "/");

    let sysroot_env_var = env::var_os("SYSROOT")
        .expect("SYSROOT not defined - needs to point to e.g. .../sysroots/1");
    let sysroot = sysroot_env_var.to_str().unwrap();
    let sysroot_include = format!("{}/usr/include", sysroot);
    let iot_include = format!("{}/usr/include/azureiot", sysroot);
    let azure_prov_client_include = format!("{}/usr/include/azure_prov_client", sysroot);

    let bindings = bindgen::Builder::default()
        .ctypes_prefix("std::os::raw")
        .use_core()
        .layout_tests(false)
        .clang_arg("--target=armv7a-linux-eabi")
        .clang_arg("--sysroot")
        .clang_arg(sysroot)
        .clang_arg(format!("-I{}", sysroot_include))
        .clang_arg(format!("-I{}", iot_include))
        .clang_arg(format!("-I{}", azure_prov_client_include))
        .clang_arg("-D__bindgen")
        .clang_arg("-fomit-frame-pointer")
        .clang_arg("-S")
        .clang_arg("-x")
        .clang_arg("c")
        .clang_arg("-nostdinc")
        .clang_arg("--verbose")
        .header(wrapper_h_str)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

const WRAPPER_H: &str = r#"
/// <summary>
/// This identifier should be defined before including any of the networking-related header files.
/// It indicates which version of the Wi-Fi data structures the application uses.
/// </summary>
#define NETWORKING_STRUCTS_VERSION 1

/// <summary>
/// This identifier must be defined before including any of the Wi-Fi related header files.
/// It indicates which version of the Wi-Fi data structures the application uses.
/// </summary>
#define WIFICONFIG_STRUCTS_VERSION 1

/// <summary>
/// This identifier must be defined before including any of the UART-related header files.
/// It indicates which version of the UART data structures the application uses.
/// </summary>
#define UART_STRUCTS_VERSION 1

/// <summary>
/// This identifier must be defined before including any of the SPI-related header files.
/// It indicates which version of the SPI data structures the application uses.
/// </summary>
#define SPI_STRUCTS_VERSION 1

// Sphere App Libs
#include <applibs/log.h>
#include <applibs/networking.h>
#include <applibs/gpio.h>
#include <applibs/uart.h>
#include <applibs/application.h>
#include <applibs/storage.h>

// Other
#include <unistd.h>

// Azure IoT SDK
#include <iothub_client_core_common.h>
#include <iothub_device_client_ll.h>
#include <iothub_client_options.h>
#include <iothubtransportmqtt.h>
#include <iothub.h>
#include <azure_sphere_provisioning.h>
#include <iothub_security_factory.h>

#include <time.h>

// CURL
#include <curl/curl.h>

// TLS
#include <tlsutils/deviceauth_curl.h>

#include <signal.h>
"#;
