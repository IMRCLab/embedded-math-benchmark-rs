use crate::tasks::{
    Atan2 as Atan2Task, QuatMul as QuatMulTask, QuatSlerp as QuatSlerpTask,
    RotateVector as RotateVectorTask, SinCos as SinCosTask, Sqrt as SqrtTask,
};
use crate::{export_tasks, BenchmarkError, BenchmarkLibrary, RawTaskImplementation};
use micromath::{F32Ext, Quaternion};

pub struct Micromath;
impl BenchmarkLibrary for Micromath {
    const IDENTIFIER: &'static str = "micromath";
}

pub struct RotateVectorLogic;

impl RawTaskImplementation<RotateVectorTask> for RotateVectorLogic {
    type PreparedInput = (Quaternion, [f32; 3]);
    type RawOutput = Quaternion;

    fn prepare(
        &self,
        input: &<RotateVectorTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        // micromath Quaternion::new takes (w, x, y, z)
        let q = Quaternion::new(input.quat[3], input.quat[0], input.quat[1], input.quat[2]);
        (q, input.point)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        let q = &input.0;
        let p = &input.1;
        let q_vec = Quaternion::new(0.0, p[0], p[1], p[2]);
        Ok(*q * q_vec * q.conj())
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<RotateVectorTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([output.x(), output.y(), output.z()])
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
        Ok(input.0.atan2(input.1))
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
        Ok((input.sin(), input.cos()))
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
            Ok(input.sqrt())
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
    type PreparedInput = (Quaternion, Quaternion);
    type RawOutput = Quaternion;

    fn prepare(&self, input: &<QuatMulTask as crate::BenchmarkTask>::Input) -> Self::PreparedInput {
        let lhs = Quaternion::new(input.lhs[3], input.lhs[0], input.lhs[1], input.lhs[2]);
        let rhs = Quaternion::new(input.rhs[3], input.rhs[0], input.rhs[1], input.rhs[2]);
        (lhs, rhs)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(input.0 * input.1)
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<QuatMulTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([output.x(), output.y(), output.z(), output.w()])
    }
}

pub struct QuatSlerpLogic;
impl RawTaskImplementation<QuatSlerpTask> for QuatSlerpLogic {
    type PreparedInput = (Quaternion, Quaternion, f32);
    type RawOutput = Quaternion;

    fn prepare(
        &self,
        input: &<QuatSlerpTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        let from = Quaternion::new(input.from[3], input.from[0], input.from[1], input.from[2]);
        let to = Quaternion::new(input.to[3], input.to[0], input.to[1], input.to[2]);
        (from, to, input.t)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        let q1 = input.0;
        let mut q2 = input.1;
        let t = input.2;

        let mut dot = q1.x() * q2.x() + q1.y() * q2.y() + q1.z() * q2.z() + q1.w() * q2.w();

        if dot < 0.0 {
            q2 = Quaternion::new(-q2.w(), -q2.x(), -q2.y(), -q2.z());
            dot = -dot;
        }

        const DOT_THRESHOLD: f32 = 0.9995;
        if dot > DOT_THRESHOLD {
            let x = q1.x() + t * (q2.x() - q1.x());
            let y = q1.y() + t * (q2.y() - q1.y());
            let z = q1.z() + t * (q2.z() - q1.z());
            let w = q1.w() + t * (q2.w() - q1.w());
            let len = (x * x + y * y + z * z + w * w).sqrt();
            return Ok(Quaternion::new(w / len, x / len, y / len, z / len));
        }

        let theta_0 = dot.acos();
        let theta = theta_0 * t;
        let sin_theta = theta.sin();
        let sin_theta_0 = theta_0.sin();

        let s0 = (theta_0 - theta).sin() / sin_theta_0;
        let s1 = sin_theta / sin_theta_0;

        let x = s0 * q1.x() + s1 * q2.x();
        let y = s0 * q1.y() + s1 * q2.y();
        let z = s0 * q1.z() + s1 * q2.z();
        let w = s0 * q1.w() + s1 * q2.w();

        Ok(Quaternion::new(w, x, y, z))
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<QuatSlerpTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([output.x(), output.y(), output.z(), output.w()])
    }
}

export_tasks!(
    Micromath,
    RotateVector => RotateVectorLogic,
    Atan2 => Atan2Logic,
    SinCos => SinCosLogic,
    Sqrt => SqrtLogic,
    QuatMul => QuatMulLogic,
    QuatSlerp => QuatSlerpLogic,
);
