#![no_std]
#![no_main]

use bootloader_api::{BootInfo, entry_point};
use core::fmt::Write;
use core::panic::PanicInfo;
use uart_16550::backend::PioBackend;
use uart_16550::{Config, Uart16550Tty};

#[allow(dead_code)]
mod vga_buffer;

entry_point!(kernel_main);

fn serial() -> Uart16550Tty<PioBackend> {
    unsafe { Uart16550Tty::new_port(0x3F8, Config::default()) }
        .expect("シリアルポートを初期化できませんでした")
}

fn kernel_main(_boot_info: &'static mut BootInfo) -> ! {
    let mut port = serial();
    writeln!(port, "my-blog-os: UEFI kernel started").unwrap();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let _ = writeln!(serial(), "PANIC: {info}");
    loop {}
}
