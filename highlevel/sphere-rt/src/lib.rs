#![no_std]
#![feature(lang_items, start, libc)]
#![feature(alloc_error_handler)]
#![feature(core_intrinsics)]
#![feature(raw)]
#![feature(wake_trait)]
#![feature(asm)]
#![feature(concat_idents)]
#![feature(format_args_nl)]
#![feature(global_asm)]
#![feature(log_syntax)]
#![feature(llvm_asm)]
#![feature(trace_macros)]

extern crate alloc as alloc;
extern crate alloc as alloc_crate;

use core::panic::PanicInfo;

mod allocator;
use allocator::MyAllocator;

extern crate sphere_sys;
use sphere_sys::Log_Debug;

pub mod prelude;

// re-exports - like libstd does
pub use alloc_crate::borrow;
pub use alloc_crate::boxed;
pub use alloc_crate::fmt;
pub use alloc_crate::format;
pub use alloc_crate::rc;
pub use alloc_crate::slice;
pub use alloc_crate::str;
pub use alloc_crate::string;
pub use alloc_crate::vec;
pub use core::any;
pub use core::arch;
pub use core::array;
pub use core::cell;
pub use core::char;
pub use core::clone;
pub use core::cmp;
pub use core::convert;
pub use core::default;
pub use core::hash;
pub use core::hint;
pub use core::i128;
pub use core::i16;
pub use core::i32;
pub use core::i64;
pub use core::i8;
pub use core::intrinsics;
pub use core::isize;
pub use core::iter;
pub use core::marker;
pub use core::mem;
pub use core::ops;
pub use core::option;
pub use core::pin;
pub use core::ptr;
pub use core::raw;
pub use core::result;
pub use core::u128;
pub use core::u16;
pub use core::u32;
pub use core::u64;
pub use core::u8;
pub use core::usize;

pub mod task {
    pub use alloc::task::*;
    pub use core::task::*;
}

// Re-export macros defined in libcore.
#[allow(deprecated, deprecated_in_future)]
pub use core::{
    assert_eq, assert_ne, debug_assert, debug_assert_eq, debug_assert_ne, matches, r#try, todo,
    unimplemented, unreachable, write, writeln,
};

// Re-export built-in macros defined through libcore.
#[allow(deprecated)]
pub use core::{
    asm, assert, cfg, column, compile_error, concat, concat_idents, env, file, format_args,
    format_args_nl, global_asm, include, include_bytes, include_str, line, llvm_asm, log_syntax,
    module_path, option_env, stringify, trace_macros,
};

pub use core::primitive;

#[cfg(not(test))]
#[global_allocator]
static GLOBAL: MyAllocator = MyAllocator;

extern "C" {
    fn start();
}

#[cfg(not(test))]
#[start]
#[no_mangle]
#[allow(unreachable_code)]
extern "C" fn main(_argc: isize, _argv: *const *const u8) -> ! {
    unsafe {
        start();
    }
    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    unsafe {
        Log_Debug(b"PANIC!\0".as_ptr() as *const i8);
        let ptr = format!("{:?}\0", info).as_ptr();
        Log_Debug(ptr as *const i8);
    }

    loop {}
}

#[macro_export]
macro_rules! print {
    () => {{}};
    ($($arg:tt)+) => {{
        $crate::_print(format_args!($($arg)+));
    }};
}

#[macro_export]
macro_rules! println {
    () => {{}};
    ($($arg:tt)+) => {{
        $crate::_println(format_args!($($arg)+));
    }};
}

#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => { print!($($arg)*) }
}

#[macro_export]
macro_rules! eprintln {
    ($($arg:tt)*) => { println!($($arg)*) }
}

#[macro_export]
macro_rules! dbg {
    ($($arg:tt)*) => { println!($($arg)*) }
}

pub fn _print(msg: core::fmt::Arguments) {
    unsafe {
        let zmsg = format!("{}\0", msg);
        Log_Debug(zmsg.as_ptr() as *const i8);
    }
}

pub fn _println(msg: core::fmt::Arguments) {
    unsafe {
        let zmsg = format!("{}\n\0", msg);
        Log_Debug(zmsg.as_ptr() as *const i8);
    }
}
