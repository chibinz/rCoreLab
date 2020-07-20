
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
mod drivers;
mod fs;

/// Needed if you want to define your own allocator
extern crate alloc;

use process::*;
use memory::PhysicalAddress;

global_asm!(include_str!("entry.S"));

#[no_mangle]
pub extern "C" fn rust_main(_hart_id: usize, dtb_pa: PhysicalAddress) -> ! {
    interrupt::init();
    memory::init();
    drivers::init(dtb_pa);
    fs::init();

    let process = Process::new_kernel().unwrap();

    PROCESSOR
        .get()
        .add_thread(Thread::new(process.clone(), simple as usize, Some(&[0])).unwrap());

    // 把多余的 process 引用丢弃掉
    drop(process);

    PROCESSOR.get().run()
}

/// 测试任何内核线程都可以操作文件系统和驱动
fn simple(id: usize) {
    println!("hello from thread id {}", id);
    // 新建一个目录
    // 输出根文件目录内容
    fs::ls("hello_world");
    fs::ls("../../notebook");

    loop {}
}