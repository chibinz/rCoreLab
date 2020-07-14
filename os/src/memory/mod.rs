mod config;
mod heap;

pub fn init() {
    heap::init();
    unsafe { riscv::register::sstatus::set_sum() };

    println!("mod memory initialized");
}
