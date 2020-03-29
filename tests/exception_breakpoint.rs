#![no_std]
#![no_main]
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]
#![feature(custom_test_frameworks)]
#![test_runner(rustos::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rustos::test::{QemuExitCode, exit_qemu};
use rustos::{test_panic, serial_println, serial_print};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    rustos::arch::initialize();
    serial_print!("testing breakpoing interrupt...");
    x86_64::instructions::interrupts::int3();

    serial_println!("ok");

    exit_qemu(QemuExitCode::Success);
    loop {}
}

test_panic!(QemuExitCode::Failed);
