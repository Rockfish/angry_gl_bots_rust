use crate::capsule::Capsule;
use crate::geom::distanceBetweenPointAndLineSegment;
use crate::{monsterSpeed, monsterY, playerCollisionRadius, State};
use glam::{vec2, vec3, Vec3};
use small_gl_core::utils::rand_float;

pub const ENEMY_COLLIDER: Capsule = Capsule { height: 0.4, radius: 0.08 };

pub struct Enemy {
    pub position: Vec3,
    pub dir: Vec3,
}

impl Enemy {
    pub fn new(position: Vec3, dir: Vec3) -> Self {
        Enemy { position, dir }
    }
}

const enemySpawnInterval: f32 = 1.0; // seconds
const spawnsPerInterval: i32 = 1;
const spawnRadius: f32 = 10.0; // from player

pub struct EnemySpawner {
    count_down: f32,
    monster_y: f32,
}

impl EnemySpawner {
    pub fn new(monster_y: f32) -> Self {
        EnemySpawner {
            count_down: enemySpawnInterval,
            monster_y,
        }
    }

    pub fn update(&mut self, state: &mut State) {
        self.count_down -= state.delta_time;
        if self.count_down <= 0.0 {
            for _i in 0..spawnsPerInterval {
                self.spawn_enemy(state)
            }
            self.count_down += enemySpawnInterval;
        }
    }

    pub fn spawn_enemy(&mut self, state: &mut State) {
        let theta = (rand_float() * 360.0).to_radians();
        let x = state.player.position.x + theta.sin() * spawnRadius;
        let z = state.player.position.z + theta.cos() * spawnRadius;
        state.enemies.push(Enemy::new(vec3(x, self.monster_y, z), vec3(0.0, 0.0, 1.0)));
    }
}

pub fn chase_player(state: &mut State) {
    let playerCollisionPosition = vec3(state.player.position.x, monsterY, state.player.position.z);

    for enemy in state.enemies.iter_mut() {
        let mut dir = state.player.position - enemy.position;
        dir.y = 0.0;
        enemy.dir = dir.normalize_or_zero();
        enemy.position += enemy.dir * state.delta_time * monsterSpeed;

        if state.player.isAlive {
            let p1 = enemy.position - enemy.dir * (ENEMY_COLLIDER.height / 2.0);
            let p2 = enemy.position + enemy.dir * (ENEMY_COLLIDER.height / 2.0);
            let dist = distanceBetweenPointAndLineSegment(&playerCollisionPosition, &p1, &p2);

            if dist <= (playerCollisionRadius + ENEMY_COLLIDER.radius) {
                println!("GOTTEM!");
                state.player.isAlive = false;
                state.player.movementDir = vec2(0.0, 0.0);
            }
        }
    }
}
