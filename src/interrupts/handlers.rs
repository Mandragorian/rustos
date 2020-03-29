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
