#![no_std]

pub use glam;
pub use micromath;
pub use nalgebra;
pub use mrs_benchmark_macros;

pub mod inputs;
pub mod suites;
pub mod tasks;

pub trait BenchmarkTask {
    const IDENTIFIER: &'static str;
    type Input;
    type Output;
}

pub trait TaskImplementation<Task: BenchmarkTask> {
    const LIBRARY_IDENTIFIER: &'static str;

    type PreparedInput;
    type RawOutput;

    /// Converts standard input into the library's internal format (not timed).
    fn prepare(&self, input: &Task::Input) -> Self::PreparedInput;

    /// Performs the core computation (timed).
    fn execute(&self, input: &Self::PreparedInput) -> Self::RawOutput;

    /// Converts the raw output back into standard output (not timed).
    fn finalize(&self, output: Self::RawOutput) -> Task::Output;
}

pub trait BenchmarkPlatform {
    type Instant: Copy;

    fn id(&self) -> &'static str;
    fn setup(&mut self) {}
    fn now(&self) -> Self::Instant;
    fn elapsed(&self, start: Self::Instant) -> u64;
    fn unit(&self) -> &'static str;
    
    /// Structured logger: formats output as "platform:library:bench:elapsed unit"
    fn log_result(&self, library: &'static str, bench: &'static str, elapsed: u64);

    /// Runs a benchmark task. Preparation and finalization are excluded from measurement.
    fn run<Task, I>(&mut self, implementation: &I, input: &Task::Input) -> u64
    where
        Task: BenchmarkTask,
        I: TaskImplementation<Task>,
        Self: Sized,
    {
        let prep_input = implementation.prepare(input);

        let start = self.now();
        let raw_output = core::hint::black_box(implementation.execute(&prep_input));
        let elapsed = self.elapsed(start);

        let std_output = implementation.finalize(raw_output);
        core::hint::black_box(std_output);

        elapsed
    }

    /// Runs a benchmark task over a set of inputs, returning the total elapsed time of all runs.
    fn run_set<Task, I>(&mut self, implementation: &I, inputs: &[Task::Input]) -> u64
    where
        Task: BenchmarkTask,
        I: TaskImplementation<Task>,
        Self: Sized,
    {
        let mut total_elapsed = 0;
        for input in inputs {
            total_elapsed += self.run(implementation, input);
        }
        total_elapsed
    }
}

pub fn run_and_log<Platform, Task, I>(
    platform: &mut Platform,
    implementation: &I,
    inputs: &[Task::Input],
) -> u64
where
    Platform: BenchmarkPlatform,
    Task: BenchmarkTask,
    I: TaskImplementation<Task>,
{
    let elapsed = platform.run_set(implementation, inputs);
    platform.log_result(I::LIBRARY_IDENTIFIER, Task::IDENTIFIER, elapsed);
    elapsed
}

mrs_benchmark_macros::generate_benchmarks!();
