#[cfg(feature = "crazyflie")]
pub mod crazyflie_fw;
pub mod glam;
pub mod libm;
pub mod micromath;
pub mod nalgebra;

pub mod constants {
    pub const KP: [f32; 3] = [7.0, 7.0, 7.0];
    pub const KD: [f32; 3] = [4.0, 4.0, 4.0];
    pub const K_R: [f32; 3] = [0.007, 0.007, 0.008];
    pub const K_W: [f32; 3] = [0.00115, 0.00115, 0.002];
    pub const GRAVITY: [f32; 3] = [0.0, 0.0, -9.81]; // matches crazyflie-fw's GRAVITY_MAGNITUDE
    pub const INERTIA: [f32; 3] = [1.657_171e-5, 16.655602e-6, 29.261652e-6];
}
