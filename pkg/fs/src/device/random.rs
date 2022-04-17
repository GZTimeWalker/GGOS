use x86_64::instructions::random::RdRand;
use super::*;

#[derive(Debug, Clone)]
pub struct Random;

impl Device for Random {
    fn read(&mut self, buf: &mut [u8], offset: usize, size: usize) -> Result<usize, BlockError> {
        if let Some(rng) = RdRand::new() {
            for i in 0..size {
                if let Some(num) = rng.get_u16() {
                    buf[i] = num as u8;
                } else {
                    return Err(BlockError::Unknown);
                }
            }
            Ok(size)
        } else {
            Err(BlockError::Unknown)
        }
    }

    fn write(&mut self, _buf: &[u8], _offset: usize, _size: usize) -> Result<usize, BlockError> {
        Err(BlockError::InvalidOperation)
    }
}

rand!(u64);
rand!(u32);
rand!(u16);

macro_rules! rand {
    ($ty:ty) => {
        paste::item! {
            pub fn [<rand_ $ty>]() -> $ty {
                if let Some(rdrand) = RdRand::new() {
                    if let Some(rand) = rdrand.get_$ty() {
                        return rand;
                    }
                }
                0
            }
        }
    }
}
