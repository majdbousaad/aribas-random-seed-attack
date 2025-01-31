use std::num::Wrapping;
use std::time::SystemTime;
use num_bigint::BigInt;
use num_traits::identities::Zero;
use crate::c_rand::{CRandomWindows, CRandomLinux};



pub enum Platform {
    Windows,
    Linux
}

pub struct AribasRandom {
    rr: u64,
    platform: Platform
}
impl AribasRandom {
    pub fn new_linux() -> Self {
        Self { rr: 0, platform: Platform::Linux }
    }

    pub fn new_windows() -> Self {
        Self { rr: 0, platform: Platform::Windows }
    }

    fn time() -> u64 {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        }
    }

    fn sysrand(&self) -> u32 {
        self.sysrand_timestamp(Self::time() as u32)
    }

    fn sysrand_timestamp(&self, timestamp: u32) -> u32 {
        match self.platform {
            Platform::Windows => {
                let mut r = CRandomWindows::new();
                r.srand(timestamp);
                r.rand()
            },
            Platform::Linux => {
                let mut r = CRandomLinux::new();
                r.srand(timestamp);
                r.rand()
            }
        }
    }

    fn sysrand_linux() -> u32 {
        Self::sysrand_timestamp_linux(Self::time() as u32)
    }

    fn sysrand_timestamp_linux(timestamp: u32) -> u32 {
        let mut r = CRandomLinux::new();
        r.srand(timestamp);
        r.rand()
    }

    fn set_nth_word(&mut self, n: u8, word: u16) {
        self.rr &= !(0xFFFFu64 << (n * 16));
        self.rr |= (word as u64) << (n * 16);
    }

    fn inirandstate(&mut self) {
        self.inirandstate_timestamp(Self::time() as u32)
    }

    pub fn print_state(&self) {
        println!("rr = {} {} {} {}", (self.rr >> (3 * 16)) & 0xFFFF, (self.rr >> (2 * 16)) & 0xFFFF, (self.rr >> (1 * 16)) & 0xFFFF, (self.rr >> (0 * 16)) & 0xFFFF);
    }

    fn inirandstate_timestamp(&mut self, timestamp: u32)
    {
        self.rr = 0;
        self.set_nth_word(1, self.sysrand_timestamp(timestamp) as u16);
        self.nextrand_3();
        self.set_nth_word(0, self.sysrand_timestamp(timestamp) as u16);
        self.nextrand_3();
        self.set_nth_word(3, 1);
    }

    fn nextrand_3(&mut self) {
        self.nextrand_mask(0xFFFF_FFFF_FFFFu64, 0xFFFF_FFFF_FFFF_FFFFu64, 48);
    }

    fn nextrand_2(&mut self) {
        self.nextrand_mask(0xFFFF_FFFFu64, 0xFFFF_FFFF_FFFFu64, 32);
    }

    fn nextrand_mask(&mut self, mask1: u64, mask2: u64, shift: u8) {
        let inc = Wrapping(57777u64);
        let scale = Wrapping(56857u64);
        let mut temp = Wrapping(self.rr);

        let a = Wrapping(temp.0 & mask1) + inc;
        if ((a.0 & !mask1) >> shift) == 0 {
            temp = Wrapping(temp.0 & !mask1) + a;
        } else {
            temp = Wrapping(temp.0 & !mask2) + a;
        }
// self.rr = temp.0; print!("temp1 = "); self.print_state();

        let a = Wrapping(temp.0 & mask1) * scale;
        if ((a.0 & !mask1) >> shift) == 0 {
            temp = Wrapping(temp.0 & !mask1) + a;
        } else {
            temp = Wrapping(temp.0 & !mask2) + a;
        }
// self.rr = temp.0; print!("temp2 = "); self.print_state();

        self.rr = temp.0;
        self.set_nth_word(3, 1);
    }

    pub fn random(&mut self, m: BigInt) -> BigInt {
        let mut result = BigInt::zero();
        let len = m.to_bytes_be().1.len();
        let len16 = (len + 1) / 2;
        if len <= 2 {
            self.nextrand_2();
            if m.is_zero() {
                return m;
            }
            return BigInt::from(((self.rr >> 16) & 0xFFFF) % m);
        }
        for i in (0..len16).step_by(2) {
            self.nextrand_3();
            let dword = ((self.rr >> 16) & 0xFFFF_FFFF) as u32;
            result |= BigInt::from(dword) << (i * 16);
        }
        result &= !(BigInt::from(0xFFFF) << (len * 8));  // somehow the leftmost 16 bit need to be cut away, idk
        result %= m;
        return result;
    }

    pub fn get_current_seed(&self) -> u64 {
        self.rr
    }

    pub fn random_seed(&mut self, seed: u64) -> u64 {
        self.rr = seed;
        self.set_nth_word(3, 1);
        self.rr
    }

    pub fn random_seed_by_timestamp(&mut self, timestamp: u32) -> u64 {
        self.inirandstate_timestamp(timestamp);
        self.rr
    }
}