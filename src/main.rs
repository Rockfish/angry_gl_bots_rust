// #![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(non_snake_case)]
// #![allow(non_camel_case_types)]
#![allow(unused_assignments)]
#![allow(clippy::zero_ptr)]
#![allow(clippy::assign_op_pattern)]

mod aabb;
mod bullet_store;
mod capsule;
mod enemy;
mod floor;
mod framebuffers;
mod geom;
mod player;
mod quads;
mod sprite_sheet;
mod texture_cache;
mod wiggly_bois;
mod muzzle_flash;

extern crate glfw;

use crate::bullet_store::BulletStore;
use crate::enemy::{chase_player, Enemy, EnemySpawner};
use crate::floor::Floor;
use crate::framebuffers::{create_depth_map_fbo, create_emission_fbo, create_horizontal_blur_fbo, create_scene_fbo, create_vertical_blur_fbo};
use crate::player::Player;
use crate::sprite_sheet::{SpriteSheet, SpriteSheetSprite};
use crate::texture_cache::TextureCache;
use crate::wiggly_bois::draw_wiggly_bois;
use glam::{vec2, vec3, vec4, Mat4, Vec3, Vec4Swizzles};
use glfw::JoystickId::Joystick1;
use glfw::{Action, Context, Key, MouseButton};
use log::error;
use small_gl_core::animator::{AnimationClip, AnimationRepeat};
use small_gl_core::camera::{Camera, CameraMovement};
use small_gl_core::gl;
use small_gl_core::model::ModelBuilder;
use small_gl_core::shader::Shader;
use small_gl_core::texture::{TextureConfig, TextureType, TextureWrap};
use std::f32::consts::PI;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;
use small_gl_core::math::{get_world_ray_from_mouse, ray_plane_intersection};

const PARALLELISM: i32 = 4;

// Viewport
const VIEW_PORT_WIDTH: i32 = 1500;
const VIEW_PORT_HEIGHT: i32 = 1000;

// Texture units
const texUnit_playerDiffuse: i32 = 0;
const texUnit_gunDiffuse: i32 = 1;
const texUnit_floorDiffuse: i32 = 2;
const texUnit_wigglyBoi: i32 = 3;
const texUnit_bullet: i32 = 4;
const texUnit_floorNormal: i32 = 5;
const texUnit_playerNormal: i32 = 6;
const texUnit_gunNormal: i32 = 7;
const texUnit_shadowMap: i32 = 8;
const texUnit_emissionFBO: i32 = 9;
const texUnit_playerEmission: i32 = 10;
const texUnit_gunEmission: i32 = 11;
const texUnit_scene: i32 = 12;
const texUnit_horzBlur: i32 = 13;
const texUnit_vertBlur: i32 = 14;
const texUnit_impactSpriteSheet: i32 = 15;
const texUnit_muzzleFlashSpriteSheet: i32 = 16;
const texUnit_floorSpec: i32 = 18;
const texUnit_playerSpec: i32 = 19;
const texUnit_gunSpec: i32 = 20;

// Player
const FIRE_INTERVAL: f32 = 0.1; // seconds
const SPREAD_AMOUNT: i32 = 20;
const PLAYER_SPEED: f32 = 1.5;
const PLAYER_COLLISION_RADIUS: f32 = 0.35;

// Models
const PLAYER_MODEL_SCALE: f32 = 0.0044;
//const PLAYER_MODEL_GUN_HEIGHT: f32 = 120.0; // un-scaled
const PLAYER_MODEL_GUN_HEIGHT: f32 = 120.0; // un-scaled
const PLAYER_MODEL_GUN_MUZZLE_OFFSET: f32 = 100.0; // un-scaled
const MONSTER_Y: f32 = PLAYER_MODEL_SCALE * PLAYER_MODEL_GUN_HEIGHT;

// Lighting
const LIGHT_FACTOR: f32 = 0.8;
const NON_BLUE: f32 = 0.9;

const FLOOR_LIGHT_FACTOR: f32 = 0.35;
const FLOOR_NON_BLUE: f32 = 0.7;

// Enemies
const MONSTER_SPEED: f32 = 0.6;

