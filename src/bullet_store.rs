use crate::geom::oriented_angle;
use glam::{vec3, Quat, Vec3};
use small_gl_core::gl::{GLsizei, GLsizeiptr, GLuint, GLvoid};
use small_gl_core::{gl, size_of_floats, SIZE_OF_FLOAT};
use std::f32::consts::PI;
use std::mem;
use crate::sprite_sheet::SpriteSheetSprite;
use crate::State;

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
}



const bulletScale: f32 = 0.3;
const bulletLifetime: f32 = 1.0;
// seconds
const bulletSpeed: f32 = 15.0;
// Game units per second
const rotPerBullet: f32 = 3.0 * PI / 180.0;

const scaleVec: Vec3 = vec3(bulletScale, bulletScale, bulletScale);
const bulletNormal: Vec3 = vec3(0.0, 1.0, 0.0);
const canonicalDir: Vec3 = vec3(0.0, 0.0, 1.0);

#[rustfmt::skip]
const bulletVertices: [f32; 20] = [
    // Positions                                         // Tex Coords
    bulletScale * (-0.243), 0.0, bulletScale * (-0.5), 1.0, 0.0,
    bulletScale * (-0.243), 0.0, bulletScale * 0.5, 0.0, 0.0,
    bulletScale * 0.243, 0.0, bulletScale * 0.5, 0.0, 1.0,
    bulletScale * 0.243, 0.0, bulletScale * (-0.5), 1.0, 1.0
];

#[rustfmt::skip]
const bulletIndices: [i32; 6] = [
    0, 1, 2,
    0, 2, 3
];

impl BulletStore {
    pub fn new(/* threadpool */) -> Self { // initialize_buffer_and_create
        let mut bullet_vao: GLuint = 0;
        let mut bullet_vbo: GLuint = 0;
        let mut bullet_ebo: GLuint = 0;
        let mut instance_vbo: GLuint = 0;
        let mut offset_vbo: GLuint = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut bullet_vao);
            gl::GenBuffers(1, &mut bullet_vbo);
            gl::GenBuffers(1, &mut bullet_ebo);
            gl::BindVertexArray(bullet_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, bullet_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size_of_floats!((bulletVertices.len())) as GLsizeiptr,
                bulletVertices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, bullet_ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                size_of_floats!(bulletIndices.len()) as GLsizeiptr,
                bulletIndices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (5 * SIZE_OF_FLOAT) as GLsizei, std::ptr::null::<GLvoid>());
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                (5 * SIZE_OF_FLOAT) as GLsizei,
                (3 * SIZE_OF_FLOAT) as *const GLvoid,
            );
            gl::EnableVertexAttribArray(1);

            gl::GenBuffers(1, &mut instance_vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, instance_vbo);
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, mem::size_of::<Quat> as GLsizei, std::ptr::null::<GLvoid>());
            gl::VertexAttribDivisor(2, 1);

            gl::GenBuffers(1, &mut offset_vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, offset_vbo);
            gl::EnableVertexAttribArray(3);
            gl::VertexAttribPointer(3, 3, gl::FLOAT, gl::FALSE, mem::size_of::<Vec3> as GLsizei, std::ptr::null::<GLvoid>());
            gl::VertexAttribDivisor(3, 1);
        }

        BulletStore {
            all_bullet_positions: Default::default(),
            all_quats: Default::default(),
            all_bullet_directions: Default::default(),
            bullet_vao,
            instance_vbo,
            offset_vbo,
            bullet_groups: vec![],
        }
    }

    pub fn create_bullets(&mut self, position: Vec3, midDir: Vec3, spreadAmount: i32) {
        let normalizedDir = midDir.normalize_or_zero();

        let mut midDirQuat = Quat::from_xyzw(1.0, 0.0, 0.0, 0.0);
        // TODO there's probably a more efficient way to calculate this...
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
    pub(crate) fn update_bullets(&self, state: &State, bullet_impact_sprites: &Vec<SpriteSheetSprite>) {

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
