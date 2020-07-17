use crate::sbi::set_timer;
use riscv::register::{sie, time};

// Ticks / Cycles til next timer interrupt
static INTERVAL: usize = 100000;
static mut TICKS: usize = 0;

pub fn init() {
    unsafe {
        // Enable timer interrupt
        sie::set_stimer();

        // Do not enable enable interrupt at boot
        // sstatus::set_sie();
    }

    set_next_timeout();
}

pub fn set_next_timeout() {
    set_timer(time::read() + INTERVAL);
}

pub fn tick() {
    set_next_timeout();

    unsafe {
        TICKS += 1;
        if TICKS % 100 == 0 {
            println!("{} ticks", TICKS);
        }
    }
}
