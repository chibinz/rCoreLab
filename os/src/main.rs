#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(global_asm)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(slice_fill)]
#[macro_use]
mod console;
mod drivers;
mod fs;
mod interrupt;
mod kernel;
mod memory;
mod panic;
mod process;
mod sbi;

/// Needed if you want to define your own allocator
extern crate alloc;

use memory::PhysicalAddress;
use process::*;
use xmas_elf::ElfFile;
use fs::INodeExt;

global_asm!(include_str!("entry.S"));

#[no_mangle]
pub extern "C" fn rust_main(_hart_id: usize, dtb_pa: PhysicalAddress) -> ! {
    interrupt::init();
    memory::init();
    drivers::init(dtb_pa);
    fs::init();

    let app = fs::ROOT_INODE.find("hello_world").unwrap();
    let data = app.readall().unwrap();
    let elf = ElfFile::new(data.as_slice()).unwrap();
    let process = Process::from_elf(&elf, true).unwrap();
    let thread = Thread::new(process, elf.header.pt2.entry_point() as usize, None).unwrap();
    PROCESSOR.get().add_thread(thread);

    PROCESSOR.get().run()
}
