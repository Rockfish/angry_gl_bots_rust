use crate::aabb::AABB;
use crate::capsule::Capsule;
use crate::enemy::{Enemy, ENEMY_COLLIDER};
use crate::geom::{distanceBetweenLineSegments, oriented_angle};
use crate::sprite_sheet::SpriteSheetSprite;
use crate::texture_cache::TextureCache;
use crate::State;
use glam::{vec3, Mat4, Quat, Vec3};
use small_gl_core::gl::{GLsizei, GLsizeiptr, GLuint, GLvoid};
use small_gl_core::shader::Shader;
use small_gl_core::texture::{Texture, TextureConfig, TextureFilter, TextureType, TextureWrap};
use small_gl_core::{gl, SIZE_OF_FLOAT, SIZE_OF_QUAT, SIZE_OF_VEC3};
use std::f32::consts::PI;
use std::rc::Rc;

pub struct BulletGroup {
    start_index: usize,
    group_size: i32,
    time_to_live: f32,
}

impl BulletGroup {
    pub fn new(start_index: usize, group_size: i32, time_to_live: f32) -> BulletGroup {
        BulletGroup {
            start_index,
            group_size,
            time_to_live,
        }
    }
}

pub struct BulletStore {
    all_bullet_positions: Vec<Vec3>,
    all_bullet_quats: Vec<Quat>,
    all_bullet_directions: Vec<Vec3>,
    // thread_pool
    bullet_vao: GLuint,
    instance_vbo: GLuint,
    offset_vbo: GLuint,
    bullet_groups: Vec<BulletGroup>,
    pub texUnit_bullet: Rc<Texture>,
}

//const bulletScale: f32 = 0.3;
const bulletScale: f32 = 1.0;
const bulletLifetime: f32 = 1.0;
// seconds
const bulletSpeed: f32 = 15.0;
// Game units per second
const Rotation_Per_Bullet: f32 = 3.0 * PI / 180.0;

const scaleVec: Vec3 = vec3(bulletScale, bulletScale, bulletScale);
const bulletNormal: Vec3 = vec3(0.0, 1.0, 0.0);
const canonicalDir: Vec3 = vec3(0.0, 0.0, 1.0);

const BULLET_COLLIDER: Capsule = Capsule { height: 0.3, radius: 0.03 };

const bulletEnemyMaxCollisionDist: f32 = BULLET_COLLIDER.height / 2.0 + BULLET_COLLIDER.radius + ENEMY_COLLIDER.height / 2.0 + ENEMY_COLLIDER.radius;

#[rustfmt::skip]
const bullet_vertices: [f32; 20] = [
    // Positions                                        // Tex Coords
    bulletScale * (-0.243), 0.1, bulletScale * (-0.5),  1.0, 0.0,
    bulletScale * (-0.243), 0.1, bulletScale * 0.5,     0.0, 0.0,
    bulletScale * 0.243,    0.1, bulletScale * 0.5,     0.0, 1.0,
    bulletScale * 0.243,    0.1, bulletScale * (-0.5),  1.0, 1.0
];

#[rustfmt::skip]
const bullet_indices: [i32; 6] = [
    0, 1, 2,
    0, 2, 3
];

impl BulletStore {
    pub fn new(texture_cache: &mut TextureCache) -> Self {
        // initialize_buffer_and_create
        let mut bullet_vao: GLuint = 0;
        let mut bullet_vbo: GLuint = 0;
        let mut bullet_ebo: GLuint = 0;

        let mut instance_vbo: GLuint = 0;
        let mut offset_vbo: GLuint = 0;

        let texture_config = TextureConfig {
            flip_v: false,
            gamma_correction: false,
            filter: TextureFilter::Linear,
            texture_type: TextureType::None,
            wrap: TextureWrap::Repeat,
        };

        // let texUnit_bullet = texture_cache
        //     .get_or_load_texture("assets/Models/Bullet/Textures/BulletTexture.png", &texture_config)
        //     .unwrap();

        let texUnit_bullet = texture_cache
            .get_or_load_texture("angrygl_assets/bullet/bullet_texture_transparent.png", &texture_config)
            .unwrap();

        unsafe {
            gl::GenVertexArrays(1, &mut bullet_vao);

            gl::GenBuffers(1, &mut bullet_vbo);
            gl::GenBuffers(1, &mut bullet_ebo);

            gl::BindVertexArray(bullet_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, bullet_vbo);

            // vertices data
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (bullet_vertices.len() * SIZE_OF_FLOAT) as GLsizeiptr,
                bullet_vertices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            // indices data
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, bullet_ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (bullet_indices.len() * SIZE_OF_FLOAT) as GLsizeiptr,
                bullet_indices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            // location 0: positions
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (5 * SIZE_OF_FLOAT) as GLsizei, std::ptr::null::<GLvoid>());

            // location 1: texture coordinates
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                (5 * SIZE_OF_FLOAT) as GLsizei,
                (3 * SIZE_OF_FLOAT) as *const GLvoid,
            );

