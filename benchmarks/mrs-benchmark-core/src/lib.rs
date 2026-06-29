#![no_std]

pub trait BenchmarkPlatform {
    type Instant: Copy;

    fn setup(&mut self) {}
    fn now(&self) -> Self::Instant;
    fn elapsed(&self, start: Self::Instant) -> u64;
    fn unit(&self) -> &'static str;

    /// Runs a benchmark task. Preparation and finalization are excluded from measurement.
    fn run<Input: Clone, Output, I>(&mut self, implementation: &I, input: Input) -> u64
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
}

pub trait TaskImplementation<Input, Output> {
    type PreparedInput;
    type RawOutput;

    /// Converts standard input into the library's internal format (not timed).
    fn prepare(&self, input: Input) -> Self::PreparedInput;

    /// Performs the core computation (timed).
    fn execute(&self, input: &Self::PreparedInput) -> Self::RawOutput;

    /// Converts the raw output back into standard output (not timed).
    fn finalize(&self, output: Self::RawOutput) -> Output;
}

// Dummy Test Case (Simple Add 1 + 1)

pub struct SimpleAdd;

impl TaskImplementation<(i32, i32), i32> for SimpleAdd {
    type PreparedInput = (i32, i32);
    type RawOutput = i32;

    fn prepare(&self, input: (i32, i32)) -> Self::PreparedInput {
        input
    }

    fn execute(&self, input: &Self::PreparedInput) -> Self::RawOutput {
        input.0 + input.1
    }

    fn finalize(&self, output: Self::RawOutput) -> i32 {
        output
    }
}
