#![allow(non_camel_case_types, unused)]

use alloc::boxed::Box;
use alloc::format;

extern crate sphere_sys;

use sphere_sys::z__UART_Config_Base;
use sphere_sys::z__UART_Config_v1;
use sphere_sys::z__UART_InitConfig;
use sphere_sys::z__UART_Open;
use sphere_sys::UART_STRUCTS_VERSION;

pub enum Isu {
    Isu0,
    Isu1,
    Isu2,
    Isu3,
    Isu4,
}

pub struct Uart {
    config_base: Box<z__UART_Config_Base>, // prevent leak of config
    fd: sphere_sys::std::os::raw::c_int,
}

impl Uart {
    // might only partially write ... for convenience a write_all would be nice
    pub fn write(&self, data: &[u8]) -> isize {
        unsafe { sphere_sys::write(self.fd, data.as_ptr() as *const _, data.len()) }
    }

    pub fn read(&self, buffer: &mut [u8]) -> isize {
        unsafe { sphere_sys::read(self.fd, buffer.as_mut_ptr() as *mut _, buffer.len()) }
    }
}

pub enum UartFlowControl {
    None,
    RTSCTS,
    XONXOFF,
}

pub struct UartConfig {
    pub baud_rate: u32,
    pub blocking_mode: bool,
    pub data_bits: u8,
    pub parity: u8,
    pub stop_bits: u8,
    pub flow_control: UartFlowControl,
}

pub fn uart_open(isu: Isu, config: UartConfig) -> Result<Uart, Uart> {
    // for MT3620 only
    let isu_id = match isu {
        Isu::Isu0 => 4,
        Isu::Isu1 => 5,
        Isu::Isu2 => 6,
        Isu::Isu3 => 7,
        Isu::Isu4 => 8,
    };

    unsafe {
        let mut ffi_config = z__UART_Config_v1 {
            z__magicAndVersion: UART_STRUCTS_VERSION,
            baudRate: 0,
            blockingMode: 0,
            dataBits: 0,
            parity: 0,
            stopBits: 0,
            flowControl: 0,
        };

        let mut fii_config_base: Box<z__UART_Config_Base> =
            Box::from_raw((&mut ffi_config as *mut _) as *mut z__UART_Config_Base);

        z__UART_InitConfig(&mut (*fii_config_base), UART_STRUCTS_VERSION);

        ffi_config.baudRate = config.baud_rate;
        ffi_config.blockingMode = match config.blocking_mode {
            true => 1,
            false => 0,
        };
        ffi_config.dataBits = config.data_bits;
        ffi_config.parity = config.parity;
        ffi_config.stopBits = config.stop_bits;
        ffi_config.flowControl = match config.flow_control {
            UartFlowControl::None => 0,
            UartFlowControl::RTSCTS => 1,
            UartFlowControl::XONXOFF => 2,
        };

        let descriptor = z__UART_Open(isu_id, &mut (*fii_config_base));

        if descriptor < 0 {
            Err(Uart {
                config_base: fii_config_base,
                fd: descriptor,
            })
        } else {
            Ok(Uart {
                config_base: fii_config_base,
                fd: descriptor,
            })
        }
    }
}
