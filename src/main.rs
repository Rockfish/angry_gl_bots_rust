// #![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(non_snake_case)]
// #![allow(non_camel_case_types)]
#![allow(unused_assignments)]
#![allow(clippy::zero_ptr)]
#![allow(clippy::assign_op_pattern)]

mod aabb;
mod bullets;
mod burn_marks;
mod capsule;
mod enemy;
mod floor;
mod framebuffers;
mod geom;
mod muzzle_flash;
mod player;
mod quads;
mod sprite_sheet;
mod texture_cache;

extern crate glfw;

use crate::bullets::BulletStore;
use crate::burn_marks::BurnMarks;
use crate::enemy::{chase_player, draw_enemies, Enemy, EnemySpawner};
use crate::floor::Floor;
use crate::framebuffers::{create_depth_map_fbo, create_emission_fbo, create_horizontal_blur_fbo, create_scene_fbo, create_vertical_blur_fbo};
use crate::muzzle_flash::{get_muzzle_position, MuzzleFlash};
use crate::player::Player;
use crate::quads::{create_moreObnoxiousQuadVAO, create_unitSquareVAO};
use glam::{vec2, vec3, vec4, Mat4, Vec3, Vec4Swizzles};
use glfw::JoystickId::Joystick1;
use glfw::{Action, Context, Key, MouseButton};
use log::error;
use small_gl_core::animator::{AnimationClip, AnimationRepeat};
use small_gl_core::camera::{Camera, CameraMovement};
use small_gl_core::gl;
use small_gl_core::gl::{GLsizei, GLuint};
use small_gl_core::math::{get_world_ray_from_mouse, ray_plane_intersection};
use small_gl_core::model::ModelBuilder;
use small_gl_core::shader::Shader;
use small_gl_core::texture::{Texture, TextureConfig, TextureFilter, TextureType, TextureWrap};
use std::f32::consts::PI;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

const PARALLELISM: i32 = 4;

// Viewport
const VIEW_PORT_WIDTH: i32 = 1500;
const VIEW_PORT_HEIGHT: i32 = 1000;
// const VIEW_PORT_WIDTH: i32 = 800;
// const VIEW_PORT_HEIGHT: i32 = 500;
//
// // Texture units
// const texUnit_playerDiffuse: u32 = 0;
// const texUnit_gunDiffuse: u32 = 1;
// const texUnit_floorDiffuse: u32 = 2;
// const texUnit_wigglyBoi: u32 = 3;
// const texUnit_bullet: u32 = 4;
// const texUnit_floorNormal: u32 = 5;
// const texUnit_playerNormal: u32 = 6;
// const texUnit_gunNormal: u32 = 7;
// const texUnit_shadowMap: u32 = 8;
// const texUnit_emissionFBO: u32 = 9;
// const texUnit_playerEmission: u32 = 10;
// const texUnit_gunEmission: i32 = 11;
// const texUnit_scene: i32 = 12;
// const texUnit_horzBlur: i32 = 13;
// const texUnit_vertBlur: i32 = 14;
// const texUnit_impactSpriteSheet: u32 = 15;
// const texUnit_muzzleFlashSpriteSheet: u32 = 16;
// const texUnit_floorSpec: u32 = 18;
// const texUnit_playerSpec: u32 = 19;
// const texUnit_gunSpec: u32 = 20;

// Player
const FIRE_INTERVAL: f32 = 0.1; // seconds
const SPREAD_AMOUNT: i32 = 20;
const PLAYER_SPEED: f32 = 1.5;
const PLAYER_COLLISION_RADIUS: f32 = 0.35;

// Models
const PLAYER_MODEL_SCALE: f32 = 0.0044;
//const PLAYER_MODEL_GUN_HEIGHT: f32 = 120.0; // un-scaled
const PLAYER_MODEL_GUN_HEIGHT: f32 = 110.0; // un-scaled
const PLAYER_MODEL_GUN_MUZZLE_OFFSET: f32 = 100.0; // un-scaled
const MONSTER_Y: f32 = PLAYER_MODEL_SCALE * PLAYER_MODEL_GUN_HEIGHT;

