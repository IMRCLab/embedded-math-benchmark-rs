use crate::tasks::{
    Atan2 as Atan2Task, MatMul3x3 as MatMul3x3Task, QuatMul as QuatMulTask,
    QuatSlerp as QuatSlerpTask, RotateVector as RotateVectorTask, SinCos as SinCosTask,
    Sqrt as SqrtTask,
};
use crate::{export_tasks, BenchmarkError, BenchmarkLibrary, RawTaskImplementation};

pub struct CrazyflieFw;
impl BenchmarkLibrary for CrazyflieFw {
    const IDENTIFIER: &'static str = "crazyflie-fw";
}

pub struct MatMul3x3Logic;
impl RawTaskImplementation<MatMul3x3Task> for MatMul3x3Logic {
    type PreparedInput = (
        mrs_benchmark_crazyflie_sys::mat33,
        mrs_benchmark_crazyflie_sys::mat33,
    );
    type RawOutput = mrs_benchmark_crazyflie_sys::mat33;

    fn prepare(
        &self,
        input: &<MatMul3x3Task as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        let lhs = mrs_benchmark_crazyflie_sys::mat33 {
            m: [
                [input.lhs[0], input.lhs[1], input.lhs[2]],
                [input.lhs[3], input.lhs[4], input.lhs[5]],
                [input.lhs[6], input.lhs[7], input.lhs[8]],
            ],
        };
        let rhs = mrs_benchmark_crazyflie_sys::mat33 {
            m: [
                [input.rhs[0], input.rhs[1], input.rhs[2]],
                [input.rhs[3], input.rhs[4], input.rhs[5]],
                [input.rhs[6], input.rhs[7], input.rhs[8]],
            ],
        };
        (lhs, rhs)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(unsafe { mrs_benchmark_crazyflie_sys::cf_mmul(input.0, input.1) })
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<MatMul3x3Task as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([
            output.m[0][0],
            output.m[0][1],
            output.m[0][2],
            output.m[1][0],
            output.m[1][1],
            output.m[1][2],
            output.m[2][0],
            output.m[2][1],
            output.m[2][2],
        ])
    }
}

pub struct RotateVectorLogic;
impl RawTaskImplementation<RotateVectorTask> for RotateVectorLogic {
    type PreparedInput = (
        mrs_benchmark_crazyflie_sys::quat,
        mrs_benchmark_crazyflie_sys::vec,
    );
    type RawOutput = mrs_benchmark_crazyflie_sys::vec;

    fn prepare(
        &self,
        input: &<RotateVectorTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        let q = mrs_benchmark_crazyflie_sys::quat {
            x: input.quat[0],
            y: input.quat[1],
            z: input.quat[2],
            w: input.quat[3],
        };
        let v = mrs_benchmark_crazyflie_sys::vec {
            x: input.point[0],
            y: input.point[1],
            z: input.point[2],
        };
        (q, v)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(unsafe { mrs_benchmark_crazyflie_sys::cf_qrot(input.0, input.1) })
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<RotateVectorTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([output.x, output.y, output.z])
    }
}

pub struct Atan2Logic;
impl RawTaskImplementation<Atan2Task> for Atan2Logic {
    type PreparedInput = (f32, f32);
    type RawOutput = f32;

    fn prepare(&self, input: &<Atan2Task as crate::BenchmarkTask>::Input) -> Self::PreparedInput {
        (input.y, input.x)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(unsafe { mrs_benchmark_crazyflie_sys::atan2f(input.0, input.1) })
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
        unsafe {
            let sin = mrs_benchmark_crazyflie_sys::sinf(*input);
            let cos = mrs_benchmark_crazyflie_sys::cosf(*input);
            Ok((sin, cos))
        }
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
            Ok(unsafe { mrs_benchmark_crazyflie_sys::sqrtf(*input) })
        }
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<SqrtTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok(output)
    }
}

pub struct QuatMulLogic;
impl RawTaskImplementation<QuatMulTask> for QuatMulLogic {
    type PreparedInput = (
        mrs_benchmark_crazyflie_sys::quat,
        mrs_benchmark_crazyflie_sys::quat,
    );
    type RawOutput = mrs_benchmark_crazyflie_sys::quat;

    fn prepare(&self, input: &<QuatMulTask as crate::BenchmarkTask>::Input) -> Self::PreparedInput {
        let lhs = mrs_benchmark_crazyflie_sys::quat {
            x: input.lhs[0],
            y: input.lhs[1],
            z: input.lhs[2],
            w: input.lhs[3],
        };
        let rhs = mrs_benchmark_crazyflie_sys::quat {
            x: input.rhs[0],
            y: input.rhs[1],
            z: input.rhs[2],
            w: input.rhs[3],
        };
        (lhs, rhs)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(unsafe { mrs_benchmark_crazyflie_sys::cf_qqmul(input.0, input.1) })
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<QuatMulTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([output.x, output.y, output.z, output.w])
    }
}

pub struct QuatSlerpLogic;
impl RawTaskImplementation<QuatSlerpTask> for QuatSlerpLogic {
    type PreparedInput = (
        mrs_benchmark_crazyflie_sys::quat,
        mrs_benchmark_crazyflie_sys::quat,
        f32,
    );
    type RawOutput = mrs_benchmark_crazyflie_sys::quat;

    fn prepare(
        &self,
        input: &<QuatSlerpTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        let from = mrs_benchmark_crazyflie_sys::quat {
            x: input.from[0],
            y: input.from[1],
            z: input.from[2],
            w: input.from[3],
        };
        let to = mrs_benchmark_crazyflie_sys::quat {
            x: input.to[0],
            y: input.to[1],
            z: input.to[2],
            w: input.to[3],
        };
        (from, to, input.t)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(unsafe { mrs_benchmark_crazyflie_sys::cf_qslerp(input.0, input.1, input.2) })
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<QuatSlerpTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([output.x, output.y, output.z, output.w])
    }
}

export_tasks!(
    CrazyflieFw,
    MatMul3x3 => MatMul3x3Logic,
    RotateVector => RotateVectorLogic,
    Atan2 => Atan2Logic,
    SinCos => SinCosLogic,
    Sqrt => SqrtLogic,
    QuatMul => QuatMulLogic,
    QuatSlerp => QuatSlerpLogic,
);
