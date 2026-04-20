#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::arch::global_asm;
use core::panic::PanicInfo;

global_asm!(include_str!("boot.S"));

use rpi_mini_os::console::CONSOLE;
use rpi_mini_os::framebuffer::{Color, FrameBuffer, fill_screen};
use rpi_mini_os::uart::Uart;
use rpi_mini_os::{print, println, serial_println};

#[unsafe(no_mangle)]
pub extern "C" fn _start_rust() -> ! {
    Uart::init();
    let _fb = FrameBuffer::init();

    #[cfg(test)]
    test_main();

    let init_text = "--- Kernel ARM64 Iniciado ---";
    let ver_text = "Version: 0.1.0";

    serial_println!("{init_text}");
    serial_println!("{ver_text}");
    fill_screen(&Color::new(0, 0, 255));

    println!("Kernel ARM64 [Version 0.1.0]");
    print!("input>");
    rpi_mini_os::graphics::draw_cursor();

    loop {
        let input = Uart.read_byte();
        CONSOLE.lock().receive_input(input);
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let _ = serial_println!("[ERROR]: {info}");
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rpi_mini_os::test_panic_handler(info)
}
