# Lab 0 学习记录
说实话，rCore-Tutorial里面基本上每一步都写得很详细了。怎么做，为什么这么做，基本上都是spoonfed到嘴边了。由于此次rCoreSummerOfCode不是一门课程，而是自己为了第二阶段学习做准备，因此我就不从rCore-Tutorial大段大段的复制代码和文字过来了。主要是记录一下自己做实验过程中遇到的一些问题，怎么解决的、对tutorial中作出指示没有细讲原因的思考和拓展、以及尝试对tutorial的代码做一些改动或改进，毕竟仓库里的代码是直接能跑的，复制黏贴过来没有太大意义。

## 问题
1. Dependency缺失
操作系统用的是Manjaro，而网上给出的教程一般都是基于Ubuntu的。直接
```bash
sudo pacman -S qemu
```
会提示`qemu-system-riscv64`并没有安装，仔细检查了一下发现只装了x86相关的，还需要
```bash
sudo pacman -S qemu-arch-extra
```

另外一位同学告诉我按照教程来就可以运行了，可能是因为个人环境配置不对或者版本太老了。
之后再make run的时候被告知找不到rust-objcopy，发现是没有装cargo-binutil,可以用以下命令解决
```bash
cargo install cargo-binutils
rustup component add llvm-tools-preview
```
群里面也有人指出了这一个问题，现在已经被添加到tutorial里面了。

## 思考
1. 关于一开始.text段只有4个字节
```
Idx Name            Size     VMA              Type
  0                 00000000 0000000000000000
  1 .text           00000004 0000000000011120 TEXT
```
当时看到.text段只有4个字节的时候很惊讶，刚才明明不是生成了`j 2; j 0`两条指令吗？RISC-V作为一种精简指令集，每条指令不是应该定长32位的吗？所以2 * 4应该是8个字节。想了想target triple里面是`riscv64imac`，开了C扩展，像无条件跳转这种常用的指令会被压缩到16位，缩小binary的大小同时提升instruction cache的性能。

2. 关于为什么要做objcopy
Tutorial里面其实讲的已经蛮详细的了，这里谈谈自己对free standing binary的理解。一个程序在硬盘上和内存中的储存方式是不同的，通常来说可执行文件更紧凑一些，而在内存中运行的程序松散一些，是被拉伸过的。比如ELF中bss段只存大小和起始地址，不存数据，因为在文件中放一大串0是没有意义的。把可执行文件加载到内存中的过程是由操作系统完成的，bss段会分配已经置零闲置的内存页。这通常没有什么问题，但是这一次是我们自己写操作系统，因此没有段的概念。kernel文件本来二进制表示什么样在bootloader(qemu)装载到内存中就是什么样。这也是为什么kernel.bin被叫做镜像。

对于与entry.S链接生成的os elf文件做objdump
```
```
以及objcopy出来的kernel.bin做hexdump，
```
```
发现两者有惊人的相似之处。原本entry.S里面只有两条指令，为什么反汇编出来有4条呢？这里做了一些注释。

3. 如果不做objcopy会怎么样
ELF文件前四个字节是.ELF这四个字符ascii构成的magic byte(注意endianness)。如果不做任何处理就丢给qemu运行的话，这四个字符(data)会被当成代码(code)执行，有很大的可能会引起undefined instruction exception。

## 改进
1. `dbg!` macro
之前提交了一个dbg! macro的pull request，现在已经被接受了。dbg！是自己平时rust编程中常用到的一个宏，简单来说就是print打法，但是更强大方便。往console输出变量值的同时显示文件名与行号并返回变量的值。同时自定义的结构体可以derive(Debug)，用dbg！显示出来也很清晰。代码大概长这样
```Rust
#[macro_export]
#[allow(unused_macros)]
macro_rules! dbgx {
    () => {
        println!("[{}:{}]", file!(), line!());
    };
    ($val:expr) => {
        match $val {
            tmp => {
                println!("[{}:{}] {} = {:#x?}",
                    file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($val:expr,) => { dbgx!($val) };
    ($($val:expr),+ $(,)?) => {
        ($(dbgx!($val)),+,)
    };
}
```

2. Enum for SBI call number
不是很喜欢C风格的纯大写字常量，改写成了enum，每次需要读写值的时候转成usize。
```Rust
enum CallNum {
    SetTimer = 0,
    ConsolePutChar = 1,
    ConsoleGetChar = 2,
    ClearIPI = 3,
    SendIPI = 4,
    RemoteFenceI = 5,
    RemoteSFenceVMA = 6,
    RemoteSFenceVMAASID = 7,
    Shutdown = 8,
}
```
