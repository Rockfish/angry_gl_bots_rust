use glam::{vec3, Mat4, Vec3};
use small_gl_core::gl::{GLsizei, GLsizeiptr, GLuint, GLvoid};
use small_gl_core::shader::Shader;
use small_gl_core::texture::{bind_texture, Texture, TextureConfig, TextureFilter, TextureType, TextureWrap};
use small_gl_core::{gl, null, SIZE_OF_FLOAT};
use std::rc::Rc;

const FLOOR_SIZE: f32 = 100.0;
const TILE_SIZE: f32 = 1.0;
const NUM_TILE_WRAPS: f32 = FLOOR_SIZE / TILE_SIZE;

#[rustfmt::skip]
const FLOOR_VERTICES: [f32; 30] = [
    // Vertices                               // TexCoord
    -FLOOR_SIZE / 2.0, 0.0, -FLOOR_SIZE / 2.0, 0.0, 0.0,
    -FLOOR_SIZE / 2.0, 0.0, FLOOR_SIZE / 2.0, NUM_TILE_WRAPS, 0.0,
    FLOOR_SIZE / 2.0, 0.0, FLOOR_SIZE / 2.0, NUM_TILE_WRAPS, NUM_TILE_WRAPS,
    -FLOOR_SIZE / 2.0, 0.0, -FLOOR_SIZE / 2.0, 0.0, 0.0,
    FLOOR_SIZE / 2.0, 0.0, FLOOR_SIZE / 2.0, NUM_TILE_WRAPS, NUM_TILE_WRAPS,
    FLOOR_SIZE / 2.0, 0.0, -FLOOR_SIZE / 2.0, 0.0, NUM_TILE_WRAPS
];

pub struct Floor {
    pub floorVAO: GLuint,
    pub floorVBO: GLuint,
    pub shader: Rc<Shader>,
    pub texture_floor_diffuse: Texture,
    pub texture_floor_normal: Texture,
    pub texture_floor_spec: Texture,
    pub texture_shadowMap: Texture,
}

impl Floor {
    pub fn new(shader: &Rc<Shader>) -> Self {
        let texture_config = TextureConfig {
            flip_v: false,
            flip_h: false,
            gamma_correction: false,
            filter: TextureFilter::Linear,
            texture_type: TextureType::None,
            wrap: TextureWrap::Repeat,
        };

        let texture_floor_diffuse = Texture::new("assets/Models/Floor D.png", &texture_config).unwrap();
        let texture_floor_normal = Texture::new("assets/Models/Floor N.png", &texture_config).unwrap();
        let texture_floor_spec = Texture::new("assets/Models/Floor M.png", &texture_config).unwrap();
        let texture_shadowMap = Texture::new("assets/Models/Floor D.png", &texture_config).unwrap();

        let mut floorVAO: GLuint = 0;
        let mut floorVBO: GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut floorVAO);
            gl::GenBuffers(1, &mut floorVBO);
            gl::BindVertexArray(floorVAO);
            gl::BindBuffer(gl::ARRAY_BUFFER, floorVBO);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (FLOOR_VERTICES.len() * SIZE_OF_FLOAT) as GLsizeiptr,
                FLOOR_VERTICES.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (5 * SIZE_OF_FLOAT) as GLsizei, null!());
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, (5 * SIZE_OF_FLOAT) as GLsizei, (3 * SIZE_OF_FLOAT) as *const GLvoid);
            gl::EnableVertexAttribArray(1);
        }

        Floor {
            floorVAO,
            floorVBO,
            shader: shader.clone(),
            texture_floor_diffuse,
            texture_floor_normal,
            texture_floor_spec,
            texture_shadowMap,
        }
    }

    pub fn draw(&self, projection_view: &Mat4, ambientColor: &Vec3) {
        //}, light_space_matrix: Option<Mat4>) {
        self.shader.use_shader();

        bind_texture(&self.shader, 0, "texture_diffuse", &self.texture_floor_diffuse);
        bind_texture(&self.shader, 1, "texture_normal", &self.texture_floor_normal);
        bind_texture(&self.shader, 2, "texture_spec", &self.texture_floor_spec);
        bind_texture(&self.shader, 3, "shadow_map", &self.texture_shadowMap);

        // self.shader.setBool("useLight", light_space_matrix.is_some());
        // self.shader.setBool("useSpec", light_space_matrix.is_some());
        // self.shader.setVec3("pointLight.worldPos", muzzleWorldPos3);
        // self.shader.setVec3("pointLight.color", muzzlePointLightColor);

        // angle floor
        let model = Mat4::from_axis_angle(vec3(0.0, 1.0, 0.0), 45.0f32.to_radians());

        let model = Mat4::IDENTITY;

        self.shader.set_bool("useLight", false);
        self.shader.set_vec3("ambient", ambientColor);
        // self.shader.setVec3("viewPos", camera_pos);
        self.shader.set_mat4("PV", projection_view);
        self.shader.set_mat4("model", &model);

        unsafe {
            gl::BindVertexArray(self.floorVAO);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }
    }
}
