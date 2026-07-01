#![no_std]
#![no_main]

use cortex_m::peripheral::DWT;
use cortex_m_rt::entry;
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};

use mrs_benchmark_core::{
    inputs::MatrixMul3x3Input,
    suites::{glam::GlamMatMul3x3, micromath::UMathMatMul3x3, nalgebra::NAlgMatMul3x3},
    BenchmarkPlatform,
};

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

    let m1 = [
        1.0, 2.0, 3.0,
        4.0, 5.0, 6.0,
        7.0, 8.0, 9.0,
    ];
    let m2 = [
        9.0, 8.0, 7.0,
        6.0, 5.0, 4.0,
        3.0, 2.0, 1.0,
    ];

    let inputs = [
        MatrixMul3x3Input { lhs: m1, rhs: m2 },
        MatrixMul3x3Input { lhs: m2, rhs: m1 },
    ];

    let total_nalg = platform.run_set(&NAlgMatMul3x3, &inputs);
    rprintln!(
        "STM32 Benchmark Set: NAlgMatMul3x3 took {} {} total",
        total_nalg,
        platform.unit()
    );

    let total_glam = platform.run_set(&GlamMatMul3x3, &inputs);
    rprintln!(
        "STM32 Benchmark Set: GlamMatMul3x3 took {} {} total",
        total_glam,
        platform.unit()
    );

    let total_umath = platform.run_set(&UMathMatMul3x3, &inputs);
    rprintln!(
        "STM32 Benchmark Set: UMathMatMul3x3 took {} {} total",
        total_umath,
        platform.unit()
    );

    rprintln!("STM32 benchmarks finished. Halting.");
    loop {}
}
