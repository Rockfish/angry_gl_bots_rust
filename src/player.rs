use crate::State;
use glam::{Vec2, Vec3};
use std::rc::Rc;

pub struct Player {
    pub lastFireTime: f32,
    pub isTryingToFire: bool,
    pub isAlive: bool,
    pub aimTheta: f32,
    pub position: Vec3,
    pub player_direction: Vec2,
    pub animation_name: String,
    pub speed: f32,
}

impl Player {
    pub fn new() -> Player {
        Player {
            lastFireTime: 0.0,
            isTryingToFire: false,
            isAlive: false,
            aimTheta: 0.0,
            position: Default::default(),
            player_direction: Default::default(),
            animation_name: "".to_string(),
            speed: 0.0,
        }
    }

    pub fn update_points_for_anim(&mut self, x: &mut State) {

        // animate player's direction changes
    }

    pub fn draw(&self) {}
}
