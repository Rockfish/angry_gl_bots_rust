use std::f32::consts::PI;
use std::rc::Rc;
use glam::{Mat4, vec3, vec4};
use small_gl_core::gl;
use small_gl_core::gl::GLuint;
use small_gl_core::shader::Shader;
use crate::sprite_sheet::SpriteSheet;

pub struct MuzzleFlash {
    unitSquareVAO: i32,
    muzzleFlashImpactSpritesheet: SpriteSheet,
}

impl MuzzleFlash {
    pub fn new(unitSquareVAO: i32, muzzleFlashImpactSpritesheet: SpriteSheet) -> Self {
        MuzzleFlash {
            unitSquareVAO,
            muzzleFlashImpactSpritesheet,
        }
    }

    pub fn draw_muzzle_flash(&self, sprite_shader: &Rc<Shader>, PV: &Mat4, muzzleTransform: &Mat4, aimTheta: f32, muzzleFlashSpritesAge: &[f32]) {

        sprite_shader.use_shader();

        unsafe {
            gl::DepthMask(gl::FALSE);
            gl::Enable(gl::BLEND);

            sprite_shader.set_mat4("PV", &PV);

            gl::ActiveTexture(gl::TEXTURE0 + self.muzzleFlashImpactSpritesheet.texture.id);
            gl::BindTexture(gl::TEXTURE_2D, self.muzzleFlashImpactSpritesheet.texture.id as GLuint);
            gl::BindVertexArray(self.unitSquareVAO as GLuint);
        }

        sprite_shader.set_int("numCols", self.muzzleFlashImpactSpritesheet.num_columns);
        sprite_shader.set_int("spritesheet", self.muzzleFlashImpactSpritesheet.texture.id as i32);
        sprite_shader.set_float("timePerSprite", self.muzzleFlashImpactSpritesheet.time_per_sprite);

        let scale = 50.0f32;

        let mut model = *muzzleTransform * Mat4::from_scale(vec3(scale, scale, scale));

        model *= Mat4::from_rotation_y(0.0f32.to_radians());
        model *= Mat4::from_rotation_x(-90.0f32.to_radians());
        model *= Mat4::from_translation(vec3(0.7f32, 0.0f32, 0.0f32));

        let thing = model * vec4(0.0, 0.0, 1.0, 1.0);
        let yRot = thing.y.acos();
        let t = if aimTheta > 0.0 { aimTheta } else { aimTheta + 2.0 * PI };
        let bbRad = 0.5f32;

        let bb = if aimTheta >= 0.0 && aimTheta <= PI {
                bbRad - 2.0 * bbRad * t / PI
        } else {
                -3.0 * bbRad + 2.0 * bbRad * t / PI
        };

        model *= Mat4::from_rotation_x(bb - yRot + 0.94);

        unsafe {
            sprite_shader.set_mat4("model", &model);
        }

        for spriteAge in muzzleFlashSpritesAge {
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