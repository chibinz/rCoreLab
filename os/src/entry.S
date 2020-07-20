# la translates into auipc, addi.
# That mean's you'll still be around 0x80000000.
# lui, on the other hand, uses absolute addressing
# Load absolute address of symbol into register
.macro la_abs rd, symbol
    lui \rd, %hi(\symbol)
    addi \rd, \rd, %lo(\symbol)
.endm

.section .text.entry
.global _start
_start:
    # Obtain physical page number of boot_page_table
    la t0, boot_page_table
    srli t0, t0, 12
    # Set mode Sv39
    li t1, (8 << 60)
    or t0, t0, t1
    # Write to satp and flush TLB
    csrw satp, t0
    sfence.vma

    # Load VIRTUAL address of boot_stack_top and rust_main
    la_abs sp, boot_stack_top
    la_abs t0, rust_main

    jr t0

.section .bss.stack
.global boot_stack
boot_stack:
    .zero 4096 * 16
    .global boot_stack_top
boot_stack_top:

# Scrap boot table for bootstrapping the kernel
.section .data
.align 12
boot_page_table:
    # Virtual mapping for NULL
    .8byte 0
    .8byte 0
    # 0x8000_0000 -> 0x8000_0000
    # 0xcf sets VRWXAD bits
    .8byte (0x80000 << 10) | 0xcf
    .zero 505 * 8
    # Device tree virtual memory map
    .8byte (0x00000 << 10) | 0xcf
    .8byte 0
    # 510th entry
    .8byte (0x80000 << 10) | 0xcf
    .8byte 0
