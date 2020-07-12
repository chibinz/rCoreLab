#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(global_asm)]

global_asm!(include_str!("entry.S"));

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    // Put os into infinite loop if something bad happens
    loop {}
}

pub fn console_putchar(ch: u8) {
    let _ret: usize;
    let arg0: usize = ch as usize;
    let arg1: usize = 0;
    let arg2: usize = 0;
    let which: usize = 1;
    unsafe {
        llvm_asm!("ecall"
            : "={x10}" (_ret)
            : "{x10}" (arg0), "{x11}" (arg1), "{x12}" (arg2), "{x17}" (which)
            : "memory"
            : "volatile"
        );
    }
}

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    console_putchar(b'O');
    console_putchar(b'K');
    console_putchar(b'\n');

    loop {}
}