            // instance vbo
            gl::GenBuffers(1, &mut instance_vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, instance_vbo);

            // location: 2: rotations
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, SIZE_OF_QUAT as GLsizei, std::ptr::null::<GLvoid>());
            gl::VertexAttribDivisor(2, 1);

            // offset vbo
            gl::GenBuffers(1, &mut offset_vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, offset_vbo);

            // location: 3: position offsets
            gl::EnableVertexAttribArray(3);
            gl::VertexAttribPointer(3, 3, gl::FLOAT, gl::FALSE, SIZE_OF_VEC3 as GLsizei, std::ptr::null::<GLvoid>());
            gl::VertexAttribDivisor(3, 1);

            println!("SIZE_OF_FLOAT: {}", SIZE_OF_FLOAT);
            println!("SIZE_OF_QUAT: {}", SIZE_OF_QUAT);
            println!("SIZE_OF_VEC3: {}", SIZE_OF_VEC3);
        }

        BulletStore {
            all_bullet_positions: Default::default(),
            all_bullet_quats: Default::default(),
            all_bullet_directions: Default::default(),
            bullet_vao,
            instance_vbo,
            offset_vbo,
            bullet_groups: vec![],
            texUnit_bullet,
        }
    }

    pub fn create_bullets(&mut self, position: Vec3, midDir: Vec3, spreadAmount: i32) {

        let direction = midDir.normalize_or_zero();

        // let x = vec3(canonicalDir.x, 0.0, canonicalDir.z).normalize_or_zero();
        let x = vec3(0.0, 0.0, 1.0); // canonical direction
        let y = vec3(direction.x, 0.0, direction.z).normalize_or_zero();
        let rotVec = vec3(0.0, 1.0, 0.0); // rotate around y

        // direction angle with respect to the canonical direction
        let theta = oriented_angle(x, y, rotVec);

        let mut midDirQuat = Quat::from_xyzw(1.0, 0.0, 0.0, 0.0);
        //midDirQuat = glm::rotate(midDirQuat, theta, rotVec);
        midDirQuat *= Quat::from_axis_angle(rotVec, theta.to_radians());

        let startIndex = self.all_bullet_positions.len();

        let bulletGroupSize = spreadAmount * spreadAmount;

        let bullet_group = BulletGroup::new(startIndex, bulletGroupSize, bulletLifetime);

        self.all_bullet_positions.resize(startIndex + bulletGroupSize as usize, Vec3::default());
        self.all_bullet_quats.resize(startIndex + bulletGroupSize as usize, Quat::default());
        self.all_bullet_directions.resize(startIndex + bulletGroupSize as usize, Vec3::default());

        let parallelism = 1; // threadPool->numWorkers();
        let workerGroupSize = spreadAmount / parallelism;

        for p in 0..parallelism {
            let iStart = p * workerGroupSize;

            let iEnd = if p == (parallelism - 1) {
                spreadAmount
            } else {
                iStart + workerGroupSize
            };

            // futures.emplace_back(threadPool->enqueue([this, &position, &midDirQuat, spreadAmount, startIndex, &g, iStart, iEnd]() {

            for i in iStart..iEnd {
                let yQuat = midDirQuat * Quat::from_axis_angle(vec3(0.0, 1.0, 0.0), Rotation_Per_Bullet * ((i - spreadAmount) as f32 / 2.0));

                for j in 0..spreadAmount {
                    let rotQuat = yQuat * Quat::from_axis_angle(vec3(1.0, 0.0, 0.0), Rotation_Per_Bullet * ((j - spreadAmount) as f32 / 2.0));

                    let dir_glam = rotQuat.mul_vec3(canonicalDir);
                    // let dir = rotate_by_quat(&canonicalDir, &rotQuat);

                    let pos = (i * spreadAmount + j) as usize + startIndex;

                    self.all_bullet_positions[pos] = position;
                    self.all_bullet_directions[pos] = dir_glam;
                    // self.all_bullet_quats[pos] = Quat::IDENTITY;
                    self.all_bullet_quats[pos] = rotQuat;
                }
            }
        }

        self.bullet_groups.push(bullet_group);
    }
    pub fn update_bullets(&mut self, state: &mut State, enemyDeathSprites: &mut Vec<SpriteSheetSprite>) {
        let use_aabb = state.enemies.len() > 0;
        let num_sub_groups = if use_aabb { 9 } else { 1 };

        let delta_position_magnitude = state.delta_time * bulletSpeed;

        let mut first_live_bullet_group: usize = 0;

        for group in self.bullet_groups.iter_mut() {
            group.time_to_live -= state.delta_time;

            if group.time_to_live <= 0.0 {
                first_live_bullet_group += 1;
            } else {
                // could make this async
                let bullet_group_start_index = group.start_index as i32;
                let num_bullets_in_group = group.group_size;
                let sub_group_size = num_bullets_in_group / num_sub_groups;

                for sub_group in 0..num_sub_groups {
                    let mut bullet_start = sub_group_size * sub_group;

                    let mut bullet_end = if sub_group == (num_sub_groups - 1) {
                        num_bullets_in_group
                    } else {
                        bullet_start + sub_group_size
                    };

                    bullet_start += bullet_group_start_index;
                    bullet_end += bullet_group_start_index;

                    for bullet_index in bullet_start..bullet_end {
                        self.all_bullet_positions[bullet_index as usize] +=
                            self.all_bullet_directions[bullet_index as usize] * delta_position_magnitude;
                    }

                    let mut subgroup_bound_box = AABB::new();

                    if use_aabb {
                        for bullet_index in bullet_start..bullet_end {
                            subgroup_bound_box.expand_to_include(self.all_bullet_positions[bullet_index as usize]);
                        }

                        subgroup_bound_box.expand_by(bulletEnemyMaxCollisionDist);
                    }

                    for i in 0..state.enemies.len() {
                        let enemy = &mut state.enemies[i];

                        if use_aabb && !subgroup_bound_box.contains_point(enemy.position) {
                            continue;
                        }
                        for bullet_index in bullet_start..bullet_end {
                            if bullet_collides_with_enemy(
                                &self.all_bullet_positions[bullet_index as usize],
                                &self.all_bullet_directions[bullet_index as usize],
                                enemy,
                            ) {
                                println!("killed enemy!");
                                enemy.is_alive = false;
                                break;
                            }
                        }
                    }
                }
            }
        }

        let mut first_live_bullet: usize = 0;

        if first_live_bullet_group != 0 {
            first_live_bullet =
                self.bullet_groups[first_live_bullet_group - 1].start_index + self.bullet_groups[first_live_bullet_group - 1].group_size as usize;
            self.bullet_groups.drain(0..first_live_bullet_group);
        }

        if first_live_bullet != 0 {
            self.all_bullet_positions.drain(0..first_live_bullet);
            self.all_bullet_directions.drain(0..first_live_bullet);
            self.all_bullet_quats.drain(0..first_live_bullet);

            for group in self.bullet_groups.iter_mut() {
                group.start_index -= first_live_bullet;
            }
        }

        for enemy in state.enemies.iter() {
            if !enemy.is_alive {
                enemyDeathSprites.push(SpriteSheetSprite::new(enemy.position));
            }
        }

        state.enemies.retain(|e| e.is_alive);
    }

    pub fn draw_bullets(&mut self, shader: &Rc<Shader>, projectionView: &Mat4) {
        // self.all_bullet_positions.clear();
        // self.all_quats.clear();
        //
        // let position = vec3(0.0, 1.0, 0.0);
        //
        // self.all_bullet_positions.push(position);
        // self.all_quats.push(Quat::IDENTITY);

        unsafe {
            gl::Enable(gl::BLEND);
            gl::DepthMask(gl::FALSE);
            gl::ActiveTexture(gl::TEXTURE0 + self.texUnit_bullet.id);
            gl::BindTexture(gl::TEXTURE_2D, self.texUnit_bullet.id);
        }

        shader.use_shader();

        shader.set_int("texture_diffuse", self.texUnit_bullet.id as i32);
        shader.set_int("texture_normal", self.texUnit_bullet.id as i32);

        shader.set_bool("useLight", false);

        shader.set_mat4("PV", projectionView);

        // let scaled_pv = *projectionView * Mat4::from_scale(vec3(2.0, 2.0, 2.0));
        // shader.set_mat4("PV", &scaled_pv);

        self.renderBulletSprites();

        unsafe {
            gl::Disable(gl::BLEND);
            gl::DepthMask(gl::TRUE);
        }
    }

    pub fn renderBulletSprites(&self) {
        // if self.bullet_groups.is_empty() {
        //     return;
        // }

        unsafe {
            gl::BindVertexArray(self.bullet_vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, self.instance_vbo);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.all_bullet_quats.len() * SIZE_OF_QUAT) as GLsizeiptr,
                self.all_bullet_quats.as_ptr() as *const GLvoid,
                gl::STREAM_DRAW,
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, self.offset_vbo);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.all_bullet_positions.len() * SIZE_OF_VEC3) as GLsizeiptr,
                self.all_bullet_positions.as_ptr() as *const GLvoid,
                gl::STREAM_DRAW,
            );

            gl::DrawElementsInstanced(
                gl::TRIANGLES,
                6,
                gl::UNSIGNED_INT,
                0 as *const GLvoid,
                self.all_bullet_positions.len() as GLsizei,
            );
        }
    }
}

