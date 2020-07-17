
#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(global_asm)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(slice_fill)]
#[macro_use]
mod console;
mod interrupt;
mod memory;
mod panic;
mod sbi;
mod process;

/// Needed if you want to define your own allocator
extern crate alloc;

use process::*;

global_asm!(include_str!("entry.S"));

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    interrupt::init();
    memory::init();

    let process = Process::new_kernel().unwrap();

    for message in 0..8 {
        let thread = Thread::new(
            process.clone(),
            sample_process as usize,
            Some(&[message]),
        ).unwrap();
        PROCESSOR.get().add_thread(thread);
    }

    drop(process);

    PROCESSOR.get().run()
}

fn sample_process(message: usize) {
    for i in 0..1000000 {
        if i % 200000 == 0 {
            println!("thread {}", message);
        }
    }
}
