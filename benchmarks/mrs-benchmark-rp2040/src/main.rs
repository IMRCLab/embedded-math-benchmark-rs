#![no_std]
#![no_main]

use cortex_m::peripheral::{syst::SystClkSource, SYST};
use cortex_m_rt::entry;
use cortex_m_semihosting::debug;
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};

// Pull in the Pico 1 BSP. The default SysTick timing path below doesn't call its
// HAL, but linking rp-pico is what provides the second-stage bootloader (boot2)
// that the RP2040 needs to boot from flash, plus the critical-section impl that
// rtt-target relies on. To measure with the 64-bit 1 µs `TIMER` instead, reach
// for `rp_pico::hal` (see the notes on the impl below).
use rp_pico as _;

use mrs_benchmark_core::{
    inputs::{MatrixMul3x3Input, RotateVectorInput},
    suites::{
        glam::{GlamMatMul3x3, GlamRotateVector},
        micromath::{UMathMatMul3x3, UMathRotateVector},
        nalgebra::{NAlgMatMul3x3, NAlgRotateVector},
    },
    BenchmarkPlatform,
};

pub struct Rp2040Platform {
    // Owned so the counter stays configured for the lifetime of the platform.
    _syst: SYST,
}

impl Rp2040Platform {
    pub fn new(mut syst: SYST) -> Self {
        syst.set_clock_source(SystClkSource::Core);
        syst.set_reload(0x00FF_FFFF); // 24-bit max: count over the full range
        syst.clear_current();
        syst.enable_counter();
        Self { _syst: syst }
    }
}

impl BenchmarkPlatform for Rp2040Platform {
    type Instant = u32;

    fn setup(&mut self) {
        rprintln!("[Platform] RP2040 SysTick cycle counter enabled (24-bit, core clock).");
    }

    fn now(&self) -> Self::Instant {
        SYST::get_current()
    }

    fn elapsed(&self, start: Self::Instant) -> u64 {
        // SysTick counts DOWN and is 24-bit, so elapsed cycles = (start - now)
        // modulo 2^24. wrapping_sub + mask handles the reload wrap correctly.
        let now = SYST::get_current();
        (start.wrapping_sub(now) & 0x00FF_FFFF) as u64
    }

    fn unit(&self) -> &'static str {
        "cycles"
    }
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Initializing RP2040 (Pico 1) Microbenchmarks...");

    // TODO(hardware, optional): to report absolute time (µs) instead of raw
    // cycles, configure the clocks/PLL here (e.g. rp_pico::hal::clocks::
    // init_clocks_and_plls) and switch the platform to rp_pico::hal::Timer. The
    // default SysTick path needs no clock setup — it counts core cycles directly.
    let cp = cortex_m::Peripherals::take().unwrap();
    let mut platform = Rp2040Platform::new(cp.SYST);
    platform.setup();

    let m1 = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
    let m2 = [9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];

    let inputs = [
        MatrixMul3x3Input { lhs: m1, rhs: m2 },
        MatrixMul3x3Input { lhs: m2, rhs: m1 },
    ];

    let total_nalg = platform.run_set(&NAlgMatMul3x3, &inputs);
    rprintln!(
        "RP2040 Benchmark Set: NAlgMatMul3x3 took {} {} total",
        total_nalg,
        platform.unit()
    );

    let total_glam = platform.run_set(&GlamMatMul3x3, &inputs);
    rprintln!(
        "RP2040 Benchmark Set: GlamMatMul3x3 took {} {} total",
        total_glam,
        platform.unit()
    );

    let total_umath = platform.run_set(&UMathMatMul3x3, &inputs);
    rprintln!(
        "RP2040 Benchmark Set: UMathMatMul3x3 took {} {} total",
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
        "RP2040 Benchmark Set: NAlgRotateVector took {} {} total",
        rot_nalg,
        platform.unit()
    );

    let rot_glam = platform.run_set(&GlamRotateVector, &rot_inputs);
    rprintln!(
        "RP2040 Benchmark Set: GlamRotateVector took {} {} total",
        rot_glam,
        platform.unit()
    );

    let rot_umath = platform.run_set(&UMathRotateVector, &rot_inputs);
    rprintln!(
        "RP2040 Benchmark Set: UMathRotateVector took {} {} total",
        rot_umath,
        platform.unit()
    );

    rprintln!("RP2040 benchmarks finished.");

    debug::exit(debug::EXIT_SUCCESS);
    // Unreachable while a debugger is attached
    #[allow(clippy::empty_loop)]
    loop {}
}