enum CameraType {
    Game,
    Floating,
    Orthographic,
}

struct State {
    game_camera: Camera,
    floating_camera: Camera,
    ortho_camera: Camera,
    active_camera: CameraType,
    delta_time: f32,
    frame_time: f32,
    first_mouse: bool,
    mouse_x: f32,
    mouse_y: f32,
    player: Player,
    enemies: Vec<Enemy>,
    viewport_width: i32,
    viewport_height: i32,
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

    let instancedTextureShader = Rc::new(Shader::new("shaders/instanced_texture_shader.vert", "shaders/basic_texture_shader.frag").unwrap());

    simpleDepthShader.use_shader();
    let lsml = simpleDepthShader.get_uniform_location("lightSpaceMatrix");

    playerShader.use_shader();

    let playerLightSpaceMatrixLocation = playerShader.get_uniform_location("lightSpaceMatrix");
    playerShader.set_vec3("directionLight.dir", &playerLightDir);
    playerShader.set_vec3("directionLight.color", &lightColor);
    playerShader.set_vec3("ambient", &ambientColor);

    // player model textures handled externally
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

    // logTimeSince("shaders loaded ", appStart);

    let textures_to_load = [
        // from angrygl
        (
            texUnit_impactSpriteSheet,
            TextureType::Diffuse,
            "angrygl_assets/bullet/impact_spritesheet_with_00.png",
        ),
        (
            texUnit_muzzleFlashSpriteSheet,
            TextureType::Diffuse,
            "angrygl_assets/Player/muzzle_spritesheet.png",
        ),
        //(texUnit_bullet, TextureType::Diffuse, "angrygl_assets/bullet/BulletTexture2.png"), todo: need to create this one. view video for details
        // from Unity
        (texUnit_bullet, TextureType::Diffuse, "assets/Models/Bullet/Textures/BulletTexture.png"), // using this temporarily to get code right
        // (texUnit_wigglyBoi, TextureType::Diffuse, "assets/Models/Eeldog/Eeldog_Green_Albedo.png"),
        (texUnit_floorNormal, TextureType::Normals, "assets/Models/Floor N.png"),
        (texUnit_floorDiffuse, TextureType::Diffuse, "assets/Models/Floor D.png"),
        (texUnit_floorSpec, TextureType::Specular, "assets/Models/Floor M.png"),
    ];

    let mut texture_cache = TextureCache::new();
    let mut texture_config = TextureConfig::new().set_wrap(TextureWrap::Repeat);

    for (_unit, texture_type, path) in textures_to_load {
        texture_config.texture_type = texture_type;
        let texture = texture_cache.get_or_load_texture(path, &texture_config);
        match texture {
            Ok(_) => println!("Loaded: {}", path),
            Err(e) => println!("Error loading: {} {}", path, e),
        }
    }

    let floor = Floor::new(&mut texture_cache, &basicTextureShader);

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
    // ----------------- Game parts ---------------
    //

    let bulletImpactSpritesheet = SpriteSheet::new(texUnit_impactSpriteSheet, 11, 0.05);
    let muzzleFlashImpactSpritesheet = SpriteSheet::new(texUnit_muzzleFlashSpriteSheet, 6, 0.05);

    let mut bulletStore = BulletStore::new(&mut texture_cache);

    // Camera
    let cameraFollowVec = vec3(-4.0, 4.3, 0.0);
    let cameraUp = vec3(0.0, 1.0, 0.0);

    let game_camera = Camera::camera_vec3_up_yaw_pitch(
        vec3(0.0, 20.0, 80.0), // for xz world
        vec3(0.0, 1.0, 0.0),
        -90.0, // seems camera starts by looking down the x-axis, so needs to turn left to see the plane
        -20.0,
    );

    let floating_camera =  Camera::camera_vec3_up_yaw_pitch(
        vec3(0.0, 10.0, 20.0), // for xz world
        vec3(0.0, 1.0, 0.0),
        -90.0, // seems camera starts by looking down the x-axis, so needs to turn left to see the plane
        -20.0,
    );

