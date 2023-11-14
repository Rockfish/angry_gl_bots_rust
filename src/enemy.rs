use crate::capsule::Capsule;
use glam::Vec3;

pub const ENEMY_COLLIDER: Capsule = Capsule {
    height: 0.4,
    radius: 0.08,
};

pub struct Enemy {
    pub position: Vec3,
    pub dir: Vec3,
}

impl Enemy {
    pub fn new(position: Vec3, dir: Vec3) -> Self {
        Enemy { position, dir }
    }
}
