use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use core::fmt::{self, Write};
use font8x8::{BASIC_FONTS, UnicodeFonts};
use spin::{Mutex, Once};

const FONT_WIDTH: usize = 8;
const FONT_HEIGHT: usize = 8;
const SCALE: usize = 4;
const TAB_WIDTH: usize = 4;
const FOREGROUND: [u8; 3] = [255, 255, 255];

static WRITER: Once<Mutex<FramebufferWriter>> = Once::new();

/// Framebufferへの出力先を初期化する。
pub fn init(framebuffer: FrameBuffer) {
    WRITER.call_once(|| Mutex::new(FramebufferWriter::new(framebuffer)));
}

/// `print!`／`println!`マクロから呼び出される出力関数。
pub fn _print(args: fmt::Arguments) {
    if let Some(writer) = WRITER.r#try() {
        let _ = writer.lock().write_fmt(args);
    }
}

/// UEFI framebufferへASCII文字列を描画するWriter。
struct FramebufferWriter {
    framebuffer: FrameBuffer,
    info: FrameBufferInfo,
    column: usize,
    row: usize,
}

impl FramebufferWriter {
    fn new(framebuffer: FrameBuffer) -> Self {
        let info = framebuffer.info();
        let mut writer = Self {
            framebuffer,
            info,
            column: 0,
            row: 0,
        };
        writer.clear();
        writer
    }

    fn clear(&mut self) {
        self.framebuffer.buffer_mut().fill(0);
    }

    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            b'\r' => self.column = 0,
            b'\t' => {
                for _ in 0..TAB_WIDTH {
                    self.write_byte(b' ');
                }
            }
            byte => {
                if self.column + FONT_WIDTH * SCALE > self.info.width {
                    self.new_line();
                }

                let glyph = if (b' '..=b'~').contains(&byte) {
                    BASIC_FONTS.get(byte as char)
                } else {
                    None
                }
                .or_else(|| BASIC_FONTS.get('?'))
                .expect("ASCIIフォントに疑問符がありません");

                self.draw_glyph(glyph);
                self.column += (FONT_WIDTH + 1) * SCALE;
            }
        }
    }

    fn new_line(&mut self) {
        self.column = 0;
        self.row += FONT_HEIGHT * SCALE;
        if self.row + FONT_HEIGHT * SCALE > self.info.height {
            self.scroll();
        }
    }

    fn scroll(&mut self) {
        let Some(row_bytes) = self.info.stride.checked_mul(self.info.bytes_per_pixel) else {
            self.clear();
            self.row = 0;
            return;
        };
        let Some(scroll_bytes) = (FONT_HEIGHT * SCALE).checked_mul(row_bytes) else {
            self.clear();
            self.row = 0;
            return;
        };

        let buffer = self.framebuffer.buffer_mut();
        if scroll_bytes >= buffer.len() {
            buffer.fill(0);
            self.row = 0;
            return;
        }

        buffer.copy_within(scroll_bytes.., 0);
        let clear_start = buffer.len() - scroll_bytes;
        buffer[clear_start..].fill(0);
        self.row = self.info.height.saturating_sub(FONT_HEIGHT * SCALE);
    }

    fn draw_glyph(&mut self, glyph: [u8; FONT_HEIGHT]) {
        let x = self.column;
        let y = self.row;
        let info = self.info;
        let buffer = self.framebuffer.buffer_mut();

        for (glyph_row, bits) in glyph.into_iter().enumerate() {
            for glyph_column in 0..FONT_WIDTH {
                if bits & (1 << glyph_column) == 0 {
                    continue;
                }

                for offset_y in 0..SCALE {
                    for offset_x in 0..SCALE {
                        write_pixel(
                            buffer,
                            info,
                            x + glyph_column * SCALE + offset_x,
                            y + glyph_row * SCALE + offset_y,
                            FOREGROUND,
                        );
                    }
                }
            }
        }
    }
}

impl fmt::Write for FramebufferWriter {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for byte in text.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

fn write_pixel(buffer: &mut [u8], info: FrameBufferInfo, x: usize, y: usize, color: [u8; 3]) {
    if x >= info.width || y >= info.height || info.bytes_per_pixel == 0 {
        return;
    }

    let Some(row_offset) = y
        .checked_mul(info.stride)
        .and_then(|offset| offset.checked_mul(info.bytes_per_pixel))
    else {
        return;
    };
    let Some(pixel_offset) = x.checked_mul(info.bytes_per_pixel) else {
        return;
    };
    let Some(offset) = row_offset.checked_add(pixel_offset) else {
        return;
    };
    let Some(end) = offset.checked_add(info.bytes_per_pixel) else {
        return;
    };
    if end > buffer.len() {
        return;
    }

    match info.pixel_format {
        PixelFormat::Rgb if info.bytes_per_pixel >= 3 => {
            buffer[offset] = color[0];
            buffer[offset + 1] = color[1];
            buffer[offset + 2] = color[2];
        }
        PixelFormat::Bgr if info.bytes_per_pixel >= 3 => {
            buffer[offset] = color[2];
            buffer[offset + 1] = color[1];
            buffer[offset + 2] = color[0];
        }
        PixelFormat::U8 => {
            let grayscale = ((color[0] as u16 + color[1] as u16 + color[2] as u16) / 3) as u8;
            buffer[offset] = grayscale;
        }
        _ => {}
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::framebuffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
