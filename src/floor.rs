use glam::{vec3, Mat4};
use small_gl_core::gl::{GLsizei, GLsizeiptr, GLuint, GLvoid};
use small_gl_core::shader::Shader;
use small_gl_core::texture::{bind_texture, Texture, TextureConfig, TextureFilter, TextureType, TextureWrap};
use small_gl_core::{gl, null, SIZE_OF_FLOAT};

const FLOOR_SIZE: f32 = 100.0;
const TILE_SIZE: f32 = 1.0;
const NUM_TILE_WRAPS: f32 = FLOOR_SIZE / TILE_SIZE;

#[rustfmt::skip]
const FLOOR_VERTICES: [f32; 30] = [
    // Vertices                                // TexCoord
    -FLOOR_SIZE / 2.0, 0.0, -FLOOR_SIZE / 2.0, 0.0, 0.0,
    -FLOOR_SIZE / 2.0, 0.0,  FLOOR_SIZE / 2.0, NUM_TILE_WRAPS, 0.0,
     FLOOR_SIZE / 2.0, 0.0,  FLOOR_SIZE / 2.0, NUM_TILE_WRAPS, NUM_TILE_WRAPS,
    -FLOOR_SIZE / 2.0, 0.0, -FLOOR_SIZE / 2.0, 0.0, 0.0,
     FLOOR_SIZE / 2.0, 0.0,  FLOOR_SIZE / 2.0, NUM_TILE_WRAPS, NUM_TILE_WRAPS,
     FLOOR_SIZE / 2.0, 0.0, -FLOOR_SIZE / 2.0, 0.0, NUM_TILE_WRAPS
];

pub struct Floor {
    pub floor_vao: GLuint,
    pub floor_vbo: GLuint,
    pub texture_floor_diffuse: Texture,
    pub texture_floor_normal: Texture,
    pub texture_floor_spec: Texture,
}

impl Floor {
    pub fn new() -> Self {
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

        let mut floor_vao: GLuint = 0;
        let mut floor_vbo: GLuint = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut floor_vao);
            gl::GenBuffers(1, &mut floor_vbo);
            gl::BindVertexArray(floor_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, floor_vbo);
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
            floor_vao,
            floor_vbo,
            texture_floor_diffuse,
            texture_floor_normal,
            texture_floor_spec,
        }
    }

    pub fn draw(&self, shader: &Shader, projection_view: &Mat4) {
        shader.use_shader();

        bind_texture(shader, 0, "texture_diffuse", &self.texture_floor_diffuse);
        bind_texture(shader, 1, "texture_normal", &self.texture_floor_normal);
        bind_texture(shader, 2, "texture_spec", &self.texture_floor_spec);

        // angle floor
        let _model = Mat4::from_axis_angle(vec3(0.0, 1.0, 0.0), 45.0f32.to_radians());

        let model = Mat4::IDENTITY;

        shader.set_mat4("PV", projection_view);
        shader.set_mat4("model", &model);

        unsafe {
            gl::BindVertexArray(self.floor_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::BindVertexArray(0);
        }
    }
}
