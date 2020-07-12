#![allow(unused)]

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
    sbi_call(1, c, 0, 0);
}

pub fn console_getchar() -> usize {
    sbi_call(2, 0, 0, 0)
}

pub fn shutdown() -> ! {
    sbi_call(8, 0, 0, 0);
    unreachable!()
}
