# Lab 2 学习记录

# 问题
1. `memory` module 中依赖声明的问题。
由于这一次lab新增的文件比较多，而且tutorial里面对依赖声明这一部分是明确省略的。所以自己手敲代码的时候遇到了不少编译错误，都是缺少mod或者use declaration造成的。一般自己写Rust的时候不太习惯用pub use这条语句来re-export names，因为这样会导致文件结构变得混乱，本来是在子目录里面的一个name现在可以从上级目录use。搞了半天，添加了`pub use {address::*, config::*, frame::FRAME_ALLOCATOR, range::Range};`之后就可以编译了。

## 思考
1. `bss`段为何不置0？
通常情况下，操作系统会给bss段的变量分配零页。但是这里是我们自己写操作系统，操作系统作为第一个运行的程序（不考虑bootloader），在使用bss段的变量之前应该先清零。然而在使用过程中，即使在没有手动置零的情况下也能正常运行。自己猜测有三种可能的原因：
1. QEMU默认未初始化的内存值为0。
2. 由于Rust RAII的特性，即使在no_std的情况下也会在变量使用之前对它进行初始化。
3. 这里的bss段只是作为操作系统随意分配的一块内存使用，初始值并不重要。

在heap的`init`函数中加上这么一句
```Rust
HEAP_SPACE.iter().for_each(|byte| assert_eq!(*byte, 0));
```
也不会panic。

2. Tutorial中提到的两个Allocator的区别
Tutorial提到的第一allocator是动态内存分配这一章里面的堆。这个堆位于.bss段里面，长度为8MiB，每次分配元素的大小随意，可以是8个字节的整型，也可以是从零长到几千个字节的Vector。具体怎么用取决于操作系统，所以这个堆的名字叫做**操作系统自用堆**可能更合适些。后面一个allocator是放在kernel外面的，因此应该是给用户程序分配内存的。颗粒度上更粗一些，每次分配4KiB大小的物理页，如果所需内存大于4KiB就要分配多个页。用文字描述的话可能还是不太清晰，这里结合linker.ld, objdump输出，以及内核运行时的debug printing做出说明：

3. 物理页外面为什么要封装一层FrameTracker？
一个物理页的PageNumber是独一无二的，因此获取和回收物理页的时候其实只需要知道页号（usize）就行了。为什么tutorial里面会用一个FrameTracker来封装页号呢？实验里是这么写到的：
> 我们设计的初衷是分配器分配给我们 FrameTracker 作为一个帧的标识，而随着不再需要这个物理页，我们需要回收，我们利用 Rust 的 drop 机制在析构的时候自动实现回收。
Rust的drop机制保证了不会再使用（Out of scope）的变量会自动回收，以避免出现内存泄露的情况。当一个变量离开作用范围之后会自动调用drop函数。因为自动隐式的调用在操作系统这种需要直接操纵内存的底层程序其实是一件比较危险的事情。Lab的思考题里面演示了物理页因为没有被返回，在match直接被drop，造成了FRAME_ALLOCATOR的dealloc被alloc死锁的情况。上周日的交流中，也有同学提出了类似的问题，因为物理页自动被回收导致程序运行错误。和那位同学观点相似，我认为在底层程序中运用高级语言特性时需要非常谨慎。就个人而言，相比之下Explicit的dealloc，有借有还，可能更直观清晰一些。
```Rust
let frame = FRAME_ALLOCATOR.lock().alloc();
// Use the frame
// ...
FRAME_ALLOCATOR.lock().dealloc(frame);
```
只不过存在着内存泄露的风险，而且内存泄露是悄无声息的，不会有任何报错。