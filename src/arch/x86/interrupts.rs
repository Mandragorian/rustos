use spin;

use lazy_static::lazy_static;
use pic8259_simple::ChainedPics;
use x86_64::structures::idt::{PageFaultErrorCode, InterruptDescriptorTable, InterruptStackFrame};

use super::gdt;
//use crate::interrupts::handlers;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub const TIMER_INTERRUPT_ID: u8 = PIC_1_OFFSET;
pub const KBD_INTERRUPT_ID: u8 = PIC_1_OFFSET + 1;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    pub static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(x86_breakpoint_handler);
	unsafe {
        idt.double_fault.set_handler_fn(double_fault_handler)
	   .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
	}
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[usize::from(TIMER_INTERRUPT_ID)].set_handler_fn(timer_interrupt_handler);
        idt[usize::from(KBD_INTERRUPT_ID)].set_handler_fn(kbd_interrupt_handler);

        idt
    };
}

//impl Into<handlers::RustosExcStackframe> for &mut InterruptStackFrame {
//    fn into(self) -> handlers::RustosExcStackframe {
//        handlers::RustosExcStackframe {
//            instruction_pointer: self.instruction_pointer.as_u64() as usize,
//            code_segment: self.code_segment as usize,
//            cpu_flags: self.cpu_flags as usize,
//            stack_pointer: self.stack_pointer.as_u64() as usize,
//            stack_segment: self.stack_segment as usize,
//            errcode: None
//        }
//    }
//}

extern "x86-interrupt" fn x86_breakpoint_handler(
    stack_frame: &mut InterruptStackFrame)
{
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame, _error_code: u64) -> !
{
    println!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    crate::arch::x86::halt_loop()
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: PageFaultErrorCode)
{
    use super::halt_loop;
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    halt_loop();
}

extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: &mut InterruptStackFrame)
{
    unsafe { PICS.lock().notify_end_of_interrupt(TIMER_INTERRUPT_ID) }
}

extern "x86-interrupt" fn kbd_interrupt_handler(
    _stack_frame: &mut InterruptStackFrame)
{
    let scancode = crate::arch::x86::read_port_u8(0x60);
    crate::cooperative::keyboard::add_scancode(scancode);
    unsafe { PICS.lock().notify_end_of_interrupt(KBD_INTERRUPT_ID) }
}

pub fn init_interrupts() {
    IDT.load();
    unsafe {PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub fn enable() {
    x86_64::instructions::interrupts::enable();
}

pub fn disable() {
    x86_64::instructions::interrupts::disable();
}

pub fn enable_interrupt_halt() {
    x86_64::instructions::interrupts::enable_interrupts_and_hlt()
}
