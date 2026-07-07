use crate::tasks::{
    MatInverse3x3 as MatInverse3x3Task, MatMul3x3 as MatMul3x3Task, QuatMul as QuatMulTask,
    QuatSlerp as QuatSlerpTask, RotateVector as RotateVectorTask,
};
use crate::{export_tasks, BenchmarkError, BenchmarkLibrary, RawTaskImplementation};

use glam::{Mat3, Quat, Vec3};

pub struct Glam;
impl BenchmarkLibrary for Glam {
    const IDENTIFIER: &'static str = "glam";
}

pub struct MatMul3x3Logic;

impl RawTaskImplementation<MatMul3x3Task> for MatMul3x3Logic {
    type PreparedInput = (Mat3, Mat3);
    type RawOutput = Mat3;

    fn prepare(
        &self,
        input: &<MatMul3x3Task as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        let lhs = Mat3::from_cols_array(&input.lhs).transpose();
        let rhs = Mat3::from_cols_array(&input.rhs).transpose();
        (lhs, rhs)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(input.0 * input.1)
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<MatMul3x3Task as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok(output.to_cols_array())
    }
}

pub struct RotateVectorLogic;

impl RawTaskImplementation<RotateVectorTask> for RotateVectorLogic {
    type PreparedInput = (Quat, Vec3);
    type RawOutput = Vec3;

    fn prepare(
        &self,
        input: &<RotateVectorTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        let q = Quat::from_xyzw(input.quat[0], input.quat[1], input.quat[2], input.quat[3]);
        let v = Vec3::new(input.point[0], input.point[1], input.point[2]);
        (q, v)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(input.0 * input.1)
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<RotateVectorTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([output.x, output.y, output.z])
    }
}

pub struct MatInverse3x3Logic;

impl RawTaskImplementation<MatInverse3x3Task> for MatInverse3x3Logic {
    type PreparedInput = Mat3;
    type RawOutput = Mat3;

    fn prepare(
        &self,
        input: &<MatInverse3x3Task as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        Mat3::from_cols_array(&input.matrix).transpose()
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        let det = input.determinant();
        let abs_det = if det < 0.0 { -det } else { det };
        if abs_det < 1e-6 {
            Err(BenchmarkError::MathError("Matrix not invertible"))
        } else {
            Ok(input.inverse())
        }
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<MatInverse3x3Task as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok(output.to_cols_array())
    }
}

pub struct QuatMulLogic;
impl RawTaskImplementation<QuatMulTask> for QuatMulLogic {
    type PreparedInput = (Quat, Quat);
    type RawOutput = Quat;

    fn prepare(&self, input: &<QuatMulTask as crate::BenchmarkTask>::Input) -> Self::PreparedInput {
        let lhs = Quat::from_xyzw(input.lhs[0], input.lhs[1], input.lhs[2], input.lhs[3]);
        let rhs = Quat::from_xyzw(input.rhs[0], input.rhs[1], input.rhs[2], input.rhs[3]);
        (lhs, rhs)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(input.0 * input.1)
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<QuatMulTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok(output.to_array())
    }
}

pub struct QuatSlerpLogic;
impl RawTaskImplementation<QuatSlerpTask> for QuatSlerpLogic {
    type PreparedInput = (Quat, Quat, f32);
    type RawOutput = Quat;

    fn prepare(
        &self,
        input: &<QuatSlerpTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        let from = Quat::from_xyzw(input.from[0], input.from[1], input.from[2], input.from[3]);
        let to = Quat::from_xyzw(input.to[0], input.to[1], input.to[2], input.to[3]);
        (from, to, input.t)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(input.0.slerp(input.1, input.2))
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<QuatSlerpTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok(output.to_array())
    }
}

export_tasks!(
    Glam,
    MatMul3x3 => MatMul3x3Logic,
    RotateVector => RotateVectorLogic,
    MatInverse3x3 => MatInverse3x3Logic,
    QuatMul => QuatMulLogic,
    QuatSlerp => QuatSlerpLogic,
);
