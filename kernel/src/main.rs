#![no_std]
#![no_main]

use bootloader_api::{BootInfo, entry_point};
use core::panic::PanicInfo;
use uart_16550::backend::PioBackend;
use uart_16550::{Config, Uart16550};

mod framebuffer;
#[allow(dead_code)]
mod vga_buffer;

entry_point!(kernel_main);

fn serial() -> Option<Uart16550<PioBackend>> {
    let mut port = unsafe { Uart16550::new_port(0x3F8).ok()? };
    port.init(Config::default()).ok()?;
    Some(port)
}

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    framebuffer::draw_hello_world(boot_info.framebuffer.as_mut());

    if let Some(mut port) = serial() {
        port.send_bytes_exact(b"my-blog-os: UEFI kernel started\r\n");
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
