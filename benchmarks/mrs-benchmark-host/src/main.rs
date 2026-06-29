use std::time::Instant;
use mrs_benchmark_core::{BenchmarkPlatform, SimpleAdd};

pub struct HostPlatform;

impl BenchmarkPlatform for HostPlatform {
    type Instant = Instant;

    fn now(&self) -> Self::Instant {
        Instant::now()
    }

    fn elapsed(&self, start: Self::Instant) -> u64 {
        start.elapsed().as_nanos() as u64
    }

    fn unit(&self) -> &'static str {
        "ns"
    }
}

fn main() {
    let mut platform = HostPlatform;
    platform.setup();

    let elapsed = platform.run(&SimpleAdd, (1, 1));

    println!(
        "Host Benchmark: SimpleAdd took {} {}",
        elapsed,
        platform.unit()
    );
}
