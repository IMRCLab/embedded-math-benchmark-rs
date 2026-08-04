#include <math.h>
#include <string.h>
#include "modules/interface/math3d.h"
#include "kalman_core.h"

void assertFail(char *exp, char *file, int line) {
    (void)exp; (void)file; (void)line;
}

// Non-inline wrapper functions to expose cmath3d's static inline functions to Rust FFI

struct mat33 cf_mmul(struct mat33 a, struct mat33 b) {
    return mmul(a, b);
}

struct vec cf_qrot(struct quat q, struct vec v) {
    return qvrot(q, v);
}

struct quat cf_qqmul(struct quat q, struct quat p) {
    return qqmul(q, p);
}

struct quat cf_qslerp(struct quat a, struct quat b, float t) {
    return qslerp(a, b, t);
}

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
) {
    kalmanCoreData_t coreData;
    kalmanCoreParams_t coreParams;
    kalmanCoreDefaultParams(&coreParams);

    memset(&coreData, 0, sizeof(coreData));
    coreData.S[KC_STATE_X] = pos[0];
    coreData.S[KC_STATE_Y] = pos[1];
    coreData.S[KC_STATE_Z] = pos[2];

    coreData.S[KC_STATE_PX] = vel_b[0];
    coreData.S[KC_STATE_PY] = vel_b[1];
    coreData.S[KC_STATE_PZ] = vel_b[2];

    coreData.S[KC_STATE_D0] = 0.0f;
    coreData.S[KC_STATE_D1] = 0.0f;
    coreData.S[KC_STATE_D2] = 0.0f;

    coreData.q[0] = quat[3]; // w
    coreData.q[1] = quat[0]; // x
    coreData.q[2] = quat[1]; // y
    coreData.q[3] = quat[2]; // z

    for (int i=0; i<9; i++) {
        for (int j=0; j<9; j++) {
            coreData.P[i][j] = cov_in[i*9 + j];
        }
    }
    arm_mat_init_f32(&coreData.Pm, KC_STATE_DIM, KC_STATE_DIM, (float *)coreData.P);

    Axis3f accAxis = { .x = acc[0], .y = acc[1], .z = acc[2] };
    Axis3f gyroAxis = { .x = gyro[0], .y = gyro[1], .z = gyro[2] };

    uint32_t dtMs = (uint32_t)(dt * 1000.0f);
    if (dtMs == 0) dtMs = 10;

    kalmanCorePredict(&coreData, &coreParams, &accAxis, &gyroAxis, dtMs, false);
    kalmanCoreFinalize(&coreData);

    float h_data[9] = {0, 0, 1.0f, 0, 0, 0, 0, 0, 0};
    arm_matrix_instance_f32 Hm;
    arm_mat_init_f32(&Hm, 1, 9, h_data);

    float expPointA = 2.5f;
    float expStdA = 0.0025f;
    float expPointB = 4.0f;
    float expStdB = 0.2f;
    float expCoeff = logf(expStdB / expStdA) / (expPointB - expPointA);
    float stdDev = expStdA * (1.0f + expf(expCoeff * (coreData.S[KC_STATE_Z] - expPointA)));

    float error = zrange - coreData.S[KC_STATE_Z];
    kalmanCoreScalarUpdate(&coreData, &Hm, error, stdDev);
    kalmanCoreFinalize(&coreData);

    out_state[0] = coreData.S[KC_STATE_X];
    out_state[1] = coreData.S[KC_STATE_Y];
    out_state[2] = coreData.S[KC_STATE_Z];

    out_state[3] = coreData.S[KC_STATE_PX];
    out_state[4] = coreData.S[KC_STATE_PY];
    out_state[5] = coreData.S[KC_STATE_PZ];

    out_state[6] = coreData.q[1]; // x
    out_state[7] = coreData.q[2]; // y
    out_state[8] = coreData.q[3]; // z
    out_state[9] = coreData.q[0]; // w
}
