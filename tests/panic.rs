#![no_std]
#![no_main]
#![allow(unreachable_code)]
#![allow(unused_imports)]
#![feature(custom_test_frameworks)]
#![test_runner(rustos::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rustos::test::{QemuExitCode, exit_qemu};
use rustos::{test_panic, serial_println, serial_print};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("testing should-fail...");
    panic!();
    serial_println!("[failed]");
}
test_panic!(QemuExitCode::Success);
