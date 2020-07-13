use super::context::Context;
use riscv::register::stvec;
use riscv::register::scause::Scause;

global_asm!(include_str!("./interrupt.S"));

pub fn init() {
    unsafe {
        extern "C" {
            // Entry for interrupt.S
            fn __interrupt();
        }

    // Set interrupt entry, and trap mode to direct
    stvec::write(__interrupt as usize, stvec::TrapMode::Direct);
    }
}

/// This function is called by __interrupt
#[no_mangle]
pub fn handle_interrupt(context: &mut Context, scause: Scause, stval: usize) {
    panic!("Interrupted: {:?}", scause.cause());
}