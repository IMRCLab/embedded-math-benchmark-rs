use crate::tasks::{
    CrossProduct as CrossProductTask, LeeController as LeeControllerTask,
    MatInverse3x3 as MatInverse3x3Task, MatMul3x3 as MatMul3x3Task,
    MatVecMul3x3 as MatVecMul3x3Task, QuatMul as QuatMulTask, QuatSlerp as QuatSlerpTask,
    QuatToRotMatrix as QuatToRotMatrixTask, RotateVector as RotateVectorTask,
    UnitQuatMul as UnitQuatMulTask, Vec3Normalize as Vec3NormalizeTask,
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

pub struct UnitQuatMulLogic;
impl RawTaskImplementation<UnitQuatMulTask> for UnitQuatMulLogic {
    type PreparedInput = (Quat, Quat);
    type RawOutput = Quat;

    fn prepare(
        &self,
        input: &<UnitQuatMulTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        let lhs =
            Quat::from_xyzw(input.lhs[0], input.lhs[1], input.lhs[2], input.lhs[3]).normalize();
        let rhs =
            Quat::from_xyzw(input.rhs[0], input.rhs[1], input.rhs[2], input.rhs[3]).normalize();
        (lhs, rhs)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(input.0 * input.1)
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<UnitQuatMulTask as crate::BenchmarkTask>::Output, BenchmarkError> {
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

pub struct LeeControllerLogic;

impl RawTaskImplementation<LeeControllerTask> for LeeControllerLogic {
    type PreparedInput = (
        Vec3,
        Vec3,
        Quat,
        Vec3, // state
        Vec3,
        Vec3,
        Vec3,
        f32,
        f32, // cmd
        f32, // mass
    );
    type RawOutput = [f32; 4];

    fn prepare(
        &self,
        input: &<LeeControllerTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        (
            Vec3::from_array(input.position),
            Vec3::from_array(input.velocity),
            Quat::from_array(input.attitude),
            Vec3::from_array(input.angular_velocity),
            Vec3::from_array(input.setpoint_position),
            Vec3::from_array(input.setpoint_velocity),
            Vec3::from_array(input.setpoint_acceleration),
            input.setpoint_yaw,
            input.setpoint_yaw_dot,
            input.mass,
        )
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        let (pos, vel, att, ang_vel, cmd_pos, cmd_vel, cmd_acc, cmd_yaw, cmd_yaw_rate, mass) =
            *input;

        use crate::suites::constants::*;
        let kp = Vec3::from_array(KP);
        let kd = Vec3::from_array(KD);
        let k_r = Vec3::from_array(K_R);
        let k_w = Vec3::from_array(K_W);
        let gravity = Vec3::from_array(GRAVITY);
        let inertia = Vec3::from_array(INERTIA);

        // --- Controller Update ---
        let e_p = cmd_pos - pos;
        let e_v = cmd_vel - vel;

        let f_d = mass * (cmd_acc + kp * e_p + kd * e_v - gravity);

        let r_mat = Mat3::from_quat(att);
        let thrust = f_d.dot(r_mat * Vec3::Z);

        let xcd = Vec3::new(libm::cosf(cmd_yaw), libm::sinf(cmd_yaw), 0.0);
        let ycd = Vec3::new(-libm::sinf(cmd_yaw), libm::cosf(cmd_yaw), 0.0);

        let xbd = ycd.cross(f_d).normalize();
        let ybd = f_d.cross(xbd).normalize();
        let zbd = xbd.cross(ybd);
        let r_d = Mat3::from_cols(xbd, ybd, zbd);

        let r_d_t_r = r_d.transpose() * r_mat;
        let r_t_r_d = r_mat.transpose() * r_d;
        let err_mat = r_d_t_r - r_t_r_d;
        // vee map of skew symmetric matrix: (M_32, M_13, M_21)
        let e_r = 0.5 * Vec3::new(err_mat.y_axis.z, err_mat.z_axis.x, err_mat.x_axis.y);

        let w_d = if f_d.length() > f32::EPSILON {
            let cmd_jerk = Vec3::ZERO;
            let c = zbd.dot(cmd_acc - gravity);
            let d1 = xbd.dot(cmd_jerk);
            let d2 = -ybd.dot(cmd_jerk);
            let d3 = cmd_yaw_rate * xcd.dot(xbd);

            let b3 = -ycd.dot(zbd);
            let c3 = ycd.cross(zbd).length();

            let wxd = d2 / c;
            let wyd = d1 / c;
            let wzd = (c * d3 - b3 * d1) / (c * c3);
            Vec3::new(wxd, wyd, wzd)
        } else {
            Vec3::ZERO
        };

        let e_w = ang_vel - r_mat.transpose() * r_d * w_d;

        let feedback = -k_r * e_r - k_w * e_w;
        let gyro = ang_vel.cross(inertia * ang_vel);
        let feed_forward = -inertia * (ang_vel.cross(r_mat.transpose() * r_d * w_d));
        let torque = feedback + gyro + feed_forward;

        Ok([thrust, torque.x, torque.y, torque.z])
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<LeeControllerTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok(output)
    }
}

pub struct CrossProductLogic;
impl RawTaskImplementation<CrossProductTask> for CrossProductLogic {
    type PreparedInput = (Vec3, Vec3);
    type RawOutput = Vec3;

    fn prepare(
        &self,
        input: &<CrossProductTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        (Vec3::from_array(input.lhs), Vec3::from_array(input.rhs))
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(input.0.cross(input.1))
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<CrossProductTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([output.x, output.y, output.z])
    }
}

pub struct Vec3NormalizeLogic;
impl RawTaskImplementation<Vec3NormalizeTask> for Vec3NormalizeLogic {
    type PreparedInput = Vec3;
    type RawOutput = Vec3;

    fn prepare(
        &self,
        input: &<Vec3NormalizeTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        Vec3::from_array(input.vector)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        if input.length() < 1e-6 {
            Err(BenchmarkError::MathError(
                "Vector magnitude too small to normalize",
            ))
        } else {
            Ok(input.normalize())
        }
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<Vec3NormalizeTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([output.x, output.y, output.z])
    }
}

pub struct MatVecMul3x3Logic;
impl RawTaskImplementation<MatVecMul3x3Task> for MatVecMul3x3Logic {
    type PreparedInput = (Mat3, Vec3);
    type RawOutput = Vec3;

    fn prepare(
        &self,
        input: &<MatVecMul3x3Task as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        let m = Mat3::from_cols_array(&input.matrix).transpose();
        let v = Vec3::from_array(input.vector);
        (m, v)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(input.0 * input.1)
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<MatVecMul3x3Task as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok([output.x, output.y, output.z])
    }
}

pub struct QuatToRotMatrixLogic;
impl RawTaskImplementation<QuatToRotMatrixTask> for QuatToRotMatrixLogic {
    type PreparedInput = Quat;
    type RawOutput = Mat3;

    fn prepare(
        &self,
        input: &<QuatToRotMatrixTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        Quat::from_xyzw(input.quat[0], input.quat[1], input.quat[2], input.quat[3]).normalize()
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(Mat3::from_quat(*input))
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<QuatToRotMatrixTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok(output.to_cols_array())
    }
}

export_tasks!(
    Glam,
    MatMul3x3 => MatMul3x3Logic,
    RotateVector => RotateVectorLogic,
    MatInverse3x3 => MatInverse3x3Logic,
    QuatMul => QuatMulLogic,
    UnitQuatMul => UnitQuatMulLogic,
    QuatSlerp => QuatSlerpLogic,
    LeeController => LeeControllerLogic,
    CrossProduct => CrossProductLogic,
    Vec3Normalize => Vec3NormalizeLogic,
    MatVecMul3x3 => MatVecMul3x3Logic,
    QuatToRotMatrix => QuatToRotMatrixLogic,
);
