#include <math.h>
#include "modules/interface/math3d.h"

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
