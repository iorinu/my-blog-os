#![no_std]
#![no_main]

use ::core::panic::PanicInfo;
use core::fmt::write;
mod vga_buffer;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
static HELLO: &[u8] = b"Hello World!";

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("Hello World {}", "!");

    loop {}
}
