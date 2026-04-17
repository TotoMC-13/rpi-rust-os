#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::arch::global_asm;
use core::panic::PanicInfo;

global_asm!(include_str!("boot.S"));

use rpi_mini_os::framebuffer::{Color, FrameBuffer, LineBuffer, fill_screen};
use rpi_mini_os::graphics::{draw_string_centered, set_cursor};
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

    draw_string_centered(
        16,
        init_text,
        &Color {
            r: 255,
            g: 255,
            b: 255,
        },
    );

    draw_string_centered(
        32,
        ver_text,
        &Color {
            r: 255,
            g: 255,
            b: 255,
        },
    );

    set_cursor(8, 48);

    let mut txt: LineBuffer = LineBuffer::new();

    loop {
        let x = Uart.read_byte();

        match x {
            13 | 10 => {
                print!("\n");
            }
            _ => print!("{}", x as char),
        }
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
