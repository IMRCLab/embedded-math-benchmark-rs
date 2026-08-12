use crate::tasks::{
    CrossProduct as CrossProductTask, DotProduct64D as DotProduct64DTask, EkfStep as EkfStepTask,
    LeeController as LeeControllerTask, MatInverse3x3 as MatInverse3x3Task,
    MatInverse9x9 as MatInverse9x9Task, MatMul3x3 as MatMul3x3Task, MatMul9x9 as MatMul9x9Task,
    MatVecMul3x3 as MatVecMul3x3Task, QuatSlerp as QuatSlerpTask,
    QuatToRotMatrix as QuatToRotMatrixTask, RotateVector as RotateVectorTask,
    UnitQuatMul as UnitQuatMulTask, Vec3Normalize as Vec3NormalizeTask,
};
use crate::{export_tasks, BenchmarkError, BenchmarkLibrary, RawTaskImplementation};

use nalgebra::{Matrix3, Vector3};
type SMatrix9 = nalgebra::SMatrix<f32, 9, 9>;
type SVector64 = nalgebra::SVector<f32, 64>;

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

pub struct EkfStepLogic;

pub struct PreparedEkfInput {
    pub position: nalgebra::Vector3<f32>,
    pub velocity: nalgebra::Vector3<f32>,
    pub attitude: nalgebra::UnitQuaternion<f32>,
    pub accelerometer: nalgebra::Vector3<f32>,
    pub gyroscope: nalgebra::Vector3<f32>,
    pub range_z: f32,
    pub dt: f32,
    pub covariance: nalgebra::SMatrix<f32, 9, 9>,
}

impl RawTaskImplementation<EkfStepTask> for EkfStepLogic {
    type PreparedInput = PreparedEkfInput;
    type RawOutput = [f32; 10];

