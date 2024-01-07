use std::f32::consts::{PI, TAU};
#[allow(dead_code)]

use glam::{Vec2, vec2};

#[allow(dead_code)]
#[derive(Debug)]
struct Weights {
    idle: f32,
    forward: f32,
    back: f32,
    right: f32,
    left: f32,
}



fn main() {

    // aim vec is always magnitude 1
    let aim_vec = vec2(1.0, 0.0);
    let move_vec = vec2(1.0, 0.0);
    let player_position = vec2(0.0, 0.0);
    let world_point = vec2(1.0, 0.0);

    let dx = world_point.x - player_position.x;
    let dz = world_point.y - player_position.y;

    let aim_theta = (dx / dz).atan() + if dz < 0.0 { PI } else { 0.0 };

    calculate_weights(aim_vec, vec2(1.0, 0.0), aim_theta);
    calculate_weights(aim_vec, vec2(1.0, 1.0), aim_theta);
    calculate_weights(aim_vec, vec2(0.0, 1.0), aim_theta);
    calculate_weights(aim_vec, vec2(0.0, -1.0), aim_theta);
    calculate_weights(aim_vec, vec2(-1.0, -1.0), aim_theta);
    calculate_weights(aim_vec, vec2(-1.0, 0.0), aim_theta);
}

fn calculate_weights(aim_vec: Vec2, move_vec: Vec2, aim_theta: f32) -> Weights {
    // let move_vec = move_vec.normalize_or_zero();

    let d = aim_vec.dot(move_vec);

    let move_theta = (move_vec.x / move_vec.y).atan() + if move_vec.y < 0.0 { PI } else { 0.0 };
    let theta_delta = move_theta - aim_theta;
    let anim_move = vec2(theta_delta.sin(), theta_delta.cos());

    let weights = Weights {
        idle: 0.0,
        forward: clamp0(anim_move.y),
        back: clamp0(-anim_move.y),
        right: clamp0(-anim_move.x),
        left: clamp0(anim_move.x),
    };

    println!("aim_vec: {:?}   move_vec: {:?}   dot: {:?}  move_theta: {:?}  anim_move: {:?}", aim_vec, move_vec, d, move_theta, anim_move);
    println!("weights: {:?}", weights);
    println!();

    weights
}

fn clamp0(value: f32) -> f32 {
    if value < 0.0001 {
        return 0.0;
    }
    value
}

fn clamp(value: f32) -> f32 {
    if value > 1.0 {
        return 1.0;
    } else if value < 0.0 {
       return 0.0;
    }
    value
}