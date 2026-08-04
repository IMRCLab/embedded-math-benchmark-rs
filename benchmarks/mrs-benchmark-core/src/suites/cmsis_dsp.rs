use crate::tasks::{
    DotProduct64D as DotProduct64DTask, MatInverse9x9 as MatInverse9x9Task,
    MatMul9x9 as MatMul9x9Task,
};
use crate::{export_tasks, BenchmarkError, BenchmarkLibrary, RawTaskImplementation};

pub struct CmsisDsp;
impl BenchmarkLibrary for CmsisDsp {
    const IDENTIFIER: &'static str = "cmsis-dsp";
}

pub struct MatMul9x9Logic;
impl RawTaskImplementation<MatMul9x9Task> for MatMul9x9Logic {
    type PreparedInput = ([f32; 81], [f32; 81]);
    type RawOutput = [f32; 81];

    fn prepare(
        &self,
        input: &<MatMul9x9Task as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        (input.lhs, input.rhs)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        let mut out = [0.0f32; 81];
        unsafe {
            mrs_benchmark_crazyflie_sys::cmsis_matmul_9x9(
                input.0.as_ptr(),
                input.1.as_ptr(),
                out.as_mut_ptr(),
            );
        }
        Ok(out)
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<MatMul9x9Task as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok(output)
    }
}

pub struct MatInverse9x9Logic;
impl RawTaskImplementation<MatInverse9x9Task> for MatInverse9x9Logic {
    type PreparedInput = [f32; 81];
    type RawOutput = [f32; 81];

    fn prepare(
        &self,
        input: &<MatInverse9x9Task as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        input.matrix
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        let mut out = [0.0f32; 81];
        let status = unsafe {
            mrs_benchmark_crazyflie_sys::cmsis_matinverse_9x9(input.as_ptr(), out.as_mut_ptr())
        };
        if status == 0 {
            Ok(out)
        } else {
            Err(BenchmarkError::MathError("Matrix non-invertible"))
        }
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<MatInverse9x9Task as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok(output)
    }
}

pub struct DotProduct64DLogic;
impl RawTaskImplementation<DotProduct64DTask> for DotProduct64DLogic {
    type PreparedInput = ([f32; 64], [f32; 64]);
    type RawOutput = f32;

    fn prepare(
        &self,
        input: &<DotProduct64DTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        (input.lhs, input.rhs)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        let res = unsafe {
            mrs_benchmark_crazyflie_sys::cmsis_dotprod_64d(input.0.as_ptr(), input.1.as_ptr())
        };
        Ok(res)
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<DotProduct64DTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok(output)
    }
}

export_tasks!(
    CmsisDsp,
    MatMul9x9 => MatMul9x9Logic,
    MatInverse9x9 => MatInverse9x9Logic,
    DotProduct64D => DotProduct64DLogic,
);
