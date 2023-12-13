use std::f32::consts::PI;
use std::rc::Rc;
use glam::{Mat4, vec3, Vec3};
use small_gl_core::model::Model;
use small_gl_core::shader::Shader;
use crate::{monsterY, State};

pub fn draw_wiggly_bois(wigglyBoi: &Model, shader: &Rc<Shader>, state: &mut State) {

    shader.use_shader();
    shader.set_vec3("nosePos", &vec3(1.0, monsterY, -2.0));

    // TODO optimise (multithreaded, instancing, SOA, etc..)
    for e in state.enemies.iter_mut() {
        let monsterTheta = (e.dir.x / e.dir.z).atan() + (if e.dir.z < 0.0 { 0.0 } else { PI });

        let mut model_transform = Mat4::from_translation(e.position);
        model_transform *= Mat4::from_scale(Vec3::splat(0.01));
        model_transform *= Mat4::from_axis_angle(vec3(0.0, 1.0, 0.0), monsterTheta);
        model_transform *= Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), PI);
        model_transform *= Mat4::from_axis_angle(vec3(1.0, 0.0, 0.0), 90.0f32.to_radians());

        let mut rot_only = Mat4::from_axis_angle(vec3(0.0, 1.0, 0.0), monsterTheta);
        rot_only = Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), PI);
        rot_only = Mat4::from_axis_angle(vec3(1.0, 0.0, 0.0), 90.0f32.to_radians());

        shader.set_mat4("aimRot", &rot_only);
        shader.set_mat4("model", &model_transform);

        wigglyBoi.render(shader);
    }
}
