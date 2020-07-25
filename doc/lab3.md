# Lab 3 学习记录

## 实验题
### 原理：在 os/src/entry.asm 中，boot_page_table 的意义是什么？当跳转执行 rust_main 时，不考虑缓存，硬件通过哪些地址找到了 rust_main 的第一条指令？
- bootstrapping. 按照`linker.ld`里面`BASE_ADDRESS`的定义，所有操作系统的符号都位于高地址，也就是0xffff开头的地址。但OpenSBI只负责把pc放到0x80200000这个地址，虚拟内存的开启和页表的加载需要操作系统来完成。为了能够让操作系统正常启动，我们把把页表硬编码进代码里面，boot_page_table充当了操作系统尚未初始化完成之前临时页表的用途。
- 跳转到`rust_main`第一条指令之前就已经开启了虚拟内存。因此rust_main的虚拟地址会经过翻译得到物理地址。cpu首先会查看satp寄存器里面的根页表地址。之后取rust_main虚拟地址的38~30位作为页表的索引查询页表项，这里应该是510项。查询之后发现第510项V位为1（有效），RWX 3位的值都为1（不全是0），所以这一项对应的是大小为1G的大虚拟页。把这一项映射的物理地址0x80000000,加上虚拟地址里面的偏移得到rust_main的物理地址，从物理地址加载第一条指令。

### 分析：为什么 Mapping 中的 page_tables 和 mapped_pairs 都保存了一些 FrameTracker？二者有何不同？
题目的更新可能没有赶上代码，这里应该说的是MemorySet里面的mapping和allocated_pairs。仅看MemorySet的new_kernel,add_segment或者remove_segment会发现两者的添加和删除是同步的, 好像没有什么区别……
```Rust
self.allocated_pairs
    .extend(self.mapping.map(&segment, init_data)?);
```
但实际上Mapping内部的操作要复杂得多。首先Mapping::new会存下root_table根页表，这个是allocated_pairs里面没有的。其次，Mapping::map里面对MapType::Linear类型映射的添加返回值是Vec::new，也就是内核镜像的那一块地址的映射会存在页表里，但是不会添加到allocated_pairs里面。再次，对于MapType::Framed类型的映射会依次调用map->map_one->find_entry。
```Rust
for vpn_slice in &vpn.levels()[1..] {
    if entry.is_empty() {
        ...
        self.page_tables.push(new_table);
    }
    ...
}
```
这个循环会往page_tables里面添加第一二级页表，但返回的时候直会返回第三级页表。一二级页表不会存在allocated_pairs里面。
综上所述，allocated_pairs保存的映射只是mapping的一个子集，仅保存进程自己分配的那一部分映射，像线程的栈等。而Mapping存的东西就比较多，包括了内核映射的部分还有页表占用的页部分等。

### 分析：假设某进程需要虚拟地址 A 到物理地址 B 的映射，这需要操作系统来完成。那么操作系统在建立映射时有没有访问 B？如果有，它是怎么在还没有映射的情况下访问 B 的呢？
- 建立映射只需要在页表中添加对应项即可，无需访问B那一块物理地址
- 没有映射一般先会在页表里面添加映射在进行访问。如果硬是要在没有映射的条件下访问物理地址的话可以参考Mapping::map里面对init_data的操作。通过线性偏移对物理内存进行访问。
```Rust
dst_slice.copy_from_slice(src_slice);
```
dst_slice的类型是&mut FrameTracker，是没有copy_from_slice的方法的。魔法发生在`.`上，Rust会自动解引用调用DerefMut。 DerefMut方法里调用了deref_kernel，会把FrameTracker所存的物理地址根据线性偏移得到虚拟地址，之后返回一个u8的数组。u8的数组会调用copy_from_slice，这个是core里面实现的，类似于c标准库的memcpy函数，完成原始数据拷贝。


## 问题
1. 用la替代lui加载地址
在修改`entry.S`的过程中，没有多想就把
```
lui sp, %hi(boot_stack_top)
addi sp, sp, %lo(boot_stack_top)
```
替换成了
```
la sp, boot_stack_top
```
这样子直观一些，但是在main里面添加这么一句`dbgx!(rust_main as usize);`发现并不在高地址，还是运行在直接映射到物理地址0x8000_0000->0x8000_0000那一块虚拟内存上。查阅RISC-V相关文档之后发现，la用的是relative addressing，assembler其实会翻译成两条指令。
```
auipc rd, %pcrel_hi(symbol)
addi  rd, rd, %pcrel_hi(symbol)
```
此时的pc还是指向物理地址，也就是0x80200000附近，所以跳转的地址也是低地址，相当于没有用上0xffff_ffff_8000_0000-> 0x8000_0000的映射。
- la
```
[src/main.rs:26] rust_main as usize = 0x80201ea8
panic: 'attempt to subtract with overflow'
```
- la_abs
```
[src/main.rs:26] rust_main as usize = 0xffffffff80201ea8
map Range { start: VirtualAddress(ffffffff80200000), end: VirtualAddress(ffffffff80211000) }
map Range { start: VirtualAddress(ffffffff80211000), end: VirtualAddress(ffffffff80216000) }
map Range { start: VirtualAddress(ffffffff80216000), end: VirtualAddress(ffffffff80217000) }
map Range { start: VirtualAddress(ffffffff80217000), end: VirtualAddress(ffffffff80a28000) }
map Range { start: VirtualAddress(ffffffff80a28000), end: VirtualAddress(ffffffff88000000) }
```
2. 实验中PageTableTracker::new()函数没有看懂……
```Rust
pub fn new(frame: FrameTracker) -> Self {
    let mut page_table = Self(frame);
    page_table.zero_init();
    page_table
}
```
这边的变量名很有迷惑性，page_table的类型其实是PageTableTracker。然而仔细看PageTableTracker的impl，发现里面并没有zero_init()这个函数。全局搜了一下，在PageTable结构体的成员函数里面找到了它。PageTableTracker包裹的是FrameTracker而不是PageTable。不太清楚为什么这个东西为什么能够通过编译。仔细看了一下文档，发现有这么一句话。
> 这个 PageTableTracker 和 PageTableEntry 也通过一些 Rust 中的自动解引用的特性为后面的实现铺平了道路，比如我们可以直接把 PageTableTracker 当成 PageTable 对待
```Rust
impl core::ops::Deref for PageTableTracker {}
impl core::ops::DerefMut for PageTableTracker {}
```
实现Deref和DerefMut这两个trait之后，竟然神奇的可以通过`.`自动解引用调用FrameTracker所存储PageTable的方法！