// Lighting
const LIGHT_FACTOR: f32 = 0.8;
const NON_BLUE: f32 = 0.9;

const BLUR_SCALE: f32 = 2.0;

const FLOOR_LIGHT_FACTOR: f32 = 0.35;
const FLOOR_NON_BLUE: f32 = 0.7;

// Enemies
const MONSTER_SPEED: f32 = 0.6;

enum CameraType {
    Game,
    Floating,
    TopDown,
    Side,
}

struct State {
    run: bool,
    window_scale: (f32, f32),
    game_camera: Camera,
    floating_camera: Camera,
    ortho_camera: Camera,
    active_camera: CameraType,
    game_projection: Mat4,
    floating_projection: Mat4,
    orthographic_projection: Mat4,
    delta_time: f32,
    frame_time: f32,
    first_mouse: bool,
    mouse_x: f32,
    mouse_y: f32,
    player: Player,
    enemies: Vec<Enemy>,
    burn_marks: BurnMarks,
    viewport_width: f32,
    viewport_height: f32,
}

fn error_callback(err: glfw::Error, description: String) {
    error!("GLFW error {:?}: {:?}", err, description);
}

fn joystick_callback(jid: glfw::JoystickId, event: glfw::JoystickEvent) {
    println!("joystick: {:?}  event: {:?}", jid, event);
}

