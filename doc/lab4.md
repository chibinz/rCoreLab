# Lab 4 学习记录

## 问题
1. Store/LoadFault
开gdb发现是在sample_process的function prologue里面产生的exception。也就是说栈的虚拟页可能没有初始化好。问题好像出在了add_segment函数上面了。为了和教程匹配, 把init_data这一项参数删去了，因为本来从alloc_page_range传进来的时候就是None，没有用上。可能删的时候不小心把物理页分配相关的代码页删去了。导致往栈上读写数据的时候会有Store/LoadFault。最后的解决办法是把rCore-Tutorial lab-4分支memory文件夹覆盖了过来，重新搞好依赖声明。

## 思考
1. Process结构体中为何没有所属线程的信息？
理论上来说进程process和线程thread是包含关系。一个进程包含一个或多个线程。但是在rCore-Tutorial的实现里面既没有所属线程的thread id，也没有线程的引用。也就是说一个进程无法知道它有几个线程在运行，也不知道是么时候所有线程会运行结束。反倒是Thread结构体里面有Process的Arc引用。这样会有一些问题，就比如说无法kill掉一个process，因为process自己也不知道哪些线程属于自己。教程里面这么做应该大概是为了简化流程，process只存储特权级（is_user）和地址空间相关内容（memory_set）。

2. Process的初始化
- Process
    - is_user
    - memory_set
Process有两个field。第一个field是一个bool，决定process是内核进程还是用户进程。lab-4教程里只考虑内核线程，所以process的初始化其实相当于memory_set的初始化。
- MemorySet
    - Mapping
    - Segment
    - allocated_pairs
Segment存的是映射类型和页使用权限，allocated_pairs存的是所有被映射的虚拟地址和其对应的物理页。后者在功能上感觉和page_table有些重叠。个人理解这是一种优化。最高级页表512个entry通常用不完，遍历allocated_pairs比遍历page table可能更快一些。Mapping是page_table的层层包裹的中间一层。下面归纳一下从process到单个页表项page_table_entry经历的indirection。
Process->MemorySet->Mapping->Vec<PageTableTracker>--取根表-->PageTableTracker--(*Tracker是指针的封装，调用deref)-->PageTable->PageTableEntry
总共7层indirection，感觉有点封装过度了……
内核进程在初始化过程中调用且只会调用一次FRAME_ALLOCATOR.alloc()，用来给自己的页表分配一个物理页，其余的虚拟地址都是内核镜像本身所包含的地址，数据已经存在物理内存中，只需要在页表中添加映射关系就行了，不需要再分配物理页。

3. Thread的初始化
- Thread
    - Context
    - Stack
之前在DailySchedule中说thread的state仅处在Context里面，做完实验之后发现不太准确。每个线程都有自己独立的栈，栈上的数据也属于线程的状态。stack的初始化和物理页分配在process的alloc_page_range函数里面，先分配物理页，再从没有被占用的连续虚拟地址取出来一块，最后建立两者的映射关系。Context没有呢么多花哨的东西，设置好栈指针，入口，传给程序的参数，以及权限就行了。

4. 内核栈
一直不太明白sscratch或者sp寄存器是什么时候被写入KERNEL_STACK的地址的。操作系统启动的时候使用的栈是entry.S里面的boot_stack，此时sp被设置为boot_stack_top这个符号的地址。然而并没有看到有一条汇编指令把sp设置成KERNEL_STACK的地址。用gdb调了一下，发现藏得蛮深的。具体过程是
rust_main -> Process::run -> Thread::prepare -> KernelStack::push_context
```Rust
let stack_top = &self.0 as *const _ as usize + size_of::<Self>();
```
self.0其实就是KERNEL_STACK，把它转为指针获得基址，随后再加上偏移获得栈顶的地址。因为Context derive了Clone，Copy，所以才可以这么写，`*push_address = context`，把上下文复制到内核栈上的一块内存。最最最重要的其实是返回值，a0寄存器里面存了push_address。Process::run函数里Thread::prepare返回之后没有任何其他代码，直接跳转到__restore的地址，因此a0寄存器的值不会被覆盖。它会在interrupt.S里面被复制到sp里面，而sp会在退出中断的时候与sscratch做交换。此时sscratch存的就是内核栈的地址。一开始没找到的原因是只看了Thread::new，而实际上内核栈地址的写入在线程开始运行的时候发生。还有一个问题没有太搞清楚，有了内核栈KERNEL_STACK之后boot_stack是不是就废弃了？

5. 线程调度
Scheduler调度的单元不是process而是thread。因为不知道上一个运行的线程和将要运行的线程是否同属于一个进程，所以每次从中断返回的时候都需要flush一下TLB。其实这个开销还是蛮大的，可以做一点优化。比如相同优先级的情况下prefer属于同一进程的线程。Scheduler出了执使线程调度功能之外还是一个存储所有线程信息的数据结构。默认实现里面的FIFO和HRRN调度器调度器内部都是用一个linked list来存线程。后期如果想要实现非阻塞式的锁，线程休眠等可能还要添加其他数据结构，比如block list，sleep list这样的。

## 改进 TODO
1. 实现Pintos 里面提到BSD scheduler
2. 减少封装的层数
    - Process -> PageTable
    - Processor -> Scheduler
    - ...
3. 实现线程的sleep，yield和exit