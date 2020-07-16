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

    dbgx!(rust_main as usize);

    let remap = memory::mapping::MemorySet::new_kernel().unwrap();
    remap.activate();

    panic!("Shutting down")
}
