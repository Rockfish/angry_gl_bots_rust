// #![feature(const_trait_impl)]
// #![feature(effects)]
// #![allow(non_upper_case_globals)]
#![allow(dead_code)]
// #![allow(non_snake_case)]
// #![allow(non_camel_case_types)]
// #![allow(unused_assignments)]
// #![allow(clippy::zero_ptr)]
// #![allow(clippy::assign_op_pattern)]

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

use std::cell::RefCell;
use crate::bullets::BulletStore;
use crate::burn_marks::BurnMarks;
use crate::enemy::{Enemy, EnemySystem};
use crate::floor::Floor;
use crate::framebuffers::{create_depth_map_fbo, create_emission_fbo, create_horizontal_blur_fbo, create_scene_fbo, create_vertical_blur_fbo};
use crate::muzzle_flash::MuzzleFlash;
use crate::player::Player;
use crate::quads::{create_more_obnoxious_quad_vao, create_unit_square_vao};
use glam::{vec2, vec3, vec4, Mat4, Vec3};
use glfw::JoystickId::Joystick1;
use glfw::{Action, Context, Key, MouseButton};
use log::error;
use small_gl_core::camera::{Camera, CameraMovement};
use small_gl_core::gl;
use small_gl_core::gl::{GLsizei, GLuint};
use small_gl_core::math::{get_world_ray_from_mouse, ray_plane_intersection};
use small_gl_core::model::ModelBuilder;
use small_gl_core::shader::Shader;
use std::f32::consts::PI;
use std::rc::Rc;
// use std::thread::sleep;
use small_gl_core::hash_map::HashSet;

const PARALLELISM: i32 = 4;

// Viewport
const VIEW_PORT_WIDTH: i32 = 1500;
const VIEW_PORT_HEIGHT: i32 = 1000;
// const VIEW_PORT_WIDTH: i32 = 800;
// const VIEW_PORT_HEIGHT: i32 = 500;

