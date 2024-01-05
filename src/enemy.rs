use crate::capsule::Capsule;
use crate::geom::distanceBetweenPointAndLineSegment;
use crate::{State, MONSTER_SPEED, MONSTER_Y, PLAYER_COLLISION_RADIUS};
use glam::{vec2, vec3, Mat4, Vec3};
use small_gl_core::model::Model;
use small_gl_core::shader::Shader;
use small_gl_core::utils::rand_float;
use std::f32::consts::PI;
use std::rc::Rc;

pub const ENEMY_COLLIDER: Capsule = Capsule { height: 0.4, radius: 0.08 };

pub struct Enemy {
    pub position: Vec3,
    pub dir: Vec3,
    pub is_alive: bool,
}

impl Enemy {
    pub fn new(position: Vec3, dir: Vec3) -> Self {
        Enemy { position, dir, is_alive: true }
    }
}

const ENEMY_SPAWN_INTERVAL: f32 = 1.0; // seconds
const SPAWNS_PER_INTERVAL: i32 = 1;
const SPAWN_RADIUS: f32 = 10.0; // from player

pub struct EnemySystem {
    count_down: f32,
    monster_y: f32,
}

impl EnemySystem {
    pub fn new(monster_y: f32) -> Self {
        EnemySystem {
            count_down: ENEMY_SPAWN_INTERVAL,
            monster_y,
        }
    }

    pub fn update(&mut self, state: &mut State) {
        self.count_down -= state.delta_time;
        if self.count_down <= 0.0 {
            for _i in 0..SPAWNS_PER_INTERVAL {
                self.spawn_enemy(state)
            }
            self.count_down += ENEMY_SPAWN_INTERVAL;
        }
    }

    pub fn spawn_enemy(&mut self, state: &mut State) {
        let theta = (rand_float() * 360.0).to_radians();
        let x = state.player.position.x + theta.sin() * SPAWN_RADIUS;
        let z = state.player.position.z + theta.cos() * SPAWN_RADIUS;
        state.enemies.push(Enemy::new(vec3(x, self.monster_y, z), vec3(0.0, 0.0, 1.0)));
    }

    pub fn chase_player(&self, state: &mut State) {
        let playerCollisionPosition = vec3(state.player.position.x, MONSTER_Y, state.player.position.z);

        for enemy in state.enemies.iter_mut() {
            let mut dir = state.player.position - enemy.position;
            dir.y = 0.0;
            enemy.dir = dir.normalize_or_zero();
            enemy.position += enemy.dir * state.delta_time * MONSTER_SPEED;

            if state.player.isAlive {
                let p1 = enemy.position - enemy.dir * (ENEMY_COLLIDER.height / 2.0);
                let p2 = enemy.position + enemy.dir * (ENEMY_COLLIDER.height / 2.0);
                let dist = distanceBetweenPointAndLineSegment(&playerCollisionPosition, &p1, &p2);

                if dist <= (PLAYER_COLLISION_RADIUS + ENEMY_COLLIDER.radius) {
                    // println!("GOTTEM!");
                    state.player.isAlive = false;
                    state.player.player_direction = vec2(0.0, 0.0);
                }
            }
        }
    }

    pub fn draw_enemies(&self, enemy_model: &Model, shader: &Shader, state: &mut State) {
        shader.use_shader();
        shader.set_vec3("nosePos", &vec3(1.0, MONSTER_Y, -2.0));
        shader.set_float("time", state.frame_time);

        // TODO optimise (multithreaded, instancing, SOA, etc..)
        for e in state.enemies.iter_mut() {
            let monsterTheta = (e.dir.x / e.dir.z).atan() + (if e.dir.z < 0.0 { 0.0 } else { PI });

            let mut model_transform = Mat4::from_translation(e.position);

            model_transform *= Mat4::from_scale(Vec3::splat(0.01));
            model_transform *= Mat4::from_axis_angle(vec3(0.0, 1.0, 0.0), monsterTheta);
            model_transform *= Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), PI);
            model_transform *= Mat4::from_axis_angle(vec3(1.0, 0.0, 0.0), 90.0f32.to_radians());

            let mut rot_only = Mat4::from_axis_angle(vec3(0.0, 1.0, 0.0), monsterTheta);
            rot_only = Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), PI);
            rot_only = Mat4::from_axis_angle(vec3(1.0, 0.0, 0.0), 90.0f32.to_radians());

            shader.set_mat4("aimRot", &rot_only);
            shader.set_mat4("model", &model_transform);

            enemy_model.render(shader);
        }
    }
}
