#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]
#![feature(custom_test_frameworks)]
#![test_runner(rustos::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

use rustos::test::{exit_qemu, QemuExitCode};
use rustos::{serial_print, serial_println, test_panic};
use core::panic::PanicInfo;
use lazy_static::lazy_static;

#[no_mangle]
#[allow(unconditional_recursion)]
pub extern "C" fn _start() -> ! {
    serial_print!("testing stack overflow double fault...");
    rustos::arch::initialize();
    init_test_idt();

    fn stack_overflow() {
        let _a = [0u64; 1024];
        stack_overflow(); // for each recursion, the return address is pushed
    }

    // trigger a stack overflow
    stack_overflow();

    serial_println!("failed");
    serial_println!("No exception occured");

    exit_qemu(QemuExitCode::Failed);

    loop {}
}

use x86_64::structures::idt::{InterruptStackFrame, InterruptDescriptorTable};

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(rustos::arch::gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}

pub fn init_test_idt() {
    TEST_IDT.load();
}

extern "x86-interrupt" fn double_fault_handler(
    _stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("ok");

    exit_qemu(QemuExitCode::Success);
    loop {}
}

test_panic!(QemuExitCode::Failed);
