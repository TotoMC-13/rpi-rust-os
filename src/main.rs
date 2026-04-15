#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::arch::global_asm;
use core::panic::PanicInfo;

global_asm!(include_str!("boot.S"));

use rpi_mini_os::println;
use rpi_mini_os::uart::Uart;

#[unsafe(no_mangle)]
pub extern "C" fn _start_rust() -> ! {
    Uart::init();

    println!("--- Kernel ARM64 Iniciado ---");
    println!("Version: {}", "0.1.0");

    #[cfg(test)]
    test_main();

    loop {
        let c = rpi_mini_os::uart::WRITER.lock().read_byte();
        println!("Caracter actual: {}", c as char);
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let _ = println!("[ERROR]: {info}");
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rpi_mini_os::test_panic_handler(info)
}
