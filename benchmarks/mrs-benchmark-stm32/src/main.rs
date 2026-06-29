#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m::peripheral::DWT;
use cortex_m_rt::entry;
use rtt_target::{rprintln, rtt_init_print};

use mrs_benchmark_core::{BenchmarkPlatform, SimpleAdd};

pub struct Stm32Platform {
    _dwt: DWT,
}

impl Stm32Platform {
    pub fn new(dwt: DWT) -> Self {
        Self { _dwt: dwt }
    }
}

impl BenchmarkPlatform for Stm32Platform {
    type Instant = u32;

    fn setup(&mut self) {
        unsafe {
            let mut p = cortex_m::Peripherals::steal();
            p.DCB.enable_trace();
            p.DWT.enable_cycle_counter();
        }
        rprintln!("[Platform] STM32 DWT cycle counter enabled.");
    }

    fn now(&self) -> Self::Instant {
        DWT::cycle_count()
    }

    fn elapsed(&self, start: Self::Instant) -> u64 {
        let now = DWT::cycle_count();
        now.wrapping_sub(start) as u64
    }

    fn unit(&self) -> &'static str {
        "cycles"
    }
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Initializing STM32 Microbenchmarks...");

    let cp = cortex_m::Peripherals::take().unwrap();
    let mut platform = Stm32Platform::new(cp.DWT);
    platform.setup();

    let elapsed = platform.run(&SimpleAdd, (1, 1));

    rprintln!(
        "STM32 Benchmark: SimpleAdd took {} {}",
        elapsed,
        platform.unit()
    );

    rprintln!("STM32 benchmarks finished. Halting.");
    loop {}
}
