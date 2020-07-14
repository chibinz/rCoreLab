#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(global_asm)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

#[macro_use]
mod console;
mod panic;
mod sbi;
mod interrupt;
mod memory;

/// Needed if you want to define your own allocator
extern crate alloc;

global_asm!(include_str!("entry.S"));

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    interrupt::init();
    memory::init();

    use alloc::boxed::Box;
    use alloc::vec::Vec;
    let v = Box::new(5);
    assert_eq!(*v, 5);
    core::mem::drop(v);

    let mut vec = Vec::new();
    (0..10000).for_each(|i| vec.push(i));
    dbg!(vec[233]);

    println!("heap test passed");

    panic!("Shutting down")
}
