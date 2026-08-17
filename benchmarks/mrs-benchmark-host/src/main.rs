use mrs_benchmark_core::BenchmarkPlatform;
use std::time::Instant;

pub struct HostPlatform;

impl BenchmarkPlatform for HostPlatform {
    type Instant = Instant;

    fn id(&self) -> &'static str {
        "host"
    }

    fn now(&self) -> Self::Instant {
        Instant::now()
    }

    fn elapsed(&self, start: Self::Instant) -> u64 {
        start.elapsed().as_nanos() as u64
    }

    fn unit(&self) -> &'static str {
        "ns"
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
                Ok(val) => println!(
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
                Err(e) => println!(
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
            println!(
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

fn main() {
    let mut platform = HostPlatform;
    platform.setup();

    println!("Starting host benchmarks...");
    println!("INFO {}", mrs_benchmark_core::CSV_HEADER);
    mrs_benchmark_core::run_all_benchmarks(&mut platform);
    println!("Finished host benchmarks.");
}
