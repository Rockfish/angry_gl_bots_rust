use glam::{vec3, Vec3};

pub struct Aabb {
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
    pub z_min: f32,
    pub z_max: f32,
    is_initialize: bool,
}

impl Aabb {
    pub fn new() -> Self {
        Aabb {
            x_min: f32::MAX,
            x_max: f32::MIN,
            y_min: f32::MAX,
            y_max: f32::MIN,
            z_min: f32::MAX,
            z_max: f32::MIN,
            is_initialize: false,
        }
    }

    pub fn expand_to_include(&mut self, v: Vec3) {
        self.x_min = self.x_min.min(v.x);
        self.x_max = self.x_max.max(v.x);
        self.y_min = self.y_min.min(v.y);
        self.y_max = self.y_max.max(v.y);
        self.z_min = self.z_min.min(v.z);
        self.z_max = self.z_max.max(v.z);
        self.is_initialize = true;
    }

    pub fn expand_by(&mut self, f: f32) {
        if self.is_initialize {
            self.x_min -= f;
            self.x_max += f;
            self.y_min -= f;
            self.y_max += f;
            self.z_min -= f;
            self.z_max += f;
        }
    }

    #[rustfmt::skip]
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.x_min
            && point.x <= self.x_max
            && point.y >= self.y_min
            && point.y <= self.y_max
            && point.z >= self.z_min
            && point.z <= self.z_max
    }
}

#[rustfmt::skip]
pub fn aabbs_intersect(a: &Aabb, b: &Aabb) -> bool {
    a.contains_point(vec3(b.x_min, b.y_min, b.z_min))
        || a.contains_point(vec3(b.x_min, b.y_min, b.z_max))
        || a.contains_point(vec3(b.x_min, b.y_max, b.z_min))
        || a.contains_point(vec3(b.x_min, b.y_max, b.z_max))
        || a.contains_point(vec3(b.x_max, b.y_min, b.z_min))
        || a.contains_point(vec3(b.x_max, b.y_min, b.z_max))
        || a.contains_point(vec3(b.x_max, b.y_max, b.z_min))
        || a.contains_point(vec3(b.x_max, b.y_max, b.z_max))
}
