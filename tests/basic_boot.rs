#![no_std]
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(rustos::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rustos::{serial_print, serial_println};
use rustos::test::{exit_qemu, QemuExitCode};
use rustos::println;

/// This function is the entry point, since the linker looks for a function
/// named `_start` by default.
#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    test_main();
    exit_qemu(QemuExitCode::Success);
    loop {}
}

#[test_case]
pub fn simple_boot() {
    serial_print!("testing simple boot...");
    serial_println!("[ok]");
}

#[test_case]
pub fn test_vga() {
    serial_print!("testing vga...");
    println!("test_println output");
    serial_println!("[ok]");
}

rustos::test_panic!(QemuExitCode::Failed);
