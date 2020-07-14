# Lab 1 学习记录

## 问题
1. `interrupt.asm`直接复制黏贴导致的store fault
因为interrupt.S大部分代码其实是在保存通用寄存器，所以不是很想一行一行的敲了，把rCore-Tutorial里面的interrupt.asm直接复制黏贴过来了。结果运行的时候遇到了store exception，看来是往禁止的地方写了东西。仔细检查发现, 在保存context之前把sp和sscratch做了交换，sp指向了后面才会实现的内核栈，因此产生了exception。后面按照洛佳同学提出的方法把30个SAVE合成了一个loop，干净很多。

## 思考
1. interrupt概念辨析
文档中关于中断（interrupt），异常（exception），陷入（trap）的定义和分类与自己上OS课的时候不太一样。在个人看来，interrupt作为这三个总称不太合适，更贴切的可能是context switch（上下文转换）。因为无论是触发中断、异常、还是陷入，cpu都会保存当前寄存状态，**提升运行权限转换模式**，比如从user mode转到supervisor mode。而当我们提到中断的时候一般都是timer interrupt，dma interrupt，默认都是硬件引起的。同时因为是外部硬件引起的变化，interrupt还是asynchronous的。硬件中断可能在任何时候发生，比如说一条指令执行到一半，这种情况下就要由cpu来保证每条指令的atomicity。Exception和Trap可以分为另一类，他们都是由软件（具体某一条指令）造成的，只不过前者是**被动**的，后者是**主动**的。Exception和Trap只能在指令间产生，因此是synchronous的。
- Context switch
    - interrupt (Caused by hardware, async)
    - exception (Caused by software, sync)
        - Exception (Involuntarily yield to OS)
        - Trap (Voluntarily yield to OS)

2. 既然trap是主动的，那么必定只有一部分指令才能产生trap，RISC-V中这些指令都是什么呢？
在RISC-V Privileged Instruction Set Listings里面找了一下，应该只有下面这两条。
- `ecall`
- `ebreak`

3. 上下文转换中如果先保存寄存器，后修改sp会怎么样？
为什么要先对对sp做处理，31个通用寄存器整整齐齐的SAVE难道不好吗？从lab 0中entry.S可知，我们现在所谓的栈其实只是内核镜像后面紧挨着的一篇内存空间。由于没有开启虚拟内存，所以直接往栈指针sp下面写其实问题不大。实际运行中这样子写也能正确输出时钟中断的结果 `100 ticks, 200 ticks...`。但是这么做存在这一定的风险。注意到上下文转化中并没有禁用中断，也就是说存在中断套娃（nested interrupt）的可能性。此时如果是往栈上写，后降低栈指针的话，前面一个中断保存的数据会被面一个覆盖掉，导致程序运行出错。

4. `breakpoint`函数中`context.sepc += 2`引发的思考
这里不加2的话会进入无限trap的死循环，但是为什么返回地址是加2而不是加4没有做出解释。还是之前的提到的原因，riscv64imac，
c extension可能对ebreak做了压缩。这样的话跨平台可能会有点问题，riscv64ima上就跑不起来了。同为trap指令的ecall执行完之后是否也需要context.sepc += 2? Exception处理完之后是重新执行当前指令（sepc不变）还是跳过当前指令（sepc += 2/4）？如何获取当前指令的长度？

5. `stvec`的Direct和Vectored模式
Rust的Match语句如果枚举项足够多的，分布足够密的话最后都会编译成一个jump table。就现在的实现而言，Direct和Vectored模式区别不是很大，都是根据scause寄存器的值在lookup table里面找到对应的中断处理函数，然后跳转到那里。只不过Direct 模式是软件实现的，而Vectored则是硬件实现的，稍微麻烦一些，需要考虑alignment。

## 改进
1. 把上下文转换中统用寄存器的保存和读取用gnu assembler的宏合成了一个loop，简洁了很多
```
```RISC-V
# Essential for substitution %i
.altmacro

# length of general purpose registers in bytes
.set reg_size, 8
# No. of registers inside a context frame
.set context_size, 34

# Load register relative to sp
.macro load reg, offset
    ld  \reg, \offset * reg_size(sp)
.endm

.macro load_gp n
    load    x\n, \n
.endm

...

.global __restore
__restore:
    # Restore csr registers
    load    t0, 32
    load    t1, 33
    csrw    sstatus, t0
    csrw    sepc, t1

    .set i, 3
    .rept 29
        load_gp %i
        .set i, i + 1
    .endr

    # Restore ra and sp last
    load    x1, 1
    load    x2, 2

    # Return to the address stored in sepc
    sret
```