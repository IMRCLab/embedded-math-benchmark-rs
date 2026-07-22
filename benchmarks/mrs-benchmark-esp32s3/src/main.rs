#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::main;
use rtt_target::{rprintln, rtt_init_print};
use xtensa_lx::timer::get_cycle_count;

use mrs_benchmark_core::BenchmarkPlatform;

esp_bootloader_esp_idf::esp_app_desc!();

pub struct Esp32s3Platform;

impl BenchmarkPlatform for Esp32s3Platform {
    type Instant = u32;

    fn id(&self) -> &'static str {
        "esp32s3"
    }

    fn setup(&mut self) {
        rprintln!("[Platform] ESP32-S3 CCOUNT cycle counter enabled.");
    }

    fn now(&self) -> Self::Instant {
        get_cycle_count()
    }

    fn elapsed(&self, start: Self::Instant) -> u64 {
        let now = get_cycle_count();
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

#[main]
fn main() -> ! {
    let _peripherals = esp_hal::init(esp_hal::Config::default());
    rtt_init_print!(rtt_target::ChannelMode::BlockIfFull, 4096);
    rprintln!("Initializing ESP32-S3 Microbenchmarks...");

    let mut platform = Esp32s3Platform;
    platform.setup();

    rprintln!("INFO {}", mrs_benchmark_core::CSV_HEADER);
    mrs_benchmark_core::run_all_benchmarks(&mut platform);

    rprintln!("ESP32-S3 benchmarks finished.");

    semihosting::process::exit(0);
    // exit() is typed `-> !` (always traps); loop kept only for parity with the other platforms.
    #[allow(unreachable_code, clippy::empty_loop)]
    loop {}
}
