use std::rc::Rc;
use glam::{Vec2, Vec3};

pub struct Player {
    pub lastFireTime: f32,
    pub isTryingToFire: bool,
    pub isAlive: bool,
    pub aimTheta: f32,
    pub position: Vec3,
    pub movementDir: Vec2,
    pub animation_name: String,
}