#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(unused_imports))]

use core::panic::PanicInfo;

use bootloader::{bootinfo::BootInfo, entry_point};

use rustos::println;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    rustos::arch::halt_loop()
}

entry_point!(kmain);

#[cfg(not(test))]
pub fn kmain(boot_info: &'static BootInfo) -> ! {
    println!("Hello World{}", "!");

    rustos::arch::initialize();

    let mut rtable = unsafe {
        rustos::arch::memory::init(boot_info.p4_table_addr as usize)
    };

    let mut frame_allocator = rustos::arch::memory::init_frame_allocator(&boot_info.memory_map);

    rustos::arch::memory::create_example_mapping(&mut rtable,  &mut frame_allocator);
    unsafe { (0xdeadbeef900 as *mut u64).write_volatile(0xf021f077f065f04e)};

    println!("It did not crash!");
    rustos::arch::halt_loop();
}
