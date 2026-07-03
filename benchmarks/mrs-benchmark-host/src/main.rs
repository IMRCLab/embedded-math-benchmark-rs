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

    fn log_result(
        &self,
        library: &'static str,
        bench: &'static str,
        id: mrs_benchmark_core::ResultId,
        elapsed: u64,
    ) {
        match id {
            mrs_benchmark_core::ResultId::Input(i) => {
                println!("{}:{}:{}, {}, {}", self.id(), library, bench, i, elapsed)
            }
            mrs_benchmark_core::ResultId::All => {
                println!("{}:{}:{}, all, {}", self.id(), library, bench, elapsed)
            }
        }
    }
}

fn main() {
    let mut platform = HostPlatform;
    platform.setup();

    println!("Starting host benchmarks...");
    mrs_benchmark_core::run_all_benchmarks(&mut platform);
    println!("Finished host benchmarks.");
}
