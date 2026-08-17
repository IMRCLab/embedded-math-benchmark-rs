#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};
use cortex_m::peripheral::{syst::SystClkSource, SYST};
use cortex_m_rt::{entry, exception};
use cortex_m_semihosting::debug;
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};

// Pull in the Pico 1 BSP. The SysTick timing path below doesn't call its HAL,
// but linking rp-pico is what provides the second-stage bootloader (boot2)
// that the RP2040 needs to boot from flash, plus the critical-section impl
// that rtt-target relies on.
use rp_pico as _;

use mrs_benchmark_core::BenchmarkPlatform;

// SysTick is 24-bit, so a bare `get_current()` reading alone can't tell a single
// wrap apart from N wraps once a measurement exceeds ~16.7M cycles (~134ms at
// 125MHz) - a prior version of this file computed elapsed as a plain mod-2^24
// difference, which silently under-reported any measurement that crossed that
// boundary (confirmed on real hardware: a task whose true cost was ~17.4M cycles
// was reported as ~670K, i.e. 17_400_000 % 2^24). TICKINT turns the reload into
// an exception so we can count wraps and extend the range to 56 bits.
const SYST_RELOAD: u32 = 0x00FF_FFFF;
const SYST_PERIOD: u64 = SYST_RELOAD as u64 + 1; // 2^24

static WRAPS: AtomicU32 = AtomicU32::new(0);

#[exception]
fn SysTick() {
    // ARMv6-M (Cortex-M0+) has no LDREX/STREX, so AtomicU32 only offers plain
    // load/store here, not fetch_add. A plain load-increment-store is fine: this
    // body can't be reentered (SysTick can't preempt itself), and every other
    // reader goes through read_ticks()'s interrupt-masked critical section.
    WRAPS.store(
        WRAPS.load(Ordering::Relaxed).wrapping_add(1),
        Ordering::Relaxed,
    );
}

pub struct Rp2040Platform {
    // Owned so the counter stays configured for the lifetime of the platform.
    _syst: SYST,
}

impl Rp2040Platform {
    pub fn new(mut syst: SYST) -> Self {
        syst.set_clock_source(SystClkSource::Core);
        syst.set_reload(SYST_RELOAD); // 24-bit max: count over the full range
        syst.clear_current();
        syst.enable_interrupt(); // fire on reload so multi-wrap spans stay accurate
        syst.enable_counter();
        Self { _syst: syst }
    }

    // Snapshots (wraps, current) together with interrupts masked so the pair is
    // never torn by a wrap landing between the two reads.
    fn read_ticks() -> (u32, u32) {
        cortex_m::interrupt::free(|_| (WRAPS.load(Ordering::Relaxed), SYST::get_current()))
    }
}

impl BenchmarkPlatform for Rp2040Platform {
    type Instant = (u32, u32);

    fn id(&self) -> &'static str {
        "rp2040"
    }

    fn setup(&mut self) {
        rprintln!(
            "[Platform] RP2040 SysTick cycle counter enabled (24-bit, core clock, wrap-extended)."
        );
    }

    fn now(&self) -> Self::Instant {
        Self::read_ticks()
    }

    fn elapsed(&self, start: Self::Instant) -> u64 {
        let (start_wraps, start_c) = start;
        let (end_wraps, end_c) = Self::read_ticks();
        // SysTick counts DOWN, so within a wrap segment elapsed = start_c - end_c;
        // each full wrap adds one more period. Signed math sidesteps the borrow.
        let wraps = end_wraps.wrapping_sub(start_wraps) as i64;
        let delta = wraps * SYST_PERIOD as i64 + (start_c as i64 - end_c as i64);
        delta as u64
    }

    fn unit(&self) -> &'static str {
        "cycles"
    }

    fn log_result<O: core::fmt::Debug>(
        &self,
        library: &'static str,
        bench: &'static str,
        input_index: usize,
        repetitions: u32,
        elapsed: u64,
        output: Option<&Result<O, mrs_benchmark_core::BenchmarkError>>,
    ) {
        if let Some(res) = output {
            match res {
                Ok(val) => rprintln!(
                    "BENCH {},{},{},{},{},{},{},{},\"{:?}\"",
                    self.id(),
                    mrs_benchmark_core::PROFILE,
                    library,
                    bench,
                    input_index,
                    repetitions,
                    elapsed,
                    self.unit(),
                    val
                ),
                Err(e) => rprintln!(
                    "BENCH {},{},{},{},{},{},{},{},\"ERROR: {:?}\"",
                    self.id(),
                    mrs_benchmark_core::PROFILE,
                    library,
                    bench,
                    input_index,
                    repetitions,
                    elapsed,
                    self.unit(),
                    e
                ),
            }
        } else {
            rprintln!(
                "BENCH {},{},{},{},{},{},{},{},\"\"",
                self.id(),
                mrs_benchmark_core::PROFILE,
                library,
                bench,
                input_index,
                repetitions,
                elapsed,
                self.unit()
            )
        }
    }
}

#[entry]
fn main() -> ! {
    rtt_init_print!(rtt_target::ChannelMode::BlockIfFull, 4096);
    rprintln!("Initializing RP2040 (Pico 1) Microbenchmarks...");

    // TODO(hardware, optional): to report absolute time (µs) instead of raw
    // cycles, configure the clocks/PLL here (e.g. rp_pico::hal::clocks::
    // init_clocks_and_plls) and switch the platform to rp_pico::hal::Timer. The
    // default SysTick path needs no clock setup — it counts core cycles directly.
    let dp = rp_pico::hal::pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut resets = dp.RESETS;
    let mut watchdog = rp_pico::hal::watchdog::Watchdog::new(dp.WATCHDOG);
    // Configure the system clock (clk_sys) to run at its nominal 125 MHz
    let _clocks = rp_pico::hal::clocks::init_clocks_and_plls(
        12_000_000,
        dp.XOSC,
        dp.CLOCKS,
        dp.PLL_SYS,
        dp.PLL_USB,
        &mut resets,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut platform = Rp2040Platform::new(cp.SYST);
    platform.setup();

    rprintln!("INFO {}", mrs_benchmark_core::CSV_HEADER);
    mrs_benchmark_core::run_all_benchmarks(&mut platform);

    rprintln!("RP2040 benchmarks finished.");

    debug::exit(debug::EXIT_SUCCESS);
    // Unreachable while a debugger is attached
    #[allow(clippy::empty_loop)]
    loop {}
}