    let ortho_camera = Camera::camera_vec3_up_yaw_pitch(
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        0.0,
        -90.0,
    );

    // Player
    let mut player = Player {
        lastFireTime: 0.0f32,
        isTryingToFire: false,
        isAlive: true,
        aimTheta: 0.0f32,
        position: vec3(0.0, 0.0, 0.0),
        player_direction: vec2(0.0, 0.0),
        animation_name: "".to_string(),
        speed: PLAYER_SPEED,
    };

    let mut state = State {
        game_camera,
        floating_camera,
        ortho_camera,
        active_camera: CameraType::Game,
        delta_time: 0.0,
        frame_time: 0.0,
        first_mouse: true,
        mouse_x: VIEW_PORT_WIDTH as f32 / 2.0,
        mouse_y: VIEW_PORT_HEIGHT as f32 / 2.0,
        player,
        enemies: vec![],
        viewport_width: VIEW_PORT_WIDTH,
        viewport_height: VIEW_PORT_HEIGHT,
    };

    let mut enemy_spawner = EnemySpawner::new(MONSTER_Y);

    let mut aimTheta = 0.0f32;

    let mut bulletImpactSprites: Vec<SpriteSheetSprite> = vec![];
    let mut muzzleFlashSpritesAge: Vec<f32> = vec![];
    let mut muzzle_world_position3: Vec3 = Vec3::ZERO;
    let mut use_point_light = false;

    player_model.play_clip(&idle);
    player_model.play_clip_with_transition(&idle, Duration::from_secs(6));
    state.player.animation_name = String::from(&idle.name);

    //
    // ----------------- render loop ---------------
    //

    println!("Assets loaded. Starting loop.");

    let game_projection = Mat4::perspective_rh_gl(state.game_camera.zoom.to_radians(), VIEW_PORT_WIDTH as f32 / VIEW_PORT_HEIGHT as f32, 0.1, 100.0);
    let floating_projection = Mat4::perspective_rh_gl(state.floating_camera.zoom.to_radians(), VIEW_PORT_WIDTH as f32 / VIEW_PORT_HEIGHT as f32, 0.1, 100.0);

    let width_half = VIEW_PORT_WIDTH as f32 / 2.0;
    let height_half = VIEW_PORT_HEIGHT as f32 / 2.0;
    let width_half = 3.0;
    let height_half = 3.0;

    let orthographic_projection = Mat4::orthographic_rh_gl(-width_half, width_half, -height_half, height_half, 0.1, 100.0);

    // let projection_inverse = game_projection.inverse();

    let mut buffer_ready = false;

