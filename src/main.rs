#![no_std]
#![no_main]
#![allow(unused_imports)]
#![feature(custom_test_frameworks)]
#![test_runner(rustos::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

#![feature(allocator_api)]

use core::panic::PanicInfo;

extern crate alloc;
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};

use bootloader::{bootinfo::BootInfo, entry_point};

use rustos::println;

entry_point!(kmain);

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    rustos::arch::halt_loop()
}

#[cfg(not(test))]
pub fn kmain(boot_info: &'static BootInfo) -> ! {
    println!("Hello World{} {}", "!", boot_info.physical_memory_offset);


    rustos::arch::initialize();

    use x86_64::{structures::paging::Page, VirtAddr};
    let mut mapper = unsafe { rustos::arch::memory::init(boot_info.physical_memory_offset) };

    use x86_64::structures::paging::mapper::MapperAllSizes;
    println!("0xb8001 - > {:?}", mapper.translate_addr(VirtAddr::new(0xb8001)));
    let mut frame_allocator = rustos::arch::memory::init_frame_allocator(&boot_info.memory_map);
    let page = Page::containing_address(VirtAddr::new(0x100000));
    rustos::arch::memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // write the string `New!` to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    rustos::arch::heap::init(&mut mapper, &mut frame_allocator)
        .expect("failed to init heap");

    let slabloc = rustos::heap::slab::SmallAllocator::new(rustos::arch::heap::SLAB_1_START,
                                                          rustos::arch::heap::SLAB_2_START,
                                                          rustos::arch::heap::SLAB_3_START,
                                                          rustos::arch::heap::SLAB_4_START,
                                                          rustos::arch::heap::LINKED_LIST_START);




    let heap_value = Box::new(41usize);
    
    let mut vec: Vec<usize> = Vec::with_capacity(500);
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());
    println!("heap_value at {:p}", heap_value);
    println!("heap_value size {:x}", core::mem::size_of::<Box<usize>>());

    drop(vec);
    drop(heap_value);

    println!("dropped===============");
    let heap_value = Box::new(41usize);
    println!("heap_value at {:p}", heap_value);

    // create a reference counted vector -> will be freed when count reaches 0
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    println!("reference count is {} now", Rc::strong_count(&cloned_reference));

    println!("It did not crash!");
    rustos::arch::halt_loop();
}


// TESTS
#[cfg(test)]
use rustos::serial_println;

#[cfg(test)]
pub fn kmain(_boot_info: &'static BootInfo) -> ! {
    #[cfg(test)]
    test_main();
    serial_println!("finishing tests...");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

use rustos::test_panic;
use rustos::test::{exit_qemu, QemuExitCode};
test_panic!(QemuExitCode::Failed);
