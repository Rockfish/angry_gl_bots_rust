use crate::State;
use glam::{vec2, vec3, Mat4, Vec2, Vec3};
use small_gl_core::animator::{AnimationClip, AnimationRepeat, WeightedAnimation};
use small_gl_core::hash_map::HashMap;
use small_gl_core::model::{Model, ModelBuilder};
use small_gl_core::shader::Shader;
use small_gl_core::texture::TextureType;
use std::f32::consts::PI;
use std::ops::Deref;
use std::rc::Rc;
use std::time::Duration;

const PLAYER_SPEED: f32 = 5.0;
// 1.5;
const ANIM_TRANSITION_TIME: f32 = 0.2;

const IDLE: &str = "idle";
const RIGHT: &str = "right";
const FORWARD: &str = "forward";
const BACK: &str = "back";
const LEFT: &str = "left";
const DEAD: &str = "dead";

pub struct Player {
    pub model: Model,
    pub position: Vec3,
    pub direction: Vec2,
    pub speed: f32,
    pub aim_theta: f32,
    pub last_fire_time: f32,
    pub is_trying_to_fire: bool,
    pub is_alive: bool,
    pub death_time: f32,
    pub animation_name: Rc<str>,
    pub animations: PlayerAnimations,
    pub anim_weights: AnimationWeights,
    pub anim_hash: HashMap<Rc<str>, Rc<AnimationClip>>,
}

pub struct PlayerAnimations {
    idle: Rc<AnimationClip>,
    right: Rc<AnimationClip>,
    forward: Rc<AnimationClip>,
    back: Rc<AnimationClip>,
    left: Rc<AnimationClip>,
    dead: Rc<AnimationClip>,
}

impl PlayerAnimations {
    pub fn new() -> Self {
        PlayerAnimations {
            idle: Rc::new(AnimationClip::new(55.0, 130.0, AnimationRepeat::Forever)),
            right: Rc::new(AnimationClip::new(184.0, 204.0, AnimationRepeat::Forever)),
            forward: Rc::new(AnimationClip::new(134.0, 154.0, AnimationRepeat::Forever)),
            back: Rc::new(AnimationClip::new(159.0, 179.0, AnimationRepeat::Forever)),
            left: Rc::new(AnimationClip::new(209.0, 229.0, AnimationRepeat::Forever)),
            dead: Rc::new(AnimationClip::new(234.0, 293.0, AnimationRepeat::Once)),
        }
    }

    pub fn get(&self, name: &str) -> &Rc<AnimationClip> {
        match name {
            "idle" => &self.idle,
            "right" => &self.right,
            "forward" => &self.forward,
            "back" => &self.back,
            "left" => &self.left,
            "dead" => &self.dead,
            _ => panic!("animation not found"),
        }
    }
}

#[derive(Debug)]
pub struct AnimationWeights {
    // Previous animation weights
    last_anim_time: f32,
    prev_idle_weight: f32,
    prev_right_weight: f32,
    prev_forward_weight: f32,
    prev_back_weight: f32,
    prev_left_weight: f32,
}

impl Default for AnimationWeights {
    fn default() -> Self {
        AnimationWeights {
            last_anim_time: 0.0,
            prev_idle_weight: 0.0,
            prev_right_weight: 0.0,
            prev_forward_weight: 0.0,
            prev_back_weight: 0.0,
            prev_left_weight: 0.0,
        }
    }
}

impl Player {
    pub fn new() -> Player {
        let player_model = ModelBuilder::new("player", "assets/Models/Player/Player.fbx")
            .add_texture("Player", TextureType::Diffuse, "Textures/Player_D.tga")
            .add_texture("Player", TextureType::Specular, "Textures/Player_M.tga")
            .add_texture("Player", TextureType::Emissive, "Textures/Player_E.tga")
            .add_texture("Player", TextureType::Normals, "Textures/Player_NRM.tga")
            .add_texture("Gun", TextureType::Diffuse, "Textures/Gun_D.tga")
            .add_texture("Gun", TextureType::Specular, "Textures/Gun_M.tga")
            .add_texture("Gun", TextureType::Emissive, "Textures/Gun_E.tga")
            .add_texture("Gun", TextureType::Normals, "Textures/Gun_NRM.tga")
            .build()
            .unwrap();

        let mut anim_hash: HashMap<Rc<str>, Rc<AnimationClip>> = HashMap::new();
        anim_hash.insert(Rc::from(IDLE), Rc::new(AnimationClip::new(55.0, 130.0, AnimationRepeat::Forever)));
        anim_hash.insert(Rc::from(FORWARD), Rc::new(AnimationClip::new(134.0, 154.0, AnimationRepeat::Forever)));
        anim_hash.insert(Rc::from(BACK), Rc::new(AnimationClip::new(159.0, 179.0, AnimationRepeat::Forever)));
        anim_hash.insert(Rc::from(RIGHT), Rc::new(AnimationClip::new(184.0, 204.0, AnimationRepeat::Forever)));
        anim_hash.insert(Rc::from(LEFT), Rc::new(AnimationClip::new(209.0, 229.0, AnimationRepeat::Forever)));
        anim_hash.insert(Rc::from(DEAD), Rc::new(AnimationClip::new(234.0, 293.0, AnimationRepeat::Once)));

        let animation_name = Rc::from("idle");

        let player = Player {
            model: player_model,
            last_fire_time: 0.0,
            is_trying_to_fire: false,
            is_alive: true,
            aim_theta: 0.0,
            position: vec3(0.0, 0.0, 0.0),
            direction: vec2(0.0, 0.0),
            death_time: -1.0,
            animation_name,
            speed: PLAYER_SPEED,
            animations: PlayerAnimations::new(),
            anim_weights: AnimationWeights::default(),
            anim_hash,
        };

        player.model.play_clip(&player.animations.idle);

        player
    }

