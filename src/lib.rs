#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    TscNotSupported,
    InvariantTscNotSupported,
    CpuidLeafTscFailed,
    CpuidLeafFreqFailed,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Debug>::fmt(self, f)
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone, Copy)]
pub struct TSC {
    freq: u64,
}

impl TSC {
    pub fn new() -> Result<Self, Error> {
        let freq = Self::cpu_freq()?;
        Ok(Self { freq })
    }

    #[cfg(target_arch = "x86_64")]
    pub fn cpu_freq() -> Result<u64, Error> {
        use core::arch::x86_64::__cpuid;

        let res = unsafe { __cpuid(0x1) };
        if res.edx & (1 << 4) == 0 {
            return Err(Error::TscNotSupported);
        }
        let res = unsafe { __cpuid(0x80000007) };
        if res.edx & (1 << 8) == 0 {
            return Err(Error::InvariantTscNotSupported);
        }
        let res = unsafe { __cpuid(0x15) };
        if res.ebx == 0 || res.eax == 0 {
            return Err(Error::CpuidLeafTscFailed);
        }

        let freq = if res.ecx != 0 {
            (res.ecx as u64 * res.ebx as u64) / (res.eax as u64)
        } else {
            let res = unsafe { __cpuid(0x16) };
            if res.eax == 0 {
                return Err(Error::CpuidLeafFreqFailed);
            }
            res.eax as u64 * 1_000_000 /* MHZ */
        };
        Ok(freq)
    }

    #[cfg(target_arch = "aarch64")]
    pub fn cpu_freq() -> Result<u64, Error> {
        use std::arch::asm;
        let freq: u64;
        unsafe {
            asm!("mrs {}, cntfrq_el0", out(reg) freq);
        }
        Ok(freq)
    }

    #[inline(always)]
    #[cfg(target_arch = "x86_64")]
    pub fn read_tsc() -> u64 {
        use core::arch::x86_64::{_mm_lfence, _rdtsc};
        unsafe {
            _mm_lfence();
            _rdtsc()
        }
    }

    #[inline(always)]
    #[cfg(target_arch = "aarch64")]
    pub fn read_tsc() -> u64 {
        use std::arch::asm;
        let value: u64;
        unsafe {
            asm!("mrs {}, cntvct_el0", out(reg) value);
        }
        value
    }

    pub fn get_freq(&self) -> u64 {
        self.freq
    }

    pub fn now_f64(&self) -> f64 {
        Self::read_tsc() as f64 / self.freq as f64
    }

    #[inline(never)]
    pub fn now_ns(&self) -> u64 {
        let tsc = Self::read_tsc();
        let (secs, rem) = (tsc / self.freq, tsc % self.freq);
        secs * 1_000_000_000 + (rem * 1_000_000_000 / self.freq)
    }
}

#[test]
fn test_perf() {
    use std::time::{Duration, Instant};
    let t = TSC::new().unwrap();

    let t0 = Instant::now();
    let mut counter = 0;
    for _ in 0..100_000 {
        let tmp = Instant::now();
        counter += tmp.duration_since(t0).as_nanos();
    }
    let t1 = Instant::now();
    assert!(counter > 10);
    println!("{:?}", t1.duration_since(t0));

    let t0 = Instant::now();
    let mut counter = 0;
    for _ in 0..100_000 {
        let tmp = t.now_ns();
        counter += tmp >> 32;
    }
    let t1 = Instant::now();
    assert!(counter > 10);
    println!("{:?}", t1.duration_since(t0));
}

#[test]
fn test_skew() {
    use std::time::{Duration, Instant};
    let t = TSC::new().unwrap();
    println!("{t:?}");

    let t0 = Instant::now();
    let n0 = t.now_ns();
    std::thread::sleep(Duration::from_secs(60));
    let n1 = t.now_ns();
    let tsc_dt = n1 - n0;
    let clock_dt = Instant::now().duration_since(t0);

    println!("tsc={} clock={clock_dt:?}", tsc_dt as f64 / 1_000_000_000.0,);
}
