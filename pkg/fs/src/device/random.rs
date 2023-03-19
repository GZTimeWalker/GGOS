use super::*;
use rand::{RngCore, SeedableRng};
use rand_hc::Hc128Rng;
use x86_64::instructions::random::RdRand;

pub static GLOBAL_RNG: spin::Once<spin::Mutex<Hc128Rng>> = spin::Once::new();

#[derive(Debug, Clone)]
pub struct Random;

impl Device<u8> for Random {
    fn read(&self, buf: &mut [u8], offset: usize, size: usize) -> Result<usize, DeviceError> {
        if let Some(rng) = RdRand::new() {
            for i in 0..size {
                if let Some(num) = rng.get_u16() {
                    buf[offset + i] = num as u8;
                } else {
                    return Err(DeviceError::ReadError);
                }
            }
            Ok(size)
        } else if let Some(mut rng) = GLOBAL_RNG.get().and_then(spin::Mutex::try_lock) {
            for i in 0..size {
                buf[offset + i] = rng.next_u32() as u8;
            }
            Ok(size)
        } else {
            Err(DeviceError::ReadError)
        }
    }

    fn write(&mut self, _: &[u8], _: usize, _: usize) -> Result<usize, DeviceError> {
        Ok(0)
    }
}

impl Random {
    pub fn new(seed: u64) -> Self {
        if !GLOBAL_RNG.is_completed() {
            GLOBAL_RNG.call_once(|| spin::Mutex::new(Hc128Rng::seed_from_u64(seed)));
        }
        Self
    }
}
