use crate::texture_cache::TextureCache;
use glam::{vec3, Mat4, Vec3};
use small_gl_core::gl::{GLsizei, GLsizeiptr, GLuint, GLvoid};
use small_gl_core::shader::Shader;
use small_gl_core::texture::{Texture, TextureConfig, TextureFilter, TextureType, TextureWrap};
use small_gl_core::{gl, null, size_of_floats, SIZE_OF_FLOAT};
use std::mem;
use std::rc::Rc;

const floorSize: f32 = 100.0;
const tileSize: f32 = 1.0;
const numTileWraps: f32 = floorSize / tileSize;

#[rustfmt::skip]
const floorVertices: [f32; 30] = [
    // Vertices                               // TexCoord
    -floorSize / 2.0, 0.0, -floorSize / 2.0, 0.0, 0.0,
    -floorSize / 2.0, 0.0, floorSize / 2.0, numTileWraps, 0.0,
    floorSize / 2.0, 0.0, floorSize / 2.0, numTileWraps, numTileWraps,
    -floorSize / 2.0, 0.0, -floorSize / 2.0, 0.0, 0.0,
    floorSize / 2.0, 0.0, floorSize / 2.0, numTileWraps, numTileWraps,
    floorSize / 2.0, 0.0, -floorSize / 2.0, 0.0, numTileWraps
];

pub struct Floor {
    pub floorVAO: GLuint,
    pub floorVBO: GLuint,
    pub shader: Rc<Shader>,
    pub texUnit_floorDiffuse: Rc<Texture>,
    pub texUnit_floorNormal: Rc<Texture>,
    pub texUnit_floorSpec: Rc<Texture>,
    pub texUnit_shadowMap: Rc<Texture>,
}

impl Floor {
    pub fn new(texture_cache: &mut TextureCache, shader: &Rc<Shader>) -> Self {
        let texture_config = TextureConfig {
            flip_v: false,
            gamma_correction: false,
            filter: TextureFilter::Linear,
            texture_type: TextureType::None,
            wrap: TextureWrap::Repeat,
        };

        let texUnit_floorDiffuse = texture_cache.get_or_load_texture("assets/Models/Floor D.png", &texture_config).unwrap();
        let texUnit_floorNormal = texture_cache.get_or_load_texture("assets/Models/Floor N.png", &texture_config).unwrap();
        let texUnit_floorSpec = texture_cache.get_or_load_texture("assets/Models/Floor M.png", &texture_config).unwrap();
        let texUnit_shadowMap = texture_cache.get_or_load_texture("assets/Models/Floor D.png", &texture_config).unwrap();

        let mut floorVAO: GLuint = 0;
        let mut floorVBO: GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut floorVAO);
            gl::GenBuffers(1, &mut floorVBO);
            gl::BindVertexArray(floorVAO);
            gl::BindBuffer(gl::ARRAY_BUFFER, floorVBO);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (floorVertices.len() * SIZE_OF_FLOAT) as GLsizeiptr,
                floorVertices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (5 * SIZE_OF_FLOAT) as GLsizei, null!());
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
        }

        Floor {
            floorVAO,
            floorVBO,
            shader: shader.clone(),
            texUnit_floorDiffuse,
            texUnit_floorNormal,
            texUnit_floorSpec,
            texUnit_shadowMap,
        }
    }

    pub fn draw(&self, projection_view: &Mat4, ambientColor: &Vec3) {
        //}, light_space_matrix: Option<Mat4>) {
        self.shader.use_shader();

        set_texture(&self.shader, 0, "texture_diffuse", &self.texUnit_floorDiffuse);
        set_texture(&self.shader, 1, "texture_normal", &self.texUnit_floorNormal);
        set_texture(&self.shader, 2, "texture_spec", &self.texUnit_floorSpec);
        set_texture(&self.shader, 3, "shadow_map", &self.texUnit_shadowMap);

        // self.shader.setBool("useLight", light_space_matrix.is_some());
        // self.shader.setBool("useSpec", light_space_matrix.is_some());
        // self.shader.setVec3("pointLight.worldPos", muzzleWorldPos3);
        // self.shader.setVec3("pointLight.color", muzzlePointLightColor);

        let model = Mat4::from_axis_angle(vec3(0.0, 1.0, 0.0), 45.0f32.to_radians());

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

pub fn set_texture(shader: &Rc<Shader>, texture_unit: i32, sample_name: &str, texture: &Rc<Texture>) {
    unsafe {
        gl::ActiveTexture(gl::TEXTURE0 + texture_unit as u32);
        gl::BindTexture(gl::TEXTURE_2D, texture.id);
        shader.set_int(sample_name, texture_unit);
    }
}
