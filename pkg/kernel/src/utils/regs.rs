use core::fmt;
use core::ops::Deref;
use volatile::Volatile;

#[repr(align(8), C)]
#[derive(Clone, Default, Copy)]
pub struct RegistersValue {
    pub r15: usize,
    pub r14: usize,
    pub r13: usize,
    pub r12: usize,
    pub r11: usize,
    pub r10: usize,
    pub r9: usize,
    pub r8: usize,
    pub rdi: usize,
    pub rsi: usize,
    pub rdx: usize,
    pub rcx: usize,
    pub rbx: usize,
    pub rax: usize,
    pub rbp: usize
}

#[repr(C)]
pub struct Registers {
    value: RegistersValue
}

impl Registers {
    #[inline]
    pub unsafe fn as_mut(&mut self) -> Volatile<&mut RegistersValue> {
        Volatile::new(&mut self.value)
    }

    #[inline]
    pub unsafe fn set_rax(&mut self, value: usize) {
        self.value.rax = value;
    }
}

impl Deref for Registers {
    type Target = RegistersValue;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl fmt::Debug for Registers {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl fmt::Debug for RegistersValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Registers {{\n")?;
        write!(f, "    r15: 0x{:016x}, ", self.r15)?;
        write!(f, "    r14: 0x{:016x}, ", self.r14)?;
        write!(f, "    r13: 0x{:016x}, ", self.r13)?;
        write!(f, "    r12: 0x{:016x},\n", self.r12)?;
        write!(f, "    r11: 0x{:016x}, ", self.r11)?;
        write!(f, "    r10: 0x{:016x}, ", self.r10)?;
        write!(f, "    r9 : 0x{:016x}, ", self.r9)?;
        write!(f, "    r8 : 0x{:016x},\n", self.r8)?;
        write!(f, "    rdi: 0x{:016x}, ", self.rdi)?;
        write!(f, "    rsi: 0x{:016x}, ", self.rsi)?;
        write!(f, "    rdx: 0x{:016x}, ", self.rdx)?;
        write!(f, "    rcx: 0x{:016x},\n", self.rcx)?;
        write!(f, "    rbx: 0x{:016x}, ", self.rbx)?;
        write!(f, "    rax: 0x{:016x}, ", self.rax)?;
        write!(f, "    rbp: 0x{:016x}\n", self.rbp)?;
        write!(f, "}}")?;
        Ok(())
    }
}


#[macro_export]
macro_rules! as_handler {
    ($fn: ident) => {
        paste::item! {
            #[naked]
            pub extern "x86-interrupt" fn [<$fn _handler>](_sf: InterruptStackFrame) {
                unsafe {
                    core::arch::asm!("
                    push rbp
                    push rax
                    push rbx
                    push rcx
                    push rdx
                    push rsi
                    push rdi
                    push r8
                    push r9
                    push r10
                    push r11
                    push r12
                    push r13
                    push r14
                    push r15
                    call {}
                    pop r15
                    pop r14
                    pop r13
                    pop r12
                    pop r11
                    pop r10
                    pop r9
                    pop r8
                    pop rdi
                    pop rsi
                    pop rdx
                    pop rcx
                    pop rbx
                    pop rax
                    pop rbp
                    iretq",
                    sym $fn, options(noreturn));
                }
            }
        }
    }
}
