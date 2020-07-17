pub mod context;
mod handler;
mod timer;

pub use context::Context;
pub fn init() {
    handler::init();
    timer::init();
    println!("mod interrupt initialized");
}