    while !window.should_close() {

        // sleep(Duration::from_millis(200));

        let current_time = glfw.get_time() as f32;
        state.delta_time = current_time - state.frame_time;
        state.frame_time = current_time;

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event, &mut state);
        }

        // --- muzzle flash

        if &muzzleFlashSpritesAge.len() > &0 {
            for i in 0..muzzleFlashSpritesAge.len() {
                muzzleFlashSpritesAge[i] += &state.delta_time;
            }
            let max_age = muzzleFlashImpactSpritesheet.num_columns as f32 * muzzleFlashImpactSpritesheet.time_per_sprite;
            muzzleFlashSpritesAge.retain(|age| *age < max_age);
        }

        // --- bullet sprites

        if bulletImpactSprites.len() > 0 {
            for sheet in bulletImpactSprites.iter_mut() {
                sheet.age += &state.delta_time;
            }
            let sprite_duration = bulletImpactSpritesheet.num_columns as f32 * bulletImpactSpritesheet.time_per_sprite;
            bulletImpactSprites.retain(|sprite| sprite.age < sprite_duration);
        }

        bulletStore.update_bullets(&mut state, &mut bulletImpactSprites);

        if state.player.isAlive {
            enemy_spawner.update(&mut state);
            chase_player(&mut state);
        }

        let game_view = Mat4::look_at_rh(state.game_camera.position, state.player.position, state.game_camera.up);

        let (projection, camera_view) = match state.active_camera {
            CameraType::Game => {
                state.game_camera.position = state.player.position + cameraFollowVec.clone();
                (game_projection, game_view)
            }
            CameraType::Floating => {
                let view = Mat4::look_at_rh(state.floating_camera.position, state.player.position, state.floating_camera.up);
                (floating_projection, view)
            }
            CameraType::Orthographic => {
                let view = Mat4::look_at_rh(state.ortho_camera.position, state.player.position, state.ortho_camera.up);

                let eye = vec3(0.0, 1.0, 0.0); // Camera positioned above the origin
                let center = vec3(0.0, 0.0, 0.0); // Looking at the origin
                let up = vec3(0.0, 0.0, -1.0); // Up direction
                let view = Mat4::look_at_rh(eye, center, up);

                // top down view
                let view = Mat4::look_at_rh(vec3(state.player.position.x, 1.0, state.player.position.z), state.player.position, up);

                // side view
                let view = Mat4::look_at_rh(vec3(0.0, 0.0, -3.0), state.player.position, vec3(0.0, 1.0, 0.0));

                (orthographic_projection, view)
            }
        };

        let projection_view = projection * camera_view;

        let mut dx: f32 = 0.0;
        let mut dz: f32 = 0.0;

        state.player.isAlive = true;

        if state.player.isAlive && buffer_ready {

            let inv = (game_view.inverse() * game_projection.inverse()).to_cols_array_2d();

            let t = (inv[0][1] * state.mouse_x + inv[1][1] * state.mouse_y + inv[3][1]
                - MONSTER_Y * (inv[0][3] * state.mouse_x + inv[1][3] * state.mouse_x + inv[3][3]))
                / (inv[2][3] * MONSTER_Y - inv[2][1]);

            let s = 1.0 / (inv[0][3] * state.mouse_x + inv[1][3] * state.mouse_y + inv[2][3] * t + inv[3][3]);

            let us = state.mouse_x * s;
            let vs = state.mouse_y * s;

            let ts = t * s;
            let worldX = inv[0][0] * us + inv[1][0] * vs + inv[2][0] * ts + inv[3][0] * s;
            let worldZ = inv[0][2] * us + inv[1][2] * vs + inv[2][2] * ts + inv[3][2] * s;

            dx = worldX - state.player.position.x;
            dz = worldZ - state.player.position.z;

            aimTheta = (dx / dz).atan() + if dz < 0.0 { PI } else { 0.0 };

            if state.mouse_x.abs() < 0.005 && state.mouse_y.abs() < 0.005 {
                aimTheta = 0.0;
            }

            let world_ray = get_world_ray_from_mouse(
                state.mouse_x,
                state.mouse_y,
                VIEW_PORT_WIDTH as f32,
                VIEW_PORT_HEIGHT as f32,
                &game_view,
                &game_projection);

            // the xz plane
            let plane_point = vec3(0.0, 0.0, 0.0);
            let plane_normal = vec3(0.0, 1.0, 0.0);

            let world_point = ray_plane_intersection(
                state.game_camera.position,
                world_ray,
                plane_point,
                plane_normal,
            ).unwrap();

            dx = world_point.x - state.player.position.x;
            dz = world_point.z - state.player.position.z;

            aimTheta = (dx / dz).atan() + if dz < 0.0 { PI } else { 0.0 };

            if state.mouse_x.abs() < 0.005 && state.mouse_y.abs() < 0.005 {
                aimTheta = 0.0;
            }
        }

        // Todo:
        // player.update_points_for_anim(&mut state);

        let aimRot = Mat4::from_axis_angle(vec3(0.0, 1.0, 0.0), aimTheta);

        let mut player_model_transform = Mat4::from_translation(state.player.position);
        player_model_transform *= Mat4::from_scale(Vec3::splat(PLAYER_MODEL_SCALE));
        player_model_transform *= aimRot;

        if state.player.isAlive && state.player.isTryingToFire && (state.player.lastFireTime + FIRE_INTERVAL) < state.frame_time {

            println!("firing!");

            bulletStore.create_bullets(dx, dz, &player_model_transform, 5); // spreadAmount);

            state.player.lastFireTime = state.frame_time;
            muzzleFlashSpritesAge.push(0.0);
        }

        // Draw phase

        unsafe {
            //gl::ClearColor(0.0, 0.02, 0.25, 1.0);
            gl::ClearColor(0.8, 0.82, 0.85, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // --- draw floor
        floor.draw(&projection_view, &ambientColor);

        // --- draw bullets
        bulletStore.draw_bullets(&instancedTextureShader, &projection_view);

        // --- draw player with shadows
        {}

        if muzzleFlashSpritesAge.len() > 0 {
            // Muzzle pos calc

            // Position in original model of gun muzzle
            let point_vec = vec3(197.0, 76.143, -3.054);
            let meshes = player_model.meshes.borrow();
            let animator = player_model.animator.borrow();
            let gun_mesh = meshes.iter().find(|m| m.name.as_str() == "Gun").unwrap();
            let final_node_matrices = animator.final_node_matrices.borrow();
            let gun_transform = final_node_matrices.get(gun_mesh.id as usize).unwrap();

            let muzzle = *gun_transform * Mat4::from_translation(point_vec);
            // Adjust for player
            let muzzle_transform = player_model_transform * muzzle;
            let muzzle_world_position = muzzle_transform * vec4(0.0, 0.0, 0.0, 1.0);

            muzzle_world_position3 = vec3(
                muzzle_world_position.x / muzzle_world_position.w,
                muzzle_world_position.y / muzzle_world_position.w,
                muzzle_world_position.z / muzzle_world_position.w,
            );
            let mut min_age = 1000f32;
            for age in muzzleFlashSpritesAge.iter() {
                min_age = min_age.min(*age);
            }
            use_point_light = min_age < 0.03;

            playerShader.set_bool("usePointLight", use_point_light);
            playerShader.set_vec3("pointLight.worldPos", &muzzle_world_position3);
            playerShader.set_vec3("pointLight.color", &muzzle_point_light_color);
        } else {
            use_point_light = false;
            playerShader.set_bool("usePointLight", use_point_light);
        }


        playerShader.use_shader();
        playerShader.set_mat4("projectionView", &projection_view);

        playerShader.set_vec3("viewPos", &state.game_camera.position);
        playerShader.set_mat4("model", &player_model_transform);
        playerShader.set_mat4("aimRot", &aimRot);

        playerShader.set_mat4("lightSpaceMatrix", &Mat4::IDENTITY);
        playerShader.set_bool("useLight", true);
        playerShader.set_vec3("ambient", &ambientColor);

        player_model.update_animation(state.delta_time);
        player_model.render(&playerShader);


        if muzzleFlashSpritesAge.len() > 0 {


        }


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

        draw_wiggly_bois(&wigglyBoi, &wigglyShader, &mut state);

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
        glfw::WindowEvent::Key(Key::Num1, _, _, modifier) => {
            state.active_camera = CameraType::Game;
        }
        glfw::WindowEvent::Key(Key::Num2, _, _, modifier) => {
            state.active_camera = CameraType::Floating;
        }
        glfw::WindowEvent::Key(Key::Num3, _, _, modifier) => {
            state.active_camera = CameraType::Orthographic;
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
        glfw::WindowEvent::Key(Key::Q, _, _, modifier) => {
            state.floating_camera.process_keyboard(CameraMovement::Up, state.delta_time);
        }
        glfw::WindowEvent::Key(Key::Z, _, _, modifier) => {
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

// glfw: whenever the window size changed (by OS or user resize) this event fires.
// ---------------------------------------------------------------------------------------------
fn framebuffer_size_event(_window: &mut glfw::Window, state: &mut State, width: i32, height: i32) {
    // make sure the viewport matches the new window dimensions; note that width and
    // height will be significantly larger than specified on retina displays.
    // println!("Framebuffer size: {}, {}", width, height);
    unsafe {
        state.viewport_width = width;
        state.viewport_height = height;
        gl::Viewport(0, 0, width, height);
    }
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

    // state.camera.process_mouse_movement(xoffset, yoffset, true);
}

fn scroll_handler(state: &mut State, _xoffset: f64, yoffset: f64) {
    state.game_camera.process_mouse_scroll(yoffset as f32);
}
