use core::fmt::{self, Write};
use lazy_static::lazy_static;
use spin::Mutex;

const UART0_BASE: u32 = 0x3F20_1000; // Direccion base para Pi 3
const UART0_DR: *mut u32 = UART0_BASE as *mut u32;
const UART0_FR: *const u32 = (UART0_BASE + 0x18) as *const u32; // Offset 0x18 para Banderas
const UART0_CR: *mut u32 = (UART0_BASE + 0x30) as *mut u32; // Offset 0x30 para Control
const FR_TXFF: u32 = 1 << 5; // Bit 5: Transmit FIFO Full
const FR_RXFE: u32 = 1 << 4; // Bit 4: Receive FIFO Empty

pub struct Uart;

impl Uart {
    pub fn init() {
        unsafe {
            // Turn UART off to configure
            core::ptr::write_volatile(UART0_CR, 0);

            // Turn on: Bit 0 (Enable), Bit 8 (TXE), Bit 9 (RXE)
            // Its equal to 0x301
            let bits_encendido = (1 << 0) | (1 << 8) | (1 << 9);
            core::ptr::write_volatile(UART0_CR, bits_encendido);
        }
    }

    pub fn read_byte(&self) -> u8 {
        unsafe {
            while (core::ptr::read_volatile(UART0_FR) & FR_RXFE) > 0 {
                core::hint::spin_loop();
            }
            (core::ptr::read_volatile(UART0_DR) & 0xFF) as u8
        }
    }

    pub fn send_byte(&self, c: u8) {
        unsafe {
            while (core::ptr::read_volatile(UART0_FR) & FR_TXFF) > 0 {
                core::hint::spin_loop();
            }
            core::ptr::write_volatile(UART0_DR, c as u32);
        }
    }
}

impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.send_byte(b'\r');
            }
            self.send_byte(byte);
        }
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Uart> = Mutex::new(Uart);
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    WRITER.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => ($crate::uart::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}

