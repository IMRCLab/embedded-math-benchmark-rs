use crate::tasks::{
    MatInverse3x3 as MatInverse3x3Task, MatMul3x3 as MatMul3x3Task, QuatMul as QuatMulTask,
    QuatSlerp as QuatSlerpTask, RotateVector as RotateVectorTask,
};
use crate::{export_tasks, BenchmarkError, BenchmarkLibrary, RawTaskImplementation};

use nalgebra::Matrix3;

pub struct Nalgebra;
impl BenchmarkLibrary for Nalgebra {
    const IDENTIFIER: &'static str = "nalgebra";
}

pub struct MatMul3x3Logic;

impl RawTaskImplementation<MatMul3x3Task> for MatMul3x3Logic {
    type PreparedInput = (Matrix3<f32>, Matrix3<f32>);
    type RawOutput = Matrix3<f32>;

    fn prepare(
        &self,
        input: &<MatMul3x3Task as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        let lhs = Matrix3::from_row_slice(&input.lhs);
        let rhs = Matrix3::from_row_slice(&input.rhs);
        (lhs, rhs)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(input.0 * input.1)
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<MatMul3x3Task as crate::BenchmarkTask>::Output, BenchmarkError> {
        let mut arr = [0.0; 9];
        arr.copy_from_slice(output.as_slice());
        Ok(arr)
    }
}

pub struct RotateVectorLogic;

impl RawTaskImplementation<RotateVectorTask> for RotateVectorLogic {
    type PreparedInput = (nalgebra::UnitQuaternion<f32>, nalgebra::Vector3<f32>);
    type RawOutput = nalgebra::Vector3<f32>;

    fn prepare(
        &self,
        input: &<RotateVectorTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        // nalgebra Quaternion::new expects (w, x, y, z)
        let q =
            nalgebra::Quaternion::new(input.quat[3], input.quat[0], input.quat[1], input.quat[2]);
        let uq = nalgebra::UnitQuaternion::from_quaternion(q);
        let v = nalgebra::Vector3::new(input.point[0], input.point[1], input.point[2]);
        (uq, v)
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
    type PreparedInput = Matrix3<f32>;
    type RawOutput = Matrix3<f32>;

    fn prepare(
        &self,
        input: &<MatInverse3x3Task as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        Matrix3::from_row_slice(&input.matrix)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        input
            .try_inverse()
            .ok_or(BenchmarkError::MathError("Matrix not invertible"))
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<MatInverse3x3Task as crate::BenchmarkTask>::Output, BenchmarkError> {
        let mut arr = [0.0; 9];
        arr.copy_from_slice(output.as_slice());
        Ok(arr)
    }
}

pub struct QuatMulLogic;
impl RawTaskImplementation<QuatMulTask> for QuatMulLogic {
    type PreparedInput = (nalgebra::UnitQuaternion<f32>, nalgebra::UnitQuaternion<f32>);
    type RawOutput = nalgebra::UnitQuaternion<f32>;

    fn prepare(&self, input: &<QuatMulTask as crate::BenchmarkTask>::Input) -> Self::PreparedInput {
        let q_lhs =
            nalgebra::Quaternion::new(input.lhs[3], input.lhs[0], input.lhs[1], input.lhs[2]);
        let uq_lhs = nalgebra::UnitQuaternion::from_quaternion(q_lhs);
        let q_rhs =
            nalgebra::Quaternion::new(input.rhs[3], input.rhs[0], input.rhs[1], input.rhs[2]);
        let uq_rhs = nalgebra::UnitQuaternion::from_quaternion(q_rhs);
        (uq_lhs, uq_rhs)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(input.0 * input.1)
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<QuatMulTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([
            output.coords.x,
            output.coords.y,
            output.coords.z,
            output.coords.w,
        ])
    }
}

pub struct QuatSlerpLogic;
impl RawTaskImplementation<QuatSlerpTask> for QuatSlerpLogic {
    type PreparedInput = (
        nalgebra::UnitQuaternion<f32>,
        nalgebra::UnitQuaternion<f32>,
        f32,
    );
    type RawOutput = nalgebra::UnitQuaternion<f32>;

    fn prepare(
        &self,
        input: &<QuatSlerpTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        let q_from =
            nalgebra::Quaternion::new(input.from[3], input.from[0], input.from[1], input.from[2]);
        let uq_from = nalgebra::UnitQuaternion::from_quaternion(q_from);
        let q_to = nalgebra::Quaternion::new(input.to[3], input.to[0], input.to[1], input.to[2]);
        let uq_to = nalgebra::UnitQuaternion::from_quaternion(q_to);
        (uq_from, uq_to, input.t)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(input.0.slerp(&input.1, input.2))
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<QuatSlerpTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([
            output.coords.x,
            output.coords.y,
            output.coords.z,
            output.coords.w,
        ])
    }
}

export_tasks!(
    Nalgebra,
    MatMul3x3 => MatMul3x3Logic,
    RotateVector => RotateVectorLogic,
    MatInverse3x3 => MatInverse3x3Logic,
    QuatMul => QuatMulLogic,
    QuatSlerp => QuatSlerpLogic,
);
