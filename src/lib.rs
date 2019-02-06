#![feature(abi_x86_interrupt)]
#![cfg_attr(not(test), no_std)]
#[macro_use]
pub mod vga_buffer;

#[macro_use]
pub mod serial;

pub mod arch;
pub mod interrupts;

pub mod test_suite;
