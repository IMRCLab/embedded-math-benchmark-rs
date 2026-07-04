use crate::tasks::{MatMul3x3 as MatMul3x3Task, RotateVector as RotateVectorTask};
use crate::{BenchmarkError, BenchmarkLibrary, RawTaskImplementation, export_tasks};

use glam::{Mat3, Quat, Vec3};

pub struct Glam;
impl BenchmarkLibrary for Glam {
    const IDENTIFIER: &'static str = "glam";
}

pub struct MatMul3x3Logic;

impl RawTaskImplementation<MatMul3x3Task> for MatMul3x3Logic {
    type PreparedInput = (Mat3, Mat3);
    type RawOutput = Mat3;

    fn prepare(&self, input: &<MatMul3x3Task as crate::BenchmarkTask>::Input) -> Self::PreparedInput {
        let lhs = Mat3::from_cols_array(&input.lhs).transpose();
        let rhs = Mat3::from_cols_array(&input.rhs).transpose();
        (lhs, rhs)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(input.0 * input.1)
    }

    fn finalize(&self, output: Self::RawOutput) -> Result<<MatMul3x3Task as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok(output.to_cols_array())
    }
}

pub struct RotateVectorLogic;

impl RawTaskImplementation<RotateVectorTask> for RotateVectorLogic {
    type PreparedInput = (Quat, Vec3);
    type RawOutput = Vec3;

    fn prepare(&self, input: &<RotateVectorTask as crate::BenchmarkTask>::Input) -> Self::PreparedInput {
        let q = Quat::from_xyzw(input.quat[0], input.quat[1], input.quat[2], input.quat[3]);
        let v = Vec3::new(input.point[0], input.point[1], input.point[2]);
        (q, v)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(input.0 * input.1)
    }

    fn finalize(&self, output: Self::RawOutput) -> Result<<RotateVectorTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([output.x, output.y, output.z])
    }
}

export_tasks!(
    Glam,
    MatMul3x3 => MatMul3x3Logic,
    RotateVector => RotateVectorLogic
);
