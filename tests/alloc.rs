#![no_std]
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(rustos::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

#![feature(allocator_api)]

use core::panic::PanicInfo;

extern crate alloc;
use alloc::{alloc::alloc, boxed::Box, vec, vec::Vec, rc::Rc};
use alloc::collections::BTreeMap;
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

struct TestStruct([u8; 256]);
struct TestStructWeirdSize([u8; 389]);

#[test_case]
pub fn simple_alloc() {

    serial_print!("testing consecutive allocations...");
    //serial_println!("linked list start: {:x}", rustos::arch::heap::LINKED_LIST_START);
    //serial_println!("usize size: {}", core::mem::size_of::<usize>());
    let size = core::mem::size_of::<TestStruct>();
    let heap_value1 = Box::new(TestStruct([0; 256]));
    let heap_value2 = Box::new(TestStruct([0; 256]));
    //serial_println!("heap_value1: ====== {:p}", heap_value1);
    //serial_println!("heap_value2: ====== {:p}", heap_value2);
    //let layout = Layout::from_size_align(64, 2048).unwrap();
    //let ptr1 = unsafe { alloc(layout) };
    //serial_println!("ptr1: ====== {:x}", ptr1 as usize);
    //let heap_value2 = Box::new(41usize);
    //serial_println!("heap_value2: ====== {:p}", heap_value2);
    //let ptr2 = unsafe { alloc(layout) };
    //serial_println!("ptr2: ====== {:x}", ptr2 as usize);
    let ptr1: *const TestStruct = &*heap_value1;
    let ptr2: *const TestStruct = &*heap_value2;
    assert_eq!(ptr1 as usize + size, ptr2 as usize);
    serial_println!("[ok]");
}

#[test_case]
pub fn alloc_weird_size() {
    serial_print!("testing consecutive allocations weird size...");
    let size = core::mem::size_of::<TestStructWeirdSize>();
    let heap_value1 = Box::new(TestStructWeirdSize([0; 389]));
    let heap_value2 = Box::new(TestStructWeirdSize([0; 389]));
    let ptr1: *const TestStructWeirdSize = &*heap_value1;
    let ptr2: *const TestStructWeirdSize = &*heap_value2;
    assert_eq!(ptr1 as usize + size, ptr2 as usize);
    serial_println!("[ok]");
}

fn prng(x: usize) -> usize {
    (8121 * x + 28411) % 134456 
}

#[test_case]
pub fn alloc_weird_size() {
    serial_print!("testing pseudorandom allocations...");

    let mut values: [(usize, usize, usize); 357] = [(0, 0, 0); 357];
    for i in 0..357 {
        let i_usize = i as usize;
        values[i] = (i_usize, i_usize, i_usize);
    }
    let mut btree: BTreeMap<usize, (usize, usize, usize)> = BTreeMap::new();

    let mut rand_raw: usize = 0;
    for _ in 0..(40000) {
        let rand = rand_raw % 1000;
        if btree.contains_key(&rand) {
            let removed = btree.remove(&rand).unwrap();
            //serial_println!("removing {}", rand);
            assert_eq!(removed, (rand, rand, rand));
        } else {
            //serial_println!("adding {}", rand);
            btree.insert(rand, (rand, rand, rand));
        }
        rand_raw = prng(rand_raw);
    }

    serial_println!("[ok]");
}
rustos::test_panic!(QemuExitCode::Failed);
