#![no_std]
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(rustos::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

#![feature(allocator_api)]

use core::panic::PanicInfo;

extern crate alloc;
use alloc::alloc::{alloc, dealloc}; 
use alloc::alloc::Layout;

use bootloader::{bootinfo::BootInfo, entry_point};

use rustos::{serial_print, serial_println};
use rustos::test::{exit_qemu, QemuExitCode};

entry_point!(test_kmain);


/// This function is the entry point, since the linker looks for a function
/// named `_start` by default.
#[no_mangle] // don't mangle the name of this function
pub fn test_kmain(boot_info: &'static BootInfo) -> ! {
    rustos::init(boot_info);
    test_main();
    serial_println!("exiting");
    exit_qemu(QemuExitCode::Success);
    loop {}
}


fn allocate_mem(size: usize, align: usize) -> usize {
    let layout = Layout::from_size_align(size, align).unwrap();
    (unsafe { alloc(layout) }) as usize
}

#[test_case]
pub fn problematic_pattern() {
    use rustos::heap::stack::align_up;


    serial_print!("testing specific problematic pattern...");
    let addr1 = allocate_mem(0xc0, 0x8);
    assert_eq!(addr1, rustos::arch::heap::LINKED_LIST_START);
    let addr2 = allocate_mem(0x38, 0x8);
    assert_eq!(addr2, addr1 + 0xc0);
    let addr3 = allocate_mem(0x3e8, 0x1);
    assert_eq!(addr3, addr2 + 0x38);
    let addr4 = allocate_mem(0x640, 0x8);
    assert_eq!(addr4, addr3 + 0x3e8);
    let addr5 = allocate_mem(0x200, 0x80);
    assert_eq!(addr5, align_up(addr4 + 0x640, 0x80));
    let addr6 = allocate_mem(0x3e8, 0x1);
    assert_eq!(addr6, addr5 + 0x200);
    let addr7 = allocate_mem(0xc0, 0x8);
    assert_eq!(addr7, addr6 + 0x3e8);
    let addr8 = allocate_mem(0x3e8, 0x1);
    assert_eq!(addr8, addr7 + 0xc0);
    let addr9 = allocate_mem(0x3e8, 0x1);
    assert_eq!(addr9, addr8 + 0x3e8);
    let layout10 = Layout::from_size_align(0x18, 0x8).unwrap();
    let addr10 = unsafe { alloc(layout10) } as usize;
    assert_eq!(addr10, align_up(addr4 + 0x640, 0x8));
    let layout11 = Layout::from_size_align(0x20, 0x8).unwrap();
    let addr11 = unsafe { alloc(layout11) } as usize;
    assert_eq!(addr11, addr10 + 0x18);
    let layout12 = Layout::from_size_align(0x118, 0x8).unwrap();
    let addr12 = unsafe { alloc(layout12) } as usize;
    assert_eq!(addr12, addr9 + 0x3e8);
    unsafe {dealloc(addr11 as *mut u8, layout11)}
    unsafe {dealloc(addr10 as *mut u8, layout10)};
    let layout14 = Layout::from_size_align(0x20, 0x8).unwrap();
    let addr14 = unsafe { alloc(layout14) } as usize;
    assert_eq!(addr14, addr11);
    serial_println!("[ok]");
}
rustos::test_panic!(QemuExitCode::Failed);