// Player
const FIRE_INTERVAL: f32 = 0.1; // seconds
const SPREAD_AMOUNT: i32 = 20;

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
    viewport_width: f32,
    viewport_height: f32,
    window_scale: (f32, f32),
    key_presses: HashSet<Key>,
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
    player: Rc<RefCell<Player>>,
    enemies: Vec<Enemy>,
    burn_marks: BurnMarks,
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
    let _light_dir: Vec3 = vec3(-0.8, 0.0, -1.0).normalize_or_zero();
    let player_light_dir: Vec3 = vec3(-1.0, -1.0, -1.0).normalize_or_zero();
    let light_color: Vec3 = LIGHT_FACTOR * 1.0 * vec3(NON_BLUE * 0.406, NON_BLUE * 0.723, 1.0);
    // const lightColor: Vec3 = LIGHT_FACTOR * 1.0 * vec3(0.406, 0.723, 1.0);
    let _floor_light_color: Vec3 = FLOOR_LIGHT_FACTOR * 1.0 * vec3(FLOOR_NON_BLUE * 0.406, FLOOR_NON_BLUE * 0.723, 1.0);
    let _floor_ambient_color: Vec3 = FLOOR_LIGHT_FACTOR * 0.50 * vec3(FLOOR_NON_BLUE * 0.7, FLOOR_NON_BLUE * 0.7, 0.7);
    let ambient_color: Vec3 = LIGHT_FACTOR * 0.10 * vec3(NON_BLUE * 0.7, NON_BLUE * 0.7, 0.7);

    let muzzle_point_light_color = vec3(1.0, 0.2, 0.0);

    println!("Loading assets");
    /*
    angrygl/main.cc
        Shader blurShader = Shader::create("angrygl/basicer_shader.vert", "angrygl/blur_shader.frag");
        Shader basicerShader = Shader::create("angrygl/basicer_shader.vert", "angrygl/basicer_shader.frag");
        Shader sceneDrawShader = Shader::create("angrygl/basicer_shader.vert", "angrygl/texture_merge_shader.frag");
        Shader simpleDepthShader = Shader::create("angrygl/depth_shader.vert", "angrygl/depth_shader.frag");
        Shader wigglyShader = Shader::create("angrygl/wiggly_shader.vert", "angrygl/player_shader.frag");
        Shader playerShader = Shader::create("angrygl/player_shader.vert", "angrygl/player_shader.frag");
        Shader instancedTextureShader = Shader::create("angrygl/instanced_texture_shader.vert", "angrygl/basic_texture_shader.frag");
        Shader nodeShader = Shader::create("angrygl/redshader.vert", "angrygl/redshader.frag");
        Shader spriteShader = Shader::create("angrygl/geom_shader2.vert", "angrygl/sprite_shader.frag");
        Shader textureShader = Shader::create("angrygl/geom_shader.vert", "angrygl/texture_shader.frag");

        // floor!
        Shader basicTextureShader = Shader::create("angrygl/basic_texture_shader.vert", "angrygl/floor_shader.frag");
     */

    // for debug
    let _basicer_shader = Shader::new("shaders/basicer_shader.vert", "shaders/basicer_shader.frag").unwrap();

    let player_shader = Shader::new("shaders/player_shader.vert", "shaders/player_shader.frag").unwrap();
    let wiggly_shader = Shader::new("shaders/wiggly_shader.vert", "shaders/player_shader.frag").unwrap();

    let floor_shader = Shader::new("shaders/basic_texture_shader.vert", "shaders/floor_shader.frag").unwrap();
    let basic_texture_shader = Shader::new("shaders/basic_texture_shader.vert", "shaders/basic_texture_shader.frag").unwrap();

    let blur_shader = Shader::new("shaders/basicer_shader.vert", "shaders/blur_shader.frag").unwrap();
    let scene_draw_shader = Shader::new("shaders/basicer_shader.vert", "shaders/texture_merge_shader.frag").unwrap();
    let simple_depth_shader = Shader::new("shaders/depth_shader.vert", "shaders/depth_shader.frag").unwrap();
    let _texture_shader = Shader::new("shaders/geom_shader.vert", "shaders/texture_shader.frag").unwrap();

    let sprite_shader = Shader::new("shaders/geom_shader2.vert", "shaders/sprite_shader.frag").unwrap();

    let instanced_texture_shader = Shader::new("shaders/instanced_texture_shader.vert", "shaders/basic_texture_shader.frag").unwrap();

    simple_depth_shader.use_shader();
    let _lsml = simple_depth_shader.get_uniform_location("lightSpaceMatrix");

    player_shader.use_shader();

    let _player_light_space_matrix_location = player_shader.get_uniform_location("lightSpaceMatrix");

    player_shader.set_vec3("directionLight.dir", &player_light_dir);
    player_shader.set_vec3("directionLight.color", &light_color);
    player_shader.set_vec3("ambient", &ambient_color);

    let enemy_model = ModelBuilder::new("enemy", "assets/Models/Eeldog/EelDog.FBX").build().unwrap();

    // let mut texture_cache = TextureCache::new();
    // let texture_config = TextureConfig::new().set_wrap(TextureWrap::Repeat);

    // let tex_unit_emission_fbo = Texture::new("", &texture_config);
    // let tex_unit_vert_blur = Texture::new("", &texture_config);
    // let tex_unit_horz_blur = Texture::new("", &texture_config);
    // let tex_unit_scene = Texture::new("", &texture_config);

    let floor = Floor::new();

    // Framebuffers

    let _depth_map_fbo = create_depth_map_fbo();
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

    let unit_square_quad = create_unit_square_vao() as i32;
    let more_obnoxious_quad_vao = create_more_obnoxious_quad_vao() as i32;

    //
    // Cameras ------------------------
    //

    let camera_follow_vec = vec3(-4.0, 4.3, 0.0);
    let _camera_up = vec3(0.0, 1.0, 0.0);

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
    let player = Player::new();

    let ortho_width = VIEW_PORT_WIDTH as f32 / 130.0;
    let ortho_height = VIEW_PORT_HEIGHT as f32 / 130.0;
    let aspect_ratio = VIEW_PORT_WIDTH as f32 / VIEW_PORT_HEIGHT as f32;
    let game_projection = Mat4::perspective_rh_gl(game_camera.zoom.to_radians(), aspect_ratio, 0.1, 100.0);
    let floating_projection = Mat4::perspective_rh_gl(floating_camera.zoom.to_radians(), aspect_ratio, 0.1, 100.0);
    let orthographic_projection = Mat4::orthographic_rh_gl(-ortho_width, ortho_width, -ortho_height, ortho_height, 0.1, 100.0);

    let mut state = State {
        run: true,
        viewport_width: VIEW_PORT_WIDTH as f32,
        viewport_height: VIEW_PORT_HEIGHT as f32,
        window_scale: window.get_content_scale(),
        key_presses: HashSet::new(),
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
        player: Rc::new(player.into()),
        enemies: vec![],
        burn_marks: BurnMarks::new(unit_square_quad),
    };

    let mut aim_theta = 0.0f32;

    // let mut muzzleFlashSpritesAge: Vec<f32> = vec![];

    let mut enemy_system = EnemySystem::new(MONSTER_Y);
    let mut muzzle_flash = MuzzleFlash::new(unit_square_quad);
    let mut bullet_store = BulletStore::new(unit_square_quad);

    // let mut use_point_light = false;

    let player = state.player.clone();
    state.player.borrow_mut().set_animation(&Rc::from("forward"), 4);

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

        bullet_store.update_bullets(&mut state);


        if player.borrow().is_alive {
            enemy_system.update(&mut state);
            enemy_system.chase_player(&mut state);
        }

        state.game_camera.position = player.borrow().position + camera_follow_vec.clone();
        let game_view = Mat4::look_at_rh(state.game_camera.position, player.borrow().position, state.game_camera.up);

        let (projection, camera_view) = match state.active_camera {
            CameraType::Game => (state.game_projection, game_view),
            CameraType::Floating => {
                let view = Mat4::look_at_rh(state.floating_camera.position, player.borrow().position, state.floating_camera.up);
                (state.floating_projection, view)
            }
            CameraType::TopDown => {
                let view = Mat4::look_at_rh(
                    vec3(player.borrow().position.x, 1.0, player.borrow().position.z),
                    player.borrow().position,
                    vec3(0.0, 0.0, -1.0),
                );
                (state.orthographic_projection, view)
            }
            CameraType::Side => {
                let view = Mat4::look_at_rh(vec3(0.0, 0.0, -3.0), player.borrow().position, vec3(0.0, 1.0, 0.0));
                (state.orthographic_projection, view)
            }
        };

        let projection_view = projection * camera_view;

        let mut dx: f32 = 0.0;
        let mut dz: f32 = 0.0;

        // player.borrow_mut().is_alive = true;

        if player.borrow().is_alive && buffer_ready {
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

            dx = world_point.x - player.borrow().position.x;
            dz = world_point.z - player.borrow().position.z;

            aim_theta = (dx / dz).atan() + if dz < 0.0 { PI } else { 0.0 };

            if state.mouse_x.abs() < 0.005 && state.mouse_y.abs() < 0.005 {
                aim_theta = 0.0;
            }
        }

        let aim_rot = Mat4::from_axis_angle(vec3(0.0, 1.0, 0.0), aim_theta);

        let mut player_transform = Mat4::from_translation(player.borrow().position);
        player_transform *= Mat4::from_scale(Vec3::splat(PLAYER_MODEL_SCALE));
        player_transform *= aim_rot;

        let muzzle_transform = player.borrow().get_muzzle_position(&player_transform);

        if player.borrow().is_alive && player.borrow().is_trying_to_fire && (player.borrow().last_fire_time + FIRE_INTERVAL) < state.frame_time {
            bullet_store.create_bullets(dx, dz, &muzzle_transform, 10); //SPREAD_AMOUNT);

            player.borrow_mut().last_fire_time = state.frame_time;
            muzzle_flash.add_flash();
        }

        // Draw phase

        unsafe {
            //gl::ClearColor(0.0, 0.02, 0.25, 1.0);
            gl::ClearColor(0.8, 0.82, 0.85, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // --- draw floor
        floor.draw(&floor_shader, &projection_view, &ambient_color);

        // --- draw player with shadows
        {}

        player_shader.use_shader();

        if muzzle_flash.muzzle_flash_sprites_age.len() > 0 {
            let min_age = muzzle_flash.get_min_age();

            let use_point_light = min_age < 0.03;

            let muzzle_world_position = muzzle_transform * vec4(0.0, 0.0, 0.0, 1.0);

            let muzzle_world_position3 = vec3(
                muzzle_world_position.x / muzzle_world_position.w,
                muzzle_world_position.y / muzzle_world_position.w,
                muzzle_world_position.z / muzzle_world_position.w,
            );

            player_shader.set_bool("usePointLight", use_point_light);
            player_shader.set_vec3("pointLight.worldPos", &muzzle_world_position3);
            player_shader.set_vec3("pointLight.color", &muzzle_point_light_color);
        } else {
            let use_point_light = false;
            player_shader.set_bool("usePointLight", use_point_light);
        }

        player_shader.set_mat4("projectionView", &projection_view);
        player_shader.set_mat4("model", &player_transform);
        player_shader.set_vec3("viewPos", &state.game_camera.position);
        player_shader.set_mat4("aimRot", &aim_rot);

        player_shader.set_mat4("lightSpaceMatrix", &Mat4::IDENTITY);
        player_shader.set_bool("useLight", true);
        player_shader.set_vec3("ambient", &ambient_color);

        player.borrow_mut().update(&mut state, aim_theta);

        player.borrow_mut().render(&player_shader);

        muzzle_flash.draw(&sprite_shader, &projection_view, &muzzle_transform);

        // if !player.borrow().is_alive {
        //     player.borrow_mut().set_animation(&dying, 1);
        // }

        wiggly_shader.use_shader();
        wiggly_shader.set_mat4("projectionView", &projection_view);
        wiggly_shader.set_mat4("lightSpaceMatrix", &Mat4::IDENTITY);
        wiggly_shader.set_bool("useLight", false);
        wiggly_shader.set_vec3("ambient", &ambient_color);

        enemy_system.draw_enemies(&enemy_model, &wiggly_shader, &mut state);

        state.burn_marks.draw_marks(&basic_texture_shader, &projection_view, state.delta_time);

        bullet_store.draw_bullet_impacts(&sprite_shader, &projection_view);

        // --- draw bullets
        bullet_store.draw_bullets(&instanced_texture_shader, &projection_view);

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

            gl::BindVertexArray(more_obnoxious_quad_vao as GLuint);

            blur_shader.use_shader();
            blur_shader.set_int("image", emissions_fbo.texture_id as i32);
            blur_shader.set_bool("horizontal", true);
            //
            gl::DrawArrays(gl::TRIANGLES, 0, 6);

            gl::BindFramebuffer(gl::FRAMEBUFFER, vertical_blur_fbo.framebuffer_id);
            gl::BindVertexArray(more_obnoxious_quad_vao as GLuint);

            blur_shader.use_shader();
            blur_shader.set_int("image", horizontal_blur_fbo.texture_id as i32);

            blur_shader.set_bool("horizontal", false);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);

            gl::Viewport(0, 0, viewport_width as GLsizei, viewport_height as GLsizei);

            // gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            //
            scene_draw_shader.use_shader();
            gl::BindVertexArray(more_obnoxious_quad_vao as GLuint);

            scene_draw_shader.set_int("base_texture", scene_fbo.texture_id as i32);
            scene_draw_shader.set_int("emission_texture", vertical_blur_fbo.texture_id as i32);
            scene_draw_shader.set_int("bright_texture", emissions_fbo.texture_id as i32);

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
    // println!("WindowEvent: {:?}", &event);
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
        glfw::WindowEvent::Key(Key::W, _, action, modifier) => {
            if modifier.is_empty() {
                handle_key_press(state, action, Key::W);
            } else {
                state.floating_camera.process_keyboard(CameraMovement::Forward, state.delta_time);
            }
        }
        glfw::WindowEvent::Key(Key::S, _, action, modifier) => {
            if modifier.is_empty() {
                handle_key_press(state, action, Key::S);
            } else {
                state.floating_camera.process_keyboard(CameraMovement::Backward, state.delta_time);
            }
        }
        glfw::WindowEvent::Key(Key::A, _, action, modifier) => {
            if modifier.is_empty() {
                handle_key_press(state, action, Key::A);
            } else {
                state.floating_camera.process_keyboard(CameraMovement::Left, state.delta_time);
            }
        }
        glfw::WindowEvent::Key(Key::D, _, action, modifier) => {
            if modifier.is_empty() {
                handle_key_press(state, action, Key::D);
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
        glfw::WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
            state.player.borrow_mut().is_trying_to_fire = true
        },
        glfw::WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
            state.player.borrow_mut().is_trying_to_fire = false
        },
        _evt => {
            // println!("WindowEvent: {:?}", _evt);
        }
    }

    if state.player.borrow().is_alive {
        let player_speed = state.player.borrow().speed;
        let mut player = state.player.borrow_mut();

        let mut direction_vec = Vec3::splat(0.0);
        for key in &state.key_presses {
            match key {
                Key::A => direction_vec += vec3(0.0, 0.0, -1.0),
                Key::D => direction_vec += vec3(0.0, 0.0, 1.0),
                Key::S => direction_vec += vec3(-1.0, 0.0, 0.0),
                Key::W => direction_vec += vec3(1.0, 0.0, 0.0),
                _ => {},
            }
        }

        if direction_vec.length_squared() > 0.01 {
            player.position += direction_vec.normalize() * player_speed * state.delta_time;
        }
        player.direction = vec2(direction_vec.x, direction_vec.z);
    }
    // println!("key presses: {:?}", &state.key_presses);
    // println!("direction: {:?}  player.direction: {:?}  delta_time: {:?}", direction_vec, player.direction, state.frame_time);
}

fn handle_key_press(state: &mut State, action: Action, key: Key) {
    match action {
        Action::Release => state.key_presses.remove(&key),
        Action::Press => state.key_presses.insert(key),
        _ => false,
    };
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

fn mouse_handler(state: &mut State, xpos_in: f64, ypos_in: f64) {
    let xpos = xpos_in as f32;
    let ypos = ypos_in as f32;

    if state.first_mouse {
        state.mouse_x = xpos;
        state.mouse_y = ypos;
        state.first_mouse = false;
    }

    // let xoffset = xpos - state.mouse_x;
    // let yoffset = state.mouse_y - ypos; // reversed since y-coordinates go from bottom to top

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