    fn prepare(&self, input: &<EkfStepTask as crate::BenchmarkTask>::Input) -> Self::PreparedInput {
        let pos = nalgebra::Vector3::new(input.position[0], input.position[1], input.position[2]);
        let vel = nalgebra::Vector3::new(input.velocity[0], input.velocity[1], input.velocity[2]);
        let q = nalgebra::Quaternion::new(
            input.attitude[3],
            input.attitude[0],
            input.attitude[1],
            input.attitude[2],
        );
        let uq = nalgebra::UnitQuaternion::from_quaternion(q);
        let acc = nalgebra::Vector3::new(
            input.accelerometer[0],
            input.accelerometer[1],
            input.accelerometer[2],
        );
        let gyro =
            nalgebra::Vector3::new(input.gyroscope[0], input.gyroscope[1], input.gyroscope[2]);
        let cov = nalgebra::SMatrix::<f32, 9, 9>::from_row_slice(&input.covariance);

        PreparedEkfInput {
            position: pos,
            velocity: vel,
            attitude: uq,
            accelerometer: acc,
            gyroscope: gyro,
            range_z: input.range_z,
            dt: input.dt,
            covariance: cov,
        }
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        let mut p = input.position;
        let mut v_b = input.velocity;
        let mut q = input.attitude;
        let acc = input.accelerometer;
        let gyro = input.gyroscope;
        let zrange = input.range_z;
        let dt = input.dt;
        let mut cov = input.covariance;
        let acc_m_s2 = acc * 9.81;

        let r_3d = q.to_rotation_matrix();
        let r = r_3d.matrix();

        let dx = v_b.x * dt;
        let dy = v_b.y * dt;
        let dz = v_b.z * dt + acc_m_s2.z * (dt * dt) / 2.0;

        p.x += r[(0, 0)] * dx + r[(0, 1)] * dy + r[(0, 2)] * dz;
        p.y += r[(1, 0)] * dx + r[(1, 1)] * dy + r[(1, 2)] * dz;
        p.z += r[(2, 0)] * dx + r[(2, 1)] * dy + r[(2, 2)] * dz - 9.81 * (dt * dt) / 2.0;

        let tmp_spx = v_b.x;
        let tmp_spy = v_b.y;
        let tmp_spz = v_b.z;

        v_b.x += dt * (gyro.z * tmp_spy - gyro.y * tmp_spz - 9.81 * r[(2, 0)]);
        v_b.y += dt * (-gyro.z * tmp_spx + gyro.x * tmp_spz - 9.81 * r[(2, 1)]);
        v_b.z += dt * (acc_m_s2.z + gyro.y * tmp_spx - gyro.x * tmp_spy - 9.81 * r[(2, 2)]);

        let dtwx = dt * gyro.x;
        let dtwy = dt * gyro.y;
        let dtwz = dt * gyro.z;
        let angle = libm::sqrtf(dtwx * dtwx + dtwy * dtwy + dtwz * dtwz);
        if angle > 1e-6 {
            let ca = libm::cosf(angle / 2.0);
            let sa = libm::sinf(angle / 2.0);
            let dq = nalgebra::Quaternion::new(
                ca,
                sa * dtwx / angle,
                sa * dtwy / angle,
                sa * dtwz / angle,
            );
            let q_curr = q.into_inner();
            let q_new = q_curr * dq;
            q = nalgebra::UnitQuaternion::from_quaternion(q_new);
        }

        let mut a_mat = nalgebra::SMatrix::<f32, 9, 9>::identity();
        a_mat[(0, 3)] = r[(0, 0)] * dt;
        a_mat[(0, 4)] = r[(0, 1)] * dt;
        a_mat[(0, 5)] = r[(0, 2)] * dt;
        a_mat[(1, 3)] = r[(1, 0)] * dt;
        a_mat[(1, 4)] = r[(1, 1)] * dt;
        a_mat[(1, 5)] = r[(1, 2)] * dt;
        a_mat[(2, 3)] = r[(2, 0)] * dt;
        a_mat[(2, 4)] = r[(2, 1)] * dt;
        a_mat[(2, 5)] = r[(2, 2)] * dt;

        a_mat[(3, 3)] = 1.0;
        a_mat[(3, 4)] = gyro.z * dt;
        a_mat[(3, 5)] = -gyro.y * dt;
        a_mat[(4, 3)] = -gyro.z * dt;
        a_mat[(4, 4)] = 1.0;
        a_mat[(4, 5)] = gyro.x * dt;
        a_mat[(5, 3)] = gyro.y * dt;
        a_mat[(5, 4)] = -gyro.x * dt;
        a_mat[(5, 5)] = 1.0;

        a_mat[(3, 6)] = 0.0;
        a_mat[(3, 7)] = 9.81 * r[(2, 2)] * dt;
        a_mat[(3, 8)] = -9.81 * r[(2, 1)] * dt;
        a_mat[(4, 6)] = -9.81 * r[(2, 2)] * dt;
        a_mat[(4, 7)] = 0.0;
        a_mat[(4, 8)] = 9.81 * r[(2, 0)] * dt;
        a_mat[(5, 6)] = 9.81 * r[(2, 1)] * dt;
        a_mat[(5, 7)] = -9.81 * r[(2, 0)] * dt;
        a_mat[(5, 8)] = 0.0;

        let d0 = gyro.x * dt / 2.0;
        let d1 = gyro.y * dt / 2.0;
        let d2 = gyro.z * dt / 2.0;
        a_mat[(6, 6)] = 1.0 - d1 * d1 / 2.0 - d2 * d2 / 2.0;
        a_mat[(6, 7)] = d2 + d0 * d1 / 2.0;
        a_mat[(6, 8)] = -d1 + d0 * d2 / 2.0;
        a_mat[(7, 6)] = -d2 + d0 * d1 / 2.0;
        a_mat[(7, 7)] = 1.0 - d0 * d0 / 2.0 - d2 * d2 / 2.0;
        a_mat[(7, 8)] = d0 + d1 * d2 / 2.0;
        a_mat[(8, 6)] = d1 + d0 * d2 / 2.0;
        a_mat[(8, 7)] = -d0 + d1 * d2 / 2.0;
        a_mat[(8, 8)] = 1.0 - d0 * d0 / 2.0 - d1 * d1 / 2.0;

        let r_proc = nalgebra::SMatrix::<f32, 9, 9>::from_diagonal(
            &nalgebra::SVector::<f32, 9>::from_row_slice(&[
                0.0, 0.0, 0.0, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            ]),
        );
        cov = a_mat * cov * a_mat.transpose() + r_proc;

        // 3. Range scalar update
        let exp_point_a = 2.5f32;
        let exp_std_a = 0.0025f32;
        let exp_point_b = 4.0f32;
        let exp_std_b = 0.2f32;
        let exp_coeff = libm::logf(exp_std_b / exp_std_a) / (exp_point_b - exp_point_a);
        let std_dev = exp_std_a * (1.0 + libm::expf(exp_coeff * (p.z - exp_point_a)));
        let r_meas = std_dev * std_dev;

        let mut h_mat = nalgebra::SMatrix::<f32, 1, 9>::zeros();
        h_mat[(0, 2)] = 1.0;

        let ht = h_mat.transpose();
        let p_ht = cov * ht;
        let hphr = r_meas + p_ht[(2, 0)];
        let k_gain = p_ht / hphr;
        let z_diff = zrange - p.z;

        p.x += k_gain[(0, 0)] * z_diff;
        p.y += k_gain[(1, 0)] * z_diff;
        p.z += k_gain[(2, 0)] * z_diff;

        v_b.x += k_gain[(3, 0)] * z_diff;
        v_b.y += k_gain[(4, 0)] * z_diff;
        v_b.z += k_gain[(5, 0)] * z_diff;

        let d0_err = k_gain[(6, 0)] * z_diff;
        let d1_err = k_gain[(7, 0)] * z_diff;
        let d2_err = k_gain[(8, 0)] * z_diff;

        let i_kh = nalgebra::SMatrix::<f32, 9, 9>::identity() - k_gain * h_mat;
        let r_meas_mat = nalgebra::SMatrix::<f32, 1, 1>::new(r_meas);
        cov = i_kh * cov * i_kh.transpose() + k_gain * r_meas_mat * k_gain.transpose();
        let _ = cov;

        // 4. Incorporate attitude error into q
        let dq_err = nalgebra::Quaternion::new(1.0, d0_err / 2.0, d1_err / 2.0, d2_err / 2.0);
        let q_curr = q.into_inner();
        let q_new_quat = q_curr * dq_err;
        q = nalgebra::UnitQuaternion::from_quaternion(q_new_quat);

        let q_final = q.into_inner();
        Ok([
            p.x, p.y, p.z, v_b.x, v_b.y, v_b.z, q_final.i, q_final.j, q_final.k, q_final.w,
        ])
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<EkfStepTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok(output)
    }
}

