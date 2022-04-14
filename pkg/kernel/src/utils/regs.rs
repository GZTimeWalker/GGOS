use core::fmt;
use core::ops::Deref;
use volatile::Volatile;

#[repr(align(8), C)]
#[derive(Debug, Clone, Default, Copy)]
pub struct RegistersValue {
    r15: usize,
    r14: usize,
    r13: usize,
    r12: usize,
    r11: usize,
    r10: usize,
    r9: usize,
    r8: usize,
    rdi: usize,
    rsi: usize,
    rdx: usize,
    rcx: usize,
    rbx: usize,
    rax: usize,
    rbp: usize
}

#[repr(C)]
pub struct  Registers {
    value: RegistersValue
}

impl Registers {
    #[inline]
    pub unsafe fn as_mut(&mut self) -> Volatile<&mut RegistersValue> {
        Volatile::new(&mut self.value)
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
