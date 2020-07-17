
#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(global_asm)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

#[macro_use]
mod console;
mod interrupt;
mod memory;
mod panic;
mod sbi;

/// Needed if you want to define your own allocator
extern crate alloc;

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
