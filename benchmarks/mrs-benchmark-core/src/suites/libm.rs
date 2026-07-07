use crate::tasks::{Atan2 as Atan2Task, SinCos as SinCosTask, Sqrt as SqrtTask};
use crate::{export_tasks, BenchmarkError, BenchmarkLibrary, RawTaskImplementation};

pub struct Libm;
impl BenchmarkLibrary for Libm {
    const IDENTIFIER: &'static str = "libm";
}

pub struct Atan2Logic;
impl RawTaskImplementation<Atan2Task> for Atan2Logic {
    type PreparedInput = (f32, f32);
    type RawOutput = f32;

    fn prepare(&self, input: &<Atan2Task as crate::BenchmarkTask>::Input) -> Self::PreparedInput {
        (input.y, input.x)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(libm::atan2f(input.0, input.1))
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<Atan2Task as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok(output)
    }
}

pub struct SinCosLogic;
impl RawTaskImplementation<SinCosTask> for SinCosLogic {
    type PreparedInput = f32;
    type RawOutput = (f32, f32);

    fn prepare(&self, input: &<SinCosTask as crate::BenchmarkTask>::Input) -> Self::PreparedInput {
        input.theta
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(libm::sincosf(*input))
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<SinCosTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([output.0, output.1])
    }
}

pub struct SqrtLogic;
impl RawTaskImplementation<SqrtTask> for SqrtLogic {
    type PreparedInput = f32;
    type RawOutput = f32;

    fn prepare(&self, input: &<SqrtTask as crate::BenchmarkTask>::Input) -> Self::PreparedInput {
        input.value
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        if *input < 0.0 {
            Err(BenchmarkError::MathError("Square root of negative number"))
        } else {
            Ok(libm::sqrtf(*input))
        }
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<SqrtTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok(output)
    }
}

export_tasks!(
    Libm,
    Atan2 => Atan2Logic,
    SinCos => SinCosLogic,
    Sqrt => SqrtLogic,
);
