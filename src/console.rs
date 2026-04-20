use core::str;
use lazy_static::lazy_static;
use spin::Mutex;

use crate::{
    framebuffer::{Color, fill_screen},
    graphics::{self, GRAPHICS_CONSOLE, draw_cursor, erase_cursor, set_cursor},
    print, println, serial_println,
};

const MAX_LINE_LEN: usize = 126;

#[derive(Copy, Clone)]
pub struct LineBuffer {
    buffer: [u8; MAX_LINE_LEN],
    len: usize,
}

impl LineBuffer {
    pub const fn new() -> Self {
        LineBuffer {
            buffer: [0; MAX_LINE_LEN],
            len: 0,
        }
    }

    pub fn push(&mut self, byte: u8) -> Result<(), &'static str> {
        if self.len < MAX_LINE_LEN {
            self.buffer[self.len] = byte;
            self.len += 1;
            Ok(())
        } else {
            Err("Buffer is Full")
        }
    }

    pub fn pop(&mut self) {
        if self.len > 0 {
            self.len -= 1;
        }
    }

    // Return contnet as a slice
    pub fn as_str(&self) -> Result<&str, str::Utf8Error> {
        str::from_utf8(&self.buffer[..self.len])
    }

    // Clear buffer
    pub fn clear(&mut self) {
        self.len = 0;
    }
}

pub struct Console {
    input_buffer: LineBuffer,
    pre_text: &'static str,
}

impl Console {
    pub fn new() -> Console {
        Console {
            input_buffer: LineBuffer::new(),
            pre_text: "input>",
        }
    }

    pub fn receive_input(&mut self, input: u8) {
        match input {
            b'\n' | b'\r' => {
                self.input_buffer.push(input).ok();
                let line = self.input_buffer.clone();
                self.input_buffer.clear();

                graphics::erase_cursor();

                println!("");

                self.process_command(line.as_str().unwrap());

                print!("{}", self.pre_text);
                draw_cursor();
            }
            8 | 127 => {
                if self.input_buffer.len > 0 {
                    erase_cursor();

                    self.input_buffer.pop();

                    let mut gc = GRAPHICS_CONSOLE.lock();
                    if gc.x >= 8 {
                        gc.x -= 8;
                    }

                    graphics::erase_char(gc.x, gc.y, &Color::new(0, 0, 255));
                    drop(gc);

                    draw_cursor();
                }
            }
            37 => {
                serial_println!("Flecha izq")
            }
            39 => {
                serial_println!("Flecha der")
            }
            _ => {
                if self.input_buffer.push(input).is_ok() {
                    erase_cursor();
                    print!("{}", input as char);
                    draw_cursor();
                }
            }
        }
    }

    fn process_command(&mut self, input: &str) {
        match input.trim() {
            "clear" | "cls" => {
                self.clear_screen();
            }
            _ => (),
        }
    }

    fn clear_screen(&self) {
        fill_screen(&Color::new(0, 0, 255));
        set_cursor(8, 8);
    }
}

lazy_static! {
    pub static ref CONSOLE: Mutex<Console> = Mutex::new(Console::new());
}
