#![no_std]
#![feature(new_uninit)]
#![feature(trait_alias)]

extern crate alloc;

extern crate sphere_sys;

pub mod application;
pub mod azureiot;
pub mod curl;
pub mod logging;
pub mod mt3620_gpio;
pub mod networking;
pub mod storage;
pub mod uart;
pub mod util;
pub mod watchdog;
