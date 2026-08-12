#pragma once

#include <math.h>
#include "modules/interface/math3d.h"
#include "modules/interface/stabilizer_types.h"

// Declarations of our wrapper functions for static inline math3d.h functions
struct mat33 cf_mmul(struct mat33 a, struct mat33 b);
struct vec cf_qrot(struct quat q, struct vec v);
struct quat cf_qqmul(struct quat q, struct quat p);
struct quat cf_qslerp(struct quat a, struct quat b, float t);
struct vec cf_vcross(struct vec a, struct vec b);
struct vec cf_vnormalize(struct vec v);
struct vec cf_mvmul(struct mat33 a, struct vec v);
struct mat33 cf_quat2rotmat(struct quat q);
struct quat cf_qnormalize(struct quat q);

void cf_ekf_step(
    float pos[3],
    float vel_b[3],
    float quat[4],
    const float acc[3],
    const float gyro[3],
    float zrange,
    float dt,
    const float cov_in[81],
    float out_state[10]
);

void cmsis_matmul_9x9(const float a[81], const float b[81], float out[81]);
int cmsis_matinverse_9x9(const float in[81], float out[81]);
float cmsis_dotprod_64d(const float a[64], const float b[64]);

// Lee position/attitude controller (modules/src/controller/controller_lee.c),
// wrapped to take flat inputs and stop at thrust+torque.
struct cf_lee_output {
    float thrust;
    struct vec torque;
};

struct cf_lee_output cf_lee_controller(
    struct vec position,
    struct vec velocity,
    struct quat attitude,
    struct vec angular_velocity,
    struct vec setpoint_position,
    struct vec setpoint_velocity,
    struct vec setpoint_acceleration,
    float setpoint_yaw,
    float setpoint_yaw_dot,
    float mass
);