fn bullet_collides_with_enemy(position: &Vec3, direction: &Vec3, enemy: &Enemy) -> bool {
    if position.distance(enemy.position) > bulletEnemyMaxCollisionDist {
        false;
    }

    let a0 = *position - *direction * (BULLET_COLLIDER.height / 2.0);
    let a1 = *position + *direction * (BULLET_COLLIDER.height / 2.0);
    let b0 = enemy.position - enemy.dir * (ENEMY_COLLIDER.height / 2.0);
    let b1 = enemy.position + enemy.dir * (ENEMY_COLLIDER.height / 2.0);

    let closet_distance = distanceBetweenLineSegments(&a0, &a1, &b0, &b1);

    closet_distance <= (BULLET_COLLIDER.radius + ENEMY_COLLIDER.radius)
}

pub fn rotate_by_quat(v: &Vec3, q: &Quat) -> Vec3 {
    let qPrime = Quat::from_xyzw(q.w, -q.x, -q.y, -q.z);
    return partialHamiltonProduct(&partial_hamilton_product2(&q, &v), &qPrime);
}

pub fn partial_hamilton_product2(quat: &Quat, vec: &Vec3 /*partial*/) -> Quat {
    Quat::from_xyzw(
        quat.w * vec.x + quat.y * vec.z - quat.z * vec.y,
        quat.w * vec.y - quat.x * vec.z + quat.z * vec.x,
        quat.w * vec.z + quat.x * vec.y - quat.y * vec.x,
        -quat.x * vec.x - quat.y * vec.y - quat.z * vec.z,
    )
}

