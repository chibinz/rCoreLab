pub mod config;
pub mod frame;
pub mod mapping;

mod range;
mod heap;
mod address;

use heap::HEAP_SPACE;
pub use {address::*, config::*, mapping::*, frame::FRAME_ALLOCATOR, range::Range};

pub fn init() {
    heap::init();
    unsafe { riscv::register::sstatus::set_sum() };

    dbgx!(*KERNEL_END_ADDRESS, unsafe {HEAP_SPACE.as_ptr()} as usize, MEMORY_START_ADDRESS, MEMORY_END_ADDRESS);
    println!("mod memory initialized");
}
