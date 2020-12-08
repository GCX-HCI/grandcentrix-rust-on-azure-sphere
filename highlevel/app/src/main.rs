#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
extern crate sphere_rt as std;
#[cfg(not(test))]
use std::prelude::v1::*;

extern crate sphere_lib;
use sphere_lib::mt3620_gpio::*;
use sphere_lib::util::sleep;

const MT3620_GPIO8: i32 = 8;
const MT3620_GPIO9: i32 = 9;
const MT3620_GPIO10: i32 = 10;
const MT3620_RDB_LED1_GREEN: i32 = MT3620_GPIO9;
const MT3620_RDB_LED1_BLUE: i32 = MT3620_GPIO10;
const MT3620_RDB_LED1_RED: i32 = MT3620_GPIO8;

#[no_mangle]
fn start() {
    println!("start");

    let red = GpioPort::open(MT3620_RDB_LED1_RED);
    let green = GpioPort::open(MT3620_RDB_LED1_GREEN);
    let blue = GpioPort::open(MT3620_RDB_LED1_BLUE);

    loop {
        red.set_low();
        green.set_low();
        blue.set_low();
        sleep(1);

        red.set_high();
        green.set_high();
        blue.set_high();
        sleep(1);

        red.set_high();
        green.set_low();
        blue.set_high();
        sleep(1);

        red.set_high();
        green.set_low();
        blue.set_low();
        sleep(1);

        red.set_low();
        green.set_low();
        blue.set_high();
        sleep(1);

        red.set_low();
        green.set_high();
        blue.set_high();
        sleep(1);
    }
}
