# Essential for substitution %i
.altmacro

# length of general purpose registers in bytes
.set reg_size, 8
# No. of registers inside a context frame
.set context_size, 34

# Save register relative to sp
.macro save reg, offset
    sd  \reg, \offset * reg_size(sp)
.endm

# Load register relative to sp
.macro load reg, offset
    ld  \reg, \offset * reg_size(sp)
.endm

.macro save_gp n
    save    x\n, \n
.endm

.macro load_gp n
    load    x\n, \n
.endm

.section .text
.global __interrupt
__interrupt:
    # Allocate stack space
    addi    sp, sp, -context_size * reg_size

    # This loop actually saves the wrong stack pointer
    .set i, 1
    .rept 31
        save_gp %i
        .set i, i + 1
    .endr

    # Correct sp
    addi    t0, sp, context_size * reg_size
    save    t0, 2

    # Save csr registers
    # Note that you cannot do
    # save sstatus, 32
    csrr    t0, sstatus
    csrr    t1, sepc
    save    t0, 32
    save    t1, 33

    # Setup parameters for handle_interrupt function
    # context: &mut Context
    mv      a0, sp
    # scause: Scause
    csrr    a1, scause
    # stval: usize
    csrr    a2, stval
    jal handle_interrupt

    # Fall through to __restore

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