#![allow(non_upper_case_globals, dead_code)]
// This is from https://github.com/rcore-os/rCore/blob/master/kernel/src/arch/x86_64/interrupt/consts.rs
// Reference: https://wiki.osdev.org/Exceptions

#[repr(u8)]
pub enum Interrupts {
    DivideError = 0,
    Debug = 1,
    NonMaskableInterrupt = 2,
    Breakpoint = 3,
    Overflow = 4,
    BoundRangeExceeded = 5,
    InvalidOpcode = 6,
    DeviceNotAvailable = 7,
    DoubleFault = 8,
    CoprocessorSegmentOverrun = 9,
    InvalidTSS = 10,
    SegmentNotPresent = 11,
    StackSegmentFault = 12,
    GeneralProtectionFault = 13,
    PageFault = 14,
    FloatingPointException = 16,
    AlignmentCheck = 17,
    MachineCheck = 18,
    SIMDFloatingPointException = 19,
    VirtualizationException = 20,
    SecurityException = 30,

    IRQ0 = 32,
    Syscall = 0x80,
}

#[repr(u8)]
pub enum IRQ {
    Timer = 0,
    Keyboard = 1,
    COM2 = 3,
    COM1 = 4,
    IDE = 14,
    Error = 19,
    Spurious = 31,
}

#[repr(u8)]
pub enum SyscallInt {
    Exit = 0x00,
    IO = 0x01,
    Misc = 0x02,
}
