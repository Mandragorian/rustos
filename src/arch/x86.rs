pub mod gdt;
pub mod memory;
pub mod interrupts;
pub mod allocator;


use x86_64::instructions::interrupts::without_interrupts;

pub fn initialize() {
    gdt::init_gtd();
    interrupts::init_interrupts();
}

pub fn no_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R
{
    without_interrupts(f)
}

pub fn halt() {
    x86_64::instructions::hlt()
}

pub fn halt_loop() -> ! {
    loop {
        x86_64::instructions::hlt()
    }
}

pub fn read_port<T>(port_addr: u16) -> T 
where T: x86_64::structures::port::PortReadWrite
{
    let port = x86_64::instructions::port::Port::new(port_addr);
    unsafe { port.read() }
}