pub struct MatMul9x9Logic;
impl RawTaskImplementation<MatMul9x9Task> for MatMul9x9Logic {
    type PreparedInput = (SMatrix9, SMatrix9);
    type RawOutput = SMatrix9;

    fn prepare(
        &self,
        input: &<MatMul9x9Task as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        let lhs = SMatrix9::from_row_slice(&input.lhs);
        let rhs = SMatrix9::from_row_slice(&input.rhs);
        (lhs, rhs)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(input.0 * input.1)
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<MatMul9x9Task as crate::BenchmarkTask>::Output, BenchmarkError> {
        let mut res = [0.0f32; 81];
        res.copy_from_slice(output.transpose().as_slice());
        Ok(res)
    }
}

pub struct MatInverse9x9Logic;
impl RawTaskImplementation<MatInverse9x9Task> for MatInverse9x9Logic {
    type PreparedInput = SMatrix9;
    type RawOutput = SMatrix9;

    fn prepare(
        &self,
        input: &<MatInverse9x9Task as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        SMatrix9::from_row_slice(&input.matrix)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        input
            .try_inverse()
            .ok_or(BenchmarkError::MathError("Matrix non-invertible"))
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<MatInverse9x9Task as crate::BenchmarkTask>::Output, BenchmarkError> {
        let mut res = [0.0f32; 81];
        res.copy_from_slice(output.transpose().as_slice());
        Ok(res)
    }
}

pub struct DotProduct64DLogic;
impl RawTaskImplementation<DotProduct64DTask> for DotProduct64DLogic {
    type PreparedInput = (SVector64, SVector64);
    type RawOutput = f32;

    fn prepare(
        &self,
        input: &<DotProduct64DTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        let lhs = SVector64::from_row_slice(&input.lhs);
        let rhs = SVector64::from_row_slice(&input.rhs);
        (lhs, rhs)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(input.0.dot(&input.1))
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<DotProduct64DTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        Ok(output)
    }
}

pub struct CrossProductLogic;
impl RawTaskImplementation<CrossProductTask> for CrossProductLogic {
    type PreparedInput = (Vector3<f32>, Vector3<f32>);
    type RawOutput = Vector3<f32>;

    fn prepare(
        &self,
        input: &<CrossProductTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        (
            Vector3::from_row_slice(&input.lhs),
            Vector3::from_row_slice(&input.rhs),
        )
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(input.0.cross(&input.1))
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
    type PreparedInput = Vector3<f32>;
    type RawOutput = Vector3<f32>;

    fn prepare(
        &self,
        input: &<Vec3NormalizeTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        Vector3::from_row_slice(&input.vector)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        if input.norm() < 1e-6 {
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
    type PreparedInput = (Matrix3<f32>, Vector3<f32>);
    type RawOutput = Vector3<f32>;

    fn prepare(
        &self,
        input: &<MatVecMul3x3Task as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        (
            Matrix3::from_row_slice(&input.matrix),
            Vector3::from_row_slice(&input.vector),
        )
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
    type PreparedInput = nalgebra::UnitQuaternion<f32>;
    type RawOutput = Matrix3<f32>;

    fn prepare(
        &self,
        input: &<QuatToRotMatrixTask as crate::BenchmarkTask>::Input,
    ) -> Self::PreparedInput {
        let q =
            nalgebra::Quaternion::new(input.quat[3], input.quat[0], input.quat[1], input.quat[2]);
        nalgebra::UnitQuaternion::from_quaternion(q)
    }

    fn execute(&self, input: &Self::PreparedInput) -> Result<Self::RawOutput, BenchmarkError> {
        Ok(input.to_rotation_matrix().into_inner())
    }

    fn finalize(
        &self,
        output: Self::RawOutput,
    ) -> Result<<QuatToRotMatrixTask as crate::BenchmarkTask>::Output, BenchmarkError> {
        let mut arr = [0.0; 9];
        arr.copy_from_slice(output.as_slice());
        Ok(arr)
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
    EkfStep => EkfStepLogic,
    MatMul9x9 => MatMul9x9Logic,
    MatInverse9x9 => MatInverse9x9Logic,
    DotProduct64D => DotProduct64DLogic,
    CrossProduct => CrossProductLogic,
    Vec3Normalize => Vec3NormalizeLogic,
    MatVecMul3x3 => MatVecMul3x3Logic,
    QuatToRotMatrix => QuatToRotMatrixLogic,
);
