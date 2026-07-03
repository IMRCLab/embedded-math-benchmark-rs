#![no_std]
#![no_main]

use cortex_m::peripheral::{syst::SystClkSource, SYST};
use cortex_m_rt::entry;
use cortex_m_semihosting::debug;
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};

// Pull in the Pico 1 BSP. The default SysTick timing path below doesn't call its
// HAL, but linking rp-pico is what provides the second-stage bootloader (boot2)
// that the RP2040 needs to boot from flash, plus the critical-section impl that
// rtt-target relies on. To measure with the 64-bit 1 µs `TIMER` instead, reach
// for `rp_pico::hal` (see the notes on the impl below).
use rp_pico as _;

use mrs_benchmark_core::BenchmarkPlatform;

pub struct Rp2040Platform {
    // Owned so the counter stays configured for the lifetime of the platform.
    _syst: SYST,
}

impl Rp2040Platform {
    pub fn new(mut syst: SYST) -> Self {
        syst.set_clock_source(SystClkSource::Core);
        syst.set_reload(0x00FF_FFFF); // 24-bit max: count over the full range
        syst.clear_current();
        syst.enable_counter();
        Self { _syst: syst }
    }
}

impl BenchmarkPlatform for Rp2040Platform {
    type Instant = u32;

    fn id(&self) -> &'static str {
        "rp2040"
    }

    fn setup(&mut self) {
        rprintln!("[Platform] RP2040 SysTick cycle counter enabled (24-bit, core clock).");
    }

    fn now(&self) -> Self::Instant {
        SYST::get_current()
    }

    fn elapsed(&self, start: Self::Instant) -> u64 {
        // SysTick counts DOWN and is 24-bit, so elapsed cycles = (start - now)
        // modulo 2^24. wrapping_sub + mask handles the reload wrap correctly.
        let now = SYST::get_current();
        (start.wrapping_sub(now) & 0x00FF_FFFF) as u64
    }

    fn unit(&self) -> &'static str {
        "cycles"
    }

    fn log_result(&self, library: &'static str, bench: &'static str, id: mrs_benchmark_core::ResultId, elapsed: u64) {
        match id {
            mrs_benchmark_core::ResultId::Input(i) => rprintln!("{}:{}:{}, {}, {}", self.id(), library, bench, i, elapsed),
            mrs_benchmark_core::ResultId::All => rprintln!("{}:{}:{}, all, {}", self.id(), library, bench, elapsed),
        }
    }
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Initializing RP2040 (Pico 1) Microbenchmarks...");

    // TODO(hardware, optional): to report absolute time (µs) instead of raw
    // cycles, configure the clocks/PLL here (e.g. rp_pico::hal::clocks::
    // init_clocks_and_plls) and switch the platform to rp_pico::hal::Timer. The
    // default SysTick path needs no clock setup — it counts core cycles directly.
    let cp = cortex_m::Peripherals::take().unwrap();
    let mut platform = Rp2040Platform::new(cp.SYST);
    platform.setup();

    mrs_benchmark_core::run_all_benchmarks(&mut platform);

    rprintln!("RP2040 benchmarks finished.");

    debug::exit(debug::EXIT_SUCCESS);
    // Unreachable while a debugger is attached
    #[allow(clippy::empty_loop)]
    loop {}
}
