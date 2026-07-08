#![no_std]
#![no_main]

use cortex_m::peripheral::DWT;
use cortex_m_rt::entry;
use cortex_m_semihosting::debug;
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};

// The RP2350 bootrom won't run an image unless it finds an IMAGE_DEF metadata
// block in a Block Loop near the start of flash — the RP2350 equivalent of the
// RP2040's boot2 blob. `secure_exe()` builds a single-block self-loop and takes
// the architecture (Arm here) from the build target. memory.x places
// `.start_block` in the first 4 KB; #[used] + KEEP stop it being GC'd. Linking
// rp235x-hal also pulls in the critical-section impl that rtt-target relies on.
#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: rp235x_hal::block::ImageDef = rp235x_hal::block::ImageDef::secure_exe();

use mrs_benchmark_core::BenchmarkPlatform;

pub struct Rp2350Platform {
    // Owned so the counter stays configured for the lifetime of the platform.
    _dwt: DWT,
}

impl Rp2350Platform {
    pub fn new(dwt: DWT) -> Self {
        Self { _dwt: dwt }
    }
}

impl BenchmarkPlatform for Rp2350Platform {
    type Instant = u32;

    fn id(&self) -> &'static str {
        // "-arm" leaves room for a future RISC-V (Hazard3) variant on the same chip.
        "rp2350-arm"
    }

    fn setup(&mut self) {
        // The Cortex-M33 has a DWT cycle counter (unlike the RP2040's M0+), so we
        // time exactly like the STM32 path. It counts core-clock cycles, so no PLL
        // or clock setup is needed to measure in cycles.
        unsafe {
            let mut p = cortex_m::Peripherals::steal();
            p.DCB.enable_trace();
            p.DWT.enable_cycle_counter();
        }
        rprintln!("[Platform] RP2350 (Cortex-M33) DWT cycle counter enabled.");
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
                    "BENCH {},{},{},{},{},{},{},\"{:?}\"",
                    self.id(),
                    library,
                    bench,
                    input_index,
                    repetitions,
                    elapsed,
                    self.unit(),
                    val
                ),
                Err(e) => rprintln!(
                    "BENCH {},{},{},{},{},{},{},\"ERROR: {:?}\"",
                    self.id(),
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
                "BENCH {},{},{},{},{},{},{},\"\"",
                self.id(),
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
    rtt_init_print!();
    rprintln!("Initializing RP2350 (Pico 2) Microbenchmarks...");

    let cp = cortex_m::Peripherals::take().unwrap();
    let mut platform = Rp2350Platform::new(cp.DWT);
    platform.setup();

    rprintln!("INFO {}", mrs_benchmark_core::CSV_HEADER);
    mrs_benchmark_core::run_all_benchmarks(&mut platform);

    rprintln!("RP2350 benchmarks finished.");

    debug::exit(debug::EXIT_SUCCESS);
    // Unreachable while a debugger is attached
    #[allow(clippy::empty_loop)]
    loop {}
}
