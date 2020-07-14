pub mod config;
pub mod frame;

mod range;
mod heap;
mod address;

pub use {address::*, config::*, frame::FRAME_ALLOCATOR, range::Range};

pub fn init() {
    heap::init();
    unsafe { riscv::register::sstatus::set_sum() };

    println!("mod memory initialized");
}
