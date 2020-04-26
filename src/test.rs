use crate::serial_println;

pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[macro_export]
macro_rules! test_panic {
    ($code:expr) => {
        #[cfg(test)]
        #[panic_handler]
        fn panic(info: &PanicInfo) -> ! {
            match $code {
                QemuExitCode::Failed => {
                    serial_println!("[failed]\n");
                    serial_println!("Error: {}\n", info);
                },
                QemuExitCode::Success => {
                    serial_println!("[ok]");
                }
            };
            exit_qemu($code);
            loop {}
        }
    }
}

pub fn breakpoint() {}
