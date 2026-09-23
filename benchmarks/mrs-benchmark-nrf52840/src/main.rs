#![no_std]
#![no_main]

use cortex_m::peripheral::DWT;
use cortex_m_rt::entry;
use cortex_m_semihosting::debug;
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};

use nrf52840_hal::clocks::Clocks;
use nrf52840_hal::pac::Peripherals;

use mrs_benchmark_core::BenchmarkPlatform;

pub struct Nrf52840Platform {
    _dwt: DWT,
}

impl Nrf52840Platform {
    pub fn new(dwt: DWT) -> Self {
        Self { _dwt: dwt }
    }
}

impl BenchmarkPlatform for Nrf52840Platform {
    type Instant = u32;

    fn id(&self) -> &'static str {
        "nrf52840"
    }

    fn setup(&mut self) {
        // Cortex-M4F DWT cycle counter, timed exactly like the STM32 and RP2350-arm
        // paths. Needs a debugger attached to run on the nRF52 (our HIL always has one).
        unsafe {
            let mut p = cortex_m::Peripherals::steal();
            p.DCB.enable_trace();
            p.DWT.enable_cycle_counter();
        }
        rprintln!("[Platform] nRF52840 DWT cycle counter enabled.");
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
    rprintln!("Initializing nRF52840 Microbenchmarks...");

    let dp = Peripherals::take().unwrap();

    // The core always runs at 64 MHz; selecting the external HFXO crystal as the
    // HFCLK source swaps the reset-default RC oscillator (~2% off) for a crystal
    // reference, so the fixed 64 MHz cycle-to-ns conversion in viz/config.py is exact.
    let _clocks = Clocks::new(dp.CLOCK).enable_ext_hfosc();

    // Flash instruction cache resets disabled; enable it to match the STM32's ART.
    dp.NVMC.icachecnf.write(|w| w.cacheen().enabled());

    let cp = cortex_m::Peripherals::take().unwrap();
    let mut platform = Nrf52840Platform::new(cp.DWT);
    platform.setup();

    rprintln!("INFO {}", mrs_benchmark_core::CSV_HEADER);
    mrs_benchmark_core::run_all_benchmarks(&mut platform);

    rprintln!("nRF52840 benchmarks finished.");

    debug::exit(debug::EXIT_SUCCESS);
    // Unreachable while a debugger is attached
    #[allow(clippy::empty_loop)]
    loop {}
}
