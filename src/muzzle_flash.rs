use crate::sprite_sheet::SpriteSheet;
use glam::{vec3, Mat4};
use small_gl_core::gl;
use small_gl_core::gl::GLuint;
use small_gl_core::model::Model;
use small_gl_core::shader::Shader;
use small_gl_core::texture::{bind_texture, Texture, TextureConfig, TextureWrap};
use std::rc::Rc;

pub struct MuzzleFlash {
    unitSquareVAO: i32,
    muzzleFlashImpactSpritesheet: SpriteSheet,
    pub muzzleFlashSpritesAge: Vec<f32>,
}

impl MuzzleFlash {
    pub fn new(unitSquareVAO: i32) -> Self {
        let texture_config = TextureConfig::new().set_wrap(TextureWrap::Repeat);
        let texture_muzzleFlashSpriteSheet = Texture::new("angrygl_assets/Player/muzzle_spritesheet.png", &texture_config).unwrap();
        let muzzleFlashImpactSpritesheet = SpriteSheet::new(texture_muzzleFlashSpriteSheet, 6, 0.05);

        MuzzleFlash {
            unitSquareVAO,
            muzzleFlashImpactSpritesheet,
            muzzleFlashSpritesAge: vec![],
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        if &self.muzzleFlashSpritesAge.len() > &0 {
            for i in 0..self.muzzleFlashSpritesAge.len() {
                self.muzzleFlashSpritesAge[i] += delta_time;
            }
            let max_age = self.muzzleFlashImpactSpritesheet.num_columns as f32 * self.muzzleFlashImpactSpritesheet.time_per_sprite;
            self.muzzleFlashSpritesAge.retain(|age| *age < max_age);
        }
    }

    pub fn get_min_age(&self) -> f32 {
        let mut min_age = 1000f32;
        for age in self.muzzleFlashSpritesAge.iter() {
            min_age = min_age.min(*age);
        }
        min_age
    }

    pub fn add_flash(&mut self) {
        self.muzzleFlashSpritesAge.push(0.0);
    }

    pub fn draw(&self, sprite_shader: &Rc<Shader>, PV: &Mat4, muzzleTransform: &Mat4) {
        if self.muzzleFlashSpritesAge.is_empty() {
            return;
        }

        sprite_shader.use_shader();
        sprite_shader.set_mat4("PV", &PV);

        unsafe {
            gl::Enable(gl::BLEND);
            gl::DepthMask(gl::FALSE);
            gl::BindVertexArray(self.unitSquareVAO as GLuint);
        }

        bind_texture(sprite_shader, 0, "spritesheet", &self.muzzleFlashImpactSpritesheet.texture);

        sprite_shader.set_int("numCols", self.muzzleFlashImpactSpritesheet.num_columns);
        sprite_shader.set_float("timePerSprite", self.muzzleFlashImpactSpritesheet.time_per_sprite);

        let scale = 50.0f32;

        let mut model = *muzzleTransform * Mat4::from_scale(vec3(scale, scale, scale));

        model *= Mat4::from_rotation_x(-90.0f32.to_radians());
        model *= Mat4::from_translation(vec3(0.7f32, 0.0f32, 0.0f32)); // adjust for position in the texture

        sprite_shader.set_mat4("model", &model);

        for spriteAge in &self.muzzleFlashSpritesAge {
            sprite_shader.set_float("age", *spriteAge);
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

pub fn get_muzzle_position(player_model: &Model, player_model_transform: &Mat4) -> Mat4 {
    // Position in original model of gun muzzle
    // let point_vec = vec3(197.0, 76.143, -3.054);
    let point_vec = vec3(191.04, 79.231, -3.4651); // center of muzzle

    let meshes = player_model.meshes.borrow();
    let animator = player_model.animator.borrow();

    let gun_mesh = meshes.iter().find(|m| m.name.as_str() == "Gun").unwrap();
    let final_node_matrices = animator.final_node_matrices.borrow();

    let gun_transform = final_node_matrices.get(gun_mesh.id as usize).unwrap();

    let muzzle = *gun_transform * Mat4::from_translation(point_vec);

    let muzzle_transform = *player_model_transform * muzzle;

    muzzle_transform
}
