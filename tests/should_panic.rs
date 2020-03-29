#![no_std]
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(rustos::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rustos::{serial_print, serial_println};
use rustos::test::{exit_qemu, QemuExitCode};

/// This function is the entry point, since the linker looks for a function
/// named `_start` by default.
#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    test_main();
    serial_println!("exiting");
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[test_case]
pub fn failed_assert() {
    serial_print!("testing should-fail...");
    assert_eq!(0, 1);
    serial_println!("[failed]");
}


rustos::test_panic!(QemuExitCode::Success);
