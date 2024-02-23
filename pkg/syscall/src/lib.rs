#![no_std]

use num_enum::FromPrimitive;

pub mod macros;

#[repr(u16)]
#[derive(Clone, Debug, FromPrimitive)]
pub enum Syscall {
    Read = 0,
    Write = 1,
    Open = 2,
    Close = 3,

    GetPid = 39,

    VFork = 58,
    Spawn = 59,
    Exit = 60,
    WaitPid = 61,
    Kill = 62,

    Sem = 66,
    Time = 201,

    Stat = 65530,
    ListDir = 65531,
    Draw = 65532,
    Allocate = 65533,
    Deallocate = 65534,

    #[num_enum(default)]
    None = 65535,
}
