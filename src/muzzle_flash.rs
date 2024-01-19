use crate::sprite_sheet::SpriteSheet;
use glam::{vec3, Mat4};
use small_gl_core::gl;
use small_gl_core::gl::GLuint;
use small_gl_core::shader::Shader;
use small_gl_core::texture::{bind_texture, Texture, TextureConfig, TextureWrap};

pub struct MuzzleFlash {
    unit_square_vao: i32,
    muzzle_flash_impact_spritesheet: SpriteSheet,
    pub muzzle_flash_sprites_age: Vec<f32>,
}

impl MuzzleFlash {
    pub fn new(unit_square_vao: i32) -> Self {
        let texture_config = TextureConfig::new().set_wrap(TextureWrap::Repeat);
        let texture_muzzle_flash_sprite_sheet = Texture::new("angrygl_assets/Player/muzzle_spritesheet.png", &texture_config).unwrap();
        let muzzle_flash_impact_spritesheet = SpriteSheet::new(texture_muzzle_flash_sprite_sheet, 6, 0.03);

        MuzzleFlash {
            unit_square_vao,
            muzzle_flash_impact_spritesheet,
            muzzle_flash_sprites_age: vec![],
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        if !self.muzzle_flash_sprites_age.is_empty() {
            for i in 0..self.muzzle_flash_sprites_age.len() {
                self.muzzle_flash_sprites_age[i] += delta_time;
            }
            let max_age = self.muzzle_flash_impact_spritesheet.num_columns as f32 * self.muzzle_flash_impact_spritesheet.time_per_sprite;
            self.muzzle_flash_sprites_age.retain(|age| *age < max_age);
        }
    }

    pub fn get_min_age(&self) -> f32 {
        let mut min_age = 1000f32;
        for age in self.muzzle_flash_sprites_age.iter() {
            min_age = min_age.min(*age);
        }
        min_age
    }

    pub fn add_flash(&mut self) {
        self.muzzle_flash_sprites_age.push(0.0);
    }

    pub fn draw(&self, sprite_shader: &Shader, projection_view: &Mat4, muzzle_transform: &Mat4) {
        if self.muzzle_flash_sprites_age.is_empty() {
            return;
        }

        sprite_shader.use_shader();
        sprite_shader.set_mat4("PV", projection_view);

        unsafe {
            gl::Enable(gl::BLEND);
            gl::DepthMask(gl::FALSE);
            gl::BindVertexArray(self.unit_square_vao as GLuint);
        }

        bind_texture(sprite_shader, 0, "spritesheet", &self.muzzle_flash_impact_spritesheet.texture);

        sprite_shader.set_int("numCols", self.muzzle_flash_impact_spritesheet.num_columns);
        sprite_shader.set_float("timePerSprite", self.muzzle_flash_impact_spritesheet.time_per_sprite);

        let scale = 50.0f32;

        let mut model = *muzzle_transform * Mat4::from_scale(vec3(scale, scale, scale));

        model *= Mat4::from_rotation_x(-90.0f32.to_radians());
        model *= Mat4::from_translation(vec3(0.7f32, 0.0f32, 0.0f32)); // adjust for position in the texture

        sprite_shader.set_mat4("model", &model);

        for sprite_age in &self.muzzle_flash_sprites_age {
            sprite_shader.set_float("age", *sprite_age);
            unsafe {
                gl::DrawArrays(gl::TRIANGLES, 0, 6);
            }
        }

        unsafe {
            gl::Disable(gl::BLEND);
            gl::DepthMask(gl::TRUE);
        }
    }
}
