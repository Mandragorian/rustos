use crate::{println, print};

#[derive(Debug, PartialEq, Eq)]
pub struct RustosExcStackframe {
    pub instruction_pointer: usize,
    pub code_segment: usize,
    pub cpu_flags: usize,
    pub stack_pointer: usize,
    pub stack_segment: usize,
    pub errcode: Option<u64>,
}

impl RustosExcStackframe {
    pub fn set_errcode(&mut self, error_code: u64) {
        self.errcode = Some(error_code);
    }
}

pub fn breakpoint_handler<T>(stack_frame: T)
where T: Into<RustosExcStackframe> {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame.into());
}

pub fn double_fault_handler<T>(stack_frame: T)
where T: Into<RustosExcStackframe> {
    println!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame.into());
    crate::arch::halt_loop();
}

pub fn timer_interrupt_handler<T>(_stack_frame: T)
where T: Into<RustosExcStackframe> {
    print!(".");
}

pub fn kbd_interrupt_handler<T>(_stack_frame: T)
where T: Into<RustosExcStackframe> {
    let scancode: u8 = crate::arch::read_port(0x60);
    println!("{}", scancode);
}
