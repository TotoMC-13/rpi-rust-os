#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rpi_mini_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::arch::global_asm;
use core::panic::PanicInfo;
use rpi_mini_os::println;

global_asm!(include_str!("../src/boot.S"));

#[unsafe(no_mangle)]
pub extern "C" fn _start_rust() -> ! {
    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rpi_mini_os::test_panic_handler(info)
}

#[test_case]
fn test_println() {
    println!("Test succesful");
}