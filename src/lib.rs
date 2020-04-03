#![feature(allocator_api)]
#![feature(alloc_error_handler)] // at the top of the file
#![no_std]
#![cfg_attr(test, no_main)]
#![cfg_attr(feature = "x86", feature(abi_x86_interrupt))]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, test_runner(crate::test::test_runner))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]

mod stack;
mod sync;
pub mod slab;

extern crate alloc;

#[macro_use]
pub mod vga_buffer;

#[macro_use]
pub mod serial;

pub mod arch;

#[macro_use]
pub mod test;

use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();


#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}


// TESTS
/// Entry point for `cargo xtest`
#[cfg(test)]
use bootloader::bootinfo::BootInfo;

#[no_mangle]
#[cfg(test)]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {

    let mut mapper = unsafe { crate::arch::memory::init(boot_info.physical_memory_offset) };
    let mut frame_allocator = crate::arch::memory::init_frame_allocator(&boot_info.memory_map);

    crate::arch::allocator::init(&mut mapper, &mut frame_allocator)
        .expect("failed to init heap");

    test_main();
    exit_qemu(QemuExitCode::Success);
    loop {}
}


#[cfg(test)]
use crate::test::{exit_qemu, QemuExitCode};
#[cfg(test)]
use core::panic::PanicInfo;
#[cfg(test)]
crate::test_panic!(QemuExitCode::Failed);
