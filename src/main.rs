#![no_std]
#![no_main]
#![allow(unused_imports)]
#![feature(custom_test_frameworks)]
#![test_runner(rustos::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

#![feature(allocator_api)]

use core::panic::PanicInfo;

extern crate alloc;
use alloc::{alloc::alloc, boxed::Box, vec, vec::Vec, rc::Rc};
use alloc::alloc::Layout;

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



    rustos::init(boot_info);
    
    let mut executor = rustos::cooperative::executor::Executor::new();
    let spawner = executor.get_spawner();
    println!("got spawner");
    let task1 = rustos::cooperative::task::Task::new(buggy_example_task(spawner.clone()));
    println!("got task1");
    let task2 = rustos::cooperative::task::Task::new(keyboard_printer(spawner.clone()));
    println!("got task2");
    executor.spawn(task1);
    println!("spawned task1");
    executor.spawn(task2);
    println!("spawned task2");
    executor.run();

    println!("It did not crash!");
    rustos::arch::halt_loop();
}

async fn keyboard_printer(mut spawner: rustos::cooperative::executor::Spawner) {
    use futures_util::stream::StreamExt;
    let mut codes = rustos::cooperative::keyboard::ScancodeStream::new();

    while let Some(scancode) = codes.next().await {
        println!("{}", scancode);
    }
}

async fn async_number() -> u32 {
    42
}

async fn buggy_example_task(mut spawner: rustos::cooperative::executor::Spawner) {
    let number = async_number().await;
    let mut i = 0;
    println!("async number: {}", number);
    let new_task = rustos::cooperative::task::Task::new(buggy_example_task(spawner.clone()));
    spawner.spawn(new_task);
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
