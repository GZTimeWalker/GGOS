use crate::*;
use core::cmp::min;
use rand::{Rng, RngCore, SeedableRng};
use rand_hc::Hc128Rng;
use x86_64::instructions::random::RdRand;

pub static GLOBAL_RNG: spin::Once<spin::Mutex<Hc128Rng>> = spin::Once::new();

#[derive(Debug, Clone)]
pub struct Random;

impl Device<u8> for Random {
    fn read(&self, buf: &mut [u8], offset: usize, size: usize) -> Result<usize> {
        if let Some(rng) = RdRand::new() {
            for i in (0..size).step_by(8) {
                if let Some(num) = rng.get_u64() {
                    for j in (i..min(i + 8, size)).rev() {
                        buf[offset + j] = (num >> (j * 8)) as u8;
                    }
                } else {
                    return Err(DeviceError::ReadError.into());
                }
            }
            Ok(size)
        } else if let Some(mut rng) = GLOBAL_RNG.get().and_then(spin::Mutex::try_lock) {
            rng.fill(&mut buf[offset..offset + size]);
            Ok(size)
        } else {
            Err(DeviceError::ReadError.into())
        }
    }

    fn write(&mut self, _: &[u8], _: usize, _: usize) -> Result<usize> {
        Ok(0)
    }
}

impl Random {
    pub fn new() -> Self {
        if !GLOBAL_RNG.is_completed() {
            let seed = RdRand::new().and_then(|rng| rng.get_u64()).unwrap_or(0);
            GLOBAL_RNG.call_once(|| spin::Mutex::new(Hc128Rng::seed_from_u64(seed)));
        }
        Self
    }
}

impl Default for Random {
    fn default() -> Self {
        Self::new()
    }
}
