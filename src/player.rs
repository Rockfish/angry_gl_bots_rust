use std::f32::consts::PI;
use crate::State;
use glam::{Mat4, Vec2, vec2, Vec3, vec3};
use std::rc::Rc;
use std::time::Duration;
use small_gl_core::animator::{AnimationClip, AnimationRepeat};
use small_gl_core::hash_map::HashMap;
use small_gl_core::model::{Model, ModelBuilder};
use small_gl_core::shader::Shader;
use small_gl_core::texture::TextureType;

const PLAYER_SPEED: f32 = 1.5;

pub struct Player {
    pub model: Model,
    pub position: Vec3,
    pub direction: Vec2,
    pub speed: f32,
    pub aim_theta: f32,
    pub last_fire_time: f32,
    pub is_trying_to_fire: bool,
    pub is_alive: bool,
    pub animation_name: Rc<str>,
    pub animations: HashMap<Rc<str>, Rc<AnimationClip>>
}


#[allow(dead_code)]
#[derive(Debug)]
struct Weights {
    idle: f32,
    forward: f32,
    back: f32,
    right: f32,
    left: f32,
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

        let mut animations: HashMap<Rc<str>, Rc<AnimationClip>> = HashMap::new();
        animations.insert(Rc::from("idle"), Rc::new(AnimationClip::new(55.0, 130.0, AnimationRepeat::Forever)));
        animations.insert(Rc::from("forward"), Rc::new(AnimationClip::new(134.0, 154.0, AnimationRepeat::Forever)));
        animations.insert(Rc::from("backwards"), Rc::new(AnimationClip::new(159.0, 179.0, AnimationRepeat::Forever)));
        animations.insert(Rc::from("right"), Rc::new(AnimationClip::new(184.0, 204.0, AnimationRepeat::Forever)));
        animations.insert(Rc::from("left"), Rc::new(AnimationClip::new(209.0, 229.0, AnimationRepeat::Forever)));
        animations.insert(Rc::from("dying"), Rc::new(AnimationClip::new(234.0, 293.0, AnimationRepeat::Once)));

        let animation_name = Rc::from("idle");
        player_model.play_clip(&animations[&animation_name]);

        let player = Player {
            model: player_model,
            last_fire_time: 0.0,
            is_trying_to_fire: false,
            is_alive: false,
            aim_theta: 0.0,
            position: vec3(0.0, 0.0, 0.0),
            direction: vec2(0.0, 0.0),
            animation_name,
            speed: PLAYER_SPEED,
            animations,
        };

        player
    }

    pub fn set_animation(&mut self, animation_name: &Rc<str>, seconds: u32) {
        if !self.animation_name.eq(animation_name) {
            self.animation_name = animation_name.clone();
            self.model.play_clip_with_transition(&self.animations[&self.animation_name], Duration::from_secs(seconds as u64));
        }
    }

    pub fn get_muzzle_position(&self, player_model_transform: &Mat4) -> Mat4 {
        // Position in original model of gun muzzle
        // let point_vec = vec3(197.0, 76.143, -3.054);
        let point_vec = vec3(191.04, 79.231, -3.4651); // center of muzzle

        let meshes = self.model.meshes.borrow();
        let animator = self.model.animator.borrow();

        let gun_mesh = meshes.iter().find(|m| m.name.as_str() == "Gun").unwrap();
        let final_node_matrices = animator.final_node_matrices.borrow();

        let gun_transform = final_node_matrices.get(gun_mesh.id as usize).unwrap();

        let muzzle = *gun_transform * Mat4::from_translation(point_vec);

        let muzzle_transform = *player_model_transform * muzzle;

        muzzle_transform
    }

    pub fn update(&mut self, state: &mut State, move_vec: Vec2, aim_theta: f32) {
        self.model.update_animation(state.delta_time);
        // animate player's direction changes
    }


    pub fn render(&self, shader: &Shader) {
        self.model.render(shader);
    }


    fn calculate_weights(move_vec: Vec2, aim_theta: f32) -> Weights {

        let move_theta = (move_vec.x / move_vec.y).atan() + if move_vec.y < 0.0 { PI } else { 0.0 };
        let theta_delta = move_theta - aim_theta;
        let anim_move = vec2(theta_delta.sin(), theta_delta.cos());
        let moving = move_vec.length_squared() > 0.0001;

        Weights {
            idle: if moving { 0.0 } else { 1.0 },
            forward: clamp0(anim_move.y),
            back: clamp0(-anim_move.y),
            right: clamp0(-anim_move.x),
            left: clamp0(anim_move.x),
        }
    }
}

    fn clamp0(value: f32) -> f32 {
        if value < 0.0001 {
            return 0.0;
        }
        value
    }
