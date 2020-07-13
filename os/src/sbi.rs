#![allow(unused)]

use CallNum::*;

enum CallNum {
    SetTimer = 0,
    ConsolePutChar = 1,
    ConsoleGetChar = 2,
    ClearIPI = 3,
    SendIPI = 4,
    RemoteFenceI = 5,
    RemoteSFenceVMA = 6,
    RemoteSFenceVMAASID = 7,
    Shutdown = 8,
}

#[inline(always)]
fn sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let ret;
    unsafe {
        llvm_asm!("ecall"
            : "={x10}" (ret)
            : "{x10}" (arg0), "{x11}" (arg1), "{x12}" (arg2), "{x17}" (which)
            : "memory"
            : "volatile");
    }
    ret
}

pub fn console_putchar(c: usize) {
    sbi_call(ConsolePutChar as usize, c, 0, 0);
}

pub fn console_getchar() -> usize {
    sbi_call(ConsoleGetChar as usize, 0, 0, 0)
}

pub fn set_timer(time: usize) {
    sbi_call(SetTimer as usize, time, 0, 0);
}

pub fn shutdown() -> ! {
    sbi_call(Shutdown as usize, 0, 0, 0);
    unreachable!()
}