    pub fn set_animation(&mut self, animation_name: &Rc<str>, seconds: u32) {
        if !self.animation_name.eq(animation_name) {
            self.animation_name = animation_name.clone();
            self.model
                .play_clip_with_transition(self.animations.get(self.animation_name.deref()), Duration::from_secs(seconds as u64));
        }
    }

    pub fn get_muzzle_position(&self, player_model_transform: &Mat4) -> Mat4 {
        // Position in original model of gun muzzle
        // let point_vec = vec3(197.0, 76.143, -3.054);
        let point_vec = vec3(191.04, 79.231, -3.4651); // center of muzzle

        let animator = self.model.animator.borrow();
        let gun_mesh = self.model.meshes.iter().find(|m| m.name.as_str() == "Gun").unwrap();

        let final_node_matrices = animator.final_node_matrices.borrow();

        let gun_transform = final_node_matrices.get(gun_mesh.id as usize).unwrap();

        let muzzle = *gun_transform * Mat4::from_translation(point_vec);

        // muzzle_transform
        *player_model_transform * muzzle
    }

    pub fn set_player_death_time(&mut self, time: f32) {
        if self.death_time < 0.0 {
            self.death_time = time;
        }
    }

    pub fn render(&self, shader: &Shader) {
        self.model.render(shader);
    }

    pub fn update(&mut self, state: &mut State, aim_theta: f32) {
        let weight_animations = self.update_animation_weights(self.direction, aim_theta, state.frame_time);
        self.model.play_weight_animations(weight_animations.as_slice(), state.frame_time);
    }

    fn update_animation_weights(&mut self, move_vec: Vec2, aim_theta: f32, frame_time: f32) -> [WeightedAnimation; 6] {
        let is_moving = move_vec.length_squared() > 0.1;

        let move_theta = (move_vec.x / move_vec.y).atan() + if move_vec.y < 0.0 { PI } else { 0.0 };
        let theta_delta = move_theta - aim_theta;
        let anim_move = vec2(theta_delta.sin(), theta_delta.cos());

        let anim_delta_time = frame_time - self.anim_weights.last_anim_time;
        self.anim_weights.last_anim_time = frame_time;

        let is_dead = self.death_time >= 0.0;

        self.anim_weights.prev_idle_weight = max(0.0, self.anim_weights.prev_idle_weight - anim_delta_time / ANIM_TRANSITION_TIME);
        self.anim_weights.prev_right_weight = max(0.0, self.anim_weights.prev_right_weight - anim_delta_time / ANIM_TRANSITION_TIME);
        self.anim_weights.prev_forward_weight = max(0.0, self.anim_weights.prev_forward_weight - anim_delta_time / ANIM_TRANSITION_TIME);
        self.anim_weights.prev_back_weight = max(0.0, self.anim_weights.prev_back_weight - anim_delta_time / ANIM_TRANSITION_TIME);
        self.anim_weights.prev_left_weight = max(0.0, self.anim_weights.prev_left_weight - anim_delta_time / ANIM_TRANSITION_TIME);

        let mut dead_weight = if is_dead { 1.0 } else { 0.0 };
        let mut idle_weight = self.anim_weights.prev_idle_weight + if is_moving || is_dead { 0.0f32 } else { 1.0 };
        let mut right_weight = self.anim_weights.prev_right_weight + if is_moving { clamp0(-anim_move.x) } else { 0.0 };
        let mut forward_weight = self.anim_weights.prev_forward_weight + if is_moving { clamp0(anim_move.y) } else { 0.0 };
        let mut back_weight = self.anim_weights.prev_back_weight + if is_moving { clamp0(-anim_move.y) } else { 0.0 };
        let mut left_weight = self.anim_weights.prev_left_weight + if is_moving { clamp0(anim_move.x) } else { 0.0 };

        let weight_sum = dead_weight + idle_weight + forward_weight + back_weight + right_weight + left_weight;
        dead_weight /= weight_sum;
        idle_weight /= weight_sum;
        forward_weight /= weight_sum;
        back_weight /= weight_sum;
        right_weight /= weight_sum;
        left_weight /= weight_sum;

        self.anim_weights.prev_idle_weight = max(self.anim_weights.prev_idle_weight, idle_weight);
        self.anim_weights.prev_right_weight = max(self.anim_weights.prev_right_weight, right_weight);
        self.anim_weights.prev_forward_weight = max(self.anim_weights.prev_forward_weight, forward_weight);
        self.anim_weights.prev_back_weight = max(self.anim_weights.prev_back_weight, back_weight);
        self.anim_weights.prev_left_weight = max(self.anim_weights.prev_left_weight, left_weight);

        // weighted animations
        [
            WeightedAnimation::new(idle_weight, 55.0, 130.0, 0.0, 0.0),
            WeightedAnimation::new(forward_weight, 134.0, 154.0, 0.0, 0.0),
            WeightedAnimation::new(back_weight, 159.0, 179.0, 10.0, 0.0),
            WeightedAnimation::new(right_weight, 184.0, 204.0, 10.0, 0.0),
            WeightedAnimation::new(left_weight, 209.0, 229.0, 0.0, 0.0),
            WeightedAnimation::new(dead_weight, 234.0, 293.0, 0.0, self.death_time),
        ]
    }
}

fn clamp0(value: f32) -> f32 {
    if value < 0.0001 {
        return 0.0;
    }
    value
}

fn max(a: f32, b: f32) -> f32 {
    if a > b { a } else { b }
}