fn main() {
    let mut glfw = glfw::init(error_callback).unwrap();

    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    let (mut window, events) = glfw
        .create_window(VIEW_PORT_WIDTH as u32, VIEW_PORT_HEIGHT as u32, "LearnOpenGL", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.set_all_polling(true);
    window.make_current();

    gl::load(|e| glfw.get_proc_address_raw(e) as *const std::os::raw::c_void);

    glfw.set_joystick_callback(joystick_callback);

    let joy = glfw.get_joystick(Joystick1);

    if joy.is_present() {
        let axes = joy.get_axes();
        println!("axes: {:?}", axes)
    }

    // Lighting
    let lightDir: Vec3 = vec3(-0.8, 0.0, -1.0).normalize_or_zero();
    let playerLightDir: Vec3 = vec3(-1.0, -1.0, -1.0).normalize_or_zero();
    let lightColor: Vec3 = LIGHT_FACTOR * 1.0 * vec3(NON_BLUE * 0.406, NON_BLUE * 0.723, 1.0);
    // const lightColor: Vec3 = LIGHT_FACTOR * 1.0 * vec3(0.406, 0.723, 1.0);
    let floorLightColor: Vec3 = FLOOR_LIGHT_FACTOR * 1.0 * vec3(FLOOR_NON_BLUE * 0.406, FLOOR_NON_BLUE * 0.723, 1.0);
    let floorAmbientColor: Vec3 = FLOOR_LIGHT_FACTOR * 0.50 * vec3(FLOOR_NON_BLUE * 0.7, FLOOR_NON_BLUE * 0.7, 0.7);
    let ambientColor: Vec3 = LIGHT_FACTOR * 0.10 * vec3(NON_BLUE * 0.7, NON_BLUE * 0.7, 0.7);

    let muzzle_point_light_color = vec3(1.0, 0.2, 0.0);

    println!("Loading assets");

    let playerShader = Rc::new(Shader::new("shaders/player_shader.vert", "shaders/player_shader.frag").unwrap());
    let wigglyShader = Rc::new(Shader::new("shaders/wiggly_shader.vert", "shaders/player_shader.frag").unwrap());

    let basicerShader = Shader::new("shaders/basicer_shader.vert", "shaders/basicer_shader.frag").unwrap();
    let basicTextureShader = Rc::new(Shader::new("shaders/basic_texture_shader.vert", "shaders/basic_texture_shader.frag").unwrap());
    let blurShader = Shader::new("shaders/basicer_shader.vert", "shaders/blur_shader.frag").unwrap();
    let sceneDrawShader = Shader::new("shaders/basicer_shader.vert", "shaders/texture_merge_shader.frag").unwrap();
    let simpleDepthShader = Rc::new(Shader::new("shaders/depth_shader.vert", "shaders/depth_shader.frag").unwrap());
    let textureShader = Rc::new(Shader::new("shaders/geom_shader.vert", "shaders/texture_shader.frag").unwrap());

    let sprite_shader = Rc::new(Shader::new("shaders/geom_shader2.vert", "shaders/sprite_shader.frag").unwrap());

    let instancedTextureShader = Rc::new(Shader::new("shaders/instanced_texture_shader.vert", "shaders/basic_texture_shader.frag").unwrap());

    simpleDepthShader.use_shader();
    let lsml = simpleDepthShader.get_uniform_location("lightSpaceMatrix");

    playerShader.use_shader();

    let playerLightSpaceMatrixLocation = playerShader.get_uniform_location("lightSpaceMatrix");

    playerShader.set_vec3("directionLight.dir", &playerLightDir);
    playerShader.set_vec3("directionLight.color", &lightColor);
    playerShader.set_vec3("ambient", &ambientColor);

    let player_model = ModelBuilder::new("player", "assets/Models/Player/Player.fbx")
        .add_texture("Player", TextureType::Diffuse, "Textures/Player_D.tga")
        .add_texture("Player", TextureType::Specular, "Textures/Player_M.tga")
        .add_texture("Player", TextureType::Emissive, "Textures/Player_E.tga")
        .add_texture("Player", TextureType::Normals, "Textures/Player_NRM.tga")
        .add_texture("Gun", TextureType::Diffuse, "Textures/Gun_D.tga")
        .add_texture("Gun", TextureType::Specular, "Textures/Gun_M.tga")
        .add_texture("Gun", TextureType::Emissive, "Textures/Gun_E.tga")
        .add_texture("Gun", TextureType::Normals, "Textures/Gun_NRM.tga")
        .build()
        .unwrap();

    let idle = Rc::new(AnimationClip::new("idle", 55.0, 130.0, AnimationRepeat::Forever));
    let forward = Rc::new(AnimationClip::new("forward", 134.0, 154.0, AnimationRepeat::Forever));
    let backwards = Rc::new(AnimationClip::new("backwards", 159.0, 179.0, AnimationRepeat::Forever));
    let right = Rc::new(AnimationClip::new("right", 184.0, 204.0, AnimationRepeat::Forever));
    let left = Rc::new(AnimationClip::new("left", 209.0, 229.0, AnimationRepeat::Forever));
    let dying = Rc::new(AnimationClip::new("dying", 234.0, 293.0, AnimationRepeat::Once));

    let wigglyBoi = ModelBuilder::new("dog", "assets/Models/Eeldog/EelDog.FBX").build().unwrap();

    // let mut texture_cache = TextureCache::new();
    let texture_config = TextureConfig::new().set_wrap(TextureWrap::Repeat);

    let texUnit_emissionFBO = Texture::new("", &texture_config);
    let texUnit_vertBlur = Texture::new("", &texture_config);
    let texUnit_horzBlur = Texture::new("", &texture_config);
    let texUnit_scene = Texture::new("", &texture_config);

    let floor = Floor::new(&basicTextureShader);

    // Framebuffers

    let depth_map_fbo = create_depth_map_fbo();
    let emissions_fbo = create_emission_fbo(VIEW_PORT_WIDTH, VIEW_PORT_HEIGHT);
    let scene_fbo = create_scene_fbo(VIEW_PORT_WIDTH, VIEW_PORT_HEIGHT);
    let horizontal_blur_fbo = create_horizontal_blur_fbo(VIEW_PORT_WIDTH, VIEW_PORT_HEIGHT);
    let vertical_blur_fbo = create_vertical_blur_fbo(VIEW_PORT_WIDTH, VIEW_PORT_HEIGHT);

    unsafe {
        gl::ActiveTexture(gl::TEXTURE0 + scene_fbo.texture_id);
        gl::Enable(gl::CULL_FACE);
        gl::ClearColor(0.1, 0.1, 0.1, 1.0);
        gl::Enable(gl::DEPTH_TEST);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        gl::ActiveTexture(gl::TEXTURE0);
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    //
    // ----------------- quads ---------------
    //

    let unit_square_quad = create_unitSquareVAO() as i32;
    let moreObnoxiousQuadVAO = create_moreObnoxiousQuadVAO() as i32;

    //
    // Cameras ------------------------
    //

    let cameraFollowVec = vec3(-4.0, 4.3, 0.0);
    let cameraUp = vec3(0.0, 1.0, 0.0);

    let game_camera = Camera::camera_vec3_up_yaw_pitch(
        vec3(0.0, 20.0, 80.0), // for xz world
        vec3(0.0, 1.0, 0.0),
        -90.0, // seems camera starts by looking down the x-axis, so needs to turn left to see the plane
        -20.0,
    );

    let floating_camera = Camera::camera_vec3_up_yaw_pitch(
        vec3(0.0, 10.0, 20.0), // for xz world
        vec3(0.0, 1.0, 0.0),
        -90.0, // seems camera starts by looking down the x-axis, so needs to turn left to see the plane
        -20.0,
    );

    let ortho_camera = Camera::camera_vec3_up_yaw_pitch(vec3(0.0, 1.0, 0.0), vec3(0.0, 1.0, 0.0), 0.0, -90.0);

    // Player
    let player = Player {
        lastFireTime: 0.0f32,
        isTryingToFire: false,
        isAlive: true,
        aimTheta: 0.0f32,
        position: vec3(0.0, 0.0, 0.0),
        player_direction: vec2(0.0, 0.0),
        animation_name: "".to_string(),
        speed: PLAYER_SPEED,
    };

    let ortho_width = VIEW_PORT_WIDTH as f32 / 130.0;
    let ortho_height = VIEW_PORT_HEIGHT as f32 / 130.0;
    let aspect_ratio = VIEW_PORT_WIDTH as f32 / VIEW_PORT_HEIGHT as f32;
    let game_projection = Mat4::perspective_rh_gl(game_camera.zoom.to_radians(), aspect_ratio, 0.1, 100.0);
    let floating_projection = Mat4::perspective_rh_gl(floating_camera.zoom.to_radians(), aspect_ratio, 0.1, 100.0);
    let orthographic_projection = Mat4::orthographic_rh_gl(-ortho_width, ortho_width, -ortho_height, ortho_height, 0.1, 100.0);

    let mut state = State {
        run: true,
        window_scale: window.get_content_scale(),
        game_camera,
        floating_camera,
        ortho_camera,
        game_projection,
        floating_projection,
        orthographic_projection,
        active_camera: CameraType::Game,
        delta_time: 0.0,
        frame_time: 0.0,
        first_mouse: true,
        mouse_x: VIEW_PORT_WIDTH as f32 / 2.0,
        mouse_y: VIEW_PORT_HEIGHT as f32 / 2.0,
        player,
        enemies: vec![],
        burn_marks: BurnMarks::new(unit_square_quad),
        viewport_width: VIEW_PORT_WIDTH as f32,
        viewport_height: VIEW_PORT_HEIGHT as f32,
    };

    let mut aimTheta = 0.0f32;

    // let mut muzzleFlashSpritesAge: Vec<f32> = vec![];

    let mut enemy_spawner = EnemySpawner::new(MONSTER_Y);
    let mut muzzle_flash = MuzzleFlash::new(unit_square_quad);
    let mut bulletStore = BulletStore::new(unit_square_quad);

    let mut use_point_light = false;

    player_model.play_clip(&idle);
    player_model.play_clip_with_transition(&idle, Duration::from_secs(6));
    state.player.animation_name = String::from(&idle.name);

    //
    // --------------------------------
    //

    println!("Assets loaded. Starting loop.");

    let mut buffer_ready = false;

    while !window.should_close() {
        // sleep(Duration::from_millis(500));

        let current_time = glfw.get_time() as f32;
        if state.run {
            state.delta_time = current_time - state.frame_time;
        } else {
            state.delta_time = 0.0;
        }
        state.frame_time = current_time;

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event, &mut state);
        }

        if joy.is_present() {
            let axes = joy.get_axes();
            println!("axes: {:?}", axes)
        }

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            // gl::ActiveTexture(gl::TEXTURE0 + scene_fbo.texture_id);
            // gl::BindTexture(gl::TEXTURE_2D, scene_fbo.texture_id as GLuint);
            // gl::BindFramebuffer(gl::FRAMEBUFFER, scene_fbo.framebuffer_id);
        }

        // --- muzzle flash

        muzzle_flash.update(state.delta_time);

        // --- bullet sprites

        bulletStore.update_bullets(&mut state);

        if state.player.isAlive {
            enemy_spawner.update(&mut state);
            chase_player(&mut state);
        }

        state.game_camera.position = state.player.position + cameraFollowVec.clone();
        let game_view = Mat4::look_at_rh(state.game_camera.position, state.player.position, state.game_camera.up);

        let (projection, camera_view) = match state.active_camera {
            CameraType::Game => (state.game_projection, game_view),
            CameraType::Floating => {
                let view = Mat4::look_at_rh(state.floating_camera.position, state.player.position, state.floating_camera.up);
                (state.floating_projection, view)
            }
            CameraType::TopDown => {
                let view = Mat4::look_at_rh(
                    vec3(state.player.position.x, 1.0, state.player.position.z),
                    state.player.position,
                    vec3(0.0, 0.0, -1.0),
                );
                (state.orthographic_projection, view)
            }
            CameraType::Side => {
                let view = Mat4::look_at_rh(vec3(0.0, 0.0, -3.0), state.player.position, vec3(0.0, 1.0, 0.0));
                (state.orthographic_projection, view)
            }
        };

        let projection_view = projection * camera_view;

        let mut dx: f32 = 0.0;
        let mut dz: f32 = 0.0;

        state.player.isAlive = true;

        if state.player.isAlive && buffer_ready {
            let world_ray = get_world_ray_from_mouse(
                state.mouse_x,
                state.mouse_y,
                state.viewport_width,
                state.viewport_height,
                &game_view,
                &state.game_projection,
            );

            // the xz plane
            let plane_point = vec3(0.0, 0.0, 0.0);
            let plane_normal = vec3(0.0, 1.0, 0.0);

            let world_point = ray_plane_intersection(state.game_camera.position, world_ray, plane_point, plane_normal).unwrap();

            dx = world_point.x - state.player.position.x;
            dz = world_point.z - state.player.position.z;

            aimTheta = (dx / dz).atan() + if dz < 0.0 { PI } else { 0.0 };

            if state.mouse_x.abs() < 0.005 && state.mouse_y.abs() < 0.005 {
                aimTheta = 0.0;
            }
        }

        // Todo: blend player animations for current action
        // player.update_points_for_anim(&mut state);

        let aimRot = Mat4::from_axis_angle(vec3(0.0, 1.0, 0.0), aimTheta);

        let mut player_model_transform = Mat4::from_translation(state.player.position);
        player_model_transform *= Mat4::from_scale(Vec3::splat(PLAYER_MODEL_SCALE));
        player_model_transform *= aimRot;

        let muzzle_transform = get_muzzle_position(&player_model, &player_model_transform);

        if state.player.isAlive && state.player.isTryingToFire && (state.player.lastFireTime + FIRE_INTERVAL) < state.frame_time {
            bulletStore.create_bullets(dx, dz, &muzzle_transform, 10); //SPREAD_AMOUNT);

            state.player.lastFireTime = state.frame_time;
            muzzle_flash.add_flash();
        }

        // Draw phase

        unsafe {
            //gl::ClearColor(0.0, 0.02, 0.25, 1.0);
            gl::ClearColor(0.8, 0.82, 0.85, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // --- draw floor
        floor.draw(&projection_view, &ambientColor);

        // --- draw player with shadows
        {}

        playerShader.use_shader();

        if muzzle_flash.muzzleFlashSpritesAge.len() > 0 {
            let min_age = muzzle_flash.get_min_age();

            use_point_light = min_age < 0.03;

            let muzzle_world_position = muzzle_transform * vec4(0.0, 0.0, 0.0, 1.0);

            let muzzle_world_position3 = vec3(
                muzzle_world_position.x / muzzle_world_position.w,
                muzzle_world_position.y / muzzle_world_position.w,
                muzzle_world_position.z / muzzle_world_position.w,
            );

            playerShader.set_bool("usePointLight", use_point_light);
            playerShader.set_vec3("pointLight.worldPos", &muzzle_world_position3);
            playerShader.set_vec3("pointLight.color", &muzzle_point_light_color);
        } else {
            use_point_light = false;
            playerShader.set_bool("usePointLight", use_point_light);
        }

        playerShader.set_mat4("projectionView", &projection_view);
        playerShader.set_vec3("viewPos", &state.game_camera.position);
        playerShader.set_mat4("model", &player_model_transform);
        playerShader.set_mat4("aimRot", &aimRot);

        playerShader.set_mat4("lightSpaceMatrix", &Mat4::IDENTITY);
        playerShader.set_bool("useLight", true);
        playerShader.set_vec3("ambient", &ambientColor);

        player_model.update_animation(state.delta_time);
        player_model.render(&playerShader);

        muzzle_flash.draw(&sprite_shader, &projection_view, &muzzle_transform);

        if !state.player.isAlive {
            if state.player.animation_name != String::from(&dying.name) {
                player_model.play_clip_with_transition(&dying, Duration::from_secs(1));
                state.player.animation_name = String::from(&dying.name);
            }
        }

        wigglyShader.use_shader();
        wigglyShader.set_mat4("projectionView", &projection_view);
        // wigglyShader.set_vec3("viewPos", &vec3(0.0, 0.0, 1.0)); //camera_position);
        // wigglyShader.set_vec3("viewPos", &camera_position);

        // let mut wiggly_transform = Mat4::from_translation(vec3(0.0, 0.5, 0.0));
        // wiggly_transform *= Mat4::from_scale(Vec3::splat(0.1));
        // wiggly_transform *= Mat4::from_axis_angle(vec3(1.0, 0.0, 0.0), -90.0f32.to_radians());
        // wigglyShader.set_mat4("model", &wiggly_transform);
        // wigglyShader.set_mat4("aimRot", &Mat4::IDENTITY);

        wigglyShader.set_mat4("lightSpaceMatrix", &Mat4::IDENTITY);
        wigglyShader.set_bool("useLight", false);
        wigglyShader.set_vec3("ambient", &ambientColor);

        draw_enemies(&wigglyBoi, &wigglyShader, &mut state);

        state.burn_marks.draw_marks(&basicTextureShader, &projection_view, state.delta_time);

        bulletStore.draw_bullet_impacts(&sprite_shader, &projection_view, &game_view);

        // --- draw bullets
        bulletStore.draw_bullets(&instancedTextureShader, &projection_view);

        // blur
        unsafe {
            // gl::Enable(gl::DEPTH_TEST);
            let viewport_width = state.viewport_width * state.window_scale.0;
            let viewport_height = state.viewport_height * state.window_scale.0;

            gl::Disable(gl::DEPTH_TEST);
            //
            gl::ActiveTexture(gl::TEXTURE0 + horizontal_blur_fbo.texture_id);
            gl::BindTexture(gl::TEXTURE_2D, horizontal_blur_fbo.texture_id as GLuint);

            gl::ActiveTexture(gl::TEXTURE0 + vertical_blur_fbo.texture_id);
            gl::BindTexture(gl::TEXTURE_2D, vertical_blur_fbo.texture_id as GLuint);

            gl::ActiveTexture(gl::TEXTURE0 + emissions_fbo.texture_id);
            gl::BindTexture(gl::TEXTURE_2D, emissions_fbo.texture_id as GLuint);
            //
            gl::ActiveTexture(gl::TEXTURE0 + scene_fbo.texture_id);
            gl::BindTexture(gl::TEXTURE_2D, scene_fbo.texture_id as GLuint);
            //
            gl::BindFramebuffer(gl::FRAMEBUFFER, horizontal_blur_fbo.framebuffer_id);

            gl::Viewport(0, 0, (viewport_width / BLUR_SCALE) as i32, (viewport_height / BLUR_SCALE) as i32);

            gl::BindVertexArray(moreObnoxiousQuadVAO as GLuint);

            blurShader.use_shader();
            blurShader.set_int("image", emissions_fbo.texture_id as i32);
            blurShader.set_bool("horizontal", true);
            //
            gl::DrawArrays(gl::TRIANGLES, 0, 6);

            gl::BindFramebuffer(gl::FRAMEBUFFER, vertical_blur_fbo.framebuffer_id);
            gl::BindVertexArray(moreObnoxiousQuadVAO as GLuint);
            blurShader.use_shader();
            blurShader.set_int("image", horizontal_blur_fbo.texture_id as i32);
            blurShader.set_bool("horizontal", false);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);

            gl::Viewport(0, 0, viewport_width as GLsizei, viewport_height as GLsizei);

            // gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            //
            sceneDrawShader.use_shader();
            gl::BindVertexArray(moreObnoxiousQuadVAO as GLuint);

            sceneDrawShader.set_int("base_texture", scene_fbo.texture_id as i32);
            sceneDrawShader.set_int("emission_texture", vertical_blur_fbo.texture_id as i32);
            sceneDrawShader.set_int("bright_texture", emissions_fbo.texture_id as i32);

            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::Enable(gl::DEPTH_TEST);

            // window.swap_buffers();
            // gl::BindFramebuffer(gl::FRAMEBUFFER, scene_fbo.framebuffer_id);

            // gl::Viewport(0, 0, state.viewport_width as i32, state.viewport_height as i32);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            gl::Viewport(0, 0, viewport_width as GLsizei, viewport_height as GLsizei);
            gl::Enable(gl::DEPTH_TEST);
        }

        buffer_ready = true;
        window.swap_buffers();
    }
}

