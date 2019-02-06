#![no_std]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

use core::panic::PanicInfo;
use rustos::test_suite::exit_qemu;
use rustos::serial_println;

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    rustos::arch::initialize();

    x86_64::instructions::int3();

    serial_println!("ok");

    unsafe { exit_qemu(); }
    loop {}
}


#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("failed");

    serial_println!("{}", info);

    unsafe { exit_qemu(); }
    loop {}
}