#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use crate::uart::Uart;
use core::panic::PanicInfo;

pub mod framebuffer;
pub mod graphics;
pub mod uart;

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        Uart::init();
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    Uart::init();
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

// Entry point for `cargo test`
#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum QemuExitCode {
    Success = 0x20026, // ADP_Stopped_ApplicationExit
    Failed = 0x20027,  // ADP_Stopped_RunTimeErrorUnknown
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    // AArch64 semihosting: SYS_EXIT (0x18)
    // x0 = op number (0x18)
    // x1 = pointer to parameters block: [reason: u64, sub_code: u64]
    const SYS_EXIT: u64 = 0x18;

    let params: [u64; 2] = [exit_code as u64, 0];

    unsafe {
        core::arch::asm!(
            "hlt #0xF000",
            in("x0") SYS_EXIT,
            in("x1") params.as_ptr(),
            options(nostack),
        );
    }
    loop {}
}
