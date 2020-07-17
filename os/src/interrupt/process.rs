lazy_static! {
    pub static ref PROCESSOR: UnsafeWrapper<Processor> = Default::default();
}

#[derive(Default)]
pub struct Processor {
    current_thread: Option<Arc<Thread>>,
    scheduler: SchedulerImpl<Arc<Thread>>,
}

pub trait Scheduler<ThreadType: Clone + Eq>: Default {
    fn add_thread<T>(&mut self, thread:ThreadType, priority: T);
    fn get_next(&mut self) -> Option<ThreadType>;
    fn remove_thread(&mut self, thread: ThreadType);
    fn set_priority<T>(&mut self, thread: ThreadType, priority: T);
}

impl Processor {
    pub fn run(&mut self) -> ! {
        // interrupt.asm 中的标签
        extern "C" {
            fn __restore(context: usize);
        }
        // 从 current_thread 中取出 Context
        let context = self.current_thread().run();
        // 从此将没有回头
        unsafe {
            __restore(context as usize);
        }
        unreachable!()
    }
}