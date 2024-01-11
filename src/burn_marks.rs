use glam::{vec3, Mat4, Vec3};
use small_gl_core::gl;
use small_gl_core::gl::GLuint;
use small_gl_core::shader::Shader;
use small_gl_core::texture::{bind_texture, Texture, TextureConfig, TextureWrap};


const BURN_MARK_TIME: f32 = 5.0;

pub struct BurnMark {
    position: Vec3,
    time_left: f32,
}

pub struct BurnMarks {
    unit_square_vao: i32,
    mark_texture: Texture,
    marks: Vec<BurnMark>,
}

impl BurnMarks {
    pub fn new(unit_square_vao: i32) -> Self {
        let texture_config = TextureConfig::new().set_wrap(TextureWrap::Repeat);
        let mark_texture = Texture::new("angrygl_assets/bullet/burn_mark.png", &texture_config).unwrap();

        BurnMarks {
            unit_square_vao,
            mark_texture,
            marks: vec![],
        }
    }

    pub fn add_mark(&mut self, position: Vec3) {
        self.marks.push(BurnMark {
            position,
            time_left: BURN_MARK_TIME,
        });
    }

    pub fn draw_marks(&mut self, shader: &Shader, projection_view: &Mat4, delta_time: f32) {
        if self.marks.is_empty() {
            return;
        }

        shader.use_shader();
        shader.set_mat4("PV", &projection_view);

        bind_texture(shader, 0, "texture_diffuse", &self.mark_texture);
        bind_texture(shader, 1, "texture_normal", &self.mark_texture);

        unsafe {
            gl::DepthMask(gl::FALSE);
            gl::Enable(gl::BLEND);
            gl::Disable(gl::CULL_FACE);
            gl::BindVertexArray(self.unit_square_vao as GLuint);
        }

        for mark in self.marks.iter_mut() {
            let scale: f32 = 0.5 * mark.time_left;
            mark.time_left -= delta_time;

            // model *= Mat4::from_translation(vec3(mark.x, 0.01, mark.z));
            let mut model = Mat4::from_translation(mark.position);

            model *= Mat4::from_rotation_x(-90.0f32.to_radians());
            model *= Mat4::from_scale(vec3(scale, scale, scale));

            shader.set_mat4("model", &model);

            unsafe {
                gl::DrawArrays(gl::TRIANGLES, 0, 6);
            }
        }

        self.marks.retain(|m| m.time_left > 0.0);

        unsafe {
            gl::Disable(gl::BLEND);
            gl::DepthMask(gl::TRUE);
            gl::Enable(gl::CULL_FACE);
        }
    }
}
