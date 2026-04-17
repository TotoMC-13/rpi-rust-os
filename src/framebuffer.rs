use core::str;

use lazy_static::lazy_static;
use spin::Mutex;

const P_BASE: u32 = 0x3F00_0000; // Peripheral Base Address
const MAIL_BASE: u32 = P_BASE + 0xB880; // Mailbox base adress
const MAIL_READ: *const u32 = MAIL_BASE as *const u32; // Mailbox read adress
const MAIL_WRITE: *mut u32 = (MAIL_BASE + 0x20) as *mut u32; // Mailbox write adress
const MAIL_STATUS: *const u32 = (MAIL_BASE + 0x18) as *const u32; // Mailbox status adress
const MAIL_FULL: u32 = 1 << 31; // El bit 31 indica que no hay espacio para escribir
const MAIL_EMPTY: u32 = 1 << 30; // El bit 30 indica que no hay nada para leer

#[repr(C, align(16))]
pub struct MailboxMessage {
    buffer: [u32; 36],
}

pub struct FrameBuffer {
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub ptr: *mut u32, // Our array pointer in the RAM
}

pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    pub fn to_u32(&self) -> u32 {
        0xFF000000 | (self.b as u32) << 16 | (self.g as u32) << 8 | (self.r as u32)
    }
}

impl Default for Color {
    fn default() -> Self {
        Color { r: 0, g: 0, b: 0 }
    }
}

unsafe impl Send for FrameBuffer {} // I tell Rust this pointer can be shared safely
unsafe impl Sync for FrameBuffer {} // I tel Rust this pointer can be seen at the same time safely (Mutex)

impl FrameBuffer {
    pub fn init() -> Result<(), &'static str> {
        // Create the empty letter, 36 spaces 4b each. 144 bytes.
        let mut msj = MailboxMessage { buffer: [0; 36] };

        // HEADER (Envelope)
        msj.buffer[0] = 36 * 4; // Mail weight (144b)
        msj.buffer[1] = 0; // 0 because we are saying that we are the cpu

        // First TAG: Screen size
        msj.buffer[2] = 0x48003; // ID Real de "Set Physical Display"
        msj.buffer[3] = 8; // Space for answer: 8b (2 integers) 
        msj.buffer[4] = 8; // Local code (8b request)
        msj.buffer[5] = 1024; // Width
        msj.buffer[6] = 768; // Height

        // Second TAG: Space color depth
        msj.buffer[7] = 0x48005; // Real ID for "Set Depth"
        msj.buffer[8] = 4; // Space for answer: 4b (1 integer) 
        msj.buffer[9] = 4; // Request code
        msj.buffer[10] = 32; // Sent value: 32 bits of depth (RGBA)

        // Third TAG: Assignm me some Memory RAM for video
        msj.buffer[11] = 0x40001; // ID for "ALLOCATE BUFFER"
        msj.buffer[12] = 8; // 8 bytes answer for Pointer and Size
        msj.buffer[13] = 8; // Request code
        msj.buffer[14] = 16; // Align my ram pointer to 16 bytes
        msj.buffer[15] = 0; // Empty: GPU overwrites this with buffer size

        // CARD END
        msj.buffer[16] = 0; // END TAG (0 marks the final, the rest is ignored)

        // SEND
        unsafe {
            let mailbox_ptr = msj.buffer.as_ptr() as u32;

            // Send through channel 8 (video)
            Self::mailbox_write(8, mailbox_ptr);

            // We wait for the answer
            Self::mailbox_read(8);
        }

        // We read the answer, if the code is now 0x8000_0000 everything went as expected
        if msj.buffer[1] == 0x8000_0000 {
            // Success
            // We go to the position in which we left the Allocate Buffer (index 11)
            // We know the pointer answer goes to the index 14

            let pointer_fb = msj.buffer[14] & 0x3FFFFFFF;

            let pitch_fb = 1024 * 4;

            *FRAMEBUFFER.lock() = Some(FrameBuffer {
                width: 1024,
                height: 768,
                pitch: pitch_fb,
                ptr: pointer_fb as *mut u32, // Framebuffer pointer
            });
            return Ok(());
        } else {
            return Err("GPU Error.");
        }
    }

    pub unsafe fn mailbox_write(channel: u8, data: u32) {
        unsafe {
            // Wait until there's mail
            while (core::ptr::read_volatile(MAIL_STATUS) & MAIL_FULL) != 0 {
                core::hint::spin_loop();
            }

            // We clean the last 4 bits and join data with channel
            let message = (data & !0xF) | (channel as u32 & 0xF);

            // Write the message
            core::ptr::write_volatile(MAIL_WRITE, message);
        }
    }

    pub unsafe fn mailbox_read(channel: u8) -> u32 {
        loop {
            unsafe {
                // Wait until mail is not empty
                while (core::ptr::read_volatile(MAIL_STATUS) & MAIL_EMPTY) != 0 {
                    core::hint::spin_loop();
                }
                // Mailbox is no longer empty, we read the mail
                let mail = core::ptr::read_volatile(MAIL_READ);
                // We compare the last 4 bits to see if the channel is correct
                if (mail & 0xF) == channel as u32 {
                    // We return the mail without channel adress
                    return mail & !0xF;
                }
            }
        }
    }
}

unsafe fn write_pixel(ptr: *mut u32, pitch: u32, x: u32, y: u32, color_u32: u32) {
    // Calcualte pixel offset
    let offset = (y * (pitch / 4) + x) as usize;
    unsafe {
        core::ptr::write_volatile(ptr.add(offset), color_u32);
    }
}

pub fn fill_screen(color: &Color) {
    let fb_lock = FRAMEBUFFER.lock();
    if let Some(fb) = fb_lock.as_ref() {
        let color_u32 = color.to_u32();
        for y in 0..fb.height {
            for x in 0..fb.width {
                unsafe {
                    write_pixel(fb.ptr, fb.pitch, x, y, color_u32);
                }
            }
        }
    }
}

pub fn draw_pixel(x: u32, y: u32, color: &Color) {
    let fb_lock = FRAMEBUFFER.lock();
    if let Some(fb) = fb_lock.as_ref() {
        unsafe {
            write_pixel(fb.ptr, fb.pitch, x, y, color.to_u32());
        }
    }
}

lazy_static! {
    pub static ref FRAMEBUFFER: Mutex<Option<FrameBuffer>> = Mutex::new(None);
}

const MAX_LINE_LEN: usize = 126;

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
