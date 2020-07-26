# Lab 6 学习记录

Lab 6其实做了3件事情：一是用户程序的环境搭建；二是操作系统调用的实现；三是标准输入和输出流。要说三者是独立的也不不完全准确。用户程序与操作系统沟通的桥梁是系统调用。标准输入和标准输出在*nix系统中被抽象成一个文件，在read/write的时候需要特殊处理。但是如果能够把他们放在单独的章节讲的话可能会更清晰一些。

## 问题
没有遇到实现上的问题，编译通过之后就能正常输出“Hello world from user program”了。

## 思考
1. 为什么要打包生成镜像？
编译生成的ELF文件不就已经是一个文件了吗？为什么需要打包生成qcow镜像？我们的操作系统是运行在qemu虚拟机上面的，其实和运行qemu的操作系统是在两台机器上面。况且文件本身是文件系统对储存介质的一种抽象，我们的文件系统sfs，与Linux的ext文件系统并不兼容。因此我们需要rcore-fs-fuse和qemu-img的帮助，来把linux里面的文件变成我们写的系统可以识别的磁盘镜像。

2. 用户程序的参数？
在 rCore-Tutorial/user/src/lib.rs 可以看到rCore-Tutorial里面为用户程序提供了简单的运行时。
```Rust
#[no_mangle]
pub extern "C" fn _start(_args: isize, _argv: *const u8) -> ! {
    unsafe {
        HEAP.lock()
            .init(HEAP_SPACE.as_ptr() as usize, USER_HEAP_SIZE);
    }
    sys_exit(main())
}
```
在进入到用户程序的main函数之后a0，a1寄存器应分别存放着_args, _argv。在Context::new函数里面的确实现了用户程序的参数设置。但是还是有一些问题，_argv是一个指针。因此光设置寄存是不够的，用户程序的初始化还需要在栈上放置相应的参数字符串。这一部分是教程里面没有实现的。

3. Lazy load
一般来说加载用户程序的时候并不是一下自全部加载到内存里的。而是在加载进程的时候保存虚拟page与可执行程序文件在磁盘上所在的block映射关系。第一次访问程序的这个地址的时候会产生page fault。此时操作系统会把这一页从磁盘fetch到内存中，把页表中相应虚拟地址项的valid bit设为true，重新执行产生exception的指令。这样做的好处有两个：可以减少程序加载时间；减少内存使用。教程里没有这么做而是eager load的原因是因为需要实现页置换算法，也就是把暂时不用的page放置在磁盘上。