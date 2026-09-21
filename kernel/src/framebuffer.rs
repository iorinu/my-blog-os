use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};

const GLYPH_WIDTH: usize = 5;
const GLYPH_HEIGHT: usize = 7;
const SCALE: usize = 8;

/// UEFIから渡されたFramebufferへ、起動確認用の文字列を描画する。
pub fn draw_hello_world(framebuffer: Option<&mut FrameBuffer>) {
    let Some(framebuffer) = framebuffer else {
        return;
    };

    let info = framebuffer.info();
    let buffer = framebuffer.buffer_mut();
    buffer.fill(0);
    draw_text(buffer, info, 40, 40, "Hello World!", [255, 255, 255]);
}

fn draw_text(
    buffer: &mut [u8],
    info: FrameBufferInfo,
    x: usize,
    y: usize,
    text: &str,
    color: [u8; 3],
) {
    let mut cursor_x = x;

    for byte in text.bytes() {
        if let Some(glyph) = glyph(byte) {
            draw_glyph(buffer, info, cursor_x, y, glyph, color);
        }
        cursor_x += (GLYPH_WIDTH + 1) * SCALE;
    }
}

fn draw_glyph(
    buffer: &mut [u8],
    info: FrameBufferInfo,
    x: usize,
    y: usize,
    glyph: [u8; GLYPH_HEIGHT],
    color: [u8; 3],
) {
    for (row, bits) in glyph.into_iter().enumerate() {
        for col in 0..GLYPH_WIDTH {
            let mask = 1 << (GLYPH_WIDTH - col - 1);
            if bits & mask == 0 {
                continue;
            }

            for offset_y in 0..SCALE {
                for offset_x in 0..SCALE {
                    write_pixel(
                        buffer,
                        info,
                        x + col * SCALE + offset_x,
                        y + row * SCALE + offset_y,
                        color,
                    );
                }
            }
        }
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

fn glyph(byte: u8) -> Option<[u8; GLYPH_HEIGHT]> {
    Some(match byte {
        b'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        b'e' => [
            0b01110, 0b10001, 0b11111, 0b10000, 0b01110, 0b00000, 0b00000,
        ],
        b'l' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01110, 0b00000,
        ],
        b'o' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b01110, 0b00000, 0b00000,
        ],
        b'W' => [
            0b10001, 0b10001, 0b10101, 0b10101, 0b01010, 0b00000, 0b00000,
        ],
        b'r' => [
            0b10110, 0b11001, 0b10000, 0b10000, 0b10000, 0b00000, 0b00000,
        ],
        b'd' => [
            0b00001, 0b01101, 0b10011, 0b10001, 0b10011, 0b01101, 0b00000,
        ],
        b'!' => [
            0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100,
        ],
        b' ' => [0; GLYPH_HEIGHT],
        _ => return None,
    })
}
