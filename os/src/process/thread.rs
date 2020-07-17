use crate::memory::range::Range;

pub struct Thread {
    pub id: ThreadID,
    pub stack: Range<VirtualAddress>,
    pub context: Mutex<Option<Context>>,
    pub process: Arc<Rwlock<Process>>,
}

impl Thread {
    pub fn park(&self, context: Context) {
        let mut slot = self.context.lock();
        assert!(slot.is_none());

        slot.replace(context);
    }

    pub fn run(&self) -> *mut Context {
        self.process.read().memory_set.activate();
        let parked_frame = self.context.lock().take().unwrap();

        if self.process.read().is_user {
            KERNEL_STACK.push_context(parked_frame)
        } else {
            let context = (parked_frame.sp() - size_of::<Context>()) as *mut Context;
            unsafe { *context = parked_frame };
            context
        }
    }
}