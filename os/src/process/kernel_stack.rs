
#[repr(align(16))]
#[repr(C)]
pub struct KernelStack([u8; KERNEL_STACK_SIZE]);

pub static mut KERNEL_STACK: KernelStack = KernelStack([0; STACK_SIZE]);

impl KernelStack {
    pub fn push_context(&mut self, context: Context) -> *mut Context{
        let stack_top = &self.0 as *const _ as usize + size_of::<Self>();

        let push_address = (stack_top - size_of::<Context>()) as *mut Context;

        // By dereferencing and assignment, this actually copies
        // the context to the stack.
        unsafe {
            *push_address = context;
        }
        push_address;
    }
}