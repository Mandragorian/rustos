pub mod gdt;
pub mod memory;
pub mod interrupts;
pub mod heap;


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

#[inline]
fn read_port<T>(port_addr: u16) -> T
where T: x86_64::structures::port::PortReadWrite
{
    let mut port = x86_64::instructions::port::Port::new(port_addr);
    unsafe { port.read() }
}

#[inline]
pub fn read_port_u8(port_addr: u16) -> u8 {
    read_port(port_addr)
}

#[inline]
pub fn write_port<T>(port_addr: u16, val: T)
where T: x86_64::structures::port::PortReadWrite
{
    let mut port = x86_64::instructions::port::Port::new(port_addr);
    unsafe { port.write(val) }
}

#[inline]
pub fn write_port_u8(port_addr: u16, val: u8) {
    write_port(port_addr, val);
}

#[inline]
pub fn read_port_u16(port_addr: u16) -> u16 {
    read_port(port_addr)
}

#[inline]
pub fn read_port_u32(port_addr: u16) -> u32 {
    read_port(port_addr)
}
