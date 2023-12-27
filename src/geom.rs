use glam::{mat3, Vec3};

pub fn distanceBetweenPointAndLineSegment(point: &Vec3, a: &Vec3, b: &Vec3) -> f32 {
    let ab = *b - *a;
    let ap = *point - *a;
    if ap.dot(ab) <= 0.0 {
        return ap.length();
    }
    let bp = *point - *b;
    if bp.dot(ab) >= 0.0 {
        return bp.length();
    }
    return ab.cross(ap).length() / ab.length();
}

pub fn distanceBetweenLineSegments(a0: &Vec3, a1: &Vec3, b0: &Vec3, b1: &Vec3) -> f32 {
    let EPS = 0.001f32;

    let A = *a1 - *a0;
    let B = *b1 - *b0;
    let magA = A.length();
    let magB = B.length();

    let _A = A / magA;
    let _B = B / magB;

    let cross = _A.cross(_B);
    let cl = cross.length();
    let denom = cl * cl;

    // If lines are parallel (denom=0) test if lines overlap.
    // If they don't overlap then there is a closest point solution.
    // If they do overlap, there are infinite closest positions, but there is a closest distance
    if denom < EPS {
        let d0 = _A.dot(*b0 - *a0);
        let d1 = _A.dot(*b1 - *a0);

        // Is segment B before A?
        if d0 <= 0.0 && 0.0 >= d1 {
            if d0.abs() < d1.abs() {
                return (*a0 - *b0).length();
            }
            return (*a0 - *b1).length();
        } else if d0 >= magA && magA <= d1 {
            if d0.abs() < d1.abs() {
                return (*a1 - *b0).length();
            }
            return (*a1 - *b1).length();
        }

        // Segments overlap, return distance between parallel segments
        return (((d0 * _A) + *a0) - *b0).length();
    }

    // Lines criss-cross: Calculate the projected closest points
    let t = *b0 - *a0;
    let detA = (mat3(t, _B, cross)).determinant();
    let detB = (mat3(t, _A, cross)).determinant();

    let t0 = detA / denom;
    let t1 = detB / denom;

    let mut pA = *a0 + (_A * t0); // Projected closest point on segment A
    let mut pB = *b0 + (_B * t1); // Projected closest point on segment B

    // Clamp projections
    if t0 < 0.0 {
        pA = *a0;
    } else if t0 > magA {
        pA = *a1;
    }

    if t1 < 0.0 {
        pB = *b0;
    } else if t1 > magB {
        pB = *b1;
    }

    // Clamp projection A
    if t0 < 0.0 || t0 > magA {
        let mut dot = _B.dot(pA - *b0);
        if dot < 0.0 {
            dot = 0.0;
        } else if dot > magB {
            dot = magB;
        }
        pB = *b0 + (_B * dot);
    }

    // Clamp projection B
    if t1 < 0.0 || t1 > magB {
        let mut dot = _A.dot(pB - *a0);
        if dot < 0.0 {
            dot = 0.0;
        } else if dot > magA {
            dot = magA;
        }
        pA = *a0 + (_A * dot);
    }

    return (pA - pB).length();
}

/// See https://github.com/icaven/glm/blob/master/glm/gtx/vector_angle.inl
pub fn oriented_angle(x: Vec3, y: Vec3, ref_axis: Vec3) -> f32 {

    let angle = x.dot(y).acos().to_degrees();

    if ref_axis.dot(x.cross(y)) < 0.0 {
        -angle
    } else {
        angle
    }
}

/*
static inline SIMD_CFUNC simd_float2 simd_mix(simd_float2 x, simd_float2 y, simd_float2 t) {
  return x + t*(y - x);
}
 */