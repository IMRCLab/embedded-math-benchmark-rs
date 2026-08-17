#![no_std]

pub use glam;
pub use micromath;
pub use mrs_benchmark_macros;
pub use nalgebra;

pub mod inputs;
pub mod suites;
pub mod tasks;

pub const CSV_HEADER: &str =
    "platform,profile,library,task,input_index,repetitions,duration,unit,result";

pub const PROFILE: &str = env!("BENCH_PROFILE");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkError {
    MathError(&'static str),
    Other(&'static str),
}

pub trait BenchmarkTask {
    const IDENTIFIER: &'static str;
    type Input;
    type Output: core::fmt::Debug;
}

pub trait BenchmarkLibrary {
    const IDENTIFIER: &'static str;
}

pub trait RawTaskImplementation<Task: BenchmarkTask> {
    type PreparedInput;
    type RawOutput;
    fn prepare(&self, input: &Task::Input) -> Self::PreparedInput;
    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError>;
    fn finalize(&self, output: Self::RawOutput) -> Result<Task::Output, BenchmarkError>;
}

pub struct LibTask<L, T>(pub T, pub core::marker::PhantomData<L>);

pub trait TaskImplementation<Task: BenchmarkTask> {
    const LIBRARY_IDENTIFIER: &'static str;
    type PreparedInput;
    type RawOutput;
    fn prepare(&self, input: &Task::Input) -> Self::PreparedInput;
    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError>;
    fn finalize(&self, output: Self::RawOutput) -> Result<Task::Output, BenchmarkError>;
}

impl<L, Task, T> TaskImplementation<Task> for LibTask<L, T>
where
    L: BenchmarkLibrary,
    Task: BenchmarkTask,
    T: RawTaskImplementation<Task>,
{
    const LIBRARY_IDENTIFIER: &'static str = L::IDENTIFIER;
    type PreparedInput = T::PreparedInput;
    type RawOutput = T::RawOutput;
    fn prepare(&self, input: &Task::Input) -> Self::PreparedInput {
        self.0.prepare(input)
    }
    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        self.0.execute(input)
    }
    fn finalize(&self, output: Self::RawOutput) -> Result<Task::Output, BenchmarkError> {
        self.0.finalize(output)
    }
}

#[macro_export]
macro_rules! export_tasks {
    ($lib:ident, $($task:ident => $logic:ident),* $(,)?) => {
        $(
            #[allow(non_upper_case_globals)]
            pub const $task: $crate::LibTask<$lib, $logic> = $crate::LibTask($logic, core::marker::PhantomData);
        )*
    };
}

pub trait BenchmarkPlatform {
    type Instant: Copy;

    fn id(&self) -> &'static str;
    fn setup(&mut self) {}
    fn now(&self) -> Self::Instant;
    fn elapsed(&self, start: Self::Instant) -> u64;
    fn unit(&self) -> &'static str;

    /// Structured logger: formats output for CSV export
    fn log_result<O: core::fmt::Debug>(
        &self,
        library: &'static str,
        bench: &'static str,
        input_index: usize,
        repetitions: u32,
        elapsed: u64,
        output: Option<&Result<O, BenchmarkError>>,
    );

    /// Runs a benchmark task. Preparation and finalization are excluded from measurement.
    fn run<Task, I>(
        &mut self,
        implementation: &I,
        input: &Task::Input,
        repetitions: u32,
    ) -> (u64, Result<Task::Output, BenchmarkError>)
    where
        Task: BenchmarkTask,
        I: TaskImplementation<Task>,
        Self: Sized,
    {
        let prep_input = implementation.prepare(input);

        let start = self.now();
        for _ in 0..repetitions {
            core::hint::black_box(
                implementation
                    .execute(core::hint::black_box(&prep_input))
                    .ok(),
            );
        }
        let elapsed = self.elapsed(start);

        let raw_output_res = implementation.execute(&prep_input);
        let output_res = raw_output_res.and_then(|raw| implementation.finalize(raw));

        (elapsed, output_res)
    }

    /// Runs a benchmark task over a set of inputs, returning the total elapsed time of all runs.
    fn run_set<Task, I>(
        &mut self,
        implementation: &I,
        inputs: &[Task::Input],
        repetitions: u32,
    ) -> u64
    where
        Task: BenchmarkTask,
        I: TaskImplementation<Task>,
        Self: Sized,
    {
        let mut total_elapsed = 0;
        for (i, input) in inputs.iter().enumerate() {
            let (elapsed, output) = self.run(implementation, input, repetitions);
            self.log_result(
                I::LIBRARY_IDENTIFIER,
                Task::IDENTIFIER,
                i,
                repetitions,
                elapsed,
                Some(&output),
            );
            total_elapsed += elapsed;
        }
        total_elapsed
    }
}

pub fn run_and_log<Platform, Task, I>(
    platform: &mut Platform,
    implementation: &I,
    inputs: &[Task::Input],
    repetitions: u32,
) -> u64
where
    Platform: BenchmarkPlatform,
    Task: BenchmarkTask,
    I: TaskImplementation<Task>,
{
    platform.run_set(implementation, inputs, repetitions)
}

mrs_benchmark_macros::generate_benchmarks!();
