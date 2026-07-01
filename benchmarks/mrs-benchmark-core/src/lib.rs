#![no_std]

pub use glam;
pub use micromath;
pub use nalgebra;



pub mod inputs;
pub mod suites;

pub trait BenchmarkPlatform {
    type Instant: Copy;

    fn setup(&mut self) {}
    fn now(&self) -> Self::Instant;
    fn elapsed(&self, start: Self::Instant) -> u64;
    fn unit(&self) -> &'static str;

    /// Runs a benchmark task. Preparation and finalization are excluded from measurement.
    fn run<Input, Output, I>(&mut self, implementation: &I, input: &Input) -> u64
    where
        I: TaskImplementation<Input, Output>,
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
    fn run_set<Input, Output, I>(&mut self, implementation: &I, inputs: &[Input]) -> u64
    where
        I: TaskImplementation<Input, Output>,
        Self: Sized,
    {
        let mut total_elapsed = 0;
        for input in inputs {
            total_elapsed += self.run(implementation, input);
        }
        total_elapsed
    }
}

pub trait TaskImplementation<Input, Output> {
    type PreparedInput;
    type RawOutput;

    /// Converts standard input into the library's internal format (not timed).
    fn prepare(&self, input: &Input) -> Self::PreparedInput;

    /// Performs the core computation (timed).
    fn execute(&self, input: &Self::PreparedInput) -> Self::RawOutput;

    /// Converts the raw output back into standard output (not timed).
    fn finalize(&self, output: Self::RawOutput) -> Output;
}
