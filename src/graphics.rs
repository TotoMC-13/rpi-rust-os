use crate::framebuffer::{Color, FRAMEBUFFER, draw_pixel};
use core::fmt::{self, Write};
use font8x8::UnicodeFonts;
use lazy_static::lazy_static;
use spin::Mutex;

pub fn draw_char(x: u32, y: u32, ch: char, color: &Color) {
    if let Some(glyph) = font8x8::BASIC_FONTS.get(ch) {
        for (row, row_byte) in glyph.iter().enumerate() {
            for col in 0..8 {
                // We check each pixel to see if we should paint it
                // Use shift to check if this pixel is on
                if (*row_byte & (1 << col)) != 0 {
                    // Calculate the position we need to paint
                    let pos_x = x + col as u32;
                    let pos_y = y + row as u32;

                    draw_pixel(pos_x, pos_y, color);
                }
            }
        }
    }
}

pub struct Console {
    pub x: u32,
    pub y: u32,
    pub fg_color: Color,
    pub x_spacing: u32,
    pub y_spacing: u32,
    pub x_margin: u32,
}

impl Console {
    pub fn init(
        x: u32,
        y: u32,
        fg_color: Color,
        x_spacing: u32,
        y_spacing: u32,
        x_margin: u32,
    ) -> Result<Console, &'static str> {
        Ok(Console {
            x,
            y,
            fg_color,
            x_spacing,
            y_spacing,
            x_margin,
        })
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for ch in text.chars() {
            if ch == '\n' {
                self.y += self.y_spacing;
                self.x = self.x_margin;
            } else {
                let lim_x = {
                    if let Some(fb) = FRAMEBUFFER.lock().as_ref() {
                        fb.width - self.x_spacing
                    } else {
                        1024 - self.x_spacing
                    }
                };

                // Check if we need to jump one line
                if self.x >= lim_x {
                    self.x = self.x_margin;
                    self.y += self.y_spacing;
                }

                draw_char(self.x, self.y, ch, &self.fg_color);
                self.x += self.x_spacing;
            }
        }
        Ok(())
    }
}

lazy_static! {
    pub static ref CONSOLE: Mutex<Console> =
        Mutex::new(Console::init(8, 16, Color::new(255, 255, 255), 8, 16, 8).unwrap());
}

pub fn set_cursor(x: u32, y: u32) {
    let mut console = CONSOLE.lock();
    console.x = x;
    console.y = y;
}

pub fn draw_string(x: u32, y: u32, txt: &str, color: &Color) {
    let mut pos_x = x;
    for ch in txt.chars() {
        draw_char(pos_x, y, ch, color);
        pos_x += 8;
    }
}

pub fn draw_string_centered(y: u32, txt: &str, color: &Color) {
    let width = {
        if let Some(fb) = FRAMEBUFFER.lock().as_ref() {
            fb.width
        } else {
            1024
        }
    };

    let centered_x = (width / 2).saturating_sub((txt.len() as u32 * 8) / 2);

    draw_string(centered_x, y, txt, color);
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    CONSOLE.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::graphics::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