pub fn partialHamiltonProduct(q1: &Quat, q2: &Quat) -> Vec3 {
    vec3(
        q1.w * q2.x + q1.x * q2.w + q1.y * q2.z - q1.z * q2.y,
        q1.w * q2.y - q1.x * q2.z + q1.y * q2.w + q1.z * q2.x,
        q1.w * q2.z + q1.x * q2.y - q1.y * q2.x + q1.z * q2.w,
    )
}

// -- from ChatGPT --
fn hamilton_product_quat_vec(quat: &Quat, vec: &Vec3) -> Quat {
    Quat {
        x:  quat.w * vec.x + quat.y * vec.z - quat.z * vec.y,
        y:  quat.w * vec.y - quat.x * vec.z + quat.z * vec.x,
        z:  quat.w * vec.z + quat.x * vec.y - quat.y * vec.x,
        w: -quat.x * vec.x - quat.y * vec.y - quat.z * vec.z,
    }
}

fn hamilton_product_quat_quat(first: Quat, other: &Quat) -> Quat {
    Quat {
        x: first.w * other.x + first.x * other.w + first.y * other.z - first.z * other.y,
        y: first.w * other.y - first.x * other.z + first.y * other.w + first.z * other.x,
        z: first.w * other.z + first.x * other.y - first.y * other.x + first.z * other.w,
        w: first.w * other.w - first.x * other.x - first.y * other.y - first.z * other.z,
    }
}

fn temp(q: Quat) {
    let v = q.xyz();
}
