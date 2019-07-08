#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)] // at the top of the file
#![cfg_attr(not(test), no_std)]

extern crate alloc;

#[macro_use]
pub mod vga_buffer;

#[macro_use]
pub mod serial;

pub mod arch;
pub mod interrupts;

pub mod test_suite;

use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();


#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
