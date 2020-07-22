use riscv::register::{
    scause::{Exception, Interrupt, Scause, Trap},
    stvec,
    sie,
};
use alloc::{format, string::String};
use crate::process::PROCESSOR;
use crate::sbi::console_getchar;
use crate::fs::STDIN;
use crate::memory::*;
use crate::kernel::syscall_handler;

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

        // Enable external interrupt
        sie::set_sext();

        // OpenSBI interrupt enable
        *PhysicalAddress(0x0c00_2080).deref_kernel() = 1u32 << 10;
        *PhysicalAddress(0x0c00_2080).deref_kernel() = 1u32 << 10;

        // Serial port
        *PhysicalAddress(0x1000_0004).deref_kernel() = 0x0bu8;
        *PhysicalAddress(0x1000_0001).deref_kernel() = 0x01u8;
        *PhysicalAddress(0x0C00_0028).deref_kernel() = 0x07u32;
        *PhysicalAddress(0x0C20_1000).deref_kernel() = 0u32;
    }
}

/// This function is called by __interrupt
#[no_mangle]
pub fn handle_interrupt(context: &mut Context, scause: Scause, stval: usize) -> *mut Context {
    match scause.cause() {
        Trap::Exception(Exception::Breakpoint) => breakpoint(context),
        Trap::Exception(Exception::UserEnvCall) => Ok(syscall_handler(context)),
        Trap::Interrupt(Interrupt::SupervisorTimer) => supervisor_timer(context),
        Trap::Interrupt(Interrupt::SupervisorExternal) => Ok(supervisor_external(context)),
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

fn supervisor_external(context: &mut Context) -> *mut Context {
    let mut c = console_getchar();
    if c <= 255 {
        if c == '\r' as usize {
            c = '\n' as usize;
        }
        STDIN.push(c as u8);
    }
    context
}