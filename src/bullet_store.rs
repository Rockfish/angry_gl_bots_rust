use crate::geom::oriented_angle;
use crate::sprite_sheet::SpriteSheetSprite;
use crate::State;
use glam::{vec3, Mat4, Quat, Vec3};
use small_gl_core::gl::{GLsizei, GLsizeiptr, GLuint, GLvoid};
use small_gl_core::shader::Shader;
use small_gl_core::texture::{Texture, TextureConfig, TextureFilter, TextureType, TextureWrap};
use small_gl_core::{gl, SIZE_OF_FLOAT, SIZE_OF_QUAT, SIZE_OF_VEC3};
use std::f32::consts::PI;
use std::rc::Rc;
use crate::texture_cache::TextureCache;

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
    all_quats: Vec<Quat>,
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
const rotPerBullet: f32 = 3.0 * PI / 180.0;

const scaleVec: Vec3 = vec3(bulletScale, bulletScale, bulletScale);
const bulletNormal: Vec3 = vec3(0.0, 1.0, 0.0);
const canonicalDir: Vec3 = vec3(0.0, 0.0, 1.0);

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

        let texUnit_bullet = texture_cache
            .get_or_load_texture("assets/Models/Bullet/Textures/BulletTexture.png", &texture_config)
            .unwrap();

        // let texUnit_bullet = texture_cache
        //     .get_or_load_texture("assets/Models/Floor N.png", &texture_config)
        //     .unwrap();

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
            all_quats: Default::default(),
            all_bullet_directions: Default::default(),
            bullet_vao,
            instance_vbo,
            offset_vbo,
            bullet_groups: vec![],
            texUnit_bullet
        }
    }

    pub fn create_bullets(&mut self, position: Vec3, midDir: Vec3, spreadAmount: i32) {

        let normalizedDir = midDir.normalize_or_zero();

        let mut midDirQuat = Quat::from_xyzw(1.0, 0.0, 0.0, 0.0);
        {
            let rotVec = vec3(0.0, 1.0, 0.0);
            let x = vec3(canonicalDir.x, 0.0, canonicalDir.z).normalize_or_zero();
            let y = vec3(normalizedDir.x, 0.0, normalizedDir.z).normalize_or_zero();

            let theta = oriented_angle(x, y, rotVec);

            //midDirQuat = glm::rotate(midDirQuat, theta, rotVec);
            midDirQuat *= Quat::from_axis_angle(rotVec, theta.to_radians());
        }

        let startIndex = self.all_bullet_positions.len();

        let bulletGroupSize = spreadAmount * spreadAmount;
        let g = BulletGroup::new(startIndex, bulletGroupSize, bulletLifetime);

        self.all_bullet_positions.resize(startIndex + bulletGroupSize as usize, Vec3::default());
        self.all_quats.resize(startIndex + bulletGroupSize as usize, Quat::default());
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
                let yQuat = midDirQuat * Quat::from_axis_angle(vec3(0.0, 1.0, 0.0), rotPerBullet * ((i - spreadAmount) as f32 / 2.0));

                for j in 0..spreadAmount {
                    let rotQuat = yQuat * Quat::from_axis_angle(vec3(1.0, 0.0, 0.0), rotPerBullet * ((j - spreadAmount) as f32 / 2.0));
                    let dir = rotate_by_quat(&canonicalDir, &rotQuat);
                    let pos = (i * spreadAmount + j) as usize + startIndex;
                    self.all_bullet_positions[pos] = position;
                    self.all_bullet_directions[pos] = dir;
                    self.all_quats[pos] = rotQuat;
                }
            }
        }
        //));

        // for ( auto & future : futures) {
        // future.get();
        // }
        self.bullet_groups.push(g);
    }
    pub(crate) fn update_bullets(&self, state: &State, bullet_impact_sprites: &Vec<SpriteSheetSprite>) {}

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
                (self.all_quats.len() * SIZE_OF_QUAT) as GLsizeiptr,
                self.all_quats.as_ptr() as *const GLvoid,
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

pub fn rotate_by_quat(v: &Vec3, q: &Quat) -> Vec3 {
    let qPrime = Quat::from_xyzw(q.w, -q.x, -q.y, -q.z);
    return partialHamiltonProduct(&partialHamiltonProduct2(&q, &v), &qPrime);
}

pub fn partialHamiltonProduct2(q1: &Quat, q2: &Vec3 /*partial*/) -> Quat {
    Quat::from_xyzw(
        -q1.x * q2.x - q1.y * q2.y - q1.z * q2.z,
        q1.w * q2.x + q1.y * q2.z - q1.z * q2.y,
        q1.w * q2.y - q1.x * q2.z + q1.z * q2.x,
        q1.w * q2.z + q1.x * q2.y - q1.y * q2.x,
    )
}

pub fn partialHamiltonProduct(q1: &Quat, q2: &Quat) -> Vec3 {
    vec3(
        q1.w * q2.x + q1.x * q2.w + q1.y * q2.z - q1.z * q2.y,
        q1.w * q2.y - q1.x * q2.z + q1.y * q2.w + q1.z * q2.x,
        q1.w * q2.z + q1.x * q2.y - q1.y * q2.x + q1.z * q2.w,
    )
}
