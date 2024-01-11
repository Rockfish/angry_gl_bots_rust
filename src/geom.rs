use glam::{mat3, Vec3};

pub fn distance_between_point_and_line_segment(point: &Vec3, a: &Vec3, b: &Vec3) -> f32 {
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

pub fn distance_between_line_segments(a0: &Vec3, a1: &Vec3, b0: &Vec3, b1: &Vec3) -> f32 {
    let eps = 0.001f32;

    let a = *a1 - *a0;
    let b = *b1 - *b0;
    let mag_a = a.length();
    let mag_b = b.length();

    let a = a / mag_a;
    let b = b / mag_b;

    let cross = a.cross(b);
    let cl = cross.length();
    let denom = cl * cl;

    // If lines are parallel (denom=0) test if lines overlap.
    // If they don't overlap then there is a closest point solution.
    // If they do overlap, there are infinite closest positions, but there is a closest distance
    if denom < eps {
        let d0 = a.dot(*b0 - *a0);
        let d1 = a.dot(*b1 - *a0);

        // Is segment B before A?
        if d0 <= 0.0 && 0.0 >= d1 {
            if d0.abs() < d1.abs() {
                return (*a0 - *b0).length();
            }
            return (*a0 - *b1).length();
        } else if d0 >= mag_a && mag_a <= d1 {
            if d0.abs() < d1.abs() {
                return (*a1 - *b0).length();
            }
            return (*a1 - *b1).length();
        }

        // Segments overlap, return distance between parallel segments
        return (((d0 * a) + *a0) - *b0).length();
    }

    // Lines criss-cross: Calculate the projected closest points
    let t = *b0 - *a0;
    let det_a = (mat3(t, b, cross)).determinant();
    let det_b = (mat3(t, a, cross)).determinant();

    let t0 = det_a / denom;
    let t1 = det_b / denom;

    let mut p_a = *a0 + (a * t0); // Projected closest point on segment A
    let mut p_b = *b0 + (b * t1); // Projected closest point on segment B

    // Clamp projections
    if t0 < 0.0 {
        p_a = *a0;
    } else if t0 > mag_a {
        p_a = *a1;
    }

    if t1 < 0.0 {
        p_b = *b0;
    } else if t1 > mag_b {
        p_b = *b1;
    }

    // Clamp projection A
    if t0 < 0.0 || t0 > mag_a {
        let mut dot = b.dot(p_a - *b0);
        if dot < 0.0 {
            dot = 0.0;
        } else if dot > mag_b {
            dot = mag_b;
        }
        p_b = *b0 + (b * dot);
    }

    // Clamp projection B
    if t1 < 0.0 || t1 > mag_b {
        let mut dot = a.dot(p_b - *a0);
        if dot < 0.0 {
            dot = 0.0;
        } else if dot > mag_a {
            dot = mag_a;
        }
        p_a = *a0 + (a * dot);
    }

    return (p_a - p_b).length();
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
