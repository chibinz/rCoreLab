pub struct Process {
    pub is_user: bool,
    pub memory_set: MemorySet,
}

impl Process {
    pub fn run(&mut self) -> ! {
        extern "C" {
            fn __restore(context: usize);
        }

        unsafe {
            __restore(context);
        }

        // A thread never returns
        unreachable!()
    }
}

struct Processor;

impl Processor {
    pub fn tick(&mut self, context: &mut Context) -> *mut Context {
        if let Some(next_thread) = self.scheduler.get_next() {
            if next_thread = self.current_thread() {
                context
            } else {
                let context = next_thread.run();
                let current_thread = self.current_thread.replace(next_thread).unwrap();
                current_thread.park(*context);
                next_context
            }
        } else {
            panic!("All threads terminated, shutting down!");
        }
    }
}