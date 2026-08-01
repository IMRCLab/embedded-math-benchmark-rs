use crate::tasks::{
    LeeController as LeeControllerTask, MatInverse3x3 as MatInverse3x3Task,
    MatMul3x3 as MatMul3x3Task, QuatSlerp as QuatSlerpTask, RotateVector as RotateVectorTask,
    UnitQuatMul as UnitQuatMulTask,
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

pub struct UnitQuatMulLogic;
impl RawTaskImplementation<UnitQuatMulTask> for UnitQuatMulLogic {
    type PreparedInput = (nalgebra::UnitQuaternion<f32>, nalgebra::UnitQuaternion<f32>);
    type RawOutput = nalgebra::UnitQuaternion<f32>;

    fn prepare(
        &self,
        input: &<UnitQuatMulTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
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
    ) -> Result<<UnitQuatMulTask as crate::BenchmarkTask>::Output, BenchmarkError> {
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

pub struct LeeControllerLogic;

impl RawTaskImplementation<LeeControllerTask> for LeeControllerLogic {
    type PreparedInput = (
        nalgebra::Vector3<f32>,
        nalgebra::Vector3<f32>,
        nalgebra::UnitQuaternion<f32>,
        nalgebra::Vector3<f32>, // state
        nalgebra::Vector3<f32>,
        nalgebra::Vector3<f32>,
        nalgebra::Vector3<f32>,
        f32,
        f32, // cmd
        f32, // mass
    );
    type RawOutput = [f32; 4];

    fn prepare(
        &self,
        input: &<LeeControllerTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        let q = nalgebra::Quaternion::new(
            input.attitude[3],
            input.attitude[0],
            input.attitude[1],
            input.attitude[2],
        );
        let uq = nalgebra::UnitQuaternion::from_quaternion(q);

        (
            nalgebra::Vector3::new(input.position[0], input.position[1], input.position[2]),
            nalgebra::Vector3::new(input.velocity[0], input.velocity[1], input.velocity[2]),
            uq,
            nalgebra::Vector3::new(
                input.angular_velocity[0],
                input.angular_velocity[1],
                input.angular_velocity[2],
            ),
            nalgebra::Vector3::new(
                input.setpoint_position[0],
                input.setpoint_position[1],
                input.setpoint_position[2],
            ),
            nalgebra::Vector3::new(
                input.setpoint_velocity[0],
                input.setpoint_velocity[1],
                input.setpoint_velocity[2],
            ),
            nalgebra::Vector3::new(
                input.setpoint_acceleration[0],
                input.setpoint_acceleration[1],
                input.setpoint_acceleration[2],
            ),
            input.setpoint_yaw,
            input.setpoint_yaw_dot,
            input.mass,
        )
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        let (pos, vel, att, ang_vel, cmd_pos, cmd_vel, cmd_acc, cmd_yaw, cmd_yaw_rate, mass) =
            *input;

        use crate::suites::constants::*;
        let kp = nalgebra::Vector3::new(KP[0], KP[1], KP[2]);
        let kd = nalgebra::Vector3::new(KD[0], KD[1], KD[2]);
        let k_r = nalgebra::Vector3::new(K_R[0], K_R[1], K_R[2]);
        let k_w = nalgebra::Vector3::new(K_W[0], K_W[1], K_W[2]);
        let gravity = nalgebra::Vector3::new(GRAVITY[0], GRAVITY[1], GRAVITY[2]);
        let inertia = nalgebra::Vector3::new(INERTIA[0], INERTIA[1], INERTIA[2]);

        // --- Controller Update ---
        let e_p = cmd_pos - pos;
        let e_v = cmd_vel - vel;

        let kp_ep = kp.component_mul(&e_p);
        let kd_ev = kd.component_mul(&e_v);

        let f_d = mass * (cmd_acc + kp_ep + kd_ev - gravity);

        let r_mat = att.to_rotation_matrix().into_inner();
        let thrust = f_d.dot(&(r_mat * nalgebra::Vector3::z()));

        let xcd = nalgebra::Vector3::new(libm::cosf(cmd_yaw), libm::sinf(cmd_yaw), 0.0);
        let ycd = nalgebra::Vector3::new(-libm::sinf(cmd_yaw), libm::cosf(cmd_yaw), 0.0);

        let xbd = ycd.cross(&f_d).normalize();
        let ybd = f_d.cross(&xbd).normalize();
        let zbd = xbd.cross(&ybd);
        let r_d = nalgebra::Matrix3::from_columns(&[xbd, ybd, zbd]);

        let r_d_t_r = r_d.transpose() * r_mat;
        let r_t_r_d = r_mat.transpose() * r_d;
        let err_mat = r_d_t_r - r_t_r_d;

        let e_r = 0.5 * nalgebra::Vector3::new(err_mat[(2, 1)], err_mat[(0, 2)], err_mat[(1, 0)]);

        let w_d = if f_d.norm() > f32::EPSILON {
            let cmd_jerk = nalgebra::Vector3::zeros();
            let c = zbd.dot(&(cmd_acc - gravity));
            let d1 = xbd.dot(&cmd_jerk);
            let d2 = -ybd.dot(&cmd_jerk);
            let d3 = cmd_yaw_rate * xcd.dot(&xbd);

            let b3 = -ycd.dot(&zbd);
            let c3 = ycd.cross(&zbd).norm();

            let wxd = d2 / c;
            let wyd = d1 / c;
            let wzd = (c * d3 - b3 * d1) / (c * c3);
            nalgebra::Vector3::new(wxd, wyd, wzd)
        } else {
            nalgebra::Vector3::zeros()
        };

        let e_w = ang_vel - r_mat.transpose() * r_d * w_d;

        let kr_er = k_r.component_mul(&e_r);
        let kw_ew = k_w.component_mul(&e_w);

        let feedback = -kr_er - kw_ew;
        let inertia_w = inertia.component_mul(&ang_vel);
        let gyro = ang_vel.cross(&inertia_w);
        let r_d_w_d = r_mat.transpose() * r_d * w_d;
        let ang_cross = ang_vel.cross(&r_d_w_d);
        let feed_forward = -inertia.component_mul(&ang_cross);
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

export_tasks!(
    Nalgebra,
    MatMul3x3 => MatMul3x3Logic,
    RotateVector => RotateVectorLogic,
    MatInverse3x3 => MatInverse3x3Logic,
    UnitQuatMul => UnitQuatMulLogic,
    QuatSlerp => QuatSlerpLogic,
    LeeController => LeeControllerLogic,
);