//
// GLFW maps callbacks to events.
//
fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent, state: &mut State) {
    let mut direction_vec = Vec3::splat(0.0);
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
        glfw::WindowEvent::FramebufferSize(width, height) => {
            framebuffer_size_event(window, state, width, height);
        }
        glfw::WindowEvent::Key(Key::Num1, _, _, _) => {
            state.active_camera = CameraType::Game;
        }
        glfw::WindowEvent::Key(Key::Num2, _, _, _) => {
            state.active_camera = CameraType::Floating;
        }
        glfw::WindowEvent::Key(Key::Num3, _, _, _) => {
            state.active_camera = CameraType::TopDown;
        }
        glfw::WindowEvent::Key(Key::Num4, _, _, _) => {
            state.active_camera = CameraType::Side;
        }
        glfw::WindowEvent::Key(Key::Space, _, Action::Press, _) => {
            state.run = !state.run;
        }
        glfw::WindowEvent::Key(Key::T, _, Action::Press, _) => {
            let width = state.viewport_width as i32;
            let height = state.viewport_height as i32;
            set_view_port(state, width, height)
        }
        glfw::WindowEvent::Key(Key::W, _, _, modifier) => {
            if modifier.is_empty() {
                direction_vec += vec3(1.0, 0.0, 0.0);
            } else {
                state.floating_camera.process_keyboard(CameraMovement::Forward, state.delta_time);
            }
        }
        glfw::WindowEvent::Key(Key::S, _, _, modifier) => {
            if modifier.is_empty() {
                direction_vec += vec3(-1.0, 0.0, 0.0);
            } else {
                state.floating_camera.process_keyboard(CameraMovement::Backward, state.delta_time);
            }
        }
        glfw::WindowEvent::Key(Key::A, _, _, modifier) => {
            if modifier.is_empty() {
                direction_vec += vec3(0.0, 0.0, -1.0);
            } else {
                state.floating_camera.process_keyboard(CameraMovement::Left, state.delta_time);
            }
        }
        glfw::WindowEvent::Key(Key::D, _, _, modifier) => {
            if modifier.is_empty() {
                direction_vec += vec3(0.0, 0.0, 1.0);
            } else {
                state.floating_camera.process_keyboard(CameraMovement::Right, state.delta_time);
            }
        }
        glfw::WindowEvent::Key(Key::Q, _, _, _) => {
            state.floating_camera.process_keyboard(CameraMovement::Up, state.delta_time);
        }
        glfw::WindowEvent::Key(Key::Z, _, _, _) => {
            state.floating_camera.process_keyboard(CameraMovement::Down, state.delta_time);
        }
        glfw::WindowEvent::CursorPos(xpos, ypos) => mouse_handler(state, xpos, ypos),
        glfw::WindowEvent::Scroll(xoffset, ysoffset) => scroll_handler(state, xoffset, ysoffset),
        glfw::WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => state.player.isTryingToFire = true,
        glfw::WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => state.player.isTryingToFire = false,
        _evt => {
            // println!("WindowEvent: {:?}", _evt);
        }
    }
    if direction_vec.length_squared() > 0.01 {
        state.player.position += direction_vec.normalize() * state.player.speed * state.delta_time;
    }
    state.player.player_direction = vec2(direction_vec.x, direction_vec.z);
}

