use crate::tasks::{
    LeeController as LeeControllerTask, MatInverse3x3 as MatInverse3x3Task,
    MatMul3x3 as MatMul3x3Task, QuatMul as QuatMulTask, QuatSlerp as QuatSlerpTask,
    RotateVector as RotateVectorTask,
};
use crate::{export_tasks, BenchmarkError, BenchmarkLibrary, RawTaskImplementation};

use glam::{Mat3, Mat4, Quat, Vec3, Vec4};

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
        // vee map of skew symmetric matrix
        let e_r = 0.5 * Vec3::new(err_mat.z_axis.y, err_mat.x_axis.z, err_mat.y_axis.x);

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

        // --- Allocation ---
        let kappa_f = KAPPA_F;
        let kappa_tau = KAPPA_TAU;
        let a = A;

        let inv_k_f = 1.0 / (4.0 * kappa_f);
        let inv_k_t_xy = 1.0 / (4.0 * kappa_f * a);
        let inv_k_t_z = 1.0 / (4.0 * kappa_tau);

        let w = Vec4::new(thrust, torque.x, torque.y, torque.z);
        let alloc_mat = Mat4::from_cols(
            Vec4::new(inv_k_f, inv_k_f, inv_k_f, inv_k_f),
            Vec4::new(-inv_k_t_xy, -inv_k_t_xy, inv_k_t_xy, inv_k_t_xy),
            Vec4::new(-inv_k_t_xy, inv_k_t_xy, inv_k_t_xy, -inv_k_t_xy),
            Vec4::new(-inv_k_t_z, inv_k_t_z, -inv_k_t_z, inv_k_t_z),
        );

        let m1234_sq = alloc_mat * w;

        let motors = Vec4::new(
            libm::sqrtf(m1234_sq.x.max(0.0)),
            libm::sqrtf(m1234_sq.y.max(0.0)),
            libm::sqrtf(m1234_sq.z.max(0.0)),
            libm::sqrtf(m1234_sq.w.max(0.0)),
        );

        Ok(motors.to_array())
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<LeeControllerTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok(output)
    }
}

export_tasks!(
    Glam,
    MatMul3x3 => MatMul3x3Logic,
    RotateVector => RotateVectorLogic,
    MatInverse3x3 => MatInverse3x3Logic,
    QuatMul => QuatMulLogic,
    QuatSlerp => QuatSlerpLogic,
    LeeController => LeeControllerLogic,
);
