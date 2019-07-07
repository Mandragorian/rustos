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

    println!("Hello World{} {}", "!", boot_info.physical_memory_offset);

    rustos::arch::initialize();

    use x86_64::{structures::paging::Page, VirtAddr};
    let mut mapper = unsafe { rustos::arch::memory::init(boot_info.physical_memory_offset) };
    let mut frame_allocator = rustos::arch::memory::init_frame_allocator(&boot_info.memory_map);
    let page = Page::containing_address(VirtAddr::new(0x1000));
    rustos::arch::memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // write the string `New!` to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    println!("It did not crash!");
    rustos::arch::halt_loop();
}
