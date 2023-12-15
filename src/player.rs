use glam::{Vec2, Vec3};
use std::rc::Rc;
use crate::State;

pub struct Player {
    pub lastFireTime: f32,
    pub isTryingToFire: bool,
    pub isAlive: bool,
    pub aimTheta: f32,
    pub position: Vec3,
    pub movementDir: Vec2,
    pub animation_name: String,
}

impl Player {
    pub fn new() -> Player {
        Player {
            lastFireTime: 0.0,
            isTryingToFire: false,
            isAlive: false,
            aimTheta: 0.0,
            position: Default::default(),
            movementDir: Default::default(),
            animation_name: "".to_string(),
        }
    }

    pub fn update_points_for_anim(&mut self, x: &mut State) {

        // animate player's direction changes

    }
}
