#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;

// Incluir el código de arranque en ensamblador
global_asm!(include_str!("boot.S"));

// Importamos las macros de la raíz del crate
use rpi_mini_os::println;
use rpi_mini_os::uart::Uart;

#[unsafe(no_mangle)]
pub extern "C" fn _start_rust() -> ! {
    Uart::init();

    println!("--- Kernel ARM64 Iniciado ---");
    println!("Version: {}", "0.1.0");

    loop {
        let c = rpi_mini_os::uart::WRITER.lock().read_byte();
        println!("Caracter actual: {}", c as char);
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let _ = println!("[ERROR]: {info}");
    loop {}
}
