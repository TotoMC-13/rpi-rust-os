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

pub fn erase_char(x: u32, y: u32, bg_color: &Color) {
    for row in 0..8 {
        for col in 0..8 {
            draw_pixel(x + col as u32, y + row as u32, bg_color);
        }
    }
}

pub fn display_cursor(x: u32, y: u32, color_texto: &Color, color_fondo: &Color, visible: bool) {
    let cursor_glyph: [u8; 8] = [0, 0, 0, 0, 0, 0, 0b11111111, 0b11111111];
    for (row, row_byte) in cursor_glyph.iter().enumerate() {
        for col in 0..8 {
            if (*row_byte & (1 << col)) != 0 {
                let pos_x = x + col as u32;
                let pos_y = y + row as u32;
                let color_a_pintar = if visible { color_texto } else { color_fondo };
                draw_pixel(pos_x, pos_y, color_a_pintar);
            }
        }
    }
}

pub fn draw_cursor() {
    let gc = GRAPHICS_CONSOLE.lock();
    display_cursor(gc.x, gc.y, &Color::new(255, 255, 255), &Color::new(0, 0, 255), true);
}

pub fn erase_cursor() {
    let gc = GRAPHICS_CONSOLE.lock();
    erase_char(gc.x, gc.y, &Color::new(0, 0, 255));
}

pub struct GraphicsConsole {
    pub x: u32,
    pub y: u32,
    pub fg_color: Color,
    pub x_spacing: u32,
    pub y_spacing: u32,
    pub x_margin: u32,
}

impl GraphicsConsole {
    pub fn init(
        x: u32,
        y: u32,
        fg_color: Color,
        x_spacing: u32,
        y_spacing: u32,
        x_margin: u32,
    ) -> Result<GraphicsConsole, &'static str> {
        Ok(GraphicsConsole {
            x,
            y,
            fg_color,
            x_spacing,
            y_spacing,
            x_margin,
        })
    }
}

impl fmt::Write for GraphicsConsole {
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

pub fn set_cursor(x: u32, y: u32) {
    let mut console = GRAPHICS_CONSOLE.lock();
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

lazy_static! {
    pub static ref GRAPHICS_CONSOLE: Mutex<GraphicsConsole> =
        Mutex::new(GraphicsConsole::init(8, 8, Color::new(255, 255, 255), 8, 16, 8).unwrap());
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    GRAPHICS_CONSOLE.lock().write_fmt(args).unwrap();
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
