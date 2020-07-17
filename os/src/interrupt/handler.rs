use riscv::register::scause::{Exception, Interrupt, Scause, Trap};
use riscv::register::stvec;

use super::context::Context;
use super::timer;

global_asm!(include_str!("interrupt.S"));

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
    match scause.cause() {
        Trap::Exception(Exception::Breakpoint) => breakpoint(context),
        Trap::Interrupt(Interrupt::SupervisorTimer) => timer::tick(),
        _ => fault(context, scause, stval),
    }
}

fn breakpoint(context: &mut Context) {
    println!("Breakpoint at 0x{:x}", context.sepc);
    // This relies on the RISC-V C extension to function correctly
    context.sepc += 2;
}

fn fault(context: &mut Context, scause: Scause, stval: usize) {
    dbgx!(scause.cause(), &context, stval);
    panic!("Unresolved interrupt");
}
