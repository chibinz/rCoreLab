mod context;
mod handler;
mod timer;

pub fn init() {
    handler::init();
    timer::init();
    println!("mod interrupt initialized");
}
