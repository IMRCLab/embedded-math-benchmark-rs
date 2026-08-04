use crate::tasks::{
    EkfStep as EkfStepTask, LeeController as LeeControllerTask, MatInverse3x3 as MatInverse3x3Task,
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

        // --- Allocation ---
        let kappa_f = KAPPA_F;
        let kappa_tau = KAPPA_TAU;
        let a = A;

        let inv_k_f = 1.0 / (4.0 * kappa_f);
        let inv_k_t_xy = 1.0 / (4.0 * kappa_f * a);
        let inv_k_t_z = 1.0 / (4.0 * kappa_tau);

        let w = nalgebra::Vector4::new(thrust, torque.x, torque.y, torque.z);
        let alloc_mat = nalgebra::Matrix4::new(
            inv_k_f,
            -inv_k_t_xy,
            -inv_k_t_xy,
            -inv_k_t_z,
            inv_k_f,
            -inv_k_t_xy,
            inv_k_t_xy,
            inv_k_t_z,
            inv_k_f,
            inv_k_t_xy,
            inv_k_t_xy,
            -inv_k_t_z,
            inv_k_f,
            inv_k_t_xy,
            -inv_k_t_xy,
            inv_k_t_z,
        );

        let m1234_sq = alloc_mat * w;

        let motors = nalgebra::Vector4::new(
            libm::sqrtf(m1234_sq.x.max(0.0)),
            libm::sqrtf(m1234_sq.y.max(0.0)),
            libm::sqrtf(m1234_sq.z.max(0.0)),
            libm::sqrtf(m1234_sq.w.max(0.0)),
        );

        Ok([motors.x, motors.y, motors.z, motors.w])
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
        let mut delta = nalgebra::Vector3::zeros();

        let r_mat = q.to_rotation_matrix();

        // 1. Position update: p_world = p_world + R * v_body * dt
        p += r_mat * v_b * dt;

        // 2. Velocity update: v_body = v_body + (a_body + R.T * g - w x v_body) * dt
        let gravity_world = nalgebra::Vector3::new(0.0, 0.0, -9.81);
        let gravity_body = r_mat.inverse() * gravity_world;
        let coriolis = gyro.cross(&v_b);
        v_b += (acc + gravity_body - coriolis) * dt;

        // 3. Attitude error: delta = delta + gyro * dt
        delta += gyro * dt;

        // 4. Covariance propagation: G * cov * G.T + R_proc
        let mut g_mat = nalgebra::SMatrix::<f32, 9, 9>::identity();
        let r_dt = r_mat.into_inner() * dt;
        for r in 0..3 {
            for c in 0..3 {
                g_mat[(r, c + 3)] = r_dt[(r, c)];
            }
        }
        g_mat[(3, 4)] += gyro.z * dt;
        g_mat[(3, 5)] -= gyro.y * dt;
        g_mat[(4, 3)] -= gyro.z * dt;
        g_mat[(4, 5)] += gyro.x * dt;
        g_mat[(5, 3)] += gyro.y * dt;
        g_mat[(5, 4)] -= gyro.x * dt;

        let r_proc = nalgebra::SMatrix::<f32, 9, 9>::from_diagonal(
            &nalgebra::SVector::<f32, 9>::from_row_slice(&[
                0.0, 0.0, 0.0, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
            ]),
        );
        cov = g_mat * cov * g_mat.transpose() + r_proc;

        // 5. Reset Step
        let q_delta = nalgebra::Quaternion::new(0.0, delta.x, delta.y, delta.z);
        let q_curr = q.into_inner();
        let q_dot_val = q_curr * q_delta;
        let q_new_quat = nalgebra::Quaternion::new(
            q_curr.w + 0.5 * q_dot_val.w,
            q_curr.i + 0.5 * q_dot_val.i,
            q_curr.j + 0.5 * q_dot_val.j,
            q_curr.k + 0.5 * q_dot_val.k,
        );
        q = nalgebra::UnitQuaternion::from_quaternion(q_new_quat);

        // 6. Height measurement update
        let exp_point_a = 2.5f32;
        let exp_std_a = 0.0025f32;
        let exp_point_b = 4.0f32;
        let exp_std_b = 0.2f32;
        let exp_coeff = libm::logf(exp_std_b / exp_std_a) / (exp_point_b - exp_point_a);
        let std_dev = exp_std_a * (1.0 + libm::expf(exp_coeff * (p.z - exp_point_a)));
        let q_height = std_dev * std_dev;

        let denom = q_height + cov[(2, 2)];
        let z_diff = zrange - p.z;

        for i in 0..3 {
            let k_p = cov[(i, 2)] / denom;
            let k_v = cov[(i + 3, 2)] / denom;
            p[i] += k_p * z_diff;
            v_b[i] += k_v * z_diff;
        }

        let mut i_kh = nalgebra::SMatrix::<f32, 9, 9>::identity();
        for i in 0..9 {
            i_kh[(i, 2)] -= cov[(i, 2)] / denom;
        }
        cov = i_kh * cov;
        let _ = cov;

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

export_tasks!(
    Nalgebra,
    MatMul3x3 => MatMul3x3Logic,
    RotateVector => RotateVectorLogic,
    MatInverse3x3 => MatInverse3x3Logic,
    UnitQuatMul => UnitQuatMulLogic,
    QuatSlerp => QuatSlerpLogic,
    LeeController => LeeControllerLogic,
    EkfStep => EkfStepLogic,
);
