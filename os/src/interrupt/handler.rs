use riscv::register::{
    scause::{Exception, Interrupt, Scause, Trap},
    stvec,
};
use alloc::{format, string::String};
use crate::process::PROCESSOR;

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
pub fn handle_interrupt(context: &mut Context, scause: Scause, stval: usize) -> *mut Context {
    match scause.cause() {
        Trap::Exception(Exception::Breakpoint) => breakpoint(context),
        Trap::Interrupt(Interrupt::SupervisorTimer) => supervisor_timer(context),
        _ => Err(format!(
            "unimplemented interrupt type: {:x?}",
            scause.cause()
        )),
    }.unwrap_or_else(|m| fault(m, scause, stval))
}

/// 处理时钟中断
fn supervisor_timer(context: &mut Context) -> Result<*mut Context, String> {
    timer::tick();
    PROCESSOR.get().park_current_thread(context);
    Ok(PROCESSOR.get().prepare_next_thread())
}


fn breakpoint(context: &mut Context) -> Result <*mut Context, String>{
    println!("Breakpoint at 0x{:x}", context.sepc);
    // This relies on the RISC-V C extension to function correctly
    context.sepc += 2;
    Ok(context)
}

fn fault(msg: String, scause: Scause, stval: usize) -> *mut Context {
    println!(
        "{:#x?} terminated: {}",
        PROCESSOR.get().current_thread(),
        msg
    );
    println!("cause: {:?}, stval: {:x}", scause.cause(), stval);

    PROCESSOR.get().kill_current_thread();
    PROCESSOR.get().prepare_next_thread()
}
