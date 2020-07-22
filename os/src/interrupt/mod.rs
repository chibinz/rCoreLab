pub mod context;
mod handler;
mod timer;

pub use context::Context;
pub fn init() {
    handler::init();
    timer::init();
    println!("mod interrupt initialized");
}

pub fn wait_for_interrupt() {
    use riscv::register::*;
    unsafe {
        sie::clear_stimer();
        sstatus::set_sie();
        llvm_asm!("wfi" :::: "volatile");
        sstatus::clear_sie();
        sie::set_stimer();
    }
}
