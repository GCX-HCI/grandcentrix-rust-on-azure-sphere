#![no_std]
#![no_main]
#![feature(const_raw_ptr_to_usize_cast)]
#![feature(asm)]
#![allow(dead_code)]

use core::panic::PanicInfo;

mod gpio;
use gpio::*;

mod uart;
use uart::*;

mod timer;
use timer::*;

mod intercore;
use intercore::*;

const SCB_BASE: usize = 0xE000ED00;

const INTERRUPT_COUNT: usize = 100;
const EXCEPTION_COUNT: usize = 16 + INTERRUPT_COUNT;

extern "C" {
    pub static StackTop: usize;
}

pub union Vector {
    handler: unsafe extern "C" fn(),
    reserved: usize,
}

#[link_section = ".vector_table"]
#[no_mangle]
pub static mut ExceptionVectorTable: [Vector; EXCEPTION_COUNT] = [
    Vector {
        reserved: 0x00100000 + 192 * 1024,
    },
    Vector { handler: main }, // RESET
    Vector {
        handler: defaultHandler,
    }, // NMI
    Vector {
        handler: defaultHandler,
    }, // HardFault
    Vector {
        handler: defaultHandler,
    }, // MPU Fault
    Vector {
        handler: defaultHandler,
    }, // Bus Fault
    Vector {
        handler: defaultHandler,
    }, // Usage Fault
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector {
        handler: defaultHandler,
    }, // SVCall
    Vector {
        handler: defaultHandler,
    }, // Debug Monitor
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    }, // PendSV
    Vector {
        handler: defaultHandler,
    }, // SysTick
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: gpt_handle_irq1,
    }, // TimerIrq
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: Uart_HandleIrq4,
    }, // UartIrq 0
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler, // INT47 ISU0 UART
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: Uart_HandleIrq51, // INT51 ISU1 UART
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler, // INT55 ISU2 UART
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler, // INT59 ISU3 UART
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler, // INT63 ISU4 UART
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler, // INT67 ISU5 UART
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
    Vector {
        handler: defaultHandler,
    },
];

#[no_mangle]
unsafe extern "C" fn defaultHandler() {}

const INTERCORE_COMM_PAYLOAD_START: usize = 20;

static mut SEND_RESULTS: bool = false;
static mut SEND_DATA_END: bool = false;

static mut RECEIVED_DATA: [u8; 256] = [0; 256];

const SELECTED_UART: UartId = UartId::UartIsu1;

#[no_mangle]
unsafe extern "C" fn main() {
    // SCB->VTOR = ExceptionVectorTable
    write_reg32(SCB_BASE, 0x08, ExceptionVectorTable.as_ptr() as u32);

    // Block includes led1RedGpio, GPIO8.
    let gpio = GpioBlock {
        base_addr: 0x38030000, // GPIO Group 5
        block_type: GPIO_BLOCK_PWM as u16,
        first_pin: 8,
        pin_count: 4,
    };
    mt3620_gpio_add_block(gpio).unwrap();
    mt3620_gpio_configure_pin_for_output(8).unwrap();

    gpt_init();

    loop {
        mt3620_gpio_write(8, false).unwrap();

        for i in 0..50000 { 
            // busy loop
        }

        mt3620_gpio_write(8, true).unwrap();

        for i in 0..50000 { 
            // busy loop
        }
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {
        // do nothing here
    }
}