## 思考
1. 为什么0xffff_ffff_8000_0000放在页表的第510项？
取虚拟地址的38~30位即为三级页表VPN[2]的索引。
(0xffff_ffff_8000_0000 >> 30) & 0x1ff = 0x1fe = 510

2. 测试page fault
在开启虚拟内存的情况下，如果访问了valid bit为0的页的话会产生一个page fault。控制流应该会跳转到interrupt模块里面handle_interrupt函数。

3. VirtualAddress, VirtualPageNumber, PhysicalAddress, PhysicalPageNumber 4者的关系与互相转换。
- Address >> 12 = PageNumber，虚拟和物理高位的有效位数不同。
- VirtualPageNumber -> PhysicalPageNumber: kernel的page可以通过线性映射(直接调用PhysicalPageNumber::from()), 用户程序的则只能通过page table lookup。
- VirtualAddress -> PhysicalAddress：
先按照上一项的办法获得对应的物理页号，再offset复制粘贴过去就行了。
- PhysicalPageNumber -> VirtualPageNumber：
因为page table是从虚拟地址到物理地址的映射，所以一般给出物理页号很难知道虚拟页号是什么。对于kernel的物理地址并不麻烦，因为是线性映射所以直接加上对应的偏移量（KERNEL_MAP_OFFSET）就行了（VirtualPageNumber::from<PhysicalPageNumber>()）。对于用户程序比较麻烦，如果一定要知道的话需要手动page walk，遍历一遍。

4. deref_kernel
本来很好奇页表这种需要对指针（地址）直接做操作的数据结构为什么tutorial可以封装的这么好，基本上没有看到unsafe。高层级的表指向底层级的表，Mapping的page_tables里面也有正在使用的表的引用，Ownership应该挺复杂的……结果发现所有的unsafe都被包装在VirtualAddress的deref函数里面了。这是一个泛型的解引用函数，rust的type inference做的很好，所以不需要像C那样把void *原始指针转成某种具体类型的指针再解引用。说实话这个函数前面应该加一个unsafe标签，用起来小心一点。

5. find_entry
这个函数的叫做find_or_insert_entry可能更贴切一些，找不到自动添加对应的虚拟页。

6. Sv39中为什么物理地址空间是56比特而不是64比特？
虚拟地址只有39位可以理解，这样可以减少页表占用空间，压缩页表层级，加快虚拟地址的翻译。但物理地址为什么只有56位？
RISC-V Privileged ISA提到物理地址的最高10位是reserved，给出来的理由是：
> We reserved several PTE bits for a possible extension that improves support for sparse address spaces by allowing page-table levels to be skipped, reducing memory usage and TLB refill latency. These reserved bits may also be used to facilitate research experimentation.  The cost is reducing the physical address space, but 64 PiB is presently ample.  When it no longer suffices, the reserved bits that remain unallocated could be used to expand the physical address space.
大意就是说空出来的这10个bit可以用来做一些优化，而且56比特对应的64PiB容量其实已经蛮大的了，如果未来物理内存空间不够用的话也可以把这10位扩充为物理地址。

7. Segment
为了知道那一段从哪里开始到哪里结束，rCore把各个段的起止地址和可读写执行信息硬编码进了memory set模块里面，这个其实就是elf header的所保存的信息。现在之所以kernel是raw binary而不是ELF的原因在于kernel的.text段必须在最前面，0x80200000对应操作系统的入口。如果能够把ELF header放在后面的话，可以写一个ELF parser，这样就不用硬编码每段信息，同时用户程序中可以复用。

## 改进 TODO
1. 现在VirtualPageNumber是通过levels()获取对应层级页表索引。如果实现Index Trait的话可以更优雅一些。
2. 把所有symbol放到同一个文件里面，方便找。
3. 写一个which_segment函数，给出page_table和虚拟地址，返回这个地址所在的segment。