fn framebuffer_size_event(_window: &mut glfw::Window, state: &mut State, width: i32, height: i32) {
    println!("resize: width, height: {}, {}", width, height);
    set_view_port(state, width, height);
}

fn set_view_port(state: &mut State, width: i32, height: i32) {
    unsafe {
        gl::Viewport(0, 0, width, height);
    }

    state.viewport_width = width as f32 / state.window_scale.0;
    state.viewport_height = height as f32 / state.window_scale.1;

    let ortho_width = state.viewport_width / 130.0;
    let ortho_height = state.viewport_height / 130.0;
    let aspect_ratio = state.viewport_width / state.viewport_height;

    state.game_projection = Mat4::perspective_rh_gl(state.game_camera.zoom.to_radians(), aspect_ratio, 0.1, 100.0);
    state.floating_projection = Mat4::perspective_rh_gl(state.floating_camera.zoom.to_radians(), aspect_ratio, 0.1, 100.0);
    state.orthographic_projection = Mat4::orthographic_rh_gl(-ortho_width, ortho_width, -ortho_height, ortho_height, 0.1, 100.0);
}

fn mouse_handler(state: &mut State, xposIn: f64, yposIn: f64) {
    let xpos = xposIn as f32;
    let ypos = yposIn as f32;

    if state.first_mouse {
        state.mouse_x = xpos;
        state.mouse_y = ypos;
        state.first_mouse = false;
    }

    let xoffset = xpos - state.mouse_x;
    let yoffset = state.mouse_y - ypos; // reversed since y-coordinates go from bottom to top

    state.mouse_x = xpos;
    state.mouse_y = ypos;

    // println!("mouse: {}, {}", xpos, ypos);

    // state.camera.process_mouse_movement(xoffset, yoffset, true);
}

fn scroll_handler(state: &mut State, _xoffset: f64, yoffset: f64) {
    state.game_camera.process_mouse_scroll(yoffset as f32);
}
/*
world_ray: Vec3(0.68110394, -0.7321868, -2.9802322e-8)
world_point: Vec3(-2.3841858e-7, 0.0, -1.7502363e-7)
dx, dz: -0.00000023841858, -0.00000017502363
aimTheta: 4.07914


world_ray: Vec3(0.66441494, -0.65366167, -0.3623247)
world_point: Vec3(0.37073898, 0.0, -2.3834906)
dx, dz: 0.37073898, -2.3834906
aimTheta: 2.9872847


 */
