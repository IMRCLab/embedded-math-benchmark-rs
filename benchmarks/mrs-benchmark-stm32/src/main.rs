#![no_std]
#![no_main]

use cortex_m::peripheral::DWT;
use cortex_m_rt::entry;
use cortex_m_semihosting::debug;
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};

use mrs_benchmark_core::BenchmarkPlatform;

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

    fn id(&self) -> &'static str {
        "stm32"
    }

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
    rtt_init_print!(rtt_target::ChannelMode::BlockIfFull, 4096);
    rprintln!("Initializing STM32 Microbenchmarks...");

    let cp = cortex_m::Peripherals::take().unwrap();
    let mut platform = Stm32Platform::new(cp.DWT);
    platform.setup();

    rprintln!("INFO {}", mrs_benchmark_core::CSV_HEADER);
    mrs_benchmark_core::run_all_benchmarks(&mut platform);

    rprintln!("STM32 benchmarks finished.");

    debug::exit(debug::EXIT_SUCCESS);
    // Unreachable while a debugger is attached
    #[allow(clippy::empty_loop)]
    loop {}
}
