use x86_64::instructions::random::RdRand;
use super::*;

#[derive(Debug, Clone)]
pub struct Random;

impl Device<u8> for Random {
    fn read(&self, buf: &mut [u8], offset: usize, size: usize) -> Result<usize, DeviceError> {
        if let Some(rng) = RdRand::new() {
            for i in 0..size {
                if let Some(num) = rng.get_u16() {
                    buf[offset + i] = num as u8;
                } else {
                    return Err(DeviceError::Unknown);
                }
            }
            Ok(size)
        } else {
            Err(DeviceError::Unknown)
        }
    }

    fn write(&mut self, _: &[u8], _: usize, _: usize) -> Result<usize, DeviceError> {
        Ok(0)
    }
}

macro_rules! rand {
    ($ty:ty) => {
        paste::item! {
            pub fn [<rand_ $ty>]() -> $ty {
                if let Some(rdrand) = RdRand::new() {
                    if let Some(rand) = rdrand.[<get_ $ty>]() {
                        return rand;
                    }
                }
                0
            }
        }
    }
}

impl Random {
    pub fn new() -> Self {
        Self
    }

    rand!(u64);
    rand!(u32);
    rand!(u16);
}
