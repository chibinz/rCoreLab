use crate::sbi::shutdown;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // Put os into infinite loop if something bad happens
    println!("\x1b[1;31mpanic: '{}'\x1b[0m", info.message().unwrap());
    shutdown()
}

#[no_mangle]
extern "C" fn abort() -> ! {
    panic!("abort()")
}
