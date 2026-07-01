#![no_std]
#![no_main]

use cortex_m::peripheral::DWT;
use cortex_m_rt::entry;
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};

use mrs_benchmark_core::{
    inputs::{MatrixMul3x3Input, RotateVectorInput},
    suites::{
        glam::{GlamMatMul3x3, GlamRotateVector},
        micromath::{UMathMatMul3x3, UMathRotateVector},
        nalgebra::{NAlgMatMul3x3, NAlgRotateVector},
    },
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

    let m1 = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
    let m2 = [9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];

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

    // Quaternion Rotation Benchmarks
    let rot_inputs = [
        RotateVectorInput {
            point: [1.0, 2.0, 3.0],
            // 90 degrees rotation around Z axis: sin(45) = cos(45) = 1/sqrt(2)
            // [x, y, z, w]
            quat: [
                0.0,
                0.0,
                core::f32::consts::FRAC_1_SQRT_2,
                core::f32::consts::FRAC_1_SQRT_2,
            ],
        },
        RotateVectorInput {
            point: [-1.5, 3.2, 0.0],
            // 45 degrees rotation around X axis: sin(22.5) = 0.3826834, cos(22.5) = 0.9238795
            // [x, y, z, w]
            quat: [0.3826834, 0.0, 0.0, 0.9238795],
        },
    ];

    let rot_nalg = platform.run_set(&NAlgRotateVector, &rot_inputs);
    rprintln!(
        "STM32 Benchmark Set: NAlgRotateVector took {} {} total",
        rot_nalg,
        platform.unit()
    );

    let rot_glam = platform.run_set(&GlamRotateVector, &rot_inputs);
    rprintln!(
        "STM32 Benchmark Set: GlamRotateVector took {} {} total",
        rot_glam,
        platform.unit()
    );

    let rot_umath = platform.run_set(&UMathRotateVector, &rot_inputs);
    rprintln!(
        "STM32 Benchmark Set: UMathRotateVector took {} {} total",
        rot_umath,
        platform.unit()
    );

    rprintln!("STM32 benchmarks finished. Halting.");
    #[allow(clippy::empty_loop)]
    loop {}
}
